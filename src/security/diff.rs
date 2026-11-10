use std::collections::HashMap;

use crate::core::event::EventModel;
use crate::core::types::OperationType;

pub struct DiffConteudo;

impl DiffConteudo {
    pub fn resumir_diferenca(tamanho_antigo: f64, tamanho_novo: f64) -> String {
        let delta = tamanho_novo - tamanho_antigo;
        if delta > 0.0 {
            format!("+{} bytes", delta as i64)
        } else if delta < 0.0 {
            format!("{} bytes", delta as i64)
        } else {
            "Sem variação de tamanho".to_string()
        }
    }

    /// Calcula o resumo de diff textual entre o snapshot anterior e o conteúdo atual
    /// (paridade com `sdiff_01_diff_conteudo.cpp`).
    pub fn calcular_diff_evento(
        evento: &EventModel,
        texto_anterior: Option<&str>,
        texto_atual: Option<&str>,
    ) -> String {
        if let (Some(antigo), Some(atual)) = (texto_anterior, texto_atual) {
            if antigo == atual {
                return "sem_alteracao_textual".to_string();
            }

            let mut freq_antigo: HashMap<&str, usize> = HashMap::new();
            for line in antigo.lines() {
                *freq_antigo.entry(line).or_insert(0) += 1;
            }

            let mut added = 0usize;
            for line in atual.lines() {
                if let Some(cnt) = freq_antigo.get_mut(line) {
                    if *cnt > 0 {
                        *cnt -= 1;
                        continue;
                    }
                }
                added += 1;
            }
            let removed: usize = freq_antigo.values().sum();

            if added > 0 || removed > 0 {
                return format!("security_diff_lines_changed|{}|{}", added, removed);
            }
        }

        let total_linhas = evento
            .total_linhas
            .parse::<usize>()
            .or_else(|_| evento.linhas.parse::<usize>())
            .unwrap_or(0);

        match evento.op_type() {
            OperationType::Adicionado if total_linhas > 0 => {
                format!("security_diff_lines_changed|{}|0", total_linhas)
            }
            OperationType::Excluido if total_linhas > 0 => {
                format!("security_diff_lines_changed|0|{}", total_linhas)
            }
            OperationType::Modificado
                if (evento.baseline_status == "changed" || evento.baseline_status == "MODIFICADO")
                    && total_linhas > 0 =>
            {
                format!("security_diff_lines_changed|1|0")
            }
            _ => "sem_alteracao_textual".to_string(),
        }
    }
}
