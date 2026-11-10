use parking_lot::Mutex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use super::db_writer::DbWriter;
use super::metadados::{
    evento_pasta_inconsistente, extensao_efetiva, extensao_tipo_preferencial,
    extrair_metadados_completos, harmonizar_metadados, obter_evento_cacheado,
    obter_mime_por_extensao, obter_tipo_e_mime, perfil_evento, preencher_metadados_item_excluido,
    salvar_evento_cache, tipo_eh_pasta,
};
use crate::core::event::EventModel;
use crate::core::types::OperationType;
use crate::security::{CorrelacaoSessoes, DiffConteudo, ExportacaoEvidencia, ScoreRisco};
use crate::utils::paths::obter_caminho_db_eventos;

fn perfil_precisa_extracao(ev: &EventModel) -> bool {
    let p = perfil_evento(ev);
    if p.pasta {
        return ev.arquivos.is_empty();
    }
    if p.dimensoes && ev.dimensoes.is_empty() {
        return true;
    }
    if p.midia && (ev.duracao.is_empty() || ev.taxa_bits.is_empty()) {
        return true;
    }
    if p.paginas && ev.paginas.is_empty() {
        return true;
    }
    if p.linhas && ev.linhas.is_empty() {
        return true;
    }
    if p.palavras && ev.palavras.is_empty() {
        return true;
    }
    if p.linhas_codigo && ev.linhas_codigo.is_empty() {
        return true;
    }
    if p.slides && ev.slides.is_empty() {
        return true;
    }
    if p.planilhas && ev.planilhas.is_empty() {
        return true;
    }
    if p.tabelas && (ev.tabelas.is_empty() || ev.colunas.is_empty() || ev.registros.is_empty()) {
        return true;
    }
    if p.arquivos && ev.arquivos.is_empty() {
        return true;
    }
    if p.descompactado && ev.unzipped_b == 0.0 && ev.size_b > 0.0 {
        return true;
    }
    if p.binario && ev.binary_file_b == 0.0 && ev.size_b > 0.0 {
        return true;
    }
    false
}

fn usuario_padrao_sistema() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Sistema".to_string())
}

fn preencher_seguranca_padrao_em_memoria(ev: &mut EventModel) {
    if ev.monitor_profile_name.is_empty() {
        ev.monitor_profile_name = Path::new(&ev.dir_atual)
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Monitoramento Local")
            .to_string();
    }
    if ev.baseline_status.is_empty() {
        ev.baseline_status = match ev.op_type() {
            OperationType::Escaneado => "verified".to_string(),
            OperationType::Excluido => "removed".to_string(),
            OperationType::Modificado => "changed".to_string(),
            _ => "new".to_string(),
        };
    }
    if ev.session_label.is_empty() {
        CorrelacaoSessoes::atribuir_sessao_evento(ev);
    }
    if ev.diff_resumo.is_empty() {
        ev.diff_resumo = DiffConteudo::calcular_diff_evento(ev, None, None);
    }
    if ev.quarantine_status.is_empty() {
        ev.quarantine_status = "not_applicable".to_string();
    }
    if ev.evidence_id.is_empty() {
        ExportacaoEvidencia::atribuir_evidencia_evento(ev);
    }
    if ev.publisher.is_empty() && !ev.autor.is_empty() {
        ev.publisher = ev.autor.clone();
    }
    if ev.actor_user.is_empty() && !ev.autor.is_empty() {
        ev.actor_user = ev.autor.clone();
    }
    if ev.actor_process.is_empty() {
        ev.actor_process = "Huginn Sentinel (Kernel Monitor)".to_string();
    }
    if ev.alerta_resumo.is_empty() || ev.risk_level.is_empty() {
        ScoreRisco::avaliar_evento(ev);
    }
}

/// Normalização ultrarrápida exclusivamente em memória para eventos carregados do SQLite.
///
/// Jamais realiza chamadas de sistema ao disco (`exists`, `is_dir`, abertura de arquivos ou
/// recálculo de SHA-256 em disco), permitindo carregar dezenas de milhares de registros em
/// poucos milissegundos sem bloquear a inicialização da janela principal.
fn normalizar_evento_do_banco(ev: &mut EventModel, default_user: &str) {
    if evento_pasta_inconsistente(ev) {
        let eh_texto = !ev.linhas.is_empty() || !ev.palavras.is_empty();
        ev.tipo = if eh_texto {
            "text".to_string()
        } else {
            "binary".to_string()
        };
        ev.mime = if eh_texto {
            "text/plain".to_string()
        } else {
            "application/octet-stream".to_string()
        };
        if ev.atributos.contains("folder")
            || ev.atributos.contains("Diretório")
            || ev.atributos.is_empty()
        {
            ev.atributos = "file".to_string();
        }
    }

    let is_dir = tipo_eh_pasta(&ev.tipo, &ev.mime)
        || ev.atributos.contains("Diretório")
        || ev.atributos.contains("folder");

    if is_dir {
        ev.tipo = "folder".to_string();
        ev.mime = "inode/directory".to_string();
    } else {
        let path_nome = Path::new(&ev.nome);
        let ext = extensao_tipo_preferencial(path_nome);
        if !ext.is_empty() {
            ev.tipo = format!(".{}", ext.trim_start_matches('.'));
            if ev.mime.is_empty() || ev.mime == "application/octet-stream" {
                ev.mime = obter_mime_por_extensao(&ext, Path::new(""));
            }
        } else if ev.tipo.is_empty() {
            let lower = ev.nome.to_lowercase();
            let (cat, m) = match lower.as_str() {
                "makefile" | "gnumakefile" | "dockerfile" | "containerfile" | "jenkinsfile"
                | "vagrantfile" | "procfile" | "rakefile" | "gemfile" | "podfile" | "brewfile"
                | "justfile" | "cmakelists.txt" | "build" | "workspace" | "sconstruct" => {
                    ("code", "text/plain")
                }
                "readme" | "license" | "licence" | "copying" | "changelog" | "authors"
                | "news" | "todo" | "notice" | "patents" | "contributing" | "maintainers" => {
                    ("document", "text/plain")
                }
                "hosts" | "fstab" | "crypttab" | "mtab" | "sudoers" | "crontab" => {
                    ("config", "text/plain")
                }
                _ => ("file", "application/octet-stream"),
            };
            ev.tipo = cat.to_string();
            if ev.mime.is_empty() {
                ev.mime = m.to_string();
            }
        } else if ev.mime.is_empty() {
            let ext_ef = extensao_efetiva(path_nome, &ev.tipo);
            ev.mime = if !ext_ef.is_empty() {
                obter_mime_por_extensao(&ext_ef, Path::new(""))
            } else if matches!(ev.tipo.as_str(), "text" | "code" | "document" | "config") {
                "text/plain".to_string()
            } else {
                "application/octet-stream".to_string()
            };
        }
    }

    if ev.op_type() == OperationType::Excluido
        && ev.dir_anterior.is_empty()
        && !ev.dir_atual.is_empty()
    {
        ev.dir_anterior = ev.dir_atual.clone();
    }

    if ev.data_modificacao.is_empty() && !ev.timestamp.is_empty() {
        ev.data_modificacao = ev.timestamp.clone();
    }
    if ev.data_criacao.is_empty() && !ev.data_modificacao.is_empty() {
        ev.data_criacao = ev.data_modificacao.clone();
    }
    if ev.data_acesso.is_empty() && !ev.data_modificacao.is_empty() {
        ev.data_acesso = ev.data_modificacao.clone();
    }
    if ev.atributos.is_empty() {
        ev.atributos = if is_dir {
            "Diretório".to_string()
        } else {
            "file".to_string()
        };
    }
    if ev.protegido.is_empty() {
        ev.protegido = "Não".to_string();
    }
    if ev.autor.is_empty() {
        ev.autor = if !ev.actor_user.is_empty() {
            ev.actor_user.clone()
        } else {
            default_user.to_string()
        };
    }

    harmonizar_metadados(ev);
    preencher_seguranca_padrao_em_memoria(ev);
}

fn normalizar_evento_carregado(ev: &mut EventModel) {
    // Repara registros históricos corrompidos em que arquivos sem extensão foram gravados como `folder`
    if evento_pasta_inconsistente(ev) {
        let eh_texto = !ev.linhas.is_empty() || !ev.palavras.is_empty();
        ev.tipo = if eh_texto {
            "text".to_string()
        } else {
            "binary".to_string()
        };
        ev.mime = if eh_texto {
            "text/plain".to_string()
        } else {
            "application/octet-stream".to_string()
        };
        if ev.atributos.contains("folder")
            || ev.atributos.contains("Diretório")
            || ev.atributos.is_empty()
        {
            ev.atributos = "file".to_string();
        }
    }

    let full = ev.full_path();
    let full_path = Path::new(&full);

    let is_dir = tipo_eh_pasta(&ev.tipo, &ev.mime)
        || ev.atributos.contains("Diretório")
        || ev.atributos.contains("folder")
        || (full_path.exists() && full_path.is_dir());

    if is_dir {
        ev.tipo = "folder".to_string();
        ev.mime = "inode/directory".to_string();
    } else {
        let tem_ext = !extensao_tipo_preferencial(full_path).is_empty();
        if (full_path.exists() && ev.op_type() != OperationType::Excluido)
            || tem_ext
            || ev.tipo.is_empty()
        {
            let (t, m) = obter_tipo_e_mime(full_path, false);
            let preservavel = !tem_ext
                && t == "binary"
                && !ev.tipo.is_empty()
                && ev.tipo != "file"
                && !tipo_eh_pasta(&ev.tipo, &ev.mime);
            if !preservavel {
                ev.tipo = t;
                if ev.mime.is_empty() || ev.mime == "application/octet-stream" {
                    ev.mime = m;
                }
            }
        } else if ev.mime.is_empty() {
            let (_, m) = obter_tipo_e_mime(full_path, false);
            ev.mime = m;
        }
    }

    if ev.op_type() == OperationType::Excluido && ev.dir_anterior.is_empty() && !ev.dir_atual.is_empty() {
        ev.dir_anterior = ev.dir_atual.clone();
    }

    // Se o evento ao vivo estiver com metadados essenciais ou específicos da categoria ausentes,
    // reconstrói a partir do arquivo em disco (se existir) ou por inferência/cache.
    if !full.is_empty()
        && (ev.autor.is_empty()
            || ev.protegido.is_empty()
            || ev.atributos.is_empty()
            || ev.data_modificacao.is_empty()
            || (!is_dir && ev.sha256_atual.is_empty())
            || perfil_precisa_extracao(ev))
    {
        if full_path.exists() && ev.op_type() != OperationType::Excluido {
            extrair_metadados_completos(full_path, ev);
        } else {
            preencher_metadados_item_excluido(full_path, ev);
        }
    }

    // Garante harmonização estrita de todas as colunas da categoria do arquivo/pasta
    harmonizar_metadados(ev);
    preencher_seguranca_padrao_em_memoria(ev);

    if !full.is_empty() {
        salvar_evento_cache(full_path, ev);
    }
}

fn restaurar_campos_ausentes_de_anterior(alvo: &mut EventModel, ant: &EventModel) {
    if evento_pasta_inconsistente(ant) {
        return;
    }

    let eh_excluido = alvo.op_type() == OperationType::Excluido;

    if alvo.dir_anterior.is_empty() && !ant.dir_atual.is_empty() {
        alvo.dir_anterior = ant.dir_atual.clone();
    }
    if (alvo.size_b == 0.0 || eh_excluido) && ant.size_b > 0.0 {
        alvo.size_b = ant.size_b;
        alvo.size_kb = ant.size_kb;
        alvo.size_mb = ant.size_mb;
        alvo.size_gb = ant.size_gb;
        alvo.size_tb = ant.size_tb;
    }

    let alvo_ext = extensao_tipo_preferencial(Path::new(&alvo.nome));
    if alvo_ext.is_empty() && !ant.tipo.is_empty() {
        if eh_excluido
            || alvo.tipo.is_empty()
            || alvo.tipo == "file"
            || (alvo.nome.eq_ignore_ascii_case(&ant.nome)
                && tipo_eh_pasta(&alvo.tipo, &alvo.mime)
                && !tipo_eh_pasta(&ant.tipo, &ant.mime))
        {
            alvo.tipo = ant.tipo.clone();
            if !ant.mime.is_empty() {
                alvo.mime = ant.mime.clone();
            }
        }
    } else if !alvo_ext.is_empty() && alvo.tipo.is_empty() {
        let (t, m) = obter_tipo_e_mime(Path::new(&alvo.nome), false);
        alvo.tipo = t;
        if alvo.mime.is_empty() {
            alvo.mime = m;
        }
    }

    if (alvo.data_criacao.is_empty() || eh_excluido) && !ant.data_criacao.is_empty() {
        alvo.data_criacao = ant.data_criacao.clone();
    }
    if (alvo.data_modificacao.is_empty() || eh_excluido) && !ant.data_modificacao.is_empty() {
        alvo.data_modificacao = ant.data_modificacao.clone();
    }
    if (alvo.data_acesso.is_empty() || eh_excluido) && !ant.data_acesso.is_empty() {
        alvo.data_acesso = ant.data_acesso.clone();
    }
    if (alvo.atributos.is_empty() || alvo.atributos == "file" || eh_excluido)
        && !ant.atributos.is_empty()
    {
        if !(!tipo_eh_pasta(&alvo.tipo, &alvo.mime)
            && (ant.atributos.contains("folder") || ant.atributos.contains("Diretório")))
        {
            alvo.atributos = ant.atributos.clone();
        }
    }
    if (alvo.autor.is_empty() || eh_excluido) && !ant.autor.is_empty() {
        alvo.autor = ant.autor.clone();
    }
    if (alvo.protegido.is_empty() || eh_excluido) && !ant.protegido.is_empty() {
        alvo.protegido = ant.protegido.clone();
    }
    if (alvo.sha256_atual.is_empty() || eh_excluido)
        && !ant.sha256_atual.is_empty()
        && !ant.sha256_atual.starts_with("size_")
    {
        alvo.sha256_atual = ant.sha256_atual.clone();
    }
    if (alvo.publisher.is_empty() || eh_excluido) && !ant.publisher.is_empty() {
        alvo.publisher = ant.publisher.clone();
    }

    // Metadados específicos de conteúdo só podem ser herdados se o perfil da categoria for idêntico
    // (impede que renomear `.tmp -> .txt` ou `.tmp -> .old` contamine colunas da nova extensão).
    if perfil_evento(alvo) != perfil_evento(ant) {
        return;
    }

    if (alvo.dimensoes.is_empty() || eh_excluido) && !ant.dimensoes.is_empty() {
        alvo.dimensoes = ant.dimensoes.clone();
    }
    if (alvo.duracao.is_empty() || eh_excluido) && !ant.duracao.is_empty() {
        alvo.duracao = ant.duracao.clone();
    }
    if (alvo.taxa_bits.is_empty() || eh_excluido) && !ant.taxa_bits.is_empty() {
        alvo.taxa_bits = ant.taxa_bits.clone();
    }
    if (alvo.paginas.is_empty() || eh_excluido) && !ant.paginas.is_empty() {
        alvo.paginas = ant.paginas.clone();
    }
    if (alvo.linhas.is_empty() || eh_excluido) && !ant.linhas.is_empty() {
        alvo.linhas = ant.linhas.clone();
    }
    if (alvo.palavras.is_empty() || eh_excluido) && !ant.palavras.is_empty() {
        alvo.palavras = ant.palavras.clone();
    }
    if (alvo.paginas_estimadas.is_empty() || eh_excluido) && !ant.paginas_estimadas.is_empty() {
        alvo.paginas_estimadas = ant.paginas_estimadas.clone();
    }
    if (alvo.linhas_codigo.is_empty() || eh_excluido) && !ant.linhas_codigo.is_empty() {
        alvo.linhas_codigo = ant.linhas_codigo.clone();
    }
    if (alvo.total_linhas.is_empty() || eh_excluido) && !ant.total_linhas.is_empty() {
        alvo.total_linhas = ant.total_linhas.clone();
    }
    if (alvo.slides_estimados.is_empty() || eh_excluido) && !ant.slides_estimados.is_empty() {
        alvo.slides_estimados = ant.slides_estimados.clone();
    }
    if (alvo.arquivos.is_empty() || eh_excluido) && !ant.arquivos.is_empty() {
        alvo.arquivos = ant.arquivos.clone();
    }
    if (alvo.unzipped_b == 0.0 || eh_excluido) && ant.unzipped_b > 0.0 {
        alvo.unzipped_b = ant.unzipped_b;
        alvo.unzipped_kb = ant.unzipped_kb;
        alvo.unzipped_mb = ant.unzipped_mb;
        alvo.unzipped_gb = ant.unzipped_gb;
        alvo.unzipped_tb = ant.unzipped_tb;
    }
    if (alvo.slides.is_empty() || eh_excluido) && !ant.slides.is_empty() {
        alvo.slides = ant.slides.clone();
    }
    if (alvo.binary_file_b == 0.0 || eh_excluido) && ant.binary_file_b > 0.0 {
        alvo.binary_file_b = ant.binary_file_b;
        alvo.binary_file_kb = ant.binary_file_kb;
        alvo.binary_file_mb = ant.binary_file_mb;
        alvo.binary_file_gb = ant.binary_file_gb;
        alvo.binary_file_tb = ant.binary_file_tb;
    }
    if (alvo.planilhas.is_empty() || eh_excluido) && !ant.planilhas.is_empty() {
        alvo.planilhas = ant.planilhas.clone();
    }
    if (alvo.colunas.is_empty() || eh_excluido) && !ant.colunas.is_empty() {
        alvo.colunas = ant.colunas.clone();
    }
    if (alvo.registros.is_empty() || eh_excluido) && !ant.registros.is_empty() {
        alvo.registros = ant.registros.clone();
    }
    if (alvo.tabelas.is_empty() || eh_excluido) && !ant.tabelas.is_empty() {
        alvo.tabelas = ant.tabelas.clone();
    }
}

struct DeletedCandidate {
    event: EventModel,
    timestamp: Instant,
}

pub struct BaseEvento {
    db_path: PathBuf,
    writer: Arc<DbWriter>,
    pending_deletions: Mutex<HashMap<String, DeletedCandidate>>, // key: nome ou size_hash
}

impl BaseEvento {
    pub fn new(db_path_opt: Option<PathBuf>) -> Self {
        let db_path = db_path_opt
            .unwrap_or_else(obter_caminho_db_eventos);

        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let writer = Arc::new(DbWriter::new(db_path.clone()));

        Self {
            db_path,
            writer,
            pending_deletions: Mutex::new(HashMap::new()),
        }
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn writer(&self) -> Arc<DbWriter> {
        self.writer.clone()
    }

    pub fn flush(&self) {
        self.writer.flush();
    }

    pub fn registrar_evento(&self, mut evento: EventModel) -> EventModel {
        let op = evento.op_type();

        // Correlação de eventos de movimentação/renomeação e restauração de metadados
        match op {
            OperationType::Excluido => {
                if evento.dir_anterior.is_empty() && !evento.dir_atual.is_empty() {
                    evento.dir_anterior = evento.dir_atual.clone();
                }
                if let Some(ant) =
                    self.buscar_metadados_anteriores(&evento.nome, &evento.dir_atual)
                {
                    restaurar_campos_ausentes_de_anterior(&mut evento, &ant);
                }
                let mut lock = self.pending_deletions.lock();
                // Limpeza preventiva de exclusões expiradas (> 3 segundos)
                lock.retain(|_, cand| cand.timestamp.elapsed().as_secs() <= 3);
                let key = evento.nome.clone();
                lock.insert(
                    key,
                    DeletedCandidate {
                        event: evento.clone(),
                        timestamp: Instant::now(),
                    },
                );
            }
            OperationType::Adicionado => {
                let mut lock = self.pending_deletions.lock();
                // Limpa expiradas antes de buscar
                lock.retain(|_, cand| cand.timestamp.elapsed().as_secs() <= 3);

                let mut found_key = None;
                for (cand_key, cand) in lock.iter() {
                    if cand.event.nome.eq_ignore_ascii_case(&evento.nome)
                        && !cand.event.dir_atual.eq_ignore_ascii_case(&evento.dir_atual)
                    {
                        found_key = Some(cand_key.clone());
                        break;
                    }
                }

                if let Some(key) = found_key {
                    if let Some(prev) = lock.remove(&key) {
                        evento.tipo_operacao = OperationType::Movido.display_pt().to_string();
                        evento.dir_anterior = prev.event.dir_atual.clone();
                        restaurar_campos_ausentes_de_anterior(&mut evento, &prev.event);
                    }
                }
            }
            OperationType::Renomeado | OperationType::Movido => {
                let mut prev_cand = {
                    let mut lock = self.pending_deletions.lock();
                    lock.retain(|_, cand| cand.timestamp.elapsed().as_secs() <= 3);
                    lock.remove(&evento.nome).or_else(|| {
                        let old_name = Path::new(&evento.dir_anterior)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default();
                        if !old_name.is_empty() {
                            lock.remove(old_name)
                        } else {
                            None
                        }
                    })
                };
                if let Some(prev) = prev_cand.take() {
                    restaurar_campos_ausentes_de_anterior(&mut evento, &prev.event);
                } else if evento.size_b == 0.0 || perfil_precisa_extracao(&evento) {
                    if let Some(ant) = self.buscar_origem_rename_move(&evento) {
                        restaurar_campos_ausentes_de_anterior(&mut evento, &ant);
                    }
                }
            }
            _ => {}
        }

        normalizar_evento_carregado(&mut evento);

        let tabela = match evento.op_type() {
            OperationType::Adicionado => "adicionado",
            OperationType::Modificado => "modificado",
            OperationType::Excluido => "excluido",
            OperationType::Renomeado => "renomeado",
            OperationType::Movido => "movido",
            OperationType::Escaneado => "escaneado",
        };

        self.writer.insert_event("monitoramento", evento.clone());
        self.writer.insert_event(tabela, evento.clone());

        evento
    }

    pub fn registrar_eventos_lote(&self, mut eventos: Vec<EventModel>) -> Vec<EventModel> {
        if eventos.is_empty() {
            return Vec::new();
        }

        {
            let mut lock = self.pending_deletions.lock();
            lock.retain(|_, cand| cand.timestamp.elapsed().as_secs() <= 3);

            for evento in &mut eventos {
                match evento.op_type() {
                    OperationType::Excluido => {
                        if evento.dir_anterior.is_empty() && !evento.dir_atual.is_empty() {
                            evento.dir_anterior = evento.dir_atual.clone();
                        }
                        if let Some(ant) =
                            self.buscar_metadados_anteriores(&evento.nome, &evento.dir_atual)
                        {
                            restaurar_campos_ausentes_de_anterior(evento, &ant);
                        }
                        let key = evento.nome.clone();
                        lock.insert(
                            key,
                            DeletedCandidate {
                                event: evento.clone(),
                                timestamp: Instant::now(),
                            },
                        );
                    }
                    OperationType::Adicionado => {
                        let mut found_key = None;
                        for (cand_key, cand) in lock.iter() {
                            if cand.event.nome.eq_ignore_ascii_case(&evento.nome)
                                && !cand.event.dir_atual.eq_ignore_ascii_case(&evento.dir_atual)
                            {
                                found_key = Some(cand_key.clone());
                                break;
                            }
                        }
                        if let Some(key) = found_key {
                            if let Some(prev) = lock.remove(&key) {
                                evento.tipo_operacao =
                                    OperationType::Movido.display_pt().to_string();
                                evento.dir_anterior = prev.event.dir_atual.clone();
                                restaurar_campos_ausentes_de_anterior(evento, &prev.event);
                            }
                        }
                    }
                    OperationType::Renomeado | OperationType::Movido => {
                        let prev_cand = lock.remove(&evento.nome).or_else(|| {
                            let old_name = Path::new(&evento.dir_anterior)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or_default();
                            if !old_name.is_empty() {
                                lock.remove(old_name)
                            } else {
                                None
                            }
                        });
                        if let Some(prev) = prev_cand {
                            restaurar_campos_ausentes_de_anterior(evento, &prev.event);
                        } else if evento.size_b == 0.0 || perfil_precisa_extracao(evento) {
                            if let Some(ant) = self.buscar_origem_rename_move(evento) {
                                restaurar_campos_ausentes_de_anterior(evento, &ant);
                            }
                        }
                    }
                    _ => {}
                }
                normalizar_evento_carregado(evento);
            }
        }

        self.writer.insert_batch("monitoramento", eventos.clone());

        let mut por_tabela: HashMap<&'static str, Vec<EventModel>> = HashMap::new();
        for ev in &eventos {
            let tab = match ev.op_type() {
                OperationType::Adicionado => "adicionado",
                OperationType::Modificado => "modificado",
                OperationType::Excluido => "excluido",
                OperationType::Renomeado => "renomeado",
                OperationType::Movido => "movido",
                OperationType::Escaneado => "escaneado",
            };
            por_tabela.entry(tab).or_default().push(ev.clone());
        }

        for (tab, lista) in por_tabela {
            self.writer.insert_batch(tab, lista);
        }

        eventos
    }

    pub fn carregar_eventos_recentes(&self, limite: usize) -> Vec<EventModel> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Vec::new();
        };

        let sql = format!(
            "SELECT id, tipo_operacao, nome, dir_anterior, dir_atual, data_criacao, data_modificacao,
                    data_acesso, tipo, mime, size_b, size_kb, size_mb, size_gb, size_tb, atributos,
                    autor, dimensoes, duracao, taxa_bits, protegido, paginas, linhas, palavras,
                    paginas_estimadas, linhas_codigo, total_linhas, slides_estimados, arquivos,
                    unzipped_b, unzipped_kb, unzipped_mb, unzipped_gb, unzipped_tb, slides,
                    binary_file_b, binary_file_kb, binary_file_mb, binary_file_gb, binary_file_tb,
                    planilhas, colunas, registros, tabelas, timestamp, monitor_profile_name,
                    sha256_atual, baseline_status, alerta_resumo, session_label, diff_resumo,
                    risk_score, risk_level, actor_user, actor_process, publisher, quarantine_status,
                    evidence_id
             FROM monitoramento ORDER BY id DESC LIMIT {limite}"
        );

        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
            Ok(EventModel {
                id: row.get(0).ok(),
                tipo_operacao: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                nome: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                dir_anterior: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                dir_atual: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                data_criacao: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                data_modificacao: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                data_acesso: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                tipo: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                mime: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                size_b: row.get::<_, Option<f64>>(10)?.unwrap_or_default(),
                size_kb: row.get::<_, Option<f64>>(11)?.unwrap_or_default(),
                size_mb: row.get::<_, Option<f64>>(12)?.unwrap_or_default(),
                size_gb: row.get::<_, Option<f64>>(13)?.unwrap_or_default(),
                size_tb: row.get::<_, Option<f64>>(14)?.unwrap_or_default(),
                atributos: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                autor: row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                dimensoes: row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                duracao: row.get::<_, Option<String>>(18)?.unwrap_or_default(),
                taxa_bits: row.get::<_, Option<String>>(19)?.unwrap_or_default(),
                protegido: row.get::<_, Option<String>>(20)?.unwrap_or_default(),
                paginas: row.get::<_, Option<String>>(21)?.unwrap_or_default(),
                linhas: row.get::<_, Option<String>>(22)?.unwrap_or_default(),
                palavras: row.get::<_, Option<String>>(23)?.unwrap_or_default(),
                paginas_estimadas: row.get::<_, Option<String>>(24)?.unwrap_or_default(),
                linhas_codigo: row.get::<_, Option<String>>(25)?.unwrap_or_default(),
                total_linhas: row.get::<_, Option<String>>(26)?.unwrap_or_default(),
                slides_estimados: row.get::<_, Option<String>>(27)?.unwrap_or_default(),
                arquivos: row.get::<_, Option<String>>(28)?.unwrap_or_default(),
                unzipped_b: row.get::<_, Option<f64>>(29)?.unwrap_or_default(),
                unzipped_kb: row.get::<_, Option<f64>>(30)?.unwrap_or_default(),
                unzipped_mb: row.get::<_, Option<f64>>(31)?.unwrap_or_default(),
                unzipped_gb: row.get::<_, Option<f64>>(32)?.unwrap_or_default(),
                unzipped_tb: row.get::<_, Option<f64>>(33)?.unwrap_or_default(),
                slides: row.get::<_, Option<String>>(34)?.unwrap_or_default(),
                binary_file_b: row.get::<_, Option<f64>>(35)?.unwrap_or_default(),
                binary_file_kb: row.get::<_, Option<f64>>(36)?.unwrap_or_default(),
                binary_file_mb: row.get::<_, Option<f64>>(37)?.unwrap_or_default(),
                binary_file_gb: row.get::<_, Option<f64>>(38)?.unwrap_or_default(),
                binary_file_tb: row.get::<_, Option<f64>>(39)?.unwrap_or_default(),
                planilhas: row.get::<_, Option<String>>(40)?.unwrap_or_default(),
                colunas: row.get::<_, Option<String>>(41)?.unwrap_or_default(),
                registros: row.get::<_, Option<String>>(42)?.unwrap_or_default(),
                tabelas: row.get::<_, Option<String>>(43)?.unwrap_or_default(),
                timestamp: row.get::<_, Option<String>>(44)?.unwrap_or_default(),
                monitor_profile_name: row.get::<_, Option<String>>(45)?.unwrap_or_default(),
                sha256_atual: row.get::<_, Option<String>>(46)?.unwrap_or_default(),
                baseline_status: row.get::<_, Option<String>>(47)?.unwrap_or_default(),
                alerta_resumo: row.get::<_, Option<String>>(48)?.unwrap_or_default(),
                session_label: row.get::<_, Option<String>>(49)?.unwrap_or_default(),
                diff_resumo: row.get::<_, Option<String>>(50)?.unwrap_or_default(),
                risk_score: row.get::<_, Option<i32>>(51)?.unwrap_or_default(),
                risk_level: row.get::<_, Option<String>>(52)?.unwrap_or_default(),
                actor_user: row.get::<_, Option<String>>(53)?.unwrap_or_default(),
                actor_process: row.get::<_, Option<String>>(54)?.unwrap_or_default(),
                publisher: row.get::<_, Option<String>>(55)?.unwrap_or_default(),
                quarantine_status: row.get::<_, Option<String>>(56)?.unwrap_or_default(),
                evidence_id: row.get::<_, Option<String>>(57)?.unwrap_or_default(),
                ..Default::default()
            })
        });

        match rows {
            Ok(it) => {
                let default_user = usuario_padrao_sistema();
                let mut vec: Vec<EventModel> = it.filter_map(Result::ok).collect();
                for ev in &mut vec {
                    normalizar_evento_do_banco(ev, &default_user);
                }
                for ev in vec.iter().take(2048) {
                    let full = ev.full_path();
                    if !full.is_empty() {
                        salvar_evento_cache(Path::new(&full), ev);
                    }
                }
                vec
            }
            Err(_) => Vec::new(),
        }
    }

    /// Carrega absolutamente todos os eventos registrados no banco SQLite sem qualquer limite
    pub fn carregar_todos_eventos(&self) -> Vec<EventModel> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Vec::new();
        };

        let sql = "SELECT id, tipo_operacao, nome, dir_anterior, dir_atual, data_criacao, data_modificacao,
                    data_acesso, tipo, mime, size_b, size_kb, size_mb, size_gb, size_tb, atributos,
                    autor, dimensoes, duracao, taxa_bits, protegido, paginas, linhas, palavras,
                    paginas_estimadas, linhas_codigo, total_linhas, slides_estimados, arquivos,
                    unzipped_b, unzipped_kb, unzipped_mb, unzipped_gb, unzipped_tb, slides,
                    binary_file_b, binary_file_kb, binary_file_mb, binary_file_gb, binary_file_tb,
                    planilhas, colunas, registros, tabelas, timestamp, monitor_profile_name,
                    sha256_atual, baseline_status, alerta_resumo, session_label, diff_resumo,
                    risk_score, risk_level, actor_user, actor_process, publisher, quarantine_status,
                    evidence_id
             FROM monitoramento ORDER BY id ASC";

        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
            Ok(EventModel {
                id: row.get(0).ok(),
                tipo_operacao: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                nome: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                dir_anterior: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                dir_atual: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                data_criacao: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                data_modificacao: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                data_acesso: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                tipo: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                mime: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                size_b: row.get::<_, Option<f64>>(10)?.unwrap_or_default(),
                size_kb: row.get::<_, Option<f64>>(11)?.unwrap_or_default(),
                size_mb: row.get::<_, Option<f64>>(12)?.unwrap_or_default(),
                size_gb: row.get::<_, Option<f64>>(13)?.unwrap_or_default(),
                size_tb: row.get::<_, Option<f64>>(14)?.unwrap_or_default(),
                atributos: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                autor: row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                dimensoes: row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                duracao: row.get::<_, Option<String>>(18)?.unwrap_or_default(),
                taxa_bits: row.get::<_, Option<String>>(19)?.unwrap_or_default(),
                protegido: row.get::<_, Option<String>>(20)?.unwrap_or_default(),
                paginas: row.get::<_, Option<String>>(21)?.unwrap_or_default(),
                linhas: row.get::<_, Option<String>>(22)?.unwrap_or_default(),
                palavras: row.get::<_, Option<String>>(23)?.unwrap_or_default(),
                paginas_estimadas: row.get::<_, Option<String>>(24)?.unwrap_or_default(),
                linhas_codigo: row.get::<_, Option<String>>(25)?.unwrap_or_default(),
                total_linhas: row.get::<_, Option<String>>(26)?.unwrap_or_default(),
                slides_estimados: row.get::<_, Option<String>>(27)?.unwrap_or_default(),
                arquivos: row.get::<_, Option<String>>(28)?.unwrap_or_default(),
                unzipped_b: row.get::<_, Option<f64>>(29)?.unwrap_or_default(),
                unzipped_kb: row.get::<_, Option<f64>>(30)?.unwrap_or_default(),
                unzipped_mb: row.get::<_, Option<f64>>(31)?.unwrap_or_default(),
                unzipped_gb: row.get::<_, Option<f64>>(32)?.unwrap_or_default(),
                unzipped_tb: row.get::<_, Option<f64>>(33)?.unwrap_or_default(),
                slides: row.get::<_, Option<String>>(34)?.unwrap_or_default(),
                binary_file_b: row.get::<_, Option<f64>>(35)?.unwrap_or_default(),
                binary_file_kb: row.get::<_, Option<f64>>(36)?.unwrap_or_default(),
                binary_file_mb: row.get::<_, Option<f64>>(37)?.unwrap_or_default(),
                binary_file_gb: row.get::<_, Option<f64>>(38)?.unwrap_or_default(),
                binary_file_tb: row.get::<_, Option<f64>>(39)?.unwrap_or_default(),
                planilhas: row.get::<_, Option<String>>(40)?.unwrap_or_default(),
                colunas: row.get::<_, Option<String>>(41)?.unwrap_or_default(),
                registros: row.get::<_, Option<String>>(42)?.unwrap_or_default(),
                tabelas: row.get::<_, Option<String>>(43)?.unwrap_or_default(),
                timestamp: row.get::<_, Option<String>>(44)?.unwrap_or_default(),
                monitor_profile_name: row.get::<_, Option<String>>(45)?.unwrap_or_default(),
                sha256_atual: row.get::<_, Option<String>>(46)?.unwrap_or_default(),
                baseline_status: row.get::<_, Option<String>>(47)?.unwrap_or_default(),
                alerta_resumo: row.get::<_, Option<String>>(48)?.unwrap_or_default(),
                session_label: row.get::<_, Option<String>>(49)?.unwrap_or_default(),
                diff_resumo: row.get::<_, Option<String>>(50)?.unwrap_or_default(),
                risk_score: row.get::<_, Option<i32>>(51)?.unwrap_or_default(),
                risk_level: row.get::<_, Option<String>>(52)?.unwrap_or_default(),
                actor_user: row.get::<_, Option<String>>(53)?.unwrap_or_default(),
                actor_process: row.get::<_, Option<String>>(54)?.unwrap_or_default(),
                publisher: row.get::<_, Option<String>>(55)?.unwrap_or_default(),
                quarantine_status: row.get::<_, Option<String>>(56)?.unwrap_or_default(),
                evidence_id: row.get::<_, Option<String>>(57)?.unwrap_or_default(),
                ..Default::default()
            })
        });

        match rows {
            Ok(it) => {
                let default_user = usuario_padrao_sistema();
                let mut vec: Vec<EventModel> = it.filter_map(Result::ok).collect();
                for ev in &mut vec {
                    normalizar_evento_do_banco(ev, &default_user);
                }
                for ev in vec.iter().rev().take(2048) {
                    let full = ev.full_path();
                    if !full.is_empty() {
                        salvar_evento_cache(Path::new(&full), ev);
                    }
                }
                vec
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn limpar_banco_eventos(&self) -> Result<(), String> {
        self.writer.recreate_and_vacuum()?;
        self.pending_deletions.lock().clear();
        Ok(())
    }

    pub fn limpar_dados(&self) {
        let _ = self.limpar_banco_eventos();
    }

    pub fn buscar_origem_rename_move(&self, evento: &EventModel) -> Option<EventModel> {
        if evento.dir_anterior.is_empty() {
            return self.buscar_metadados_anteriores(&evento.nome, &evento.dir_atual);
        }
        let p_ant = Path::new(&evento.dir_anterior);
        if let (Some(parent), Some(fname)) = (p_ant.parent(), p_ant.file_name()) {
            let parent_str = parent.to_string_lossy();
            let fname_str = fname.to_string_lossy();
            if !parent_str.is_empty() && !p_ant.is_dir() {
                if let Some(ev) = self.buscar_metadados_anteriores(&fname_str, &parent_str) {
                    return Some(ev);
                }
            }
        }
        self.buscar_metadados_anteriores(&evento.nome, &evento.dir_anterior)
    }

    pub fn buscar_metadados_anteriores(&self, nome: &str, dir: &str) -> Option<EventModel> {
        if !dir.is_empty() && !nome.is_empty() {
            let full_candidate = Path::new(dir).join(nome);
            if let Some(cached) = obter_evento_cacheado(&full_candidate) {
                return Some(cached);
            }
        }
        let conn = Connection::open(&self.db_path).ok()?;
        let sql_exact = "SELECT id, tipo_operacao, nome, dir_anterior, dir_atual, data_criacao, data_modificacao,
                    data_acesso, tipo, mime, size_b, size_kb, size_mb, size_gb, size_tb, atributos,
                    autor, dimensoes, duracao, taxa_bits, protegido, paginas, linhas, palavras,
                    paginas_estimadas, linhas_codigo, total_linhas, slides_estimados, arquivos,
                    unzipped_b, unzipped_kb, unzipped_mb, unzipped_gb, unzipped_tb, slides,
                    binary_file_b, binary_file_kb, binary_file_mb, binary_file_gb, binary_file_tb,
                    planilhas, colunas, registros, tabelas, timestamp, monitor_profile_name,
                    sha256_atual, baseline_status, alerta_resumo, session_label, diff_resumo,
                    risk_score, risk_level, actor_user, actor_process, publisher, quarantine_status,
                    evidence_id
              FROM monitoramento WHERE nome = ?1 AND dir_atual = ?2 ORDER BY id DESC LIMIT 1";

        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<EventModel> {
            Ok(EventModel {
                id: row.get(0).ok(),
                tipo_operacao: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                nome: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                dir_anterior: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                dir_atual: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                data_criacao: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                data_modificacao: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                data_acesso: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                tipo: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                mime: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                size_b: row.get::<_, Option<f64>>(10)?.unwrap_or_default(),
                size_kb: row.get::<_, Option<f64>>(11)?.unwrap_or_default(),
                size_mb: row.get::<_, Option<f64>>(12)?.unwrap_or_default(),
                size_gb: row.get::<_, Option<f64>>(13)?.unwrap_or_default(),
                size_tb: row.get::<_, Option<f64>>(14)?.unwrap_or_default(),
                atributos: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                autor: row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                dimensoes: row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                duracao: row.get::<_, Option<String>>(18)?.unwrap_or_default(),
                taxa_bits: row.get::<_, Option<String>>(19)?.unwrap_or_default(),
                protegido: row.get::<_, Option<String>>(20)?.unwrap_or_default(),
                paginas: row.get::<_, Option<String>>(21)?.unwrap_or_default(),
                linhas: row.get::<_, Option<String>>(22)?.unwrap_or_default(),
                palavras: row.get::<_, Option<String>>(23)?.unwrap_or_default(),
                paginas_estimadas: row.get::<_, Option<String>>(24)?.unwrap_or_default(),
                linhas_codigo: row.get::<_, Option<String>>(25)?.unwrap_or_default(),
                total_linhas: row.get::<_, Option<String>>(26)?.unwrap_or_default(),
                slides_estimados: row.get::<_, Option<String>>(27)?.unwrap_or_default(),
                arquivos: row.get::<_, Option<String>>(28)?.unwrap_or_default(),
                unzipped_b: row.get::<_, Option<f64>>(29)?.unwrap_or_default(),
                unzipped_kb: row.get::<_, Option<f64>>(30)?.unwrap_or_default(),
                unzipped_mb: row.get::<_, Option<f64>>(31)?.unwrap_or_default(),
                unzipped_gb: row.get::<_, Option<f64>>(32)?.unwrap_or_default(),
                unzipped_tb: row.get::<_, Option<f64>>(33)?.unwrap_or_default(),
                slides: row.get::<_, Option<String>>(34)?.unwrap_or_default(),
                binary_file_b: row.get::<_, Option<f64>>(35)?.unwrap_or_default(),
                binary_file_kb: row.get::<_, Option<f64>>(36)?.unwrap_or_default(),
                binary_file_mb: row.get::<_, Option<f64>>(37)?.unwrap_or_default(),
                binary_file_gb: row.get::<_, Option<f64>>(38)?.unwrap_or_default(),
                binary_file_tb: row.get::<_, Option<f64>>(39)?.unwrap_or_default(),
                planilhas: row.get::<_, Option<String>>(40)?.unwrap_or_default(),
                colunas: row.get::<_, Option<String>>(41)?.unwrap_or_default(),
                registros: row.get::<_, Option<String>>(42)?.unwrap_or_default(),
                tabelas: row.get::<_, Option<String>>(43)?.unwrap_or_default(),
                timestamp: row.get::<_, Option<String>>(44)?.unwrap_or_default(),
                monitor_profile_name: row.get::<_, Option<String>>(45)?.unwrap_or_default(),
                sha256_atual: row.get::<_, Option<String>>(46)?.unwrap_or_default(),
                baseline_status: row.get::<_, Option<String>>(47)?.unwrap_or_default(),
                alerta_resumo: row.get::<_, Option<String>>(48)?.unwrap_or_default(),
                session_label: row.get::<_, Option<String>>(49)?.unwrap_or_default(),
                diff_resumo: row.get::<_, Option<String>>(50)?.unwrap_or_default(),
                risk_score: row.get::<_, Option<i32>>(51)?.unwrap_or_default(),
                risk_level: row.get::<_, Option<String>>(52)?.unwrap_or_default(),
                actor_user: row.get::<_, Option<String>>(53)?.unwrap_or_default(),
                actor_process: row.get::<_, Option<String>>(54)?.unwrap_or_default(),
                publisher: row.get::<_, Option<String>>(55)?.unwrap_or_default(),
                quarantine_status: row.get::<_, Option<String>>(56)?.unwrap_or_default(),
                evidence_id: row.get::<_, Option<String>>(57)?.unwrap_or_default(),
                ..Default::default()
            })
        };

        if let Ok(mut stmt) = conn.prepare(sql_exact) {
            if let Ok(mut ev) = stmt.query_row(rusqlite::params![nome, dir], &map_row) {
                let default_user = usuario_padrao_sistema();
                normalizar_evento_do_banco(&mut ev, &default_user);
                return Some(ev);
            }
        }

        let sql_name = "SELECT id, tipo_operacao, nome, dir_anterior, dir_atual, data_criacao, data_modificacao,
                    data_acesso, tipo, mime, size_b, size_kb, size_mb, size_gb, size_tb, atributos,
                    autor, dimensoes, duracao, taxa_bits, protegido, paginas, linhas, palavras,
                    paginas_estimadas, linhas_codigo, total_linhas, slides_estimados, arquivos,
                    unzipped_b, unzipped_kb, unzipped_mb, unzipped_gb, unzipped_tb, slides,
                    binary_file_b, binary_file_kb, binary_file_mb, binary_file_gb, binary_file_tb,
                    planilhas, colunas, registros, tabelas, timestamp, monitor_profile_name,
                    sha256_atual, baseline_status, alerta_resumo, session_label, diff_resumo,
                    risk_score, risk_level, actor_user, actor_process, publisher, quarantine_status,
                    evidence_id
              FROM monitoramento WHERE nome = ?1 ORDER BY id DESC LIMIT 1";

        if let Ok(mut stmt) = conn.prepare(sql_name) {
            if let Ok(mut ev) = stmt.query_row(rusqlite::params![nome], &map_row) {
                let default_user = usuario_padrao_sistema();
                normalizar_evento_do_banco(&mut ev, &default_user);
                return Some(ev);
            }
        }

        None
    }
}
