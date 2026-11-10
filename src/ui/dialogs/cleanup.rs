use egui::{Color32, Context, RichText, Ui, Vec2};
use crate::locale::Localizer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimpezaAlvo {
    Ambos,
    BancoDados,
    TabelaUI,
    BancoSeguranca,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanupStep {
    SelecionarAlvo,
    Confirmacao(LimpezaAlvo),
    Sucesso(LimpezaAlvo),
    Erro(String),
}

pub struct CleanupDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub step: CleanupStep,
    pub aberto_direto: bool,
    ultimo_step: Option<CleanupStep>,
}

impl Default for CleanupDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            step: CleanupStep::SelecionarAlvo,
            aberto_direto: false,
            ultimo_step: None,
        }
    }
}

fn render_cleanup_button_opt(
    ui: &mut Ui,
    emoji: &str,
    text: impl AsRef<str>,
    width: f32,
    height: f32,
    fill_color: Option<Color32>,
    text_color_override: Option<Color32>,
) -> egui::Response {
    let text = text.as_ref();
    let font_id = egui::FontId::proportional(12.0);
    let emoji_font_id = egui::FontId::proportional(13.0);

    let emoji_galley = if !emoji.is_empty() {
        Some(ui.painter().layout_no_wrap(emoji.to_string(), emoji_font_id, Color32::PLACEHOLDER))
    } else {
        None
    };
    let text_galley = if !text.is_empty() {
        Some(ui.painter().layout_no_wrap(text.to_string(), font_id, Color32::PLACEHOLDER))
    } else {
        None
    };

    let spacing = if emoji_galley.is_some() && text_galley.is_some() { 6.0 } else { 0.0 };
    let emoji_w = emoji_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let text_w = text_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);

    let content_width = emoji_w + spacing + text_w;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let bg_fill = if let Some(fill) = fill_color {
            if response.is_pointer_button_down_on() {
                Color32::from_rgb(180, 30, 30)
            } else if response.hovered() {
                Color32::from_rgb(230, 60, 60)
            } else {
                fill
            }
        } else {
            visuals.bg_fill
        };

        let stroke = if fill_color.is_some() {
            egui::Stroke::new(1.0, Color32::from_white_alpha(40))
        } else {
            visuals.bg_stroke
        };

        let text_color = text_color_override.unwrap_or_else(|| visuals.text_color());

        ui.painter().rect_filled(rect, 4_u8, bg_fill);
        ui.painter().rect_stroke(rect, 4_u8, stroke, egui::StrokeKind::Inside);

        let center_x = rect.center().x;
        let center_y = rect.center().y;

        // Centralização milimétrica no eixo vertical da caixa do botão (X = center_x)
        let start_x = center_x - content_width / 2.0;

        let mut cur_x = start_x;
        if let Some(eg) = emoji_galley {
            let emoji_visual_y = if eg.mesh_bounds.is_positive() {
                eg.mesh_bounds.center().y
            } else {
                eg.rect.center().y
            };
            let emoji_pos = egui::Pos2::new(cur_x, center_y - emoji_visual_y);
            ui.painter().galley(emoji_pos, eg, text_color);
            cur_x += emoji_w + spacing;
        }

        if let Some(tg) = text_galley {
            let text_visual_y = if tg.mesh_bounds.is_positive() {
                tg.mesh_bounds.center().y
            } else {
                tg.rect.center().y
            };
            let text_pos = egui::Pos2::new(cur_x, center_y - text_visual_y);
            ui.painter().galley(text_pos, tg, text_color);
        }
    }

    response
}

fn render_cleanup_button(
    ui: &mut Ui,
    emoji: &str,
    text: impl AsRef<str>,
    width: f32,
    height: f32,
) -> egui::Response {
    render_cleanup_button_opt(ui, emoji, text, width, height, None, None)
}

impl CleanupDialog {
    pub fn abrir_selecao(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        self.aberto_direto = false;
        self.step = CleanupStep::SelecionarAlvo;
        self.ultimo_step = None;
    }

    pub fn abrir_alvo(&mut self, alvo: LimpezaAlvo) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        self.aberto_direto = true;
        self.step = CleanupStep::Confirmacao(alvo);
        self.ultimo_step = None;
    }

    pub fn fechar(&mut self) {
        self.aberto = false;
        self.minimizado = false;
        self.step = CleanupStep::SelecionarAlvo;
        self.aberto_direto = false;
        self.ultimo_step = None;
    }

    pub fn processar_resultado(&mut self, alvo: LimpezaAlvo, res: Result<(), String>) {
        match res {
            Ok(()) => self.step = CleanupStep::Sucesso(alvo),
            Err(e) => self.step = CleanupStep::Erro(e),
        }
    }

    /// Calcula o menor comprimento do eixo horizontal dos botões baseado na maior string de UI,
    /// e calcula as dimensões da janela (largura e altura mínima específica de cada passo).
    pub fn calcular_geometria(
        ctx: &Context,
        loc: &Localizer,
        step: &CleanupStep,
    ) -> (f32, f32, f32) {
        let font_id = egui::FontId::proportional(12.0);
        let emoji_font_id = egui::FontId::proportional(13.0);

        let itens = [
            ("🧹", loc.t("clear_ui_db_option")),
            ("🗄️", loc.t("clear_db_option")),
            ("📋", loc.t("clear_ui_table_option")),
            ("🛡️", loc.t("security_clear_database")),
            ("❌", loc.t("cancel")),
        ];

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut max_content_w: f32 = 0.0;
            ctx.fonts_mut(|fonts| {
                for (emoji, texto) in &itens {
                    let w_emoji = if !emoji.is_empty() {
                        fonts
                            .layout_no_wrap(emoji.to_string(), emoji_font_id.clone(), Color32::PLACEHOLDER)
                            .size()
                            .x
                    } else {
                        0.0
                    };
                    let w_text = if !texto.is_empty() {
                        fonts
                            .layout_no_wrap(texto.clone(), font_id.clone(), Color32::PLACEHOLDER)
                            .size()
                            .x
                    } else {
                        0.0
                    };
                    let spacing = if w_emoji > 0.0 && w_text > 0.0 { 6.0 } else { 0.0 };
                    let total = w_emoji + spacing + w_text;
                    if total > max_content_w {
                        max_content_w = total;
                    }
                }
            });

            let prompt_w = ctx.fonts_mut(|fonts| {
                let w_ico = fonts
                    .layout_no_wrap("🗑️".to_string(), egui::FontId::proportional(18.0), Color32::PLACEHOLDER)
                    .size()
                    .x;
                let w_txt = fonts
                    .layout_no_wrap(
                        loc.t("clear_target_prompt"),
                        egui::FontId::proportional(13.0),
                        Color32::PLACEHOLDER,
                    )
                    .size()
                    .x;
                w_ico + 6.0 + w_txt
            });

            // Menor comprimento possível para a caixa dos botões sem vazar ou estourar sobre a string mais longa
            let btn_w = (max_content_w + 16.0).ceil();

            // O comprimento horizontal da janela acompanha o comprimento dos botões mais margens internas
            let win_w = (btn_w.max(prompt_w) + 28.0).max(340.0).ceil();

            (btn_w, win_w)
        }));

        let (btn_w, win_w) = match result {
            Ok(dims) => dims,
            Err(_) => {
                let max_chars = itens
                    .iter()
                    .map(|(e, t)| e.chars().count() + t.chars().count())
                    .max()
                    .unwrap_or(45);
                let btn_w = ((max_chars as f32) * 7.5 + 24.0).max(280.0).ceil();
                let win_w = (btn_w + 28.0).max(340.0).ceil();
                (btn_w, win_w)
            }
        };

        // Comprimento do eixo vertical otimizado para o menor comprimento possível de cada janela,
        // garantindo um pequeno espaço elegante entre os botões inferiores e a borda inferior da janela,
        // sem vazar ou estourar sobre os widgets internos.
        let win_h = match step {
            CleanupStep::SelecionarAlvo => 226.0,
            CleanupStep::Confirmacao(LimpezaAlvo::Ambos) => 198.0,
            CleanupStep::Confirmacao(_) => 168.0,
            CleanupStep::Sucesso(_) => 126.0,
            CleanupStep::Erro(_) => 132.0,
        };

        (btn_w, win_w, win_h)
    }

    pub fn mostrar(
        &mut self,
        ctx: &Context,
        loc: &Localizer,
        mut on_confirm: impl FnMut(LimpezaAlvo),
    ) {
        if !self.aberto {
            return;
        }

        let (btn_w, win_w, win_h) = Self::calcular_geometria(ctx, loc, &self.step);
        let win_size = [win_w, win_h];
        let min_win_size = [win_w.min(300.0), 100.0];

        let sub_titulo = match &self.step {
            CleanupStep::SelecionarAlvo => String::new(),
            CleanupStep::Confirmacao(_) => format!(": ⚠️ {}", loc.t("confirm")),
            CleanupStep::Sucesso(_) => format!(": ✅ {}", loc.t("success")),
            CleanupStep::Erro(_) => format!(": ❌ {}", loc.t("error")),
        };
        let titulo_janela = format!("🗑️ {}{}", loc.t("cleanup_system_title"), sub_titulo);

        // Atualiza as dimensões e título do viewport no SO caso o passo tenha mudado
        let viewport_id = egui::ViewportId::from_hash_of("dialog_cleanup");
        if self.ultimo_step.as_ref() != Some(&self.step) {
            self.ultimo_step = Some(self.step.clone());
            ctx.send_viewport_cmd_to(
                viewport_id,
                egui::ViewportCommand::InnerSize(Vec2::new(win_w, win_h)),
            );
            ctx.send_viewport_cmd_to(
                viewport_id,
                egui::ViewportCommand::Title(titulo_janela.clone()),
            );
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
            "dialog_cleanup",
            titulo_janela,
            win_size,
            min_win_size,
            &mut state,
            |ui, fechar| {
                if !self.aberto {
                    *fechar = true;
                    return;
                }

                match self.step {
                    CleanupStep::SelecionarAlvo => {
                        self.render_selecionar_alvo(ui, loc, btn_w, fechar);
                    }
                    CleanupStep::Confirmacao(alvo) => {
                        self.render_confirmacao(ui, loc, alvo, fechar, &mut on_confirm);
                    }
                    CleanupStep::Sucesso(alvo) => {
                        self.render_sucesso(ui, loc, alvo, fechar);
                    }
                    CleanupStep::Erro(ref msg) => {
                        let erro_str = msg.clone();
                        self.render_erro(ui, loc, &erro_str, fechar);
                    }
                }

                if *fechar || !self.aberto {
                    *fechar = true;
                    self.fechar();
                }
            },
        );

        if !state.aberto || !self.aberto {
            self.aberto = false;
            state.aberto = false;
        } else {
            self.aberto = state.aberto;
        }
        self.elevar_z_order = state.elevar_z_order;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
    }

    fn render_selecionar_alvo(
        &mut self,
        ui: &mut Ui,
        loc: &Localizer,
        btn_w: f32,
        fechar: &mut bool,
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🗑️").size(18.0));
            ui.label(RichText::new(loc.t("clear_target_prompt")).strong().size(13.0));
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(5.0);

        let btn_h = 27.0;

        ui.vertical_centered(|ui| {
            // Opção 1: Limpar Ambos (Tabela, Banco de Eventos e de Segurança)
            if render_cleanup_button(ui, "🧹", loc.t("clear_ui_db_option"), btn_w, btn_h).clicked() {
                self.step = CleanupStep::Confirmacao(LimpezaAlvo::Ambos);
            }
            ui.add_space(4.0);

            // Opção 2: Limpar Banco de Dados (huginn_events.db)
            if render_cleanup_button(ui, "🗄️", loc.t("clear_db_option"), btn_w, btn_h).clicked() {
                self.step = CleanupStep::Confirmacao(LimpezaAlvo::BancoDados);
            }
            ui.add_space(4.0);

            // Opção 3: Limpar Tabela da UI (Apenas memória)
            if render_cleanup_button(ui, "📋", loc.t("clear_ui_table_option"), btn_w, btn_h).clicked() {
                self.step = CleanupStep::Confirmacao(LimpezaAlvo::TabelaUI);
            }
            ui.add_space(4.0);

            // Opção 4: Limpar Banco de Segurança (huginn_security.db)
            if render_cleanup_button(ui, "🛡️", loc.t("security_clear_database"), btn_w, btn_h).clicked() {
                self.step = CleanupStep::Confirmacao(LimpezaAlvo::BancoSeguranca);
            }
            ui.add_space(4.0);

            // Opção 5: Cancelar
            if render_cleanup_button(ui, "❌", loc.t("cancel"), btn_w, btn_h).clicked() {
                *fechar = true;
                self.fechar();
            }
        });
    }

    fn render_confirmacao(
        &mut self,
        ui: &mut Ui,
        loc: &Localizer,
        alvo: LimpezaAlvo,
        fechar: &mut bool,
        on_confirm: &mut impl FnMut(LimpezaAlvo),
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚠️").size(20.0));
            ui.label(RichText::new(loc.t("confirm")).heading().size(16.0));
        });
        ui.add_space(3.0);
        ui.separator();
        ui.add_space(4.0);

        let (alvo_label, desc_confirm) = match alvo {
            LimpezaAlvo::Ambos => (
                loc.t("clear_ui_db_option"),
                loc.t("clear_all_confirm"),
            ),
            LimpezaAlvo::BancoDados => (
                loc.t("clear_db_option"),
                loc.t("clear_events_db_confirm"),
            ),
            LimpezaAlvo::TabelaUI => (
                loc.t("clear_ui_table_option"),
                loc.t("clear_ui_table_confirm"),
            ),
            LimpezaAlvo::BancoSeguranca => (
                loc.t("security_clear_database"),
                loc.t("security_clear_database_confirm"),
            ),
        };

        ui.label(
            RichText::new(format!("🎯 {}", alvo_label))
                .strong()
                .size(13.0)
                .color(Color32::from_rgb(255, 152, 0)),
        );
        ui.add_space(3.0);

        ui.label(RichText::new(desc_confirm).size(12.0));

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        // Botões "⚠️ Confirmar" e "↩️ Voltar" com centralização milimétrica no eixo vertical
        ui.horizontal(|ui| {
            let spacing = ui.spacing().item_spacing.x.max(8.0);
            let avail_w = ui.available_width();
            let btn_w = ((avail_w - spacing) / 2.0).floor();
            let btn_h = 27.0;

            let leftover = avail_w - (btn_w * 2.0 + spacing);
            if leftover > 1.0 {
                ui.add_space(leftover / 2.0);
            }

            // Botão "⚠️ Confirmar" (destaque vermelho de perigo)
            let btn_confirm_res = render_cleanup_button_opt(
                ui,
                "⚠️",
                loc.t("confirm"),
                btn_w,
                btn_h,
                Some(Color32::from_rgb(211, 47, 47)),
                Some(Color32::WHITE),
            );
            if btn_confirm_res.clicked() {
                on_confirm(alvo);
            }

            let (voltar_ico, voltar_texto) = if self.aberto_direto {
                ("❌", loc.t("cancel"))
            } else {
                ("↩️", loc.t("back"))
            };

            // Botão "↩️ Voltar" (ou "❌ Cancelar" caso aberto diretamente)
            let btn_back_res = render_cleanup_button_opt(
                ui,
                voltar_ico,
                voltar_texto,
                btn_w,
                btn_h,
                None,
                None,
            );
            if btn_back_res.clicked() {
                if self.aberto_direto {
                    *fechar = true;
                    self.fechar();
                } else {
                    self.step = CleanupStep::SelecionarAlvo;
                }
            }
        });
        ui.add_space(6.0);
    }

    fn render_sucesso(&mut self, ui: &mut Ui, loc: &Localizer, alvo: LimpezaAlvo, fechar: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("✅").size(20.0));
            ui.label(RichText::new(loc.t("success")).heading().size(16.0).color(Color32::from_rgb(76, 175, 80)));
        });
        ui.add_space(3.0);
        ui.separator();
        ui.add_space(5.0);

        let msg_sucesso = match alvo {
            LimpezaAlvo::Ambos => loc.t("clear_success"),
            LimpezaAlvo::BancoDados => loc.t("clear_db_success"),
            LimpezaAlvo::TabelaUI => loc.t("clear_ui_success"),
            LimpezaAlvo::BancoSeguranca => loc.t("security_database_cleared"),
        };

        ui.label(RichText::new(msg_sucesso).strong().size(13.0));
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        ui.vertical_centered(|ui| {
            if render_cleanup_button(ui, "✔️", loc.t("ok"), 100.0, 26.0).clicked() {
                *fechar = true;
                self.fechar();
            }
        });
        ui.add_space(6.0);
    }

    fn render_erro(&mut self, ui: &mut Ui, loc: &Localizer, msg: &str, fechar: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("❌").size(20.0));
            ui.label(RichText::new(loc.t("error")).heading().size(16.0).color(Color32::from_rgb(244, 67, 54)));
        });
        ui.add_space(3.0);
        ui.separator();
        ui.add_space(5.0);

        let err_desc = format!("{}: {}", loc.t("clear_error"), msg);
        ui.colored_label(Color32::from_rgb(244, 67, 54), RichText::new(err_desc).size(12.0));

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        ui.vertical_centered(|ui| {
            if render_cleanup_button(ui, "✔️", loc.t("ok"), 100.0, 26.0).clicked() {
                *fechar = true;
                self.fechar();
            }
        });
        ui.add_space(6.0);
    }
}
