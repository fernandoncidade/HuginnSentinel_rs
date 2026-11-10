use chrono::NaiveDateTime;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::core::event::EventModel;
use crate::core::types::OperationType;
use crate::services::metadados::extensao_tipo_preferencial;
use crate::services::metadados::types::{
    is_backup_ext, is_compressed_ext, is_executable_ext, is_log_ext, is_source_code_ext,
    is_temp_ext, is_virtual_disk_ext,
};

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub session_id: String,
    pub inicio: String,
    pub fim: String,
    pub total_eventos: usize,
    pub adicionados: usize,
    pub modificados: usize,
    pub excluidos: usize,
    pub renomeados: usize,
    pub movidos: usize,
    pub escaneados: usize,
    pub max_risco: i32,
    pub nivel_risco: String,
    pub diretorios: Vec<String>,
}

struct LiveSessionTracker {
    current_index: usize,
    last_event_at: Option<Instant>,
}

static LIVE_SESSION: LazyLock<Mutex<LiveSessionTracker>> = LazyLock::new(|| {
    Mutex::new(LiveSessionTracker {
        current_index: 1,
        last_event_at: None,
    })
});

pub struct CorrelacaoSessoes;

impl CorrelacaoSessoes {
    /// Atribui `session_id` e `session_label` ao evento em tempo real ou durante varredura
    /// (paridade com `sess_01_correlacao_sessoes.cpp`).
    pub fn atribuir_sessao_evento(evento: &mut EventModel) {
        let mut tracker = LIVE_SESSION.lock();
        let now = Instant::now();
        if let Some(last) = tracker.last_event_at {
            if now.duration_since(last) > Duration::from_secs(300) {
                tracker.current_index += 1;
            }
        }
        tracker.last_event_at = Some(now);
        let idx = tracker.current_index;
        drop(tracker);

        evento.session_id = format!("SESS-{}", idx);

        let ext = extensao_tipo_preferencial(Path::new(&evento.nome));
        let dir_lower = evento.dir_atual.to_lowercase();
        let op = evento.op_type();

        let label = if op == OperationType::Excluido
            && (is_temp_ext(&ext) || is_log_ext(&ext) || is_backup_ext(&ext))
        {
            "limpeza_automatica"
        } else if is_source_code_ext(&ext)
            || dir_lower.contains("\\target\\")
            || dir_lower.contains("\\build\\")
            || dir_lower.contains("\\node_modules\\")
            || dir_lower.contains("\\.git\\")
        {
            "build_refatoracao"
        } else if (op == OperationType::Adicionado || op == OperationType::Modificado)
            && (is_executable_ext(&ext) || is_compressed_ext(&ext) || is_virtual_disk_ext(&ext))
        {
            "instalacao_ou_extracao"
        } else {
            "sessao_monitoramento"
        };

        evento.session_label = label.to_string();
    }

    pub fn agrupar_em_sessoes(eventos: &[EventModel]) -> Vec<SessionSummary> {
        if eventos.is_empty() {
            return Vec::new();
        }

        let mut ordenados: Vec<&EventModel> = eventos.iter().collect();
        ordenados.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut sessoes: Vec<SessionSummary> = Vec::new();
        let mut session_index = 1;

        let mut sessao_atual: Option<SessionSummary> = None;
        let mut ultimo_instante: Option<NaiveDateTime> = None;

        for ev in ordenados {
            let dt = NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default();

            let nova_sessao = if let Some(last) = ultimo_instante {
                let diff_segundos = dt.signed_duration_since(last).num_seconds();
                diff_segundos > 300
            } else {
                true
            };

            if nova_sessao {
                if let Some(s) = sessao_atual.take() {
                    sessoes.push(s);
                }
                let id = format!("SESS-{}", session_index);
                session_index += 1;
                sessao_atual = Some(SessionSummary {
                    session_id: id,
                    inicio: ev.timestamp.clone(),
                    fim: ev.timestamp.clone(),
                    total_eventos: 0,
                    adicionados: 0,
                    modificados: 0,
                    excluidos: 0,
                    renomeados: 0,
                    movidos: 0,
                    escaneados: 0,
                    max_risco: 0,
                    nivel_risco: "Baixo".to_string(),
                    diretorios: Vec::new(),
                });
            }

            if let Some(s) = sessao_atual.as_mut() {
                s.fim = ev.timestamp.clone();
                s.total_eventos += 1;
                match ev.op_type() {
                    OperationType::Adicionado => s.adicionados += 1,
                    OperationType::Modificado => s.modificados += 1,
                    OperationType::Excluido => s.excluidos += 1,
                    OperationType::Renomeado => s.renomeados += 1,
                    OperationType::Movido => s.movidos += 1,
                    OperationType::Escaneado => s.escaneados += 1,
                }
                if ev.risk_score > s.max_risco {
                    s.max_risco = ev.risk_score;
                    s.nivel_risco = ev.risk_level.clone();
                }
                if !ev.dir_atual.is_empty() && !s.diretorios.contains(&ev.dir_atual) {
                    s.diretorios.push(ev.dir_atual.clone());
                }
            }

            ultimo_instante = Some(dt);
        }

        if let Some(s) = sessao_atual {
            sessoes.push(s);
        }

        sessoes.reverse();
        sessoes
    }
}
