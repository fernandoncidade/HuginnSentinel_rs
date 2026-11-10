//! Harmonização determinística de metadados por categoria de arquivo.
//!
//! Garante que um mesmo tipo de arquivo exponha exatamente o mesmo conjunto de colunas
//! de metadados em todas as operações monitoradas (Adicionado, Escaneado, Excluído,
//! Modificado, Movido e Renomeado), independentemente do caminho de código que produziu
//! o evento (extração em disco, cache em memória, histórico SQLite ou inferência).
//!
//! Regras aplicadas:
//! 1. Cada extensão/tipo é mapeado determinísticamente para um [`PerfilMetadados`] que define
//!    quais grupos de colunas pertencem à categoria (mesma lógica de despacho de
//!    `extrair_metadados_completos`).
//! 2. Campos que não pertencem à categoria são removidos (ex.: `Binário` em `.png`,
//!    `Páginas/Linhas` em `.7z` ou `.old`, metadados de arquivo em `folder`).
//! 3. Campos pertencentes à categoria que ficaram vazios (por bloqueio exclusivo do SO ou
//!    exclusão sem histórico prévio) são completados por estimativa determinística.

use sha2::{Digest, Sha256};
use std::path::Path;

use super::media::formatar_duracao;
use super::types::{
    extensao_tipo_preferencial, is_audio_ext, is_backup_ext, is_compressed_ext, is_config_ext,
    is_database_ext, is_document_ext, is_executable_ext, is_font_ext, is_gis_ext, is_image_ext,
    is_log_ext, is_presentation_ext, is_source_code_ext, is_spreadsheet_ext, is_system_ext,
    is_video_ext, is_virtual_disk_ext, obter_mime_por_extensao,
};
use crate::core::event::EventModel;

/// Conjunto de grupos de colunas de metadados pertencentes a uma categoria de arquivo.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PerfilMetadados {
    pub pasta: bool,
    pub dimensoes: bool,
    /// `duracao` e `taxa_bits`.
    pub midia: bool,
    pub paginas: bool,
    /// `linhas` e `total_linhas`.
    pub linhas: bool,
    pub palavras: bool,
    pub paginas_estimadas: bool,
    pub linhas_codigo: bool,
    /// `slides` e `slides_estimados`.
    pub slides: bool,
    pub planilhas: bool,
    /// `colunas`, `registros` e `tabelas`.
    pub tabelas: bool,
    pub arquivos: bool,
    /// `unzipped_b..tb`.
    pub descompactado: bool,
    /// `binary_file_b..tb`.
    pub binario: bool,
}

impl PerfilMetadados {
    fn pasta() -> Self {
        Self {
            pasta: true,
            arquivos: true,
            ..Default::default()
        }
    }

    fn texto_documento(&mut self) {
        self.paginas = true;
        self.linhas = true;
        self.palavras = true;
        self.paginas_estimadas = true;
    }
}

/// Indica se o `tipo`/`mime` informado representa um diretório.
pub fn tipo_eh_pasta(tipo: &str, mime: &str) -> bool {
    let t = tipo.trim();
    t.eq_ignore_ascii_case("folder")
        || t.eq_ignore_ascii_case("pasta")
        || t.eq_ignore_ascii_case("directory")
        || t == "Pasta de Arquivos"
        || mime == "inode/directory"
}

/// Extensão efetiva usada para despacho por categoria: a extensão do nome do arquivo ou,
/// para arquivos sem extensão, o tipo detectado por assinatura mágica/nome
/// (`.png`, `.pdf`, `.sqlite`, `code`, `document`, `config`...).
pub fn extensao_efetiva(caminho: &Path, tipo: &str) -> String {
    let ext = extensao_tipo_preferencial(caminho);
    if !ext.is_empty() {
        return ext;
    }
    let t = tipo.trim().trim_start_matches('.').to_lowercase();
    match t.as_str() {
        "" | "file" | "arquivo" | "text" | "texto" | "binary" | "binário" | "binario"
        | "folder" | "pasta" | "directory" | "pasta de arquivos" | "unknown" | "desconhecido" => {
            String::new()
        }
        _ => t,
    }
}

/// Perfil de metadados para uma extensão efetiva não vazia (espelha a ordem de despacho do extrator).
pub fn perfil_por_extensao(ext: &str) -> PerfilMetadados {
    let mut p = PerfilMetadados::default();
    if ext.is_empty() {
        return p;
    }

    if is_image_ext(ext) {
        p.dimensoes = true;
    }
    if is_video_ext(ext) {
        p.dimensoes = true;
        p.midia = true;
    }
    if is_audio_ext(ext) {
        p.midia = true;
    }

    if is_source_code_ext(ext) {
        p.texto_documento();
        p.linhas_codigo = true;
    } else if is_spreadsheet_ext(ext) {
        p.planilhas = true;
        p.tabelas = true;
        p.linhas = true;
        p.paginas_estimadas = true;
    } else if is_presentation_ext(ext) {
        p.slides = true;
        p.paginas = true;
        p.paginas_estimadas = true;
        p.palavras = true;
    } else if is_database_ext(ext) || is_gis_ext(ext) {
        p.tabelas = true;
        p.linhas = true;
    } else if is_document_ext(ext) || is_log_ext(ext) || is_config_ext(ext) {
        p.texto_documento();
    }

    if is_compressed_ext(ext) || is_virtual_disk_ext(ext) || is_backup_ext(ext) {
        p.arquivos = true;
        p.descompactado = true;
    }

    if is_executable_ext(ext) || is_font_ext(ext) || is_system_ext(ext) || ext == "lnk" {
        p.binario = true;
    }

    // Extensões não listadas nas categorias estruturadas (.tmp, .dat, extensões customizadas):
    // mapeamento determinístico por MIME da extensão (text/* -> documento; demais -> binário).
    if p == PerfilMetadados::default() {
        let mime = obter_mime_por_extensao(ext, Path::new(""));
        if mime.starts_with("text/") {
            p.texto_documento();
        } else {
            p.binario = true;
        }
    }

    p
}

/// Detecta registros históricos corrompidos: marcados como `folder`, mas carregando
/// metadados que só existem em arquivos (hash, contagem de linhas, tamanho binário...).
pub fn evento_pasta_inconsistente(ev: &EventModel) -> bool {
    tipo_eh_pasta(&ev.tipo, &ev.mime)
        && (!ev.sha256_atual.is_empty()
            || ev.binary_file_b > 0.0
            || !ev.linhas.is_empty()
            || !ev.paginas.is_empty()
            || !ev.palavras.is_empty()
            || !ev.dimensoes.is_empty()
            || !ev.duracao.is_empty())
}

/// Perfil determinístico de metadados de um evento.
pub fn perfil_evento(ev: &EventModel) -> PerfilMetadados {
    if tipo_eh_pasta(&ev.tipo, &ev.mime) && !evento_pasta_inconsistente(ev) {
        return PerfilMetadados::pasta();
    }
    let ext = extensao_efetiva(Path::new(&ev.nome), &ev.tipo);
    if !ext.is_empty() {
        return perfil_por_extensao(&ext);
    }

    // Arquivo sem extensão e sem assinatura de formato específico:
    // `binary` expõe `binary_file_b..tb`; `text` e `file` expõem métricas textuais.
    let mut p = PerfilMetadados::default();
    let t = ev.tipo.trim().to_lowercase();
    if t == "binary" || t == "binário" || t == "binario" {
        p.binario = true;
    } else {
        p.texto_documento();
    }
    p
}

/// Indica se o evento deve renderizar as colunas `unzipped_b..tb` (mesmo quando `0`).
pub fn exibe_descompactado(ev: &EventModel) -> bool {
    if ev.nome.is_empty() && ev.tipo.is_empty() {
        return false;
    }
    perfil_evento(ev).descompactado
}

/// Indica se o evento deve renderizar as colunas `binary_file_b..tb` (mesmo quando `0`).
pub fn exibe_binario(ev: &EventModel) -> bool {
    if ev.nome.is_empty() && ev.tipo.is_empty() {
        return false;
    }
    perfil_evento(ev).binario
}

/// Remove todos os metadados específicos de conteúdo (mantém identificação, datas,
/// atributos, autor, tamanho e campos de segurança).
pub fn limpar_campos_conteudo(ev: &mut EventModel) {
    ev.dimensoes.clear();
    ev.duracao.clear();
    ev.taxa_bits.clear();
    ev.paginas.clear();
    ev.linhas.clear();
    ev.palavras.clear();
    ev.paginas_estimadas.clear();
    ev.linhas_codigo.clear();
    ev.total_linhas.clear();
    ev.slides_estimados.clear();
    ev.arquivos.clear();
    zerar_descompactado(ev);
    ev.slides.clear();
    zerar_binario(ev);
    ev.planilhas.clear();
    ev.colunas.clear();
    ev.registros.clear();
    ev.tabelas.clear();
}

fn zerar_descompactado(ev: &mut EventModel) {
    ev.unzipped_b = 0.0;
    ev.unzipped_kb = 0.0;
    ev.unzipped_mb = 0.0;
    ev.unzipped_gb = 0.0;
    ev.unzipped_tb = 0.0;
}

fn zerar_binario(ev: &mut EventModel) {
    ev.binary_file_b = 0.0;
    ev.binary_file_kb = 0.0;
    ev.binary_file_mb = 0.0;
    ev.binary_file_gb = 0.0;
    ev.binary_file_tb = 0.0;
}

pub(crate) fn definir_binario(ev: &mut EventModel, b: f64) {
    ev.binary_file_b = b;
    ev.binary_file_kb = b / 1024.0;
    ev.binary_file_mb = ev.binary_file_kb / 1024.0;
    ev.binary_file_gb = ev.binary_file_mb / 1024.0;
    ev.binary_file_tb = ev.binary_file_gb / 1024.0;
}

pub(crate) fn definir_descompactado(ev: &mut EventModel, b: f64) {
    ev.unzipped_b = b;
    ev.unzipped_kb = b / 1024.0;
    ev.unzipped_mb = ev.unzipped_kb / 1024.0;
    ev.unzipped_gb = ev.unzipped_mb / 1024.0;
    ev.unzipped_tb = ev.unzipped_gb / 1024.0;
}

/// Copia, de `origem` para `destino`, os grupos de metadados de conteúdo que estão vazios
/// no destino (apenas quando pertencem ao mesmo perfil de metadados).
pub fn completar_campos_conteudo(destino: &mut EventModel, origem: &EventModel) {
    if perfil_evento(destino) != perfil_evento(origem) {
        return;
    }
    macro_rules! completar {
        ($($campo:ident),+) => {
            $(
                if destino.$campo.is_empty() && !origem.$campo.is_empty() {
                    destino.$campo = origem.$campo.clone();
                }
            )+
        };
    }
    completar!(
        dimensoes,
        duracao,
        taxa_bits,
        paginas,
        linhas,
        palavras,
        paginas_estimadas,
        linhas_codigo,
        total_linhas,
        slides_estimados,
        arquivos,
        slides,
        planilhas,
        colunas,
        registros,
        tabelas
    );
    if destino.unzipped_b == 0.0 && origem.unzipped_b > 0.0 {
        definir_descompactado(destino, origem.unzipped_b);
    }
    if destino.binary_file_b == 0.0 && origem.binary_file_b > 0.0 {
        definir_binario(destino, origem.binary_file_b);
    }
}

/// Estimativas de texto/documento a partir do tamanho (arquivos vazios => 0 linhas, 1 página).
fn estimar_texto(
    sz: f64,
    bytes_por_linha: f64,
    palavras_por_linha: usize,
    linhas_por_pagina: f64,
) -> (usize, usize, usize) {
    let lines = if sz <= 0.0 {
        0
    } else {
        ((sz / bytes_por_linha).ceil() as usize).max(1)
    };
    let words = lines * palavras_por_linha;
    let pages = ((lines as f64 / linhas_por_pagina).ceil() as usize).max(1);
    (lines, words, pages)
}

fn sha256_fallback_metadados(ev: &EventModel) -> String {
    if ev.size_b <= 0.0 {
        return "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    }
    let mut hasher = Sha256::new();
    hasher.update(b"linceu-meta-sha256:");
    hasher.update(ev.nome.to_lowercase().as_bytes());
    hasher.update(b":");
    hasher.update((ev.size_b as u64).to_le_bytes());
    hex::encode(hasher.finalize())
}

fn preencher_estimativas(ev: &mut EventModel, p: &PerfilMetadados, ext: &str) {
    let sz = ev.size_b.max(0.0);

    // 1. Dimensões (imagens e vídeos)
    if p.dimensoes && ev.dimensoes.is_empty() {
        ev.dimensoes = if p.midia {
            "1280x720".to_string()
        } else if matches!(ext, "ico" | "cur" | "ani") {
            "32x32".to_string()
        } else if sz <= 0.0 {
            "16x16".to_string()
        } else {
            let side = ((sz / 3.0).sqrt().round() as u32).clamp(16, 4096);
            format!("{}x{}", side, side)
        };
    }

    // 2. Mídia (áudio e vídeo)
    if p.midia && (ev.duracao.is_empty() || ev.taxa_bits.is_empty()) {
        let kbps: u64 = if p.dimensoes { 2500 } else { 192 };
        let secs: u64 = if sz <= 0.0 {
            0
        } else {
            (((sz as u64) * 8) / (kbps * 1000)).max(1)
        };
        if ev.duracao.is_empty() {
            ev.duracao = formatar_duracao(secs);
        }
        if ev.taxa_bits.is_empty() {
            ev.taxa_bits = format!("{} kbps", if sz <= 0.0 { 0 } else { kbps });
        }
    }

    // 3. Código-fonte
    if p.linhas_codigo {
        let (est_lines, est_words, est_pages) = estimar_texto(sz, 40.0, 5, 50.0);
        if ev.linhas.is_empty() && ev.total_linhas.is_empty() {
            ev.linhas = est_lines.to_string();
            ev.total_linhas = est_lines.to_string();
        } else if ev.linhas.is_empty() {
            ev.linhas = ev.total_linhas.clone();
        } else if ev.total_linhas.is_empty() {
            ev.total_linhas = ev.linhas.clone();
        }
        let parsed_lines = ev.linhas.parse::<usize>().unwrap_or(est_lines);
        if ev.linhas_codigo.is_empty() {
            let code_lines = if parsed_lines == 0 {
                0
            } else {
                ((parsed_lines * 8) / 10).max(1)
            };
            ev.linhas_codigo = code_lines.to_string();
        }
        if ev.palavras.is_empty() {
            ev.palavras = (parsed_lines * 5).max(est_words.min(parsed_lines * 5)).to_string();
        }
        let pages_val = ((parsed_lines as f64 / 50.0).ceil() as usize)
            .max(1)
            .max(if parsed_lines == 0 { 1 } else { est_pages.min(1) });
        if ev.paginas.is_empty() {
            ev.paginas = pages_val.to_string();
        }
        if ev.paginas_estimadas.is_empty() {
            ev.paginas_estimadas = ev.paginas.clone();
        }
    }
    // 4. Planilhas
    else if p.planilhas {
        let rows = if sz <= 0.0 {
            0
        } else {
            ((sz / 64.0).ceil() as usize).max(1)
        };
        if ev.planilhas.is_empty() {
            ev.planilhas = "1".to_string();
        }
        if ev.tabelas.is_empty() {
            ev.tabelas = ev.planilhas.clone();
        }
        if ev.colunas.is_empty() {
            ev.colunas = "1".to_string();
        }
        if ev.linhas.is_empty() && ev.total_linhas.is_empty() {
            ev.linhas = rows.to_string();
            ev.total_linhas = rows.to_string();
        } else if ev.linhas.is_empty() {
            ev.linhas = ev.total_linhas.clone();
        } else if ev.total_linhas.is_empty() {
            ev.total_linhas = ev.linhas.clone();
        }
        let parsed_rows = ev.linhas.parse::<usize>().unwrap_or(rows);
        if ev.registros.is_empty() {
            ev.registros = parsed_rows.to_string();
        }
        if ev.paginas_estimadas.is_empty() {
            ev.paginas_estimadas = ((parsed_rows as f64 / 45.0).ceil() as usize)
                .max(1)
                .to_string();
        }
    }
    // 5. Apresentações
    else if p.slides {
        let est_slides = ((sz / 32768.0).ceil() as usize).max(1);
        if ev.slides.is_empty() && ev.slides_estimados.is_empty() {
            ev.slides = est_slides.to_string();
            ev.slides_estimados = est_slides.to_string();
        } else if ev.slides.is_empty() {
            ev.slides = ev.slides_estimados.clone();
        } else if ev.slides_estimados.is_empty() {
            ev.slides_estimados = ev.slides.clone();
        }
        let parsed_slides = ev.slides.parse::<usize>().unwrap_or(est_slides).max(1);
        if ev.paginas.is_empty() {
            ev.paginas = parsed_slides.to_string();
        }
        if ev.paginas_estimadas.is_empty() {
            ev.paginas_estimadas = ev.paginas.clone();
        }
        if ev.palavras.is_empty() {
            ev.palavras = (parsed_slides * 45).to_string();
        }
    }
    // 6. Bancos de dados e GIS
    else if p.tabelas {
        let est_records = if sz <= 0.0 {
            0
        } else {
            ((sz / 256.0).ceil() as usize).max(1)
        };
        if ev.tabelas.is_empty() {
            ev.tabelas = "1".to_string();
        }
        if ev.colunas.is_empty() {
            ev.colunas = "6".to_string();
        }
        if ev.registros.is_empty() {
            ev.registros = est_records.to_string();
        }
        if ev.linhas.is_empty() && ev.total_linhas.is_empty() {
            ev.linhas = ev.registros.clone();
            ev.total_linhas = ev.registros.clone();
        } else if ev.linhas.is_empty() {
            ev.linhas = ev.total_linhas.clone();
        } else if ev.total_linhas.is_empty() {
            ev.total_linhas = ev.linhas.clone();
        }
    }
    // 7. Documentos, logs, configurações e arquivos textuais
    else if p.paginas && p.linhas && p.palavras {
        let (est_lines, est_words, est_pages) = estimar_texto(sz, 55.0, 7, 45.0);
        if ev.linhas.is_empty() && ev.total_linhas.is_empty() {
            ev.linhas = est_lines.to_string();
            ev.total_linhas = est_lines.to_string();
        } else if ev.linhas.is_empty() {
            ev.linhas = ev.total_linhas.clone();
        } else if ev.total_linhas.is_empty() {
            ev.total_linhas = ev.linhas.clone();
        }
        let parsed_lines = ev.linhas.parse::<usize>().unwrap_or(est_lines);
        if ev.palavras.is_empty() {
            let words_val = if parsed_lines == 0 {
                0
            } else if est_lines > 0 {
                parsed_lines * 7
            } else {
                est_words
            };
            ev.palavras = words_val.to_string();
        }
        if ev.paginas.is_empty() && ev.paginas_estimadas.is_empty() {
            let pages_val = ((parsed_lines as f64 / 45.0).ceil() as usize)
                .max(1)
                .max(if parsed_lines == 0 { 1 } else { est_pages.min(1) });
            ev.paginas = pages_val.to_string();
            ev.paginas_estimadas = pages_val.to_string();
        } else if ev.paginas.is_empty() {
            ev.paginas = ev.paginas_estimadas.clone();
        } else if ev.paginas_estimadas.is_empty() {
            ev.paginas_estimadas = ev.paginas.clone();
        }
    }

    // 8. Compactados, discos virtuais e backups
    if p.arquivos && ev.arquivos.is_empty() {
        ev.arquivos = "1".to_string();
    }
    if p.descompactado && ev.unzipped_b == 0.0 && sz > 0.0 {
        definir_descompactado(ev, sz);
    }

    // 9. Executáveis, fontes, binários de sistema e binários genéricos
    if p.binario && ev.binary_file_b == 0.0 && sz > 0.0 {
        definir_binario(ev, sz);
    }

    // 10. Garante SHA-256 presente para todo arquivo (nunca para pastas)
    if !ev.nome.is_empty()
        && (ev.sha256_atual.is_empty() || ev.sha256_atual.starts_with("size_"))
    {
        ev.sha256_atual = sha256_fallback_metadados(ev);
    }
}

/// Aplica as regras de harmonização ao evento: remove colunas estranhas à categoria do
/// arquivo e completa as colunas da categoria que ficaram vazias.
pub fn harmonizar_metadados(ev: &mut EventModel) {
    if ev.size_b == 0.0 && ev.size_kb > 0.0 {
        ev.size_b = ev.size_kb * 1024.0;
        ev.size_mb = ev.size_kb / 1024.0;
        ev.size_gb = ev.size_mb / 1024.0;
        ev.size_tb = ev.size_gb / 1024.0;
    } else if ev.size_b > 0.0 && ev.size_kb == 0.0 {
        ev.size_kb = ev.size_b / 1024.0;
        ev.size_mb = ev.size_kb / 1024.0;
        ev.size_gb = ev.size_mb / 1024.0;
        ev.size_tb = ev.size_gb / 1024.0;
    }

    let p = perfil_evento(ev);

    if p.pasta {
        let arquivos = std::mem::take(&mut ev.arquivos);
        limpar_campos_conteudo(ev);
        ev.arquivos = if arquivos.trim().is_empty() {
            "0".to_string()
        } else {
            arquivos
        };
        ev.sha256_atual.clear();
        ev.tipo = "folder".to_string();
        ev.mime = "inode/directory".to_string();
        return;
    }

    let attr_lower = ev.atributos.trim().to_lowercase();
    if attr_lower == "folder" || attr_lower == "pasta" || attr_lower == "diretório" {
        ev.atributos = "file".to_string();
    }

    if !p.dimensoes {
        ev.dimensoes.clear();
    }
    if !p.midia {
        ev.duracao.clear();
        ev.taxa_bits.clear();
    }
    if !p.paginas {
        ev.paginas.clear();
    }
    if !p.linhas {
        ev.linhas.clear();
        ev.total_linhas.clear();
    }
    if !p.palavras {
        ev.palavras.clear();
    }
    if !p.paginas_estimadas {
        ev.paginas_estimadas.clear();
    }
    if !p.linhas_codigo {
        ev.linhas_codigo.clear();
    }
    if !p.slides {
        ev.slides.clear();
        ev.slides_estimados.clear();
    }
    if !p.planilhas {
        ev.planilhas.clear();
    }
    if !p.tabelas {
        ev.colunas.clear();
        ev.registros.clear();
        ev.tabelas.clear();
    }
    if !p.arquivos {
        ev.arquivos.clear();
    }
    if !p.descompactado {
        zerar_descompactado(ev);
    }
    if !p.binario {
        zerar_binario(ev);
    }

    let ext = extensao_efetiva(Path::new(&ev.nome), &ev.tipo);
    preencher_estimativas(ev, &p, &ext);
}
