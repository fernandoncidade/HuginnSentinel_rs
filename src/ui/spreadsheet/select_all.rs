// Módulo do Botão Selecionar Tudo (SelectAllButton)
// Posicionado na interseção superior esquerda entre cabeçalhos de linhas e colunas (canto da planilha Excel).

use egui::{Color32, Id, Pos2, Rect, Response, Stroke, Ui, Vec2};

pub struct SelectAllButton {
    pub is_all_selected: bool,
    pub selection_color: Option<Color32>,
}

impl SelectAllButton {
    pub fn new(is_all_selected: bool) -> Self {
        Self {
            is_all_selected,
            selection_color: None,
        }
    }

    pub fn with_color(is_all_selected: bool, color: Option<Color32>) -> Self {
        Self {
            is_all_selected,
            selection_color: color,
        }
    }

    pub fn show(&self, ui: &mut Ui, size: Vec2) -> Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        self.paint_rect(ui, rect, &response);
        response
    }

    pub fn show_at_rect(&self, ui: &mut Ui, rect: Rect) -> Response {
        let id = Id::unique("spreadsheet_select_all_corner_btn");
        let response = ui.interact(rect, id, egui::Sense::click());
        self.paint_rect(ui, rect, &response);
        response
    }

    fn paint_rect(&self, ui: &mut Ui, rect: Rect, response: &Response) {
        let dark_mode = ui.visuals().dark_mode;
        let (bg_color, border_color, tri_color) = if self.is_all_selected {
            let sel_col = self.selection_color.unwrap_or_else(|| Color32::from_rgb(16, 124, 65));
            (
                sel_col,
                sel_col,
                Color32::WHITE,
            )
        } else if response.hovered() {
            if dark_mode {
                (
                    Color32::from_rgb(48, 52, 64),
                    Color32::from_rgb(70, 78, 96),
                    Color32::from_rgb(180, 190, 210),
                )
            } else {
                (
                    Color32::from_rgb(230, 233, 238),
                    Color32::from_rgb(195, 200, 210),
                    Color32::from_rgb(100, 108, 120),
                )
            }
        } else {
            if dark_mode {
                (
                    Color32::from_rgb(34, 36, 44),
                    Color32::from_rgb(54, 58, 70),
                    Color32::from_rgb(140, 150, 175),
                )
            } else {
                (
                    Color32::from_rgb(243, 244, 246),
                    Color32::from_rgb(214, 218, 224),
                    Color32::from_rgb(130, 138, 150),
                )
            }
        };

        // Desenha fundo do cabeçalho de canto
        ui.painter().rect_filled(rect, 0_u8, bg_color);
        ui.painter().rect_stroke(
            rect,
            0_u8,
            Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );

        // Desenha triângulo clássico do Excel no canto inferior direito do botão
        let p1 = Pos2::new(rect.right() - 4.0, rect.bottom() - 4.0);
        let p2 = Pos2::new(rect.right() - 14.0, rect.bottom() - 4.0);
        let p3 = Pos2::new(rect.right() - 4.0, rect.bottom() - 14.0);

        ui.painter().add(egui::Shape::convex_polygon(
            vec![p1, p2, p3],
            tri_color,
            Stroke::NONE,
        ));
    }
}
