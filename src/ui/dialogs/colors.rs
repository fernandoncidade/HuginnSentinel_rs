use egui::{Color32, RichText, Vec2};

use crate::core::types::OperationType;
use crate::locale::Localizer;
use crate::services::gerenciador_tabela::GerenciadorTabela;

pub struct ColorDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
}

impl Default for ColorDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
        }
    }
}

impl ColorDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, gt: &mut GerenciadorTabela) {
        if !self.aberto {
            return;
        }

        #[allow(deprecated)]
        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        super::show_modeless_dialog(
            ctx,
            "dialog_colors",
            format!("🎨 {}", loc.t("colors_config")),
            [480.0, 320.0],
            [380.0, 250.0],
            &mut state,
            |ui, fechar| {
                ui.label(loc.t("colors_description"));
                ui.add_space(4.0);
                ui.separator();

                let footer_h = 36.0;
                let scroll_h = (ui.available_height() - footer_h).max(100.0);

                egui::ScrollArea::vertical()
                    .id_salt("colors_dialog_scroll")
                    .max_height(scroll_h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for op in OperationType::all() {
                            let (bg, fg) = gt.obter_cores_operacao(*op);
                            let mut color = bg;

                            ui.horizontal(|ui| {
                                let label = loc.traduzir_tipo_operacao(op.as_str());
                                ui.label(RichText::new(&label).strong().size(13.0));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Seletor de cores customizado e localizado com suporte a HTML, U8, 📋 e ❌
                                    if crate::ui::widgets::render_custom_color_edit_button(
                                        ui,
                                        ui.scope_id().with(op.as_str()),
                                        &mut color,
                                        loc,
                                    )
                                    .changed()
                                    {
                                        gt.set_cor_operacao(*op, color);
                                    }

                                    ui.add_space(8.0);

                                    // Chip de demonstração proporcional e com boa legibilidade
                                    let chip_rect = ui.allocate_space(Vec2::new(130.0, 24.0)).1;
                                    ui.painter().rect_filled(chip_rect, 4_u8, bg);
                                    ui.painter().rect_stroke(
                                        chip_rect,
                                        4_u8,
                                        egui::Stroke::new(1.0, Color32::from_white_alpha(35)),
                                        egui::StrokeKind::Inside,
                                    );
                                    ui.painter().text(
                                        chip_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        &label,
                                        egui::FontId::proportional(11.5),
                                        fg,
                                    );
                                });
                            });
                            ui.separator();
                        }
                    });

                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if crate::ui::widgets::render_centered_emoji_button(
                        ui,
                        "",
                        loc.t("reset_defaults"),
                    )
                    .clicked()
                    {
                        gt.resetar_cores_padrao();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if crate::ui::widgets::render_centered_emoji_button_sized(
                            ui,
                            "",
                            loc.t("close"),
                            Vec2::new(85.0, 24.0),
                        )
                        .clicked()
                        {
                            *fechar = true;
                        }
                    });
                });
            },
        );

        self.aberto = state.aberto;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
        self.elevar_z_order = state.elevar_z_order;
    }
}
