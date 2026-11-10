use egui::{Context, Id, LayerId, Order, Rect, RichText, Ui, Vec2, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelessState {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
}

impl Default for ModelessState {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
        }
    }
}

impl ModelessState {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn fechar(&mut self) {
        self.aberto = false;
        self.minimizado = false;
    }
}

/// Renderiza um Diálogo Modeless Nativo do Windows 11 utilizando viewports imediatos independentes.
///
/// Características:
/// 1. Não fica restrito à área da janela principal da GUI: pode ser arrastado livremente para qualquer monitor.
/// 2. Barra de títulos padrão do Windows 11 com botões nativos de Minimizar (🗕), Maximizar/Restaurar (🗖/🗗) e Fechar (🗙).
/// 3. Suporte completo a Z-Order dinâmico gerenciado pelo SO (Windows Window Manager): ao clicar na janela, ela sobe para o topo.
/// 4. Modeless: execução assíncrona/imediata no event-loop sem bloquear a janela principal ou outros diálogos.
pub fn show_modeless_dialog<R>(
    ctx: &Context,
    viewport_id_str: &str,
    title: impl Into<String>,
    default_size: [f32; 2],
    min_size: [f32; 2],
    state: &mut ModelessState,
    mut content: impl FnMut(&mut Ui, &mut bool) -> R,
) -> Option<R> {
    if !state.aberto {
        return None;
    }

    let viewport_id = egui::ViewportId::from_hash_of(viewport_id_str);
    let title_str = title.into();

    // Obtém a posição e área da GUI principal (ROOT) para garantir que todas as janelas secundárias,
    // diálogos, avisos e alertas sejam gerados exatamente na mesma tela/monitor em que a GUI principal está aberta.
    let root_rect = ctx.input_for(egui::ViewportId::ROOT, |i| {
        i.viewport().outer_rect.or(i.viewport().inner_rect)
    });

    let target_pos = if let Some(root) = root_rect {
        let cx = root.center().x;
        let cy = root.center().y;
        let mut x = cx - default_size[0] / 2.0;
        let mut y = cy - default_size[1] / 2.0;

        // Se a janela secundária ficar muito alta em relação ao topo da janela principal, ajusta margem
        if y < root.min.y {
            y = root.min.y + 20.0;
        }
        if x < root.min.x {
            x = root.min.x + 20.0;
        }

        egui::pos2(x, y)
    } else {
        let srect = screen_rect(ctx);
        egui::pos2(
            (srect.center().x - default_size[0] / 2.0).max(0.0),
            (srect.center().y - default_size[1] / 2.0).max(0.0),
        )
    };

    // Verifica se a viewport secundária já está ativa e existente na sessão
    let viewport_ja_existe = ctx
        .input_for(viewport_id, |i| {
            i.viewport().outer_rect.or(i.viewport().inner_rect)
        })
        .is_some();

    if state.elevar_z_order {
        ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Focus);
        ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::OuterPosition(target_pos));
        state.elevar_z_order = false;
    }

    let mut builder = egui::ViewportBuilder::default()
        .with_title(title_str)
        .with_inner_size(default_size)
        .with_min_inner_size(min_size)
        .with_decorations(true)
        .with_resizable(true)
        .with_close_button(true)
        .with_minimize_button(true)
        .with_maximize_button(true);

    // Quando gerada pela primeira vez, define explicitamente a posição na mesma tela da GUI principal
    if !viewport_ja_existe || state.elevar_z_order {
        builder = builder.with_position([target_pos.x, target_pos.y]);
    }

    if let Some(icon) = crate::utils::icons::carregar_icone_janela() {
        builder = builder.with_icon(icon);
    }

    let mut fechar = false;
    let mut result = None;

    ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
        let dark_mode = crate::utils::detectar_modo_escuro_windows();
        if ctx.style().visuals.dark_mode != dark_mode {
            crate::ui::widgets::aplicar_tema_visuals(ctx, dark_mode);
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            fechar = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            result = Some(content(ui, &mut fechar));
        });
    });

    if fechar {
        state.fechar();
    }

    result
}

/// Fallback / Compatibilidade para renderização em janela interna caso necessário
pub fn render_modeless_header(ui: &mut Ui, state: &mut ModelessState) -> bool {
    let mut fechar = false;
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button(
                    RichText::new("🗙")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(240, 80, 80)),
                )
                .on_hover_text("Fechar Janela")
                .clicked()
            {
                fechar = true;
            }
            if state.maximizado {
                if ui
                    .button(RichText::new("🗗").size(13.0))
                    .on_hover_text("Restaurar Tamanho")
                    .clicked()
                {
                    state.maximizado = false;
                }
            } else {
                if ui
                    .button(RichText::new("🗖").size(13.0))
                    .on_hover_text("Maximizar")
                    .clicked()
                {
                    state.maximizado = true;
                }
            }
            if ui
                .button(RichText::new("🗕").size(13.0))
                .on_hover_text("Minimizar")
                .clicked()
            {
                state.minimizado = true;
            }
        });
    });
    ui.separator();
    fechar
}

pub fn screen_rect(ctx: &Context) -> Rect {
    ctx.input(|i| i.raw.screen_rect)
        .unwrap_or(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(1280.0, 800.0),
        ))
}

pub fn build_modeless_window<'open>(
    ctx: &Context,
    title: impl Into<egui::WidgetText>,
    id: Id,
    state: &ModelessState,
    default_size: Vec2,
    min_size: Vec2,
    open: &'open mut bool,
) -> Window<'open> {
    let mut win = Window::new(title)
        .id(id)
        .order(Order::Middle)
        .movable(!state.maximizado)
        .resizable(!state.maximizado)
        .collapsible(false)
        .constrain(false)
        .open(open);

    if state.maximizado {
        let srect = screen_rect(ctx);
        win = win.fixed_rect(srect.shrink(10.0));
    } else {
        win = win.default_size(default_size).min_size(min_size);
        if let Some(root) = ctx.input_for(egui::ViewportId::ROOT, |i| {
            i.viewport().outer_rect.or(i.viewport().inner_rect)
        }) {
            let center = root.center();
            win = win.default_pos(egui::pos2(
                center.x - default_size.x / 2.0,
                center.y - default_size.y / 2.0,
            ));
        }
    }

    win
}

pub fn show_minimized_chip(ctx: &Context, id: Id, title: &str, state: &mut ModelessState) {
    let layer_id = LayerId::new(Order::Middle, id);
    let mut restaurar = false;
    let mut fechar = false;

    let mini_id = id.with("_minimized_chip");
    let resp = Window::new(format!("🗕 {}", title))
        .id(mini_id)
        .order(Order::Middle)
        .movable(true)
        .resizable(false)
        .collapsible(false)
        .constrain(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("🗖 Restaurar").size(11.0)).clicked() {
                    restaurar = true;
                }
                if ui
                    .button(
                        RichText::new("🗙 Fechar")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(240, 80, 80)),
                    )
                    .clicked()
                {
                    fechar = true;
                }
            });
        });

    if let Some(r) = resp {
        if r.response.clicked() || r.response.dragged() || state.elevar_z_order {
            ctx.move_to_top(layer_id);
            state.elevar_z_order = false;
        }
    }

    if restaurar {
        state.minimizado = false;
        state.elevar_z_order = true;
    }
    if fechar {
        state.aberto = false;
        state.minimizado = false;
    }
}
