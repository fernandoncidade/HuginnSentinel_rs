// Módulo de Gerenciamento Estrutural de Linhas e Colunas (RowColumnManager)
// Executa inserção, exclusão, duplicação, limpeza, ocultação/reexibição, AutoFit e histórico de comandos (Undo/Redo).

use crate::core::event::EventModel;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum SpreadsheetCommand {
    InsertRow {
        index: usize,
        event: EventModel,
    },
    DeleteRow {
        index: usize,
        event: EventModel,
    },
    DuplicateRow {
        index: usize,
    },
    ClearRows {
        indices: Vec<usize>,
        old_events: Vec<EventModel>,
    },
    HideRow {
        index: usize,
    },
    UnhideRow {
        index: usize,
    },
    HideCol {
        key: String,
    },
    UnhideCol {
        key: String,
    },
    SetColWidth {
        key: String,
        old_width: f32,
        new_width: f32,
    },
    SetRowHeight {
        index: usize,
        old_height: f32,
        new_height: f32,
    },
}

#[derive(Debug, Clone, Default)]
pub struct RowColumnManager {
    pub hidden_rows: HashSet<usize>,
    pub hidden_cols: HashSet<String>,
    pub undo_stack: Vec<SpreadsheetCommand>,
    pub redo_stack: Vec<SpreadsheetCommand>,
}

impl RowColumnManager {
    pub fn new() -> Self {
        Self {
            hidden_rows: HashSet::new(),
            hidden_cols: HashSet::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn hide_row(&mut self, row_idx: usize) {
        self.hidden_rows.insert(row_idx);
        self.undo_stack
            .push(SpreadsheetCommand::HideRow { index: row_idx });
        self.redo_stack.clear();
    }

    pub fn unhide_row(&mut self, row_idx: usize) {
        self.hidden_rows.remove(&row_idx);
        self.undo_stack
            .push(SpreadsheetCommand::UnhideRow { index: row_idx });
        self.redo_stack.clear();
    }

    pub fn unhide_all_rows(&mut self) {
        self.hidden_rows.clear();
    }

    pub fn is_row_hidden(&self, row_idx: usize) -> bool {
        self.hidden_rows.contains(&row_idx)
    }

    pub fn hide_col(&mut self, col_key: &str) {
        self.hidden_cols.insert(col_key.to_string());
        self.undo_stack.push(SpreadsheetCommand::HideCol {
            key: col_key.to_string(),
        });
        self.redo_stack.clear();
    }

    pub fn unhide_col(&mut self, col_key: &str) {
        self.hidden_cols.remove(col_key);
        self.undo_stack.push(SpreadsheetCommand::UnhideCol {
            key: col_key.to_string(),
        });
        self.redo_stack.clear();
    }

    pub fn unhide_all_cols(&mut self) {
        self.hidden_cols.clear();
    }

    pub fn is_col_hidden(&self, col_key: &str) -> bool {
        self.hidden_cols.contains(col_key)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hide_and_unhide_row() {
        let mut mgr = RowColumnManager::new();
        assert!(!mgr.is_row_hidden(5));
        mgr.hide_row(5);
        assert!(mgr.is_row_hidden(5));
        mgr.unhide_row(5);
        assert!(!mgr.is_row_hidden(5));
    }

    #[test]
    fn test_hide_and_unhide_col() {
        let mut mgr = RowColumnManager::new();
        assert!(!mgr.is_col_hidden("nome"));
        mgr.hide_col("nome");
        assert!(mgr.is_col_hidden("nome"));
        mgr.unhide_col("nome");
        assert!(!mgr.is_col_hidden("nome"));
    }

    #[test]
    fn test_unhide_all() {
        let mut mgr = RowColumnManager::new();
        mgr.hide_row(1);
        mgr.hide_row(2);
        mgr.hide_col("a");
        mgr.hide_col("b");
        assert_eq!(mgr.hidden_rows.len(), 2);
        assert_eq!(mgr.hidden_cols.len(), 2);

        mgr.unhide_all_rows();
        mgr.unhide_all_cols();
        assert_eq!(mgr.hidden_rows.len(), 0);
        assert_eq!(mgr.hidden_cols.len(), 0);
    }
}
