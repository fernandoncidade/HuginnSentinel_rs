use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{OperationType, RiskLevel};

/// Modelo representativo de um evento de monitoramento com todas as 58 colunas suportadas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventModel {
    pub id: Option<i64>,
    pub tipo_operacao: String,
    pub nome: String,
    pub dir_anterior: String,
    pub dir_atual: String,
    pub data_criacao: String,
    pub data_modificacao: String,
    pub data_acesso: String,
    pub tipo: String,
    pub mime: String,
    pub size_b: f64,
    pub size_kb: f64,
    pub size_mb: f64,
    pub size_gb: f64,
    pub size_tb: f64,
    pub atributos: String,
    pub autor: String,
    pub dimensoes: String,
    pub duracao: String,
    pub taxa_bits: String,
    pub protegido: String,
    pub paginas: String,
    pub linhas: String,
    pub palavras: String,
    pub paginas_estimadas: String,
    pub linhas_codigo: String,
    pub total_linhas: String,
    pub slides_estimados: String,
    pub arquivos: String,
    pub unzipped_b: f64,
    pub unzipped_kb: f64,
    pub unzipped_mb: f64,
    pub unzipped_gb: f64,
    pub unzipped_tb: f64,
    pub slides: String,
    pub binary_file_b: f64,
    pub binary_file_kb: f64,
    pub binary_file_mb: f64,
    pub binary_file_gb: f64,
    pub binary_file_tb: f64,
    pub planilhas: String,
    pub colunas: String,
    pub registros: String,
    pub tabelas: String,
    pub timestamp: String,
    pub monitor_profile_name: String,
    pub sha256_atual: String,
    pub baseline_status: String,
    pub alerta_resumo: String,
    pub session_label: String,
    pub diff_resumo: String,
    pub risk_score: i32,
    pub risk_level: String,
    pub actor_user: String,
    pub actor_process: String,
    pub publisher: String,
    pub quarantine_status: String,
    pub evidence_id: String,

    // Campos adicionais forenses de auditoria
    pub monitor_profile_id: String,
    pub monitor_diretorios: String,
    pub sha256_baseline: String,
    pub integrity_flags: String,
    pub alerta_regras: String,
    pub alerta_nivel: String,
    pub session_id: String,
    pub risk_reasons: String,
    pub actor_pid: i32,
    pub attribution_source: String,
    pub quarantine_path: String,
    pub signature_state: String,
    pub script_risk: String,
    pub evidence_manifest: String,
    pub evidence_hash: String,
}

impl Default for EventModel {
    fn default() -> Self {
        Self {
            id: None,
            tipo_operacao: String::new(),
            nome: String::new(),
            dir_anterior: String::new(),
            dir_atual: String::new(),
            data_criacao: String::new(),
            data_modificacao: String::new(),
            data_acesso: String::new(),
            tipo: String::new(),
            mime: String::new(),
            size_b: 0.0,
            size_kb: 0.0,
            size_mb: 0.0,
            size_gb: 0.0,
            size_tb: 0.0,
            atributos: String::new(),
            autor: String::new(),
            dimensoes: String::new(),
            duracao: String::new(),
            taxa_bits: String::new(),
            protegido: String::new(),
            paginas: String::new(),
            linhas: String::new(),
            palavras: String::new(),
            paginas_estimadas: String::new(),
            linhas_codigo: String::new(),
            total_linhas: String::new(),
            slides_estimados: String::new(),
            arquivos: String::new(),
            unzipped_b: 0.0,
            unzipped_kb: 0.0,
            unzipped_mb: 0.0,
            unzipped_gb: 0.0,
            unzipped_tb: 0.0,
            slides: String::new(),
            binary_file_b: 0.0,
            binary_file_kb: 0.0,
            binary_file_mb: 0.0,
            binary_file_gb: 0.0,
            binary_file_tb: 0.0,
            planilhas: String::new(),
            colunas: String::new(),
            registros: String::new(),
            tabelas: String::new(),
            timestamp: String::new(),
            monitor_profile_name: String::new(),
            sha256_atual: String::new(),
            baseline_status: String::new(),
            alerta_resumo: String::new(),
            session_label: String::new(),
            diff_resumo: String::new(),
            risk_score: 0,
            risk_level: RiskLevel::Baixo.as_str().to_string(),
            actor_user: String::new(),
            actor_process: String::new(),
            publisher: String::new(),
            quarantine_status: String::new(),
            evidence_id: String::new(),

            monitor_profile_id: String::new(),
            monitor_diretorios: String::new(),
            sha256_baseline: String::new(),
            integrity_flags: String::new(),
            alerta_regras: String::new(),
            alerta_nivel: String::new(),
            session_id: String::new(),
            risk_reasons: String::new(),
            actor_pid: 0,
            attribution_source: String::new(),
            quarantine_path: String::new(),
            signature_state: String::new(),
            script_risk: String::new(),
            evidence_manifest: String::new(),
            evidence_hash: String::new(),
        }
    }
}

impl EventModel {
    pub fn new(
        tipo_operacao: OperationType,
        nome: impl Into<String>,
        dir: impl Into<String>,
    ) -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            tipo_operacao: tipo_operacao.display_pt().to_string(),
            nome: nome.into(),
            dir_atual: dir.into(),
            timestamp: now.clone(),
            data_modificacao: now,
            ..Default::default()
        }
    }

    pub fn op_type(&self) -> OperationType {
        let clean = self.tipo_operacao.trim();
        if !clean.is_empty() {
            OperationType::from_raw(clean)
        } else if !self.dir_anterior.is_empty() && self.dir_anterior != self.dir_atual {
            OperationType::Movido
        } else {
            OperationType::Modificado
        }
    }

    pub fn full_path(&self) -> String {
        if self.dir_atual.is_empty() {
            self.nome.clone()
        } else {
            let sep = if self.dir_atual.ends_with('\\') || self.dir_atual.ends_with('/') {
                ""
            } else {
                "\\"
            };
            format!("{}{}{}", self.dir_atual, sep, self.nome)
        }
    }

    pub fn get_field(&self, key: &str) -> String {
        match key {
            "id" => self.id.map(|v| v.to_string()).unwrap_or_default(),
            "tipo_operacao" => self.tipo_operacao.clone(),
            "nome" => self.nome.clone(),
            "dir_anterior" => self.dir_anterior.clone(),
            "dir_atual" => self.dir_atual.clone(),
            "data_criacao" => self.data_criacao.clone(),
            "data_modificacao" => self.data_modificacao.clone(),
            "data_acesso" => self.data_acesso.clone(),
            "tipo" => self.tipo.clone(),
            "mime" => self.mime.clone(),
            "size_b" => format!("{:.0}", self.size_b),
            "size_kb" => format!("{:.2}", self.size_kb),
            "size_mb" => format!("{:.2}", self.size_mb),
            "size_gb" => format!("{:.3}", self.size_gb),
            "size_tb" => format!("{:.4}", self.size_tb),
            "atributos" => self.atributos.clone(),
            "autor" => self.autor.clone(),
            "dimensoes" => self.dimensoes.clone(),
            "duracao" => self.duracao.clone(),
            "taxa_bits" => self.taxa_bits.clone(),
            "protegido" => self.protegido.clone(),
            "paginas" => self.paginas.clone(),
            "linhas" => self.linhas.clone(),
            "palavras" => self.palavras.clone(),
            "paginas_estimadas" => self.paginas_estimadas.clone(),
            "linhas_codigo" => self.linhas_codigo.clone(),
            "total_linhas" => self.total_linhas.clone(),
            "slides_estimados" => self.slides_estimados.clone(),
            "arquivos" => self.arquivos.clone(),
            "unzipped_b" => {
                if self.unzipped_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_descompactado(self)
                {
                    format!("{:.0}", self.unzipped_b)
                } else {
                    String::new()
                }
            }
            "unzipped_kb" => {
                if self.unzipped_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_descompactado(self)
                {
                    format!("{:.2}", self.unzipped_kb)
                } else {
                    String::new()
                }
            }
            "unzipped_mb" => {
                if self.unzipped_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_descompactado(self)
                {
                    format!("{:.2}", self.unzipped_mb)
                } else {
                    String::new()
                }
            }
            "unzipped_gb" => {
                if self.unzipped_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_descompactado(self)
                {
                    format!("{:.3}", self.unzipped_gb)
                } else {
                    String::new()
                }
            }
            "unzipped_tb" => {
                if self.unzipped_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_descompactado(self)
                {
                    format!("{:.4}", self.unzipped_tb)
                } else {
                    String::new()
                }
            }
            "slides" => self.slides.clone(),
            "binary_file_b" => {
                if self.binary_file_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_binario(self)
                {
                    format!("{:.0}", self.binary_file_b)
                } else {
                    String::new()
                }
            }
            "binary_file_kb" => {
                if self.binary_file_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_binario(self)
                {
                    format!("{:.2}", self.binary_file_kb)
                } else {
                    String::new()
                }
            }
            "binary_file_mb" => {
                if self.binary_file_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_binario(self)
                {
                    format!("{:.2}", self.binary_file_mb)
                } else {
                    String::new()
                }
            }
            "binary_file_gb" => {
                if self.binary_file_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_binario(self)
                {
                    format!("{:.3}", self.binary_file_gb)
                } else {
                    String::new()
                }
            }
            "binary_file_tb" => {
                if self.binary_file_b > 0.0
                    || crate::services::metadados::harmonizacao::exibe_binario(self)
                {
                    format!("{:.4}", self.binary_file_tb)
                } else {
                    String::new()
                }
            }
            "planilhas" => self.planilhas.clone(),
            "colunas" => self.colunas.clone(),
            "registros" => self.registros.clone(),
            "tabelas" => self.tabelas.clone(),
            "timestamp" => self.timestamp.clone(),
            "monitor_profile_name" => self.monitor_profile_name.clone(),
            "sha256_atual" => self.sha256_atual.clone(),
            "baseline_status" => self.baseline_status.clone(),
            "alerta_resumo" => self.alerta_resumo.clone(),
            "session_label" => self.session_label.clone(),
            "diff_resumo" => self.diff_resumo.clone(),
            "risk_score" => self.risk_score.to_string(),
            "risk_level" => self.risk_level.clone(),
            "actor_user" => self.actor_user.clone(),
            "actor_process" => self.actor_process.clone(),
            "publisher" => self.publisher.clone(),
            "quarantine_status" => self.quarantine_status.clone(),
            "evidence_id" => self.evidence_id.clone(),
            _ => String::new(),
        }
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        for col in crate::services::gerenciador_colunas::LISTA_COLUNAS_TODAS {
            m.insert(col.key.to_string(), self.get_field(col.key));
        }
        m
    }
}
