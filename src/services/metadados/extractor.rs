use chrono::{DateTime, Local};
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use super::archives::extrair_arquivo_compactado_com_ext;
use super::documents::{
    extrair_apresentacao, extrair_banco_dados, extrair_codigo_fonte, extrair_documento,
    extrair_planilha, ler_texto_flexivel,
};
use super::executables::extrair_executavel;
use super::harmonizacao::{
    completar_campos_conteudo, definir_binario, extensao_efetiva, harmonizar_metadados,
    perfil_evento, tipo_eh_pasta,
};
use super::images::extrair_imagem_com_ext;
use super::media::extrair_midia_com_ext;
use super::types::{
    extensao_tipo_preferencial, is_audio_ext, is_backup_ext, is_compressed_ext, is_config_ext,
    is_database_ext, is_document_ext, is_executable_ext, is_font_ext, is_gis_ext, is_image_ext,
    is_log_ext, is_presentation_ext, is_source_code_ext, is_spreadsheet_ext, is_system_ext,
    is_video_ext, is_virtual_disk_ext, obter_tipo_e_mime,
};
use crate::core::event::EventModel;

#[derive(Clone)]
struct DirStatsCacheEntry {
    total_bytes: u64,
    total_files: usize,
    updated_at: Instant,
}

static DIR_STATS_CACHE: LazyLock<Mutex<HashMap<String, DirStatsCacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static METADATA_EVENT_CACHE: LazyLock<Mutex<HashMap<String, EventModel>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn chave_cache_caminho(caminho: &Path) -> String {
    caminho
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase()
}

pub fn obter_evento_cacheado(caminho: &Path) -> Option<EventModel> {
    let key = chave_cache_caminho(caminho);
    METADATA_EVENT_CACHE.lock().get(&key).cloned()
}

pub fn salvar_evento_cache(caminho: &Path, evento: &EventModel) {
    let key = chave_cache_caminho(caminho);
    let mut cache = METADATA_EVENT_CACHE.lock();
    if cache.len() > 25_000 {
        cache.clear();
    }
    cache.insert(key, evento.clone());
}

/// Abre arquivo permitindo compartilhamento de leitura/escrita/exclusão no Windows
/// (evita `ERROR_SHARING_VIOLATION` durante cópias e gravações concorrentes).
pub fn abrir_arquivo_compartilhado(caminho: &Path) -> std::io::Result<File> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_SHARE_READ (1) | FILE_SHARE_WRITE (2) | FILE_SHARE_DELETE (4) = 7
        std::fs::OpenOptions::new()
            .read(true)
            .share_mode(7)
            .open(caminho)
    }
    #[cfg(not(target_os = "windows"))]
    {
        File::open(caminho)
    }
}

/// Tenta abrir o arquivo com retentativas breves para contornar locks temporários do Windows durante gravação.
pub fn abrir_arquivo_com_espera(
    caminho: &Path,
    max_tentativas: usize,
    delay_ms: u64,
) -> std::io::Result<File> {
    let mut ultimo_erro = None;
    for i in 0..max_tentativas {
        match abrir_arquivo_compartilhado(caminho) {
            Ok(f) => return Ok(f),
            Err(e) => {
                ultimo_erro = Some(e);
                if i + 1 < max_tentativas {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    }
    Err(ultimo_erro.unwrap_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Falha ao abrir arquivo compartilhado",
        )
    }))
}

/// Verifica se o arquivo está acessível para leitura compartilhada.
pub fn arquivo_acessivel(caminho: &Path) -> bool {
    abrir_arquivo_compartilhado(caminho).is_ok()
}

/// Extração voltada a eventos em tempo real (`Observador`): caso o arquivo exista mas esteja
/// sob lock exclusivo transitório durante cópia/gravação no Windows (`dwShareMode = 0`),
/// aguarda brevemente a liberação do handle antes de extrair os metadados completos.
pub fn extrair_metadados_ao_vivo(caminho: &Path, evento: &mut EventModel) {
    if caminho.exists() && !caminho.is_dir() && !arquivo_acessivel(caminho) {
        const ATRASOS_MS: [u64; 9] = [10, 20, 30, 50, 80, 120, 180, 250, 350];
        for delay in ATRASOS_MS {
            std::thread::sleep(Duration::from_millis(delay));
            if !caminho.exists() || arquivo_acessivel(caminho) {
                break;
            }
        }
    }
    extrair_metadados_completos(caminho, evento);
}

/// Preenche metadados para um item recém-excluído usando o cache em memória ou inferência pelo nome/extensão.
pub fn preencher_metadados_item_excluido(caminho: &Path, evento: &mut EventModel) {
    if let Some(cached) = obter_evento_cacheado(caminho) {
        let id = evento.id;
        let op = evento.tipo_operacao.clone();
        let nome = evento.nome.clone();
        let ts = evento.timestamp.clone();
        let dir_ant = evento.dir_anterior.clone();
        let dir_at = evento.dir_atual.clone();
        let pre_sha256 = evento.sha256_atual.clone();
        let pre_sha256_base = evento.sha256_baseline.clone();
        let pre_size_b = evento.size_b;
        let pre_data_mod = evento.data_modificacao.clone();

        *evento = cached;
        evento.id = id;
        evento.tipo_operacao = op;
        if !nome.is_empty() {
            let mudou_nome = !evento.nome.eq_ignore_ascii_case(&nome);
            evento.nome = nome;
            if mudou_nome {
                let is_dir_cached = tipo_eh_pasta(&evento.tipo, &evento.mime);
                let nova_ext = extensao_tipo_preferencial(Path::new(&evento.nome));
                if is_dir_cached || !nova_ext.is_empty() {
                    let (tipo_str, mime_str) =
                        obter_tipo_e_mime(Path::new(&evento.nome), is_dir_cached);
                    evento.tipo = tipo_str;
                    evento.mime = mime_str;
                }
            }
        }
        if evento.size_b == 0.0 && pre_size_b > 0.0 {
            evento.size_b = pre_size_b;
            evento.size_kb = pre_size_b / 1024.0;
            evento.size_mb = evento.size_kb / 1024.0;
            evento.size_gb = evento.size_mb / 1024.0;
            evento.size_tb = evento.size_gb / 1024.0;
        }
        if evento.sha256_atual.is_empty() && !pre_sha256.is_empty() {
            evento.sha256_atual = pre_sha256;
        }
        if evento.sha256_baseline.is_empty() && !pre_sha256_base.is_empty() {
            evento.sha256_baseline = pre_sha256_base;
        }
        if evento.data_modificacao.is_empty() && !pre_data_mod.is_empty() {
            evento.data_modificacao = pre_data_mod;
        }
        evento.timestamp = ts;
        evento.dir_anterior = dir_ant;
        evento.dir_atual = dir_at;
        harmonizar_metadados(evento);
        return;
    }

    // Determina se é pasta a partir dos atributos ou tipo pré-existentes, nunca adivinhando que sem ponto é pasta
    let is_dir = tipo_eh_pasta(&evento.tipo, &evento.mime)
        || evento.atributos.contains("Diretório")
        || evento.atributos.contains("folder");

    if is_dir || !extensao_tipo_preferencial(caminho).is_empty() || evento.tipo.is_empty() {
        let (tipo_str, mime_str) = obter_tipo_e_mime(caminho, is_dir);
        evento.tipo = tipo_str;
        if evento.mime.is_empty() || evento.mime == "application/octet-stream" || is_dir {
            evento.mime = mime_str;
        }
    } else if evento.mime.is_empty() {
        let (_, mime_str) = obter_tipo_e_mime(caminho, false);
        evento.mime = mime_str;
    }

    if evento.autor.is_empty() {
        evento.autor = obter_usuario_sistema();
    }
    if evento.protegido.is_empty() {
        evento.protegido = "Não".to_string();
    }
    if evento.atributos.is_empty() {
        evento.atributos = if is_dir {
            "folder".to_string()
        } else {
            "file".to_string()
        };
    }
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    if evento.data_modificacao.is_empty() {
        evento.data_modificacao = now.clone();
    }
    if evento.data_criacao.is_empty() {
        evento.data_criacao = now.clone();
    }
    if evento.data_acesso.is_empty() {
        evento.data_acesso = now;
    }

    harmonizar_metadados(evento);
}

fn obter_usuario_sistema() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Sistema".to_string())
}

pub fn extrair_metadados_completos(caminho: &Path, evento: &mut EventModel) {
    let sys_user = obter_usuario_sistema();

    let Ok(mut metadata) = fs::metadata(caminho) else {
        preencher_metadados_item_excluido(caminho, evento);
        return;
    };

    let is_dir = metadata.is_dir();
    if !is_dir
        && metadata.len() == 0
        && matches!(
            evento.op_type(),
            crate::core::types::OperationType::Adicionado
                | crate::core::types::OperationType::Modificado
        )
    {
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(10));
            if let Ok(m) = fs::metadata(caminho) {
                metadata = m;
                if metadata.len() > 0 {
                    break;
                }
            }
        }
    }

    let (tipo_str, mime_str) = obter_tipo_e_mime(caminho, is_dir);
    evento.tipo = tipo_str;
    evento.mime = mime_str;

    // Se o arquivo sem extensão está bloqueado no momento da leitura mas já havia sido
    // identificado em cache (ex.: `.png`, `.sqlite`, `document`), preserva o tipo real.
    if !is_dir && extensao_tipo_preferencial(caminho).is_empty() && !arquivo_acessivel(caminho) {
        if let Some(cached) = obter_evento_cacheado(caminho) {
            if !cached.tipo.is_empty()
                && cached.tipo != "file"
                && !tipo_eh_pasta(&cached.tipo, &cached.mime)
            {
                evento.tipo = cached.tipo;
                if !cached.mime.is_empty() {
                    evento.mime = cached.mime;
                }
            }
        }
    }

    if let Ok(modified) = metadata.modified() {
        let dt: DateTime<Local> = modified.into();
        evento.data_modificacao = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    if let Ok(created) = metadata.created() {
        let dt: DateTime<Local> = created.into();
        evento.data_criacao = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    if let Ok(accessed) = metadata.accessed() {
        let dt: DateTime<Local> = accessed.into();
        evento.data_acesso = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    // Atributos e proteção baseados no sistema de arquivos (chaves canônicas traduzidas pelo Localizer)
    let mut attr_flags = Vec::new();
    if is_dir {
        attr_flags.push("folder");
    } else {
        attr_flags.push("file");
    }

    let mut is_protected = metadata.permissions().readonly();
    if metadata.permissions().readonly() {
        attr_flags.push("readonly");
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        let attrs = metadata.file_attributes();
        if (attrs & 0x00000001) != 0 && !attr_flags.contains(&"readonly") {
            attr_flags.push("readonly");
            is_protected = true;
        }
        if (attrs & 0x00000002) != 0 {
            attr_flags.push("hidden");
            is_protected = true;
        }
        if (attrs & 0x00000004) != 0 {
            attr_flags.push("system");
            is_protected = true;
        }
        if (attrs & 0x00000020) != 0 {
            attr_flags.push("archive");
        }
        if (attrs & 0x00000800) != 0 {
            attr_flags.push("compressed");
        }
        if (attrs & 0x00004000) != 0 {
            attr_flags.push("encrypted");
            is_protected = true;
        }
    }
    evento.atributos = attr_flags.join(", ");

    // Tratamento de diretórios (paridade com gmet_38_GetMetadados.cpp e gmet_47_GetTamanhoBytes.cpp)
    if is_dir {
        let (dir_bytes, dir_files) = calcular_estatisticas_diretorio(caminho);
        evento.size_b = dir_bytes as f64;
        evento.size_kb = evento.size_b / 1024.0;
        evento.size_mb = evento.size_kb / 1024.0;
        evento.size_gb = evento.size_mb / 1024.0;
        evento.size_tb = evento.size_gb / 1024.0;
        evento.arquivos = dir_files.to_string();
        if evento.autor.is_empty() {
            evento.autor = sys_user;
        }
        evento.protegido = if is_protected {
            "Sim".to_string()
        } else {
            "Não".to_string()
        };
        harmonizar_metadados(evento);
        salvar_evento_cache(caminho, evento);
        return;
    }

    let size_bytes = metadata.len();
    evento.size_b = size_bytes as f64;
    evento.size_kb = evento.size_b / 1024.0;
    evento.size_mb = evento.size_kb / 1024.0;
    evento.size_gb = evento.size_mb / 1024.0;
    evento.size_tb = evento.size_gb / 1024.0;

    let ext = extensao_efetiva(caminho, &evento.tipo);
    let mut format_author = String::new();

    // 1. Imagens (dimensoes, autor EXIF/PNG/SVG)
    if is_image_ext(&ext) {
        if let Some(img_meta) = extrair_imagem_com_ext(caminho, &ext) {
            if !img_meta.dimensoes.is_empty() {
                evento.dimensoes = img_meta.dimensoes;
            }
            if !img_meta.autor.is_empty() {
                format_author = img_meta.autor;
            }
        }
    }

    // 2. Áudio e Vídeo (duracao, taxa_bits, dimensoes, autor)
    if is_video_ext(&ext) || is_audio_ext(&ext) {
        if let Some(media_meta) = extrair_midia_com_ext(caminho, &ext, size_bytes) {
            if !media_meta.duracao.is_empty() {
                evento.duracao = media_meta.duracao;
            }
            if !media_meta.taxa_bits.is_empty() {
                evento.taxa_bits = media_meta.taxa_bits;
            }
            if !media_meta.dimensoes.is_empty() && evento.dimensoes.is_empty() {
                evento.dimensoes = media_meta.dimensoes;
            }
            if !media_meta.autor.is_empty() && format_author.is_empty() {
                format_author = media_meta.autor;
            }
        }
    }

    // 3. Código-fonte (total_linhas, linhas_codigo, linhas, palavras, paginas, paginas_estimadas)
    if is_source_code_ext(&ext) {
        if let Some(code_meta) = extrair_codigo_fonte(caminho, size_bytes) {
            evento.linhas = code_meta.linhas;
            evento.total_linhas = code_meta.total_linhas;
            evento.linhas_codigo = code_meta.linhas_codigo;
            evento.palavras = code_meta.palavras;
            evento.paginas = code_meta.paginas;
            evento.paginas_estimadas = code_meta.paginas_estimadas;
        }
    }
    // 4. Planilhas (planilhas, colunas, registros, tabelas, linhas, total_linhas)
    else if is_spreadsheet_ext(&ext) {
        if let Some(sp_meta) = extrair_planilha(caminho, &ext, size_bytes) {
            evento.planilhas = sp_meta.planilhas;
            evento.tabelas = sp_meta.tabelas;
            evento.colunas = sp_meta.colunas;
            evento.registros = sp_meta.registros;
            evento.linhas = sp_meta.linhas;
            evento.total_linhas = sp_meta.total_linhas;
            if !sp_meta.paginas_estimadas.is_empty() {
                evento.paginas_estimadas = sp_meta.paginas_estimadas;
            }
            if !sp_meta.autor.is_empty() && format_author.is_empty() {
                format_author = sp_meta.autor;
            }
        }
    }
    // 5. Apresentações (slides, slides_estimados, paginas, paginas_estimadas, palavras)
    else if is_presentation_ext(&ext) {
        if let Some(pr_meta) = extrair_apresentacao(caminho, &ext, size_bytes) {
            evento.slides = pr_meta.slides;
            evento.slides_estimados = pr_meta.slides_estimados;
            evento.paginas = pr_meta.paginas;
            evento.paginas_estimadas = pr_meta.paginas_estimadas;
            if !pr_meta.palavras.is_empty() {
                evento.palavras = pr_meta.palavras;
            }
            if !pr_meta.autor.is_empty() && format_author.is_empty() {
                format_author = pr_meta.autor;
            }
        }
    }
    // 6. Bancos de Dados e GIS (tabelas, colunas, registros, linhas, total_linhas)
    else if is_database_ext(&ext) || is_gis_ext(&ext) {
        if let Some(db_meta) = extrair_banco_dados(caminho, &ext, size_bytes) {
            evento.tabelas = db_meta.tabelas;
            evento.colunas = db_meta.colunas;
            evento.registros = db_meta.registros;
            if evento.linhas.is_empty() {
                evento.linhas = db_meta.linhas;
            }
            if evento.total_linhas.is_empty() {
                evento.total_linhas = db_meta.total_linhas;
            }
        }
    }
    // 7. Documentos, Logs e Configurações (paginas, paginas_estimadas, linhas, total_linhas, palavras, autor)
    else if is_document_ext(&ext) || is_log_ext(&ext) || is_config_ext(&ext) {
        if let Some(doc_meta) = extrair_documento(caminho, &ext, size_bytes) {
            if !doc_meta.paginas.is_empty() {
                evento.paginas = doc_meta.paginas;
            }
            if !doc_meta.paginas_estimadas.is_empty() {
                evento.paginas_estimadas = doc_meta.paginas_estimadas;
            }
            if !doc_meta.linhas.is_empty() {
                evento.linhas = doc_meta.linhas;
            }
            if !doc_meta.total_linhas.is_empty() {
                evento.total_linhas = doc_meta.total_linhas;
            }
            if !doc_meta.palavras.is_empty() {
                evento.palavras = doc_meta.palavras;
            }
            if doc_meta.protegido {
                is_protected = true;
            }
            if !doc_meta.autor.is_empty() && format_author.is_empty() {
                format_author = doc_meta.autor;
            }
        }
    }

    // 8. Compactados, Discos Virtuais e Backups (arquivos, unzipped_b..tb)
    if is_compressed_ext(&ext) || is_virtual_disk_ext(&ext) || is_backup_ext(&ext) {
        if let Some(arch_meta) = extrair_arquivo_compactado_com_ext(caminho, &ext) {
            evento.arquivos = arch_meta.arquivos;
            evento.unzipped_b = arch_meta.unzipped_b;
            evento.unzipped_kb = arch_meta.unzipped_kb;
            evento.unzipped_mb = arch_meta.unzipped_mb;
            evento.unzipped_gb = arch_meta.unzipped_gb;
            evento.unzipped_tb = arch_meta.unzipped_tb;
            if arch_meta.protegido {
                is_protected = true;
            }
        }
    }

    // 9. Executáveis, Fontes e Binários de Sistema (binary_file_b..tb, publisher)
    if is_executable_ext(&ext) || is_font_ext(&ext) || is_system_ext(&ext) || ext == "lnk" {
        if let Some(exec_meta) = extrair_executavel(caminho, size_bytes) {
            evento.binary_file_b = exec_meta.binary_file_b;
            evento.binary_file_kb = exec_meta.binary_file_kb;
            evento.binary_file_mb = exec_meta.binary_file_mb;
            evento.binary_file_gb = exec_meta.binary_file_gb;
            evento.binary_file_tb = exec_meta.binary_file_tb;
            if !exec_meta.publisher.is_empty() {
                evento.publisher = exec_meta.publisher.clone();
                if format_author.is_empty() {
                    format_author = exec_meta.publisher;
                }
            }
            if exec_meta.assinado {
                is_protected = true;
            }
        }
    }

    // 10. Arquivos não estruturados (texto sem extensão vs binário/temporário):
    // segue estritamente o perfil determinístico do tipo/extensão, nunca contaminando outras categorias.
    let p = perfil_evento(evento);
    if p.paginas && p.linhas && p.palavras && evento.linhas.is_empty() {
        let max_read = size_bytes.min(2 * 1024 * 1024) as usize;
        if let Some(text) = ler_texto_flexivel(caminho, max_read) {
            let lines = text.lines().count().max(usize::from(size_bytes > 0));
            let words = text.split_whitespace().count();
            let pages = ((lines as f64) / 45.0).ceil().max(1.0) as usize;
            evento.linhas = lines.to_string();
            evento.total_linhas = lines.to_string();
            evento.palavras = words.to_string();
            evento.paginas = pages.to_string();
            evento.paginas_estimadas = pages.to_string();
        }
    } else if p.binario && evento.binary_file_b == 0.0 && size_bytes > 0 {
        definir_binario(evento, size_bytes as f64);
    }

    if !format_author.is_empty() {
        evento.autor = format_author;
    } else if evento.autor.is_empty() {
        evento.autor = sys_user;
    }

    evento.protegido = if is_protected {
        "Sim".to_string()
    } else {
        "Não".to_string()
    };

    // Calcula SHA-256 para todos os arquivos (inclusive 0 bytes e arquivos grandes)
    if metadata.is_file() {
        if let Some(hash) = calcular_sha256(caminho) {
            evento.sha256_atual = hash;
        }
    }

    // Se houver registro anterior em cache para o caminho (ex.: arquivo sob lock exclusivo no SO),
    // preserva os metadados compatíveis que não puderam ser lidos diretamente
    if let Some(cached) = obter_evento_cacheado(caminho) {
        completar_campos_conteudo(evento, &cached);
        if evento.sha256_atual.is_empty() && !cached.sha256_atual.is_empty() {
            evento.sha256_atual = cached.sha256_atual;
        }
    }

    harmonizar_metadados(evento);
    salvar_evento_cache(caminho, evento);
}

/// Calcula tamanho total (bytes) e contagem de arquivos de um diretório com cache de curta duração.
pub fn calcular_estatisticas_diretorio(caminho: &Path) -> (u64, usize) {
    let key = chave_cache_caminho(caminho);
    {
        let cache = DIR_STATS_CACHE.lock();
        if let Some(entry) = cache.get(&key) {
            if entry.updated_at.elapsed() < Duration::from_secs(3) {
                return (entry.total_bytes, entry.total_files);
            }
        }
    }

    let mut total_bytes = 0u64;
    let mut total_files = 0usize;
    let mut stack = vec![(caminho.to_path_buf(), 0usize)];
    let mut visited_entries = 0usize;

    while let Some((dir, depth)) = stack.pop() {
        if depth > 8 || visited_entries > 15_000 {
            break;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry_res in entries {
            let Ok(entry) = entry_res else {
                continue;
            };
            visited_entries += 1;
            if visited_entries > 15_000 {
                break;
            }
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push((entry.path(), depth + 1));
            } else if ft.is_file() {
                total_files += 1;
                if let Ok(m) = entry.metadata() {
                    total_bytes = total_bytes.saturating_add(m.len());
                }
            }
        }
    }

    let mut cache = DIR_STATS_CACHE.lock();
    if cache.len() > 4096 {
        cache.clear();
    }
    cache.insert(
        key,
        DirStatsCacheEntry {
            total_bytes,
            total_files,
            updated_at: Instant::now(),
        },
    );

    (total_bytes, total_files)
}

pub fn calcular_sha256(caminho: &Path) -> Option<String> {
    let mut file = abrir_arquivo_compartilhado(caminho).ok()?;
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut hasher = Sha256::new();

    const LIMITE_HASH_COMPLETO: u64 = 64 * 1024 * 1024;
    const BLOCO_AMOSTRA: usize = 4 * 1024 * 1024;

    if file_size <= LIMITE_HASH_COMPLETO {
        let mut buffer = [0u8; 65536];
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return None,
            }
        }
    } else {
        // Para arquivos > 64 MB, combina tamanho exato + 3 janelas de 4 MB (início, meio e fim)
        // para produzir hash SHA-256 hexadecimal de 64 caracteres sem gargalo de I/O.
        hasher.update(b"linceu-large-sha256:");
        hasher.update(file_size.to_le_bytes());
        let offsets = [
            0u64,
            (file_size / 2).saturating_sub((BLOCO_AMOSTRA / 2) as u64),
            file_size.saturating_sub(BLOCO_AMOSTRA as u64),
        ];
        let mut buf = vec![0u8; BLOCO_AMOSTRA];
        for off in offsets {
            if file.seek(SeekFrom::Start(off)).is_ok() {
                let mut lido = 0usize;
                while lido < BLOCO_AMOSTRA {
                    match file.read(&mut buf[lido..]) {
                        Ok(0) => break,
                        Ok(n) => lido += n,
                        Err(_) => break,
                    }
                }
                hasher.update(&buf[..lido]);
            }
        }
    }

    Some(hex::encode(hasher.finalize()))
}

pub fn extrair_metadados_scan(caminho: &Path, evento: &mut EventModel) {
    extrair_metadados_completos(caminho, evento);
}
