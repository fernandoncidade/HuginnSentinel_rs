// Módulo de Geometria e Layout da Planilha (GridLayout)
// Gerencia dimensões, posições, rolagem virtual e congelamento de painéis (linhas e colunas fixas).

use super::col_header::ColumnHeader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PainelFixoConfig {
    pub freeze_header_row: bool,
    pub freeze_row_headers: bool,
    pub pinned_rows: Vec<usize>,
    pub pinned_cols: Vec<String>,
    #[serde(default)]
    pub frozen_rows: usize,
    #[serde(default)]
    pub frozen_cols: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridLayout {
    pub default_row_height: f32,
    pub default_header_height: f32,
    pub row_header_width: f32,
    pub custom_row_heights: HashMap<usize, f32>,
    pub custom_col_widths: HashMap<String, f32>,
    pub frozen_rows: usize,
    pub frozen_cols: usize,
    pub freeze_header_row: bool,
    pub freeze_row_headers: bool,
    pub pinned_rows: BTreeSet<usize>,
    pub pinned_cols: BTreeSet<String>,
    pub min_col_width: f32,
    pub max_col_width: f32,
    pub min_row_height: f32,
    pub max_row_height: f32,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl GridLayout {
    pub fn new() -> Self {
        Self {
            default_row_height: 26.0,
            default_header_height: 34.0,
            row_header_width: 52.0,
            custom_row_heights: HashMap::new(),
            custom_col_widths: HashMap::new(),
            frozen_rows: 0,
            frozen_cols: 0,
            freeze_header_row: true,
            freeze_row_headers: true,
            pinned_rows: BTreeSet::new(),
            pinned_cols: BTreeSet::new(),
            min_col_width: 40.0,
            max_col_width: 1200.0,
            min_row_height: 18.0,
            max_row_height: 140.0,
        }
    }

    pub fn get_row_height(&self, row_idx: usize) -> f32 {
        self.custom_row_heights
            .get(&row_idx)
            .copied()
            .unwrap_or(self.default_row_height)
    }

    pub fn set_row_height(&mut self, row_idx: usize, height: f32) {
        let clamped = height.clamp(self.min_row_height, self.max_row_height);
        self.custom_row_heights.insert(row_idx, clamped);
    }

    pub fn get_col_width(&self, col_key: &str, default_val: f32) -> f32 {
        self.custom_col_widths
            .get(col_key)
            .copied()
            .unwrap_or(default_val)
            .clamp(self.min_col_width, self.max_col_width)
    }

    pub fn set_col_width(&mut self, col_key: &str, width: f32) {
        let clamped = width.clamp(self.min_col_width, self.max_col_width);
        self.custom_col_widths.insert(col_key.to_string(), clamped);
    }

    pub fn toggle_freeze_cols(&mut self, count: usize) {
        if self.frozen_cols == count {
            self.frozen_cols = 0;
        } else {
            self.frozen_cols = count;
        }
    }

    pub fn toggle_freeze_rows(&mut self, count: usize) {
        if self.frozen_rows == count {
            self.frozen_rows = 0;
        } else {
            self.frozen_rows = count;
        }
    }

    /// Verifica se uma linha de dados (0-based) está fixada individualmente ou por prefixo
    pub fn is_row_pinned(&self, row_idx: usize) -> bool {
        row_idx < self.frozen_rows || self.pinned_rows.contains(&row_idx)
    }

    /// Define se uma linha de dados específica (0-based) permanece fixa ao rolar verticalmente
    pub fn set_row_pinned(&mut self, row_idx: usize, pinned: bool) {
        if pinned {
            self.pinned_rows.insert(row_idx);
        } else {
            self.pinned_rows.remove(&row_idx);
            if row_idx < self.frozen_rows {
                for r in 0..self.frozen_rows {
                    if r != row_idx {
                        self.pinned_rows.insert(r);
                    }
                }
                self.frozen_rows = 0;
            }
        }
    }

    /// Alterna o estado fixo de uma linha de dados específica (0-based)
    pub fn toggle_pin_row(&mut self, row_idx: usize) -> bool {
        let next = !self.is_row_pinned(row_idx);
        self.set_row_pinned(row_idx, next);
        next
    }

    /// Desafixa absolutamente todas as linhas (incluindo a 1ª linha de cabeçalhos das colunas)
    pub fn unfreeze_all_rows(&mut self) {
        self.freeze_header_row = false;
        self.frozen_rows = 0;
        self.pinned_rows.clear();
    }

    /// Restaura a configuração padrão de linhas (apenas a 1ª linha de cabeçalhos fixa)
    pub fn reset_freeze_rows_default(&mut self) {
        self.freeze_header_row = true;
        self.frozen_rows = 0;
        self.pinned_rows.clear();
    }

    /// Verifica se uma coluna específica está marcada como fixa pela chave
    pub fn is_col_pinned(&self, col_key: &str) -> bool {
        self.pinned_cols.contains(col_key)
    }

    /// Verifica se uma coluna em `logical_idx` com chave `col_key` está efetivamente fixa
    pub fn is_col_effectively_pinned(&self, logical_idx: usize, col_key: &str) -> bool {
        logical_idx < self.frozen_cols || self.pinned_cols.contains(col_key)
    }

    /// Define se uma coluna específica permanece fixa ao rolar horizontalmente
    pub fn set_col_pinned(&mut self, col_key: &str, pinned: bool) {
        if pinned {
            self.pinned_cols.insert(col_key.to_string());
        } else {
            self.pinned_cols.remove(col_key);
        }
    }

    /// Alterna o estado fixo de uma coluna específica pela chave
    pub fn toggle_pin_col(&mut self, col_key: &str) -> bool {
        let next = !self.is_col_pinned(col_key);
        self.set_col_pinned(col_key, next);
        next
    }

    /// Desafixa absolutamente todas as colunas (incluindo a 1ª coluna numérica #)
    pub fn unfreeze_all_cols(&mut self) {
        self.freeze_row_headers = false;
        self.frozen_cols = 0;
        self.pinned_cols.clear();
    }

    /// Restaura a configuração padrão de colunas (apenas a 1ª coluna numérica # fixa)
    pub fn reset_freeze_cols_default(&mut self) {
        self.freeze_row_headers = true;
        self.frozen_cols = 0;
        self.pinned_cols.clear();
    }

    /// Atualiza índices de linhas fixas ao inserir uma linha na tabela
    pub fn on_row_inserted(&mut self, at_row: usize) {
        let mut updated = BTreeSet::new();
        for &r in &self.pinned_rows {
            if r >= at_row {
                updated.insert(r + 1);
            } else {
                updated.insert(r);
            }
        }
        self.pinned_rows = updated;
    }

    /// Atualiza índices de linhas fixas ao remover uma linha da tabela
    pub fn on_row_deleted(&mut self, at_row: usize) {
        let mut updated = BTreeSet::new();
        for &r in &self.pinned_rows {
            if r == at_row {
                continue;
            } else if r > at_row {
                updated.insert(r - 1);
            } else {
                updated.insert(r);
            }
        }
        self.pinned_rows = updated;
    }

    /// Interpreta uma especificação 1-based digitada pelo usuário (ex: "1, 5, 10-12") em índices 0-based
    pub fn parse_row_numbers_spec(spec: &str, max_rows: Option<usize>) -> Vec<usize> {
        let mut out = BTreeSet::new();
        for token in spec.split([',', ';']) {
            let trimmed = token.trim().trim_start_matches('#');
            if trimmed.is_empty() {
                continue;
            }
            if let Some((a_str, b_str)) = trimmed.split_once('-') {
                if let (Ok(a), Ok(b)) = (a_str.trim().parse::<usize>(), b_str.trim().parse::<usize>())
                {
                    let start = a.min(b).max(1);
                    let end = a.max(b);
                    // Limita intervalo de expansão manual a no máximo 500 linhas por segurança
                    for one_based in start..=end.min(start.saturating_add(499)) {
                        let idx = one_based - 1;
                        if max_rows.map_or(true, |m| idx < m) {
                            out.insert(idx);
                        }
                    }
                }
            } else if let Ok(one_based) = trimmed.parse::<usize>() {
                if one_based >= 1 {
                    let idx = one_based - 1;
                    if max_rows.map_or(true, |m| idx < m) {
                        out.insert(idx);
                    }
                }
            }
        }
        out.into_iter().collect()
    }

    /// Salva a configuração persistente de linhas e colunas fixas em `%LOCALAPPDATA%\HuginnSentinel`
    pub fn salvar_configuracao_painel_fixo(&self) {
        let p = crate::utils::paths::obter_caminho_persistente().join("painel_fixo_config.json");
        let cfg = PainelFixoConfig {
            freeze_header_row: self.freeze_header_row,
            freeze_row_headers: self.freeze_row_headers,
            pinned_rows: self.pinned_rows.iter().copied().collect(),
            pinned_cols: self.pinned_cols.iter().cloned().collect(),
            frozen_rows: self.frozen_rows,
            frozen_cols: self.frozen_cols,
        };
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(p, json);
        }
    }

    /// Carrega a configuração persistente de linhas e colunas fixas caso exista
    pub fn carregar_configuracao_painel_fixo(&mut self) {
        let p = crate::utils::paths::obter_caminho_persistente().join("painel_fixo_config.json");
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(cfg) = serde_json::from_slice::<PainelFixoConfig>(&bytes) {
                self.freeze_header_row = cfg.freeze_header_row;
                self.freeze_row_headers = cfg.freeze_row_headers;
                self.pinned_rows = cfg.pinned_rows.into_iter().collect();
                self.pinned_cols = cfg.pinned_cols.into_iter().collect();
                self.frozen_rows = cfg.frozen_rows;
                self.frozen_cols = cfg.frozen_cols;
            }
        }
    }

    /// Calcula o intervalo de linhas visíveis para virtualização de alto desempenho
    pub fn compute_visible_rows(
        &self,
        offset_y: f32,
        view_height: f32,
        total_rows: usize,
    ) -> (usize, usize) {
        if total_rows == 0 {
            return (0, 0);
        }
        let row_h = self.default_row_height;
        let first = ((offset_y / row_h).floor() as usize).saturating_sub(2);
        let visible_count = ((view_height / row_h).ceil() as usize) + 5;
        let last = (first + visible_count).min(total_rows);
        (first, last)
    }

    /// Calcula a largura total de uma lista de colunas
    pub fn total_columns_width(&self, columns: &[ColumnHeader]) -> f32 {
        columns
            .iter()
            .filter(|c| c.visible)
            .map(|c| self.get_col_width(&c.key, c.width))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_layout_row_height_clamps() {
        let mut layout = GridLayout::new();
        layout.set_row_height(0, 10.0);
        assert_eq!(layout.get_row_height(0), layout.min_row_height);

        layout.set_row_height(0, 500.0);
        assert_eq!(layout.get_row_height(0), layout.max_row_height);

        layout.set_row_height(0, 30.0);
        assert_eq!(layout.get_row_height(0), 30.0);
    }

    #[test]
    fn test_grid_layout_col_width_clamps() {
        let mut layout = GridLayout::new();
        layout.set_col_width("col_test", 10.0);
        assert_eq!(
            layout.get_col_width("col_test", 100.0),
            layout.min_col_width
        );

        layout.set_col_width("col_test", 2000.0);
        assert_eq!(
            layout.get_col_width("col_test", 100.0),
            layout.max_col_width
        );

        layout.set_col_width("col_test", 150.0);
        assert_eq!(layout.get_col_width("col_test", 100.0), 150.0);
    }

    #[test]
    fn test_visible_rows_computation() {
        let layout = GridLayout::new();
        let (first, last) = layout.compute_visible_rows(260.0, 520.0, 1000);
        assert!(first <= 10);
        assert!(last >= 30);
    }
}
