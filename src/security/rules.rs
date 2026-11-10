use std::path::Path;

use crate::core::event::EventModel;
use crate::core::types::OperationType;
use crate::services::metadados::extensao_tipo_preferencial;
use crate::services::metadados::types::{is_config_ext, is_executable_ext, is_system_ext};

#[derive(Debug, Clone)]
pub struct SecurityRuleMatch {
    pub rule_id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub points: i32,
    pub severity: &'static str,
}

pub struct MotorRegras;

impl MotorRegras {
    pub fn avaliar(evento: &EventModel) -> Vec<SecurityRuleMatch> {
        let mut matches = Vec::new();
        let nome_lower = evento.nome.to_lowercase();
        let dir_lower = evento.dir_atual.to_lowercase();
        let ext = extensao_tipo_preferencial(Path::new(&evento.nome));

        // R01: Extensões típicas de Ransomware
        let ransomware_exts = [
            ".locked",
            ".crypto",
            ".enc",
            ".locky",
            ".crypted",
            ".ransom",
            ".wnry",
            ".stop",
            ".cerber",
            ".zepto",
            ".lockbit",
            ".blackcat",
            ".clop",
        ];
        if ransomware_exts.iter().any(|r| nome_lower.ends_with(r)) {
            matches.push(SecurityRuleMatch {
                rule_id: "SEC-R01",
                name: "rule_ransomware",
                description: "Arquivo possui extensão associada a sequestro de dados (ransomware).",
                points: 95,
                severity: "critical",
            });
        }

        // R02: Extensão dupla dissimulada ou renomeação para executável
        let double_exts = [
            ".pdf.exe",
            ".docx.exe",
            ".xlsx.exe",
            ".jpg.exe",
            ".png.exe",
            ".pdf.vbs",
            ".txt.vbs",
            ".doc.js",
            ".pdf.bat",
            ".txt.exe",
        ];
        if double_exts.iter().any(|d| nome_lower.ends_with(d))
            || (evento.op_type() == OperationType::Renomeado && is_executable_ext(&ext))
        {
            matches.push(SecurityRuleMatch {
                rule_id: "SEC-R02",
                name: "rule_double_ext",
                description: "Arquivo executável renomeado ou com extensão dupla dissimulada.",
                points: 85,
                severity: "critical",
            });
        }

        // R03: Executável não assinado ou em diretório temporário
        let is_binary = is_executable_ext(&ext) || is_system_ext(&ext);
        let in_temp = dir_lower.contains("temp") || dir_lower.contains("appdata\\local");
        if is_binary && (in_temp || evento.signature_state == "unsigned") {
            let pts = if in_temp { 75 } else { 45 };
            matches.push(SecurityRuleMatch {
                rule_id: "SEC-R03",
                name: "rule_binary_temp",
                description: "Executável ou biblioteca sem assinatura digital verificada.",
                points: pts,
                severity: if in_temp { "high" } else { "medium" },
            });
        }

        // R04: Script de automação com indicadores de risco
        let is_script = matches!(
            ext.as_str(),
            "ps1" | "psm1" | "vbs" | "vbe" | "bat" | "cmd" | "wsf" | "hta"
        );
        if is_script && (in_temp || evento.script_risk == "high" || evento.script_risk == "medium")
        {
            let pts = if evento.script_risk == "high" { 80 } else { 55 };
            matches.push(SecurityRuleMatch {
                rule_id: "SEC-R04",
                name: "rule_script_temp",
                description: "Script de automação ou comandos com potencial de execução privilegiada.",
                points: pts,
                severity: "high",
            });
        }

        // R05: Executável oculto
        if is_binary
            && (evento.atributos.contains("hidden") || evento.atributos.contains("Oculto"))
        {
            matches.push(SecurityRuleMatch {
                rule_id: "SEC-R05",
                name: "rule_hidden_executable",
                description: "Arquivo executável configurado com atributo oculto no sistema.",
                points: 70,
                severity: "high",
            });
        }

        // R06: Alteração de baseline criptográfico ou configuração crítica
        if evento.baseline_status == "changed" || evento.baseline_status == "MODIFICADO" {
            let pts = if is_config_ext(&ext) || is_binary {
                65
            } else {
                35
            };
            matches.push(SecurityRuleMatch {
                rule_id: "SEC-R06",
                name: "rule_baseline_divergence",
                description: "O hash criptográfico SHA-256 do arquivo diverge do baseline inicial.",
                points: pts,
                severity: if pts >= 60 { "high" } else { "medium" },
            });
        }

        matches
    }
}
