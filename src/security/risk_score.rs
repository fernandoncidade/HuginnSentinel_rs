use super::rules::MotorRegras;
use crate::core::event::EventModel;
use crate::core::types::{OperationType, RiskLevel};

pub struct ScoreRisco;

impl ScoreRisco {
    pub fn avaliar_evento(evento: &mut EventModel) {
        let matches = MotorRegras::avaliar(evento);

        let mut max_score = 0i32;
        let mut reasons = Vec::new();
        let mut alert_names = Vec::new();
        let mut rule_ids = Vec::new();

        for m in &matches {
            if m.points > max_score {
                max_score = m.points;
            }
            reasons.push(format!("{}: {}", m.name, m.description));
            alert_names.push(m.name.to_string());
            rule_ids.push(m.rule_id.to_string());
        }

        // Pontuação complementar de contexto operacional (paridade com risk_01_score_risco.cpp)
        let mut bonus = 0i32;
        match evento.op_type() {
            OperationType::Excluido => bonus += 10,
            OperationType::Renomeado | OperationType::Movido => bonus += 8,
            OperationType::Modificado => bonus += 5,
            OperationType::Adicionado => bonus += 5,
            OperationType::Escaneado => {}
        }

        if evento.baseline_status == "changed" && max_score == 0 {
            bonus += 20;
        } else if evento.baseline_status == "missing" {
            bonus += 15;
        }

        let final_score = (max_score + bonus).clamp(0, 100);
        let risk_lvl = RiskLevel::from_score(final_score);

        evento.risk_score = final_score;
        evento.risk_level = risk_lvl.as_str().to_string();

        if !matches.is_empty() {
            evento.alerta_regras = rule_ids.join(", ");
            evento.alerta_nivel = risk_lvl.as_str().to_string();
            evento.alerta_resumo = alert_names.join("; ");
            evento.risk_reasons = reasons.join(" | ");
        } else {
            evento.alerta_regras = "none".to_string();
            evento.alerta_nivel = "baixo".to_string();
            evento.alerta_resumo = match evento.baseline_status.as_str() {
                "directory" => "security_baseline_directory".to_string(),
                "changed" => "security_baseline_changed".to_string(),
                "missing" => "security_baseline_missing".to_string(),
                "new" => "security_baseline_new".to_string(),
                _ => "security_baseline_unchanged".to_string(),
            };
            evento.risk_reasons = "Nenhuma anomalia crítica detectada".to_string();
        }
    }
}
