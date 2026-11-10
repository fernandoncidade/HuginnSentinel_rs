// Módulo de Navegação por Teclado e Mouse (GridNavigation)
// Implementa navegação bidimensional inspirada no Excel: setas, Shift+setas, Tab, Enter, Home, End, PageUp/Down, Ctrl+A.

use super::cell_selection::CellSelectionModel;
use egui::{Key, Ui};

pub struct GridNavigation;

impl GridNavigation {
    pub fn handle_keyboard(
        ui: &mut Ui,
        selection: &mut CellSelectionModel,
        total_rows: usize,
        total_cols: usize,
        rows_per_page: usize,
    ) -> bool {
        if total_rows == 0 || total_cols == 0 {
            return false;
        }

        let mut changed = false;

        ui.input(|i| {
            let ctrl = i.modifiers.command || i.modifiers.ctrl;
            let shift = i.modifiers.shift;

            // Ctrl + A -> Selecionar Tudo
            if ctrl && !shift && i.key_pressed(Key::A) {
                selection.select_all();
                changed = true;
                return;
            }

            // Garante célula ativa inicial caso não exista
            let current = selection
                .active_cell
                .unwrap_or(super::cell_selection::CellCoord::new(0, 0));
            let mut next_row = current.row;
            let mut next_col = current.col;

            // Ctrl + Home -> Topo esquerdo (0, 0)
            if ctrl && !shift && i.key_pressed(Key::Home) {
                next_row = 0;
                next_col = 0;
            }
            // Ctrl + End -> Última célula da planilha
            else if ctrl && !shift && i.key_pressed(Key::End) {
                next_row = total_rows.saturating_sub(1);
                next_col = total_cols.saturating_sub(1);
            }
            // Home -> Primeira coluna da linha atual
            else if !ctrl && !shift && i.key_pressed(Key::Home) {
                next_col = 0;
            }
            // End -> Última coluna da linha atual
            else if !ctrl && !shift && i.key_pressed(Key::End) {
                next_col = total_cols.saturating_sub(1);
            }
            // Page Up -> Rola para cima 1 página
            else if i.key_pressed(Key::PageUp) {
                next_row = next_row.saturating_sub(rows_per_page.max(5));
            }
            // Page Down -> Rola para baixo 1 página
            else if i.key_pressed(Key::PageDown) {
                next_row = (next_row + rows_per_page.max(5)).min(total_rows.saturating_sub(1));
            }
            // Tab / Shift + Tab
            else if i.key_pressed(Key::Tab) {
                if shift {
                    if next_col > 0 {
                        next_col -= 1;
                    } else if next_row > 0 {
                        next_row -= 1;
                        next_col = total_cols.saturating_sub(1);
                    }
                } else {
                    if next_col + 1 < total_cols {
                        next_col += 1;
                    } else if next_row + 1 < total_rows {
                        next_row += 1;
                        next_col = 0;
                    }
                }
            }
            // Enter / Shift + Enter
            else if i.key_pressed(Key::Enter) {
                if shift {
                    if next_row > 0 {
                        next_row -= 1;
                    }
                } else if next_row + 1 < total_rows {
                    next_row += 1;
                }
            }
            // Setas direcionais (Arrow keys)
            else if i.key_pressed(Key::ArrowUp) {
                if next_row > 0 {
                    next_row -= 1;
                }
            } else if i.key_pressed(Key::ArrowDown) {
                if next_row + 1 < total_rows {
                    next_row += 1;
                }
            } else if i.key_pressed(Key::ArrowLeft) {
                if next_col > 0 {
                    next_col -= 1;
                }
            } else if i.key_pressed(Key::ArrowRight) {
                if next_col + 1 < total_cols {
                    next_col += 1;
                }
            }

            if next_row != current.row || next_col != current.col {
                if shift {
                    // Shift pressionado expande o intervalo retangular a partir da âncora
                    selection.expand_to(next_row, next_col);
                } else {
                    selection.select_cell(next_row, next_col);
                }
                changed = true;
            }
        });

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_bounds() {
        let mut selection = CellSelectionModel::new();
        selection.select_cell(0, 0);

        // Testa expansão de seleção programática simulando setas
        selection.expand_to(3, 4);
        assert!(selection.is_cell_selected(0, 0));
        assert!(selection.is_cell_selected(3, 4));
        assert_eq!(
            selection.active_cell,
            Some(super::super::cell_selection::CellCoord::new(3, 4))
        );
    }
}
