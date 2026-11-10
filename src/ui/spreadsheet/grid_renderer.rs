// Módulo de Renderização da Planilha (GridRenderer) — Inspiração Microsoft Excel & C++ Original
// Renderiza cabeçalhos estilizados, linhas de grade, célula ativa com borda destacada e intervalos de seleção.
// Suporte integral e dinâmico a temas Claro (Excel Classic) e Escuro (Excel Dark).

use egui::{Color32, FontId, Pos2, Rect, Stroke, Ui, Vec2};

pub struct GridRenderer;

impl GridRenderer {
    /// Renderiza o cabeçalho de uma coluna (Título + Indicador de ordenação)
    /// Sem as letras "A", "B", "C", com alinhamento vertical centralizado profissional.
    pub fn render_column_header(
        ui: &mut Ui,
        rect: Rect,
        title: &str,
        is_selected: bool,
        is_sorted: bool,
        sort_ascending: bool,
        is_hovered: bool,
        selection_color: Option<Color32>,
    ) {
        let dark_mode = ui.visuals().dark_mode;

        let (bg, border_color, title_color) = if is_selected {
            let sel_col = selection_color.unwrap_or_else(|| Color32::from_rgb(16, 124, 65));
            if dark_mode {
                (
                    Color32::from_rgba_unmultiplied(sel_col.r(), sel_col.g(), sel_col.b(), 65),
                    sel_col,
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(sel_col.r(), sel_col.g(), sel_col.b(), 35),
                    sel_col,
                    Color32::from_rgb(
                        (sel_col.r() as f32 * 0.7).clamp(0.0, 255.0) as u8,
                        (sel_col.g() as f32 * 0.7).clamp(0.0, 255.0) as u8,
                        (sel_col.b() as f32 * 0.7).clamp(0.0, 255.0) as u8,
                    ),
                )
            }
        } else if is_sorted {
            if dark_mode {
                (
                    Color32::from_rgb(36, 48, 70),
                    Color32::from_rgb(60, 120, 200),
                    Color32::from_rgb(130, 200, 255),
                )
            } else {
                (
                    Color32::from_rgb(230, 240, 255),
                    Color32::from_rgb(40, 120, 220),
                    Color32::from_rgb(16, 80, 180),
                )
            }
        } else if is_hovered {
            if dark_mode {
                (
                    Color32::from_rgb(46, 50, 62),
                    Color32::from_rgb(70, 78, 96),
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::from_rgb(232, 235, 240),
                    Color32::from_rgb(195, 200, 210),
                    Color32::from_rgb(20, 24, 32),
                )
            }
        } else {
            if dark_mode {
                (
                    Color32::from_rgb(36, 38, 46),
                    Color32::from_rgb(54, 58, 70),
                    Color32::from_rgb(230, 235, 245),
                )
            } else {
                (
                    Color32::from_rgb(243, 244, 246),
                    Color32::from_rgb(214, 218, 224),
                    Color32::from_rgb(40, 44, 52),
                )
            }
        };

        // Fundo e borda do cabeçalho
        ui.painter().rect_filled(rect, 0_u8, bg);
        ui.painter().rect_stroke(
            rect,
            0_u8,
            Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );

        let seta = if is_sorted {
            if sort_ascending {
                " ▲"
            } else {
                " ▼"
            }
        } else {
            ""
        };

        // Título Traduzido da Coluna perfeitamente centralizado verticalmente
        let text_max_chars = ((rect.width() - 14.0) / 7.2).max(1.0) as usize;
        let display_title = format!("{}{}", truncate_text(title, text_max_chars), seta);

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            display_title,
            FontId::proportional(12.0),
            title_color,
        );
    }

    /// Renderiza o cabeçalho de uma linha com numeração sequencial (1, 2, 3...)
    pub fn render_row_header(
        ui: &mut Ui,
        rect: Rect,
        row_number: usize,
        is_selected: bool,
        is_active: bool,
        is_hovered: bool,
        selection_color: Option<Color32>,
    ) {
        let dark_mode = ui.visuals().dark_mode;

        let (bg, fg, border_color) = if is_selected {
            let sel_col = selection_color.unwrap_or_else(|| Color32::from_rgb(16, 124, 65));
            if dark_mode {
                (
                    Color32::from_rgba_unmultiplied(sel_col.r(), sel_col.g(), sel_col.b(), 65),
                    Color32::WHITE,
                    sel_col,
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(sel_col.r(), sel_col.g(), sel_col.b(), 35),
                    Color32::from_rgb(
                        (sel_col.r() as f32 * 0.7).clamp(0.0, 255.0) as u8,
                        (sel_col.g() as f32 * 0.7).clamp(0.0, 255.0) as u8,
                        (sel_col.b() as f32 * 0.7).clamp(0.0, 255.0) as u8,
                    ),
                    sel_col,
                )
            }
        } else if is_active {
            if dark_mode {
                (
                    Color32::from_rgb(38, 54, 80),
                    Color32::from_rgb(130, 200, 255),
                    Color32::from_rgb(60, 120, 200),
                )
            } else {
                (
                    Color32::from_rgb(220, 235, 255),
                    Color32::from_rgb(20, 80, 180),
                    Color32::from_rgb(40, 120, 220),
                )
            }
        } else if is_hovered {
            if dark_mode {
                (
                    Color32::from_rgb(44, 48, 60),
                    Color32::WHITE,
                    Color32::from_rgb(65, 72, 90),
                )
            } else {
                (
                    Color32::from_rgb(230, 233, 238),
                    Color32::from_rgb(20, 24, 32),
                    Color32::from_rgb(195, 200, 210),
                )
            }
        } else {
            if dark_mode {
                (
                    Color32::from_rgb(34, 36, 44),
                    Color32::from_rgb(150, 160, 180),
                    Color32::from_rgb(52, 56, 68),
                )
            } else {
                (
                    Color32::from_rgb(243, 244, 246),
                    Color32::from_rgb(100, 108, 120),
                    Color32::from_rgb(214, 218, 224),
                )
            }
        };

        ui.painter().rect_filled(rect, 0_u8, bg);
        ui.painter().rect_stroke(
            rect,
            0_u8,
            Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            row_number.to_string(),
            FontId::proportional(11.5),
            fg,
        );
    }

    /// Desenha o contorno clássico personalizável (padrão Verde Excel) na célula ativa
    pub fn draw_active_cell_indicator(ui: &mut Ui, cell_rect: Rect, stroke_color: Color32) {
        ui.painter().rect_stroke(
            cell_rect,
            0_u8,
            Stroke::new(2.0, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Alça clássica de preenchimento automático no canto inferior direito
        let handle_rect = Rect::from_min_size(
            Pos2::new(cell_rect.right() - 4.0, cell_rect.bottom() - 4.0),
            Vec2::new(4.0, 4.0),
        );
        ui.painter().rect_filled(handle_rect, 0_u8, stroke_color);
    }

    /// Desenha a caixa de preenchimento suave do intervalo retangular selecionado
    pub fn draw_range_selection(
        ui: &mut Ui,
        range_rect: Rect,
        fill_color: Color32,
        stroke_color: Color32,
    ) {
        ui.painter().rect_filled(range_rect, 0_u8, fill_color);
        ui.painter().rect_stroke(
            range_rect,
            0_u8,
            Stroke::new(1.5, stroke_color),
            egui::StrokeKind::Inside,
        );
    }

    /// Desenha as linhas de grade sutis da planilha adaptadas ao tema claro/escuro
    pub fn draw_grid_lines(ui: &mut Ui, rect: Rect) {
        let dark_mode = ui.visuals().dark_mode;
        let grid_color = if dark_mode {
            Color32::from_rgb(48, 52, 64)
        } else {
            Color32::from_rgb(226, 229, 234) // Linhas sutis e nítidas estilo Excel
        };
        let grid_stroke = Stroke::new(0.8, grid_color);
        ui.painter()
            .rect_stroke(rect, 0_u8, grid_stroke, egui::StrokeKind::Inside);
    }
}

pub fn truncate_text(s: &str, max_chars: usize) -> &str {
    if s.chars().count() <= max_chars {
        return s;
    }
    if max_chars <= 2 {
        return s;
    }
    match s.char_indices().nth(max_chars - 1) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
