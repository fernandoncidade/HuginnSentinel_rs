use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::paths::obter_caminho_persistente;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColunaInfo {
    pub key: &'static str,
    pub translation_key: &'static str,
    pub visivel: bool,
    pub ordem: usize,
    pub largura_padrao: f32,
}

pub const LISTA_COLUNAS_TODAS: &[ColunaInfo] = &[
    ColunaInfo {
        key: "tipo_operacao",
        translation_key: "operation_type",
        visivel: true,
        ordem: 0,
        largura_padrao: 110.0,
    },
    ColunaInfo {
        key: "nome",
        translation_key: "name",
        visivel: true,
        ordem: 1,
        largura_padrao: 200.0,
    },
    ColunaInfo {
        key: "dir_anterior",
        translation_key: "prev_dir",
        visivel: true,
        ordem: 2,
        largura_padrao: 180.0,
    },
    ColunaInfo {
        key: "dir_atual",
        translation_key: "curr_dir",
        visivel: true,
        ordem: 3,
        largura_padrao: 220.0,
    },
    ColunaInfo {
        key: "data_criacao",
        translation_key: "creation_date",
        visivel: false,
        ordem: 4,
        largura_padrao: 140.0,
    },
    ColunaInfo {
        key: "data_modificacao",
        translation_key: "modification_date",
        visivel: true,
        ordem: 5,
        largura_padrao: 140.0,
    },
    ColunaInfo {
        key: "data_acesso",
        translation_key: "access_date",
        visivel: false,
        ordem: 6,
        largura_padrao: 140.0,
    },
    ColunaInfo {
        key: "tipo",
        translation_key: "type",
        visivel: true,
        ordem: 7,
        largura_padrao: 140.0,
    },
    ColunaInfo {
        key: "mime",
        translation_key: "mime",
        visivel: true,
        ordem: 8,
        largura_padrao: 160.0,
    },
    ColunaInfo {
        key: "size_b",
        translation_key: "size_b",
        visivel: false,
        ordem: 9,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "size_kb",
        translation_key: "size_kb",
        visivel: true,
        ordem: 10,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "size_mb",
        translation_key: "size_mb",
        visivel: false,
        ordem: 11,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "size_gb",
        translation_key: "size_gb",
        visivel: false,
        ordem: 12,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "size_tb",
        translation_key: "size_tb",
        visivel: false,
        ordem: 13,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "atributos",
        translation_key: "attributes",
        visivel: false,
        ordem: 14,
        largura_padrao: 120.0,
    },
    ColunaInfo {
        key: "autor",
        translation_key: "author",
        visivel: false,
        ordem: 15,
        largura_padrao: 120.0,
    },
    ColunaInfo {
        key: "dimensoes",
        translation_key: "dimensions",
        visivel: false,
        ordem: 16,
        largura_padrao: 110.0,
    },
    ColunaInfo {
        key: "duracao",
        translation_key: "duration",
        visivel: false,
        ordem: 17,
        largura_padrao: 100.0,
    },
    ColunaInfo {
        key: "taxa_bits",
        translation_key: "bit_rate",
        visivel: false,
        ordem: 18,
        largura_padrao: 100.0,
    },
    ColunaInfo {
        key: "protegido",
        translation_key: "protected",
        visivel: false,
        ordem: 19,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "paginas",
        translation_key: "pages",
        visivel: false,
        ordem: 20,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "linhas",
        translation_key: "lines",
        visivel: false,
        ordem: 21,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "palavras",
        translation_key: "words",
        visivel: false,
        ordem: 22,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "paginas_estimadas",
        translation_key: "pages_estimated",
        visivel: false,
        ordem: 23,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "linhas_codigo",
        translation_key: "lines_code",
        visivel: false,
        ordem: 24,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "total_linhas",
        translation_key: "total_lines",
        visivel: false,
        ordem: 25,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "slides_estimados",
        translation_key: "slides_estimated",
        visivel: false,
        ordem: 26,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "arquivos",
        translation_key: "files",
        visivel: false,
        ordem: 27,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "unzipped_b",
        translation_key: "unzipped_b",
        visivel: false,
        ordem: 28,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "unzipped_kb",
        translation_key: "unzipped_kb",
        visivel: false,
        ordem: 29,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "unzipped_mb",
        translation_key: "unzipped_mb",
        visivel: false,
        ordem: 30,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "unzipped_gb",
        translation_key: "unzipped_gb",
        visivel: false,
        ordem: 31,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "unzipped_tb",
        translation_key: "unzipped_tb",
        visivel: false,
        ordem: 32,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "slides",
        translation_key: "slides",
        visivel: false,
        ordem: 33,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "binary_file_b",
        translation_key: "binary_file_b",
        visivel: false,
        ordem: 34,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "binary_file_kb",
        translation_key: "binary_file_kb",
        visivel: false,
        ordem: 35,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "binary_file_mb",
        translation_key: "binary_file_mb",
        visivel: false,
        ordem: 36,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "binary_file_gb",
        translation_key: "binary_file_gb",
        visivel: false,
        ordem: 37,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "binary_file_tb",
        translation_key: "binary_file_tb",
        visivel: false,
        ordem: 38,
        largura_padrao: 90.0,
    },
    ColunaInfo {
        key: "planilhas",
        translation_key: "spreadsheets",
        visivel: false,
        ordem: 39,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "colunas",
        translation_key: "columns",
        visivel: false,
        ordem: 40,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "registros",
        translation_key: "records",
        visivel: false,
        ordem: 41,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "tabelas",
        translation_key: "tables",
        visivel: false,
        ordem: 42,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "timestamp",
        translation_key: "timestamp",
        visivel: true,
        ordem: 43,
        largura_padrao: 140.0,
    },
    ColunaInfo {
        key: "monitor_profile_name",
        translation_key: "security_monitoring_profile",
        visivel: false,
        ordem: 44,
        largura_padrao: 130.0,
    },
    ColunaInfo {
        key: "sha256_atual",
        translation_key: "security_sha256_current",
        visivel: false,
        ordem: 45,
        largura_padrao: 180.0,
    },
    ColunaInfo {
        key: "baseline_status",
        translation_key: "security_baseline_status",
        visivel: false,
        ordem: 46,
        largura_padrao: 110.0,
    },
    ColunaInfo {
        key: "alerta_resumo",
        translation_key: "security_alert_summary",
        visivel: true,
        ordem: 47,
        largura_padrao: 160.0,
    },
    ColunaInfo {
        key: "session_label",
        translation_key: "security_session_label",
        visivel: false,
        ordem: 48,
        largura_padrao: 110.0,
    },
    ColunaInfo {
        key: "diff_resumo",
        translation_key: "security_diff_summary",
        visivel: false,
        ordem: 49,
        largura_padrao: 110.0,
    },
    ColunaInfo {
        key: "risk_score",
        translation_key: "security_risk_score",
        visivel: false,
        ordem: 50,
        largura_padrao: 80.0,
    },
    ColunaInfo {
        key: "risk_level",
        translation_key: "security_risk_level",
        visivel: true,
        ordem: 51,
        largura_padrao: 100.0,
    },
    ColunaInfo {
        key: "actor_user",
        translation_key: "security_actor_user",
        visivel: false,
        ordem: 52,
        largura_padrao: 100.0,
    },
    ColunaInfo {
        key: "actor_process",
        translation_key: "security_actor_process",
        visivel: false,
        ordem: 53,
        largura_padrao: 120.0,
    },
    ColunaInfo {
        key: "publisher",
        translation_key: "security_publisher_name",
        visivel: false,
        ordem: 54,
        largura_padrao: 140.0,
    },
    ColunaInfo {
        key: "quarantine_status",
        translation_key: "security_quarantine_status",
        visivel: false,
        ordem: 55,
        largura_padrao: 110.0,
    },
    ColunaInfo {
        key: "evidence_id",
        translation_key: "security_evidence_id",
        visivel: false,
        ordem: 56,
        largura_padrao: 110.0,
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColunaCustomizada {
    pub key: String,
    pub visivel: bool,
    pub ordem: usize,
    #[serde(default)]
    pub largura: Option<f32>,
}

pub struct GerenciadorColunas {
    colunas: Vec<ColunaInfo>,
}

impl Default for GerenciadorColunas {
    fn default() -> Self {
        Self::new()
    }
}

impl GerenciadorColunas {
    pub fn new() -> Self {
        let mut gc = Self {
            colunas: LISTA_COLUNAS_TODAS.to_vec(),
        };
        gc.carregar_configuracoes();
        gc
    }

    pub fn colunas(&self) -> &[ColunaInfo] {
        &self.colunas
    }

    pub fn colunas_mut(&mut self) -> &mut [ColunaInfo] {
        &mut self.colunas
    }

    pub fn colunas_visiveis(&self) -> Vec<&ColunaInfo> {
        self.colunas.iter().filter(|c| c.visivel).collect()
    }

    pub fn alternar_visibilidade(&mut self, key: &str) {
        if let Some(col) = self.colunas.iter_mut().find(|c| c.key == key) {
            col.visivel = !col.visivel;
            self.salvar_configuracoes();
        }
    }

    pub fn set_visibilidade(&mut self, key: &str, visivel: bool) {
        if let Some(col) = self.colunas.iter_mut().find(|c| c.key == key) {
            col.visivel = visivel;
        }
    }

    pub fn set_largura(&mut self, key: &str, largura: f32) {
        if let Some(col) = self.colunas.iter_mut().find(|c| c.key == key) {
            col.largura_padrao = largura.clamp(50.0, 1200.0);
        }
    }

    pub fn autofit_coluna_ao_cabecalho(&mut self, key: &str, loc: &crate::locale::Localizer) {
        if let Some(col) = self.colunas.iter_mut().find(|c| c.key == key) {
            let texto = loc.get_text(col.translation_key);
            let char_len = texto.chars().count();
            let estimated_w = (char_len as f32 * 8.5) + 42.0;
            col.largura_padrao = estimated_w.clamp(60.0, 1200.0);
        }
    }

    pub fn autofit_todas_colunas_ao_cabecalho(&mut self, loc: &crate::locale::Localizer) {
        for col in self.colunas.iter_mut() {
            let texto = loc.get_text(col.translation_key);
            let char_len = texto.chars().count();
            let estimated_w = (char_len as f32 * 8.5) + 42.0;
            col.largura_padrao = estimated_w.clamp(60.0, 1200.0);
        }
        self.salvar_configuracoes();
    }

    pub fn set_largura_medida(&mut self, key: &str, text_pixel_width: f32) {
        if let Some(col) = self.colunas.iter_mut().find(|c| c.key == key) {
            col.largura_padrao = (text_pixel_width + 42.0).clamp(60.0, 1200.0);
        }
    }

    pub fn resetar_padrao(&mut self) {
        self.colunas = LISTA_COLUNAS_TODAS.to_vec();
        self.salvar_configuracoes();
    }

    pub fn resetar_larguras(&mut self) {
        for col in self.colunas.iter_mut() {
            if let Some(padrao) = LISTA_COLUNAS_TODAS.iter().find(|c| c.key == col.key) {
                col.largura_padrao = padrao.largura_padrao;
            }
        }
        self.salvar_configuracoes();
    }

    pub fn salvar_configuracoes(&self) {
        let p = obter_caminho_persistente().join("colunas_config.json");
        let list: Vec<ColunaCustomizada> = self
            .colunas
            .iter()
            .map(|c| ColunaCustomizada {
                key: c.key.to_string(),
                visivel: c.visivel,
                ordem: c.ordem,
                largura: Some(c.largura_padrao),
            })
            .collect();
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = std::fs::write(p, json);
        }
    }

    pub fn carregar_configuracoes(&mut self) {
        let p = obter_caminho_persistente().join("colunas_config.json");
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(list) = serde_json::from_slice::<Vec<ColunaCustomizada>>(&bytes) {
                let map: HashMap<String, ColunaCustomizada> =
                    list.into_iter().map(|c| (c.key.clone(), c)).collect();
                for col in self.colunas.iter_mut() {
                    if let Some(saved) = map.get(col.key) {
                        col.visivel = saved.visivel;
                        col.ordem = saved.ordem;
                        if let Some(largura) = saved.largura {
                            col.largura_padrao = largura.clamp(50.0, 1200.0);
                        }
                    }
                }
                self.colunas.sort_by_key(|c| c.ordem);
            }
        }
    }
}
