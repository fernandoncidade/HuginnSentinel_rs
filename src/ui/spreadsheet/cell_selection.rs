// Módulo de Seleção de Células (CellSelectionModel) — Padrão Microsoft Excel
// Gerencia seleção individual, intervalos retangulares, múltiplas seleções, linhas e colunas completas, e seleção integral.

use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Tonalidade de verde clássico do Microsoft Excel (#107C41) utilizada como cor padrão de seleção
pub const COR_SELECAO_PADRAO: [u8; 3] = [16, 124, 65];

fn default_cor_selecao() -> [u8; 3] {
    COR_SELECAO_PADRAO
}

/// Modo de acompanhamento ou fixação estática de seleções na planilha
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SelectionFollowMode {
    /// Acompanha dinamicamente a seleção (na rolagem e no rastreamento de dados)
    #[default]
    Acompanhar,
    /// Permanece estática (mantém posição/índice fixo na grade)
    Estatica,
}

/// Configuração do comportamento de seleção (células, linhas e rolagem automática)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionBehaviorConfig {
    /// Modo principal: se ativado, a seleção acompanha os eventos/células e a rolagem os segue
    pub acompanhar_selecao: bool,
    /// Se ativado, a seleção de célula individual ou conjunto de células acompanha a seleção
    pub acompanhar_celulas: bool,
    /// Se ativado, a seleção de linha individual ou conjunto de linhas acompanha a seleção
    pub acompanhar_linhas: bool,
    /// Se ativado, a rolagem/visualização rola automaticamente para manter a seleção visível
    pub auto_scroll: bool,
    /// Cor RGB utilizada para realçar a seleção de células, intervalos e cabeçalhos
    #[serde(default = "default_cor_selecao")]
    pub cor_selecao: [u8; 3],
}

impl Default for SelectionBehaviorConfig {
    fn default() -> Self {
        Self {
            acompanhar_selecao: true,
            acompanhar_celulas: true,
            acompanhar_linhas: true,
            auto_scroll: true,
            cor_selecao: COR_SELECAO_PADRAO,
        }
    }
}

impl SelectionBehaviorConfig {
    /// Retorna verdadeiro se a seleção de células ou conjunto de células deve acompanhar a seleção
    pub fn deve_acompanhar_celulas(&self) -> bool {
        self.acompanhar_selecao && self.acompanhar_celulas
    }

    /// Retorna verdadeiro se a seleção de linhas ou conjunto de linhas deve acompanhar a seleção
    pub fn deve_acompanhar_linhas(&self) -> bool {
        self.acompanhar_selecao && self.acompanhar_linhas
    }

    /// Retorna verdadeiro se a rolagem da grade deve acompanhar a seleção ativa
    pub fn deve_acompanhar_rolagem(&self) -> bool {
        self.acompanhar_selecao && self.auto_scroll
    }

    /// Retorna a cor sólida de seleção em formato `egui::Color32`
    pub fn cor_selecao_color32(&self) -> Color32 {
        Color32::from_rgb(self.cor_selecao[0], self.cor_selecao[1], self.cor_selecao[2])
    }

    /// Retorna a cor semitransparente para preenchimento de intervalos retangulares selecionados
    pub fn cor_selecao_preenchimento(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.cor_selecao[0],
            self.cor_selecao[1],
            self.cor_selecao[2],
            30,
        )
    }

    /// Retorna a cor de fundo da célula selecionada adaptada dinamicamente ao tema Claro/Escuro
    pub fn cor_selecao_fundo_celula(&self, dark_mode: bool) -> Color32 {
        let r = self.cor_selecao[0] as f32;
        let g = self.cor_selecao[1] as f32;
        let b = self.cor_selecao[2] as f32;
        if dark_mode {
            let r_tint = (r * 0.22 + 20.0 * 0.78).clamp(0.0, 255.0) as u8;
            let g_tint = (g * 0.22 + 22.0 * 0.78).clamp(0.0, 255.0) as u8;
            let b_tint = (b * 0.22 + 26.0 * 0.78).clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r_tint, g_tint, b_tint)
        } else {
            let r_tint = (r * 0.12 + 255.0 * 0.88).clamp(0.0, 255.0) as u8;
            let g_tint = (g * 0.12 + 255.0 * 0.88).clamp(0.0, 255.0) as u8;
            let b_tint = (b * 0.12 + 255.0 * 0.88).clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r_tint, g_tint, b_tint)
        }
    }

    /// Retorna a cor do texto da célula selecionada com legibilidade e contraste ideais
    pub fn cor_selecao_texto(&self, dark_mode: bool) -> Color32 {
        let r = self.cor_selecao[0] as f32;
        let g = self.cor_selecao[1] as f32;
        let b = self.cor_selecao[2] as f32;
        if dark_mode {
            let r_text = (r * 0.35 + 255.0 * 0.65).clamp(0.0, 255.0) as u8;
            let g_text = (g * 0.35 + 255.0 * 0.65).clamp(0.0, 255.0) as u8;
            let b_text = (b * 0.35 + 255.0 * 0.65).clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r_text, g_text, b_text)
        } else {
            let r_text = (r * 0.65).clamp(0.0, 255.0) as u8;
            let g_text = (g * 0.65).clamp(0.0, 255.0) as u8;
            let b_text = (b * 0.65).clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r_text, g_text, b_text)
        }
    }

    /// Restaura a cor de seleção para a tonalidade padrão original (Verde Excel)
    pub fn restaurar_cor_padrao(&mut self) {
        self.cor_selecao = COR_SELECAO_PADRAO;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellCoord {
    pub row: usize,
    pub col: usize,
}

impl CellCoord {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRange {
    pub min_row: usize,
    pub max_row: usize,
    pub min_col: usize,
    pub max_col: usize,
}

impl SelectionRange {
    pub fn from_points(r1: usize, c1: usize, r2: usize, c2: usize) -> Self {
        Self {
            min_row: r1.min(r2),
            max_row: r1.max(r2),
            min_col: c1.min(c2),
            max_col: c1.max(c2),
        }
    }

    pub fn contains(&self, row: usize, col: usize) -> bool {
        row >= self.min_row && row <= self.max_row && col >= self.min_col && col <= self.max_col
    }

    pub fn contains_row(&self, row: usize, total_cols: usize) -> bool {
        row >= self.min_row
            && row <= self.max_row
            && self.min_col == 0
            && self.max_col + 1 >= total_cols
    }

    pub fn contains_col(&self, col: usize, total_rows: usize) -> bool {
        col >= self.min_col
            && col <= self.max_col
            && self.min_row == 0
            && self.max_row + 1 >= total_rows
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    None,
    Cell,
    Range,
    Rows,
    Cols,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellSelectionModel {
    pub active_cell: Option<CellCoord>,
    pub anchor_cell: Option<CellCoord>,
    pub primary_range: Option<SelectionRange>,
    pub additional_ranges: Vec<SelectionRange>,
    pub selected_rows: HashSet<usize>,
    pub selected_cols: HashSet<usize>,
    pub is_all_selected: bool,
}

impl Default for CellSelectionModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CellSelectionModel {
    pub fn new() -> Self {
        Self {
            active_cell: None,
            anchor_cell: None,
            primary_range: None,
            additional_ranges: Vec::new(),
            selected_rows: HashSet::new(),
            selected_cols: HashSet::new(),
            is_all_selected: false,
        }
    }

    pub fn clear(&mut self) {
        self.active_cell = None;
        self.anchor_cell = None;
        self.primary_range = None;
        self.additional_ranges.clear();
        self.selected_rows.clear();
        self.selected_cols.clear();
        self.is_all_selected = false;
    }

    pub fn select_cell(&mut self, row: usize, col: usize) {
        self.clear();
        let coord = CellCoord::new(row, col);
        self.active_cell = Some(coord);
        self.anchor_cell = Some(coord);
        self.primary_range = Some(SelectionRange::from_points(row, col, row, col));
    }

    pub fn select_range(&mut self, r1: usize, c1: usize, r2: usize, c2: usize) {
        self.clear();
        self.active_cell = Some(CellCoord::new(r2, c2));
        self.anchor_cell = Some(CellCoord::new(r1, c1));
        self.primary_range = Some(SelectionRange::from_points(r1, c1, r2, c2));
    }

    pub fn expand_to(&mut self, row: usize, col: usize) {
        let anchor = self.anchor_cell.unwrap_or(CellCoord::new(row, col));
        self.anchor_cell = Some(anchor);
        self.active_cell = Some(CellCoord::new(row, col));
        self.primary_range = Some(SelectionRange::from_points(
            anchor.row, anchor.col, row, col,
        ));
        self.is_all_selected = false;
    }

    pub fn select_row(&mut self, row: usize, extend: bool, toggle: bool, total_cols: usize) {
        if toggle {
            if self.selected_rows.contains(&row) {
                self.selected_rows.remove(&row);
            } else {
                self.selected_rows.insert(row);
            }
            self.active_cell = Some(CellCoord::new(row, 0));
            self.is_all_selected = false;
        } else if extend {
            let start = self.anchor_cell.map(|a| a.row).unwrap_or(row);
            let min_r = start.min(row);
            let max_r = start.max(row);
            self.selected_rows.clear();
            for r in min_r..=max_r {
                self.selected_rows.insert(r);
            }
            self.active_cell = Some(CellCoord::new(row, 0));
            self.primary_range = Some(SelectionRange::from_points(
                min_r,
                0,
                max_r,
                total_cols.saturating_sub(1),
            ));
            self.is_all_selected = false;
        } else {
            self.clear();
            self.selected_rows.insert(row);
            let coord = CellCoord::new(row, 0);
            self.active_cell = Some(coord);
            self.anchor_cell = Some(coord);
            self.primary_range = Some(SelectionRange::from_points(
                row,
                0,
                row,
                total_cols.saturating_sub(1),
            ));
        }
    }

    pub fn select_col(&mut self, col: usize, extend: bool, toggle: bool, total_rows: usize) {
        if toggle {
            if self.selected_cols.contains(&col) {
                self.selected_cols.remove(&col);
            } else {
                self.selected_cols.insert(col);
            }
            self.active_cell = Some(CellCoord::new(0, col));
            self.is_all_selected = false;
        } else if extend {
            let start = self.anchor_cell.map(|a| a.col).unwrap_or(col);
            let min_c = start.min(col);
            let max_c = start.max(col);
            self.selected_cols.clear();
            for c in min_c..=max_c {
                self.selected_cols.insert(c);
            }
            self.active_cell = Some(CellCoord::new(0, col));
            self.primary_range = Some(SelectionRange::from_points(
                0,
                min_c,
                total_rows.saturating_sub(1),
                max_c,
            ));
            self.is_all_selected = false;
        } else {
            self.clear();
            self.selected_cols.insert(col);
            let coord = CellCoord::new(0, col);
            self.active_cell = Some(coord);
            self.anchor_cell = Some(coord);
            self.primary_range = Some(SelectionRange::from_points(
                0,
                col,
                total_rows.saturating_sub(1),
                col,
            ));
        }
    }

    pub fn select_all(&mut self) {
        self.clear();
        self.is_all_selected = true;
        self.active_cell = Some(CellCoord::new(0, 0));
        self.anchor_cell = Some(CellCoord::new(0, 0));
    }

    pub fn select_row_range(&mut self, r1: usize, r2: usize, total_cols: usize) {
        let min_r = r1.min(r2);
        let max_r = r1.max(r2);
        self.clear();
        for r in min_r..=max_r {
            self.selected_rows.insert(r);
        }
        self.anchor_cell = Some(CellCoord::new(r1, 0));
        self.active_cell = Some(CellCoord::new(r2, 0));
        self.primary_range = Some(SelectionRange::from_points(
            min_r,
            0,
            max_r,
            total_cols.saturating_sub(1),
        ));
        self.is_all_selected = false;
    }

    pub fn select_col_range(&mut self, c1: usize, c2: usize, total_rows: usize) {
        let min_c = c1.min(c2);
        let max_c = c1.max(c2);
        self.clear();
        for c in min_c..=max_c {
            self.selected_cols.insert(c);
        }
        self.anchor_cell = Some(CellCoord::new(0, c1));
        self.active_cell = Some(CellCoord::new(0, c2));
        self.primary_range = Some(SelectionRange::from_points(
            0,
            min_c,
            total_rows.saturating_sub(1),
            max_c,
        ));
        self.is_all_selected = false;
    }

    pub fn is_cell_selected(&self, row: usize, col: usize) -> bool {
        if self.is_all_selected {
            return true;
        }
        if self.selected_rows.contains(&row) || self.selected_cols.contains(&col) {
            return true;
        }
        if let Some(ref range) = self.primary_range {
            if range.contains(row, col) {
                return true;
            }
        }
        for range in &self.additional_ranges {
            if range.contains(row, col) {
                return true;
            }
        }
        false
    }

    pub fn is_row_selected(&self, row: usize) -> bool {
        if self.is_all_selected || self.selected_rows.contains(&row) {
            return true;
        }
        if let Some(ref range) = self.primary_range {
            if row >= range.min_row && row <= range.max_row {
                return true;
            }
        }
        false
    }

    pub fn is_col_selected(&self, col: usize) -> bool {
        if self.is_all_selected || self.selected_cols.contains(&col) {
            return true;
        }
        if let Some(ref range) = self.primary_range {
            if col >= range.min_col && col <= range.max_col {
                return true;
            }
        }
        false
    }

    pub fn is_active_cell(&self, row: usize, col: usize) -> bool {
        self.active_cell == Some(CellCoord::new(row, col))
    }

    pub fn on_row_inserted(&mut self, at_index: usize) {
        if let Some(ref mut ac) = self.active_cell {
            if ac.row >= at_index {
                ac.row += 1;
            }
        }
        if let Some(ref mut anc) = self.anchor_cell {
            if anc.row >= at_index {
                anc.row += 1;
            }
        }
        if let Some(ref mut range) = self.primary_range {
            if range.min_row >= at_index {
                range.min_row += 1;
            }
            if range.max_row >= at_index {
                range.max_row += 1;
            }
        }
        let updated_rows: HashSet<usize> = self
            .selected_rows
            .iter()
            .map(|&r| if r >= at_index { r + 1 } else { r })
            .collect();
        self.selected_rows = updated_rows;
    }

    pub fn on_row_deleted(&mut self, at_index: usize) {
        if let Some(ref mut ac) = self.active_cell {
            if ac.row > at_index {
                ac.row -= 1;
            } else if ac.row == at_index {
                // Mantém active_cell nos limites
            }
        }
        if let Some(ref mut anc) = self.anchor_cell {
            if anc.row > at_index {
                anc.row -= 1;
            }
        }
        if let Some(ref mut range) = self.primary_range {
            if range.min_row > at_index {
                range.min_row = range.min_row.saturating_sub(1);
            }
            if range.max_row >= at_index {
                range.max_row = range.max_row.saturating_sub(1);
            }
        }
        let mut updated_rows = HashSet::new();
        for &r in &self.selected_rows {
            if r < at_index {
                updated_rows.insert(r);
            } else if r > at_index {
                updated_rows.insert(r - 1);
            }
        }
        self.selected_rows = updated_rows;
    }

    pub fn on_col_inserted(&mut self, at_index: usize) {
        if let Some(ref mut ac) = self.active_cell {
            if ac.col >= at_index {
                ac.col += 1;
            }
        }
        if let Some(ref mut anc) = self.anchor_cell {
            if anc.col >= at_index {
                anc.col += 1;
            }
        }
        if let Some(ref mut range) = self.primary_range {
            if range.min_col >= at_index {
                range.min_col += 1;
            }
            if range.max_col >= at_index {
                range.max_col += 1;
            }
        }
        let updated_cols: HashSet<usize> = self
            .selected_cols
            .iter()
            .map(|&c| if c >= at_index { c + 1 } else { c })
            .collect();
        self.selected_cols = updated_cols;
    }

    pub fn on_col_deleted(&mut self, at_index: usize) {
        if let Some(ref mut ac) = self.active_cell {
            if ac.col > at_index {
                ac.col -= 1;
            }
        }
        if let Some(ref mut anc) = self.anchor_cell {
            if anc.col > at_index {
                anc.col -= 1;
            }
        }
        if let Some(ref mut range) = self.primary_range {
            if range.min_col > at_index {
                range.min_col = range.min_col.saturating_sub(1);
            }
            if range.max_col >= at_index {
                range.max_col = range.max_col.saturating_sub(1);
            }
        }
        let mut updated_cols = HashSet::new();
        for &c in &self.selected_cols {
            if c < at_index {
                updated_cols.insert(c);
            } else if c > at_index {
                updated_cols.insert(c - 1);
            }
        }
        self.selected_cols = updated_cols;
    }

    /// Desloca todos os índices de linhas da seleção por um delta positivo (ex.: ao receber novos eventos no topo)
    pub fn shift_rows(&mut self, delta: usize) {
        if delta == 0 || self.is_all_selected {
            return;
        }
        if let Some(ref mut ac) = self.active_cell {
            ac.row += delta;
        }
        if let Some(ref mut anc) = self.anchor_cell {
            anc.row += delta;
        }
        if let Some(ref mut range) = self.primary_range {
            range.min_row += delta;
            range.max_row += delta;
        }
        for r in &mut self.additional_ranges {
            r.min_row += delta;
            r.max_row += delta;
        }
        let old_rows: Vec<usize> = self.selected_rows.drain().collect();
        for r in old_rows {
            self.selected_rows.insert(r + delta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_cell_selection() {
        let mut model = CellSelectionModel::new();
        model.select_cell(2, 3);
        assert!(model.is_cell_selected(2, 3));
        assert!(!model.is_cell_selected(2, 4));
        assert!(!model.is_cell_selected(3, 3));
        assert_eq!(model.active_cell, Some(CellCoord::new(2, 3)));
    }

    #[test]
    fn test_range_selection() {
        let mut model = CellSelectionModel::new();
        model.select_range(1, 1, 3, 4);
        assert!(model.is_cell_selected(1, 1));
        assert!(model.is_cell_selected(2, 3));
        assert!(model.is_cell_selected(3, 4));
        assert!(!model.is_cell_selected(0, 0));
        assert!(!model.is_cell_selected(4, 4));
    }

    #[test]
    fn test_select_all() {
        let mut model = CellSelectionModel::new();
        model.select_all();
        assert!(model.is_all_selected);
        assert!(model.is_cell_selected(999, 999));
        assert!(model.is_row_selected(10));
        assert!(model.is_col_selected(5));
    }

    #[test]
    fn test_select_row_and_column() {
        let mut model = CellSelectionModel::new();
        model.select_row(4, false, false, 10);
        assert!(model.is_row_selected(4));
        assert!(model.is_cell_selected(4, 0));
        assert!(model.is_cell_selected(4, 9));
        assert!(!model.is_cell_selected(3, 0));

        model.select_col(2, false, false, 10);
        assert!(model.is_col_selected(2));
        assert!(model.is_cell_selected(0, 2));
        assert!(model.is_cell_selected(9, 2));
        assert!(!model.is_cell_selected(0, 3));
    }

    #[test]
    fn test_expand_selection() {
        let mut model = CellSelectionModel::new();
        model.select_cell(2, 2);
        model.expand_to(4, 5);
        assert!(model.is_cell_selected(2, 2));
        assert!(model.is_cell_selected(3, 3));
        assert!(model.is_cell_selected(4, 5));
        assert!(!model.is_cell_selected(5, 5));
    }

    #[test]
    fn test_row_insert_and_delete() {
        let mut model = CellSelectionModel::new();
        model.select_cell(3, 2);
        model.on_row_inserted(2);
        assert_eq!(model.active_cell, Some(CellCoord::new(4, 2)));

        model.on_row_deleted(2);
        assert_eq!(model.active_cell, Some(CellCoord::new(3, 2)));
    }
}
