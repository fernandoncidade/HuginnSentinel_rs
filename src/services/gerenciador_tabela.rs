use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::core::types::OperationType;
use crate::utils::paths::obter_caminho_persistente;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationColorConfig {
    pub bg_hex: String,
    pub fg_hex: String,
}

pub struct GerenciadorTabela {
    cores_operacoes: HashMap<OperationType, (Color32, Color32)>, // (bg, fg)
    colunas_para_colorir: HashSet<String>,
    pub coluna_ordenacao: Option<String>,
    pub ordem_ascendente: bool,
    pub ordenacao_habilitada: bool,
}

impl Default for GerenciadorTabela {
    fn default() -> Self {
        Self::new()
    }
}

impl GerenciadorTabela {
    pub fn new() -> Self {
        let mut colunas_para_colorir = HashSet::new();
        colunas_para_colorir.insert("tipo_operacao".to_string());

        let mut gt = Self {
            cores_operacoes: HashMap::new(),
            colunas_para_colorir,
            coluna_ordenacao: Some("timestamp".to_string()),
            ordem_ascendente: false, // mais recentes primeiro por padrão
            ordenacao_habilitada: true,
        };
        gt.inicializar_cores_padrao();
        gt.carregar_configuracao_cores();
        gt.carregar_configuracao_colunas_coloridas();
        gt
    }

    fn inicializar_cores_padrao(&mut self) {
        for op in OperationType::all() {
            let (bg_hex, fg_hex) = op.default_color_hex();
            let bg = parse_hex_color(bg_hex);
            let fg = parse_hex_color(fg_hex);
            self.cores_operacoes.insert(*op, (bg, fg));
        }
    }

    pub fn obter_cores_operacao(&self, op: OperationType) -> (Color32, Color32) {
        self.cores_operacoes
            .get(&op)
            .copied()
            .unwrap_or_else(|| (Color32::from_rgb(40, 40, 40), Color32::WHITE))
    }

    pub fn set_cor_operacao(&mut self, op: OperationType, bg: Color32) {
        let fg = calcular_cor_texto_ideal(bg);
        self.cores_operacoes.insert(op, (bg, fg));
        self.salvar_configuracao_cores();
    }

    pub fn resetar_cores_padrao(&mut self) {
        self.inicializar_cores_padrao();
        self.salvar_configuracao_cores();
    }

    pub fn salvar_configuracao_cores(&self) {
        let p = obter_caminho_persistente().join("cores_config.json");
        let mut map: HashMap<String, OperationColorConfig> = HashMap::new();
        for (op, (bg, fg)) in &self.cores_operacoes {
            map.insert(
                op.as_str().to_string(),
                OperationColorConfig {
                    bg_hex: color_to_hex(*bg),
                    fg_hex: color_to_hex(*fg),
                },
            );
        }
        if let Ok(json) = serde_json::to_string_pretty(&map) {
            let _ = std::fs::write(p, json);
        }
    }

    pub fn carregar_configuracao_cores(&mut self) {
        let p = obter_caminho_persistente().join("cores_config.json");
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(map) = serde_json::from_slice::<HashMap<String, OperationColorConfig>>(&bytes)
            {
                for (key, cfg) in map {
                    let op = OperationType::from_raw(&key);
                    let bg = parse_hex_color(&cfg.bg_hex);
                    let fg = parse_hex_color(&cfg.fg_hex);
                    self.cores_operacoes.insert(op, (bg, fg));
                }
            }
        }
    }

    pub fn coluna_deve_colorir(&self, key: &str) -> bool {
        self.colunas_para_colorir.contains(key)
    }

    pub fn set_coluna_colorir(&mut self, key: &str, colorir: bool) {
        if colorir {
            self.colunas_para_colorir.insert(key.to_string());
        } else {
            self.colunas_para_colorir.remove(key);
        }
        self.salvar_configuracao_colunas_coloridas();
    }

    pub fn selecionar_todas_colunas_colorir(&mut self, keys: &[&str]) {
        for k in keys {
            self.colunas_para_colorir.insert(k.to_string());
        }
        self.salvar_configuracao_colunas_coloridas();
    }

    pub fn resetar_colunas_colorir(&mut self) {
        self.colunas_para_colorir.clear();
        self.colunas_para_colorir
            .insert("tipo_operacao".to_string());
        self.salvar_configuracao_colunas_coloridas();
    }

    pub fn alternar_ordenacao_linhas(&mut self) -> bool {
        self.ordenacao_habilitada = !self.ordenacao_habilitada;
        self.ordenacao_habilitada
    }

    /// Alterna a direção da ordenação entre ascendente (true) e descendente (false)
    pub fn inverter_ordem(&mut self) -> bool {
        self.ordem_ascendente = !self.ordem_ascendente;
        self.ordenacao_habilitada = true;
        self.ordem_ascendente
    }

    /// Define a coluna de ordenação e sua direção (ascendente ou descendente)
    pub fn definir_ordenacao_coluna(&mut self, key: impl Into<String>, ascendente: bool) {
        self.coluna_ordenacao = Some(key.into());
        self.ordem_ascendente = ascendente;
        self.ordenacao_habilitada = true;
    }

    pub fn is_ordenacao_habilitada(&self) -> bool {
        self.ordenacao_habilitada
    }

    pub fn salvar_configuracao_colunas_coloridas(&self) {
        let p = obter_caminho_persistente().join("colunas_coloridas_config.json");
        let list: Vec<String> = self.colunas_para_colorir.iter().cloned().collect();
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = std::fs::write(p, json);
        }
    }

    pub fn carregar_configuracao_colunas_coloridas(&mut self) {
        let p = obter_caminho_persistente().join("colunas_coloridas_config.json");
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(list) = serde_json::from_slice::<Vec<String>>(&bytes) {
                self.colunas_para_colorir = list.into_iter().collect();
            }
        }
        // Garante que tipo_operacao esteja presente por padrão se o conjunto estiver vazio
        if self.colunas_para_colorir.is_empty() {
            self.colunas_para_colorir
                .insert("tipo_operacao".to_string());
        }
    }
}

pub fn calcular_cor_texto_ideal(bg: Color32) -> Color32 {
    let r = bg.r() as f32;
    let g = bg.g() as f32;
    let b = bg.b() as f32;
    // Luminância relativa perceptiva
    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    if y > 140.0 {
        Color32::from_rgb(18, 18, 18) // Texto escuro em fundo claro
    } else {
        Color32::from_rgb(255, 255, 255) // Texto claro em fundo escuro
    }
}

pub fn parse_hex_color(hex_str: &str) -> Color32 {
    let clean = hex_str.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return Color32::from_rgb(r, g, b);
        }
    }
    Color32::from_rgb(46, 125, 50)
}

pub fn color_to_hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}
