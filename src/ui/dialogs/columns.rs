use egui::{RichText, ScrollArea, Vec2};

use crate::locale::Localizer;
use crate::services::gerenciador_colunas::GerenciadorColunas;

pub struct ColumnDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    filtro_busca: String,
}

impl Default for ColumnDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            filtro_busca: String::new(),
        }
    }
}

impl ColumnDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, gc: &mut GerenciadorColunas) {
        if !self.aberto {
            return;
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        super::show_modeless_dialog(
            ctx,
            "dialog_columns",
            format!("📑 {}", loc.t("column_manager_title")),
            [420.0, 520.0],
            [340.0, 300.0],
            &mut state,
            |ui, fechar| {
                // 1. Campo de busca ocupando toda a largura útil disponível
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    let clear_btn_w = if self.filtro_busca.is_empty() { 0.0 } else { 26.0 };
                    let input_w = (ui.available_width() - clear_btn_w).max(140.0);
                    ui.add_sized(
                        [input_w, 24.0],
                        egui::TextEdit::singleline(&mut self.filtro_busca)
                            .hint_text(loc.t("filter_column_by_name")),
                    );
                    if !self.filtro_busca.is_empty() {
                        if ui.small_button("✖").clicked() {
                            self.filtro_busca.clear();
                        }
                    }
                });

                ui.add_space(4.0);

                // 2. Botões de ação em lote com centralização vertical no botão
                ui.horizontal(|ui| {
                    if crate::ui::widgets::render_centered_emoji_button(ui, "", loc.t("select_all")).clicked() {
                        for col in gc.colunas().to_vec() {
                            gc.set_visibilidade(col.key, true);
                        }
                        gc.salvar_configuracoes();
                    }
                    if crate::ui::widgets::render_centered_emoji_button(ui, "", loc.t("deselect_all")).clicked() {
                        for col in gc.colunas().to_vec() {
                            gc.set_visibilidade(col.key, false);
                        }
                        // Mantém ao menos nome e tipo visíveis
                        gc.set_visibilidade("nome", true);
                        gc.set_visibilidade("tipo_operacao", true);
                        gc.salvar_configuracoes();
                    }
                    if crate::ui::widgets::render_centered_emoji_button(ui, "", loc.t("reset_defaults")).clicked() {
                        gc.resetar_padrao();
                    }
                });

                ui.separator();

                // 3. Área de rolagem vertical ajustada ocupando toda a largura e altura disponíveis
                let footer_h = 36.0;
                let scroll_h = (ui.available_height() - footer_h).max(120.0);

                let colunas = gc.colunas().to_vec();
                let busca_lower = self.filtro_busca.to_lowercase();

                ScrollArea::vertical()
                    .id_salt("dialog_columns_scroll")
                    .max_height(scroll_h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for col in &colunas {
                            let nome_traduzido = loc.get_text(col.translation_key);
                            if !self.filtro_busca.is_empty()
                                && !nome_traduzido.to_lowercase().contains(&busca_lower)
                                && !col.key.contains(&busca_lower)
                            {
                                continue;
                            }

                            let mut visivel = col.visivel;
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut visivel, &nome_traduzido).changed() {
                                    gc.set_visibilidade(col.key, visivel);
                                    gc.salvar_configuracoes();
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("({})", col.key))
                                            .size(11.0)
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                });
                            });
                        }
                    });

                ui.separator();

                // 4. Rodapé informativo com contador de colunas visíveis e botão Fechar alinhado à direita
                ui.horizontal(|ui| {
                    let total_visiveis = colunas.iter().filter(|c| c.visivel).count();
                    ui.label(
                        RichText::new(format!("{}: {} / {}", loc.t("columns"), total_visiveis, colunas.len()))
                            .size(11.5)
                            .color(ui.visuals().weak_text_color()),
                    );

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
        self.elevar_z_order = state.elevar_z_order;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
    }
}
