// Módulo de Widgets Customizados (Huginn)
// Implementa renderização de botões com emojis rigorosamente centralizados verticalmente
// em relação à altura dos seus widgets, desacoplados da linha de base tipográfica da fonte.

use egui::{
    pos2, Color32, FontId, Id, Pos2, Rect, Response, RichText, Stroke, StrokeKind, Ui, Vec2,
};

/// Renderiza um botão com emoji centralizado verticalmente dentro de seu widget,
/// com base no centro visual geométrico dos vértices (mesh_bounds) do glifo,
/// eliminando completamente a dependência da linha de base tipográfica da string.
pub fn render_centered_emoji_button(ui: &mut Ui, emoji: &str, text: impl AsRef<str>) -> Response {
    render_centered_emoji_button_sized(ui, emoji, text, Vec2::ZERO)
}

pub fn render_centered_emoji_button_sized(
    ui: &mut Ui,
    emoji: &str,
    text: impl AsRef<str>,
    min_size: Vec2,
) -> Response {
    let text = text.as_ref();
    let font_id = FontId::proportional(11.5);
    let emoji_font_id = FontId::proportional(12.5);

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

    let padding = Vec2::new(8.0, 4.0);
    let spacing = if emoji_galley.is_some() && text_galley.is_some() { 5.0 } else { 0.0 };
    let emoji_w = emoji_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let emoji_h = emoji_galley.as_ref().map(|g| g.size().y).unwrap_or(0.0);
    let text_w = text_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let text_h = text_galley.as_ref().map(|g| g.size().y).unwrap_or(0.0);

    let content_width = emoji_w + spacing + text_w;
    let desired_width = (padding.x * 2.0 + content_width).max(min_size.x);
    let desired_height = (emoji_h.max(text_h) + padding.y * 2.0).max(min_size.y).max(24.0);

    let (rect, response) = ui.allocate_exact_size(Vec2::new(desired_width, desired_height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter().rect_filled(rect, 3_u8, visuals.bg_fill);
        ui.painter().rect_stroke(rect, 3_u8, visuals.bg_stroke, egui::StrokeKind::Inside);

        let center_y = rect.center().y;

        let start_x = if min_size.x > content_width + padding.x * 2.0 {
            rect.center().x - content_width / 2.0
        } else {
            rect.left() + padding.x
        };

        let mut cur_x = start_x;
        if let Some(eg) = emoji_galley {
            let emoji_visual_y = if eg.mesh_bounds.is_positive() {
                eg.mesh_bounds.center().y
            } else {
                eg.rect.center().y
            };
            let emoji_pos = Pos2::new(cur_x, center_y - emoji_visual_y);
            ui.painter().galley(emoji_pos, eg, visuals.text_color());
            cur_x += emoji_w + spacing;
        }

        if let Some(tg) = text_galley {
            let text_visual_y = if tg.mesh_bounds.is_positive() {
                tg.mesh_bounds.center().y
            } else {
                tg.rect.center().y
            };
            let text_pos = Pos2::new(cur_x, center_y - text_visual_y);
            ui.painter().galley(text_pos, tg, visuals.text_color());
        }
    }

    response
}

/// Helper para botões com cores destacadas (ex: Iniciar, Parar)
pub fn render_colored_emoji_button(
    ui: &mut Ui,
    emoji: &str,
    text: impl AsRef<str>,
    accent_color: Color32,
) -> Response {
    let text = text.as_ref();
    let font_id = FontId::proportional(12.0);
    let emoji_font_id = FontId::proportional(13.0);

    let emoji_galley = ui.painter().layout_no_wrap(emoji.to_string(), emoji_font_id, Color32::PLACEHOLDER);
    let text_galley = if !text.is_empty() {
        Some(ui.painter().layout_no_wrap(text.to_string(), font_id, Color32::PLACEHOLDER))
    } else {
        None
    };

    let padding = Vec2::new(9.0, 4.5);
    let spacing = if text.is_empty() { 0.0 } else { 6.0 };
    let text_width = text_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let text_height = text_galley.as_ref().map(|g| g.size().y).unwrap_or(0.0);

    let desired_width = padding.x * 2.0 + emoji_galley.size().x + spacing + text_width;
    let desired_height = (emoji_galley.size().y.max(text_height) + padding.y * 2.0).max(26.0);

    let (rect, response) = ui.allocate_exact_size(Vec2::new(desired_width, desired_height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter().rect_filled(rect, 3_u8, visuals.bg_fill);
        ui.painter().rect_stroke(rect, 3_u8, visuals.bg_stroke, egui::StrokeKind::Inside);

        let center_y = rect.center().y;

        let emoji_w = emoji_galley.size().x;
        let emoji_visual_y = if emoji_galley.mesh_bounds.is_positive() {
            emoji_galley.mesh_bounds.center().y
        } else {
            emoji_galley.rect.center().y
        };

        if text.is_empty() {
            let emoji_visual_x = if emoji_galley.mesh_bounds.is_positive() {
                emoji_galley.mesh_bounds.center().x
            } else {
                emoji_galley.rect.center().x
            };
            let pos = Pos2::new(rect.center().x - emoji_visual_x, center_y - emoji_visual_y);
            ui.painter().galley(pos, emoji_galley, accent_color);
        } else {
            let emoji_x = rect.left() + padding.x;
            let emoji_pos = Pos2::new(emoji_x, center_y - emoji_visual_y);
            ui.painter().galley(emoji_pos, emoji_galley, accent_color);

            if let Some(tg) = text_galley {
                let text_visual_y = if tg.mesh_bounds.is_positive() {
                    tg.mesh_bounds.center().y
                } else {
                    tg.rect.center().y
                };
                let text_x = emoji_x + emoji_w + spacing;
                let text_pos = Pos2::new(text_x, center_y - text_visual_y);
                ui.painter().galley(text_pos, tg, visuals.text_color());
            }
        }
    }

    response
}

/// Converte uma string hexadecimal de cor (ex: "#00ff00", "00ff00", "#0f0", "0f0", "#00ff00ff") para Color32
pub fn parse_hex_color(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        3 => {
            let mut chars = s.chars();
            let r_c = chars.next()?;
            let g_c = chars.next()?;
            let b_c = chars.next()?;
            let r = u8::from_str_radix(&format!("{}{}", r_c, r_c), 16).ok()?;
            let g = u8::from_str_radix(&format!("{}{}", g_c, g_c), 16).ok()?;
            let b = u8::from_str_radix(&format!("{}{}", b_c, b_c), 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        _ => None,
    }
}

/// Renderiza um seletor de cor customizado e localizado com janela pop-up completa:
/// - Mostrador de amostra de cor (Color Well)
/// - Cabeçalho com botão "❌" para fechar a janela da paleta
/// - Área de seleção 2D (Saturação / Brilho) e seletor 1D de Matiz (Hue)
/// - Botões "U8" e "📋" com alinhamento vertical rigoroso na caixa do botão
/// - Tooltips traduzidos dinamicamente em tempo de execução
/// - Controles numéricos e saída de canais "R, G, B" (ex: "R 46, G 125, B 50")
/// - Saída e entrada editável de formato "HTML" (ex: "HTML: #00ff00")
pub fn render_custom_color_edit_button(
    ui: &mut Ui,
    id: Id,
    color: &mut Color32,
    loc: &crate::locale::Localizer,
) -> Response {
    let popup_id = id.with("custom_color_picker_popup");
    let is_open = egui::Popup::is_id_open(ui.ctx(), popup_id);

    // 1. Botão amostra de cor (Color Well)
    let button_size = Vec2::new(34.0, 22.0);
    let (rect, mut button_response) = ui.allocate_exact_size(button_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = if is_open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&button_response)
        };

        ui.painter().rect_filled(rect, 4_u8, *color);
        ui.painter().rect_stroke(
            rect,
            4_u8,
            egui::Stroke::new(1.5, visuals.bg_stroke.color),
            StrokeKind::Inside,
        );
    }

    // 2. Janela Popup da Paleta de Cores
    let mut changed = false;
    egui::Popup::menu(&button_response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(260.0);
            ui.set_max_width(280.0);

            // Carrega estado HSV do cache temporário para preservar Matiz mesmo em tons neutros
            let hsva_id = id.with("hsva_state");
            let mut hsva = ui
                .ctx()
                .data(|d| d.get_temp::<[f32; 3]>(hsva_id))
                .unwrap_or_else(|| {
                    let h = egui::ecolor::Hsva::from(*color);
                    [h.h, h.s, h.v]
                });

            let [r, g, b, _] = color.to_srgba_unmultiplied();
            let html_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);

            let hex_id = id.with("html_hex_edit_buf");
            let mut hex_buf: String = ui
                .ctx()
                .data(|d| d.get_temp::<String>(hex_id))
                .unwrap_or_else(|| html_hex.clone());

            // Cabeçalho da janela com botão ❌ para fechar
            ui.horizontal(|ui| {
                let chip_size = Vec2::new(20.0, 16.0);
                let (chip_rect, _) = ui.allocate_exact_size(chip_size, egui::Sense::hover());
                ui.painter().rect_filled(chip_rect, 3_u8, *color);
                ui.painter().rect_stroke(
                    chip_rect,
                    3_u8,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    StrokeKind::Inside,
                );

                ui.label(
                    RichText::new(loc.t("color_palette_title"))
                        .strong()
                        .size(12.5),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if render_centered_emoji_button(ui, "❌", "")
                        .on_hover_text(loc.t("close"))
                        .clicked()
                    {
                        egui::Popup::close_id(ui.ctx(), popup_id);
                    }
                });
            });

            ui.separator();
            ui.add_space(2.0);

            let picker_width = 250.0;

            // Área 2D: Saturação (horizontal) e Brilho/Valor (vertical)
            let (picker_rect, picker_resp) =
                ui.allocate_exact_size(Vec2::new(picker_width, 150.0), egui::Sense::click_and_drag());

            if let Some(mpos) = picker_resp.interact_pointer_pos() {
                hsva[1] = ((mpos.x - picker_rect.left()) / picker_rect.width()).clamp(0.0, 1.0);
                hsva[2] = ((picker_rect.bottom() - mpos.y) / picker_rect.height()).clamp(0.0, 1.0);
                let new_c = Color32::from(egui::ecolor::Hsva::new(hsva[0], hsva[1], hsva[2], 1.0));
                *color = new_c;
                changed = true;
            }

            if ui.is_rect_visible(picker_rect) {
                let n = 24_usize;
                let mut pixels = Vec::with_capacity((n + 1) * (n + 1));
                for yi in 0..=n {
                    let cur_v = 1.0 - yi as f32 / n as f32;
                    for xi in 0..=n {
                        let cur_s = xi as f32 / n as f32;
                        pixels.push(Color32::from(egui::ecolor::Hsva::new(hsva[0], cur_s, cur_v, 1.0)));
                    }
                }
                let image = egui::ColorImage::new([n + 1, n + 1], pixels);
                let texture = ui.ctx().load_texture_cached(
                    "picker_2d_grad",
                    id.with("grad"),
                    image,
                    egui::TextureOptions::LINEAR,
                );
                ui.painter().image(
                    texture.id(),
                    picker_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.painter().rect_stroke(
                    picker_rect,
                    4_u8,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    StrokeKind::Inside,
                );

                // Marcador circular sobre a cor selecionada
                let cx = picker_rect.left() + hsva[1] * picker_rect.width();
                let cy = picker_rect.bottom() - hsva[2] * picker_rect.height();
                let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                let ring_col = if lum < 128.0 { Color32::WHITE } else { Color32::BLACK };
                ui.painter().circle(
                    pos2(cx, cy),
                    5.0,
                    *color,
                    egui::Stroke::new(2.0, ring_col),
                );
            }

            ui.add_space(4.0);

            // Barra 1D: Matiz (Hue)
            let (hue_rect, hue_resp) =
                ui.allocate_exact_size(Vec2::new(picker_width, 18.0), egui::Sense::click_and_drag());

            if let Some(mpos) = hue_resp.interact_pointer_pos() {
                hsva[0] = ((mpos.x - hue_rect.left()) / hue_rect.width()).clamp(0.0, 1.0);
                let new_c = Color32::from(egui::ecolor::Hsva::new(hsva[0], hsva[1], hsva[2], 1.0));
                *color = new_c;
                changed = true;
            }

            if ui.is_rect_visible(hue_rect) {
                let n = 36_usize;
                let mut pixels = Vec::with_capacity(n + 1);
                for xi in 0..=n {
                    let cur_h = xi as f32 / n as f32;
                    pixels.push(Color32::from(egui::ecolor::Hsva::new(cur_h, 1.0, 1.0, 1.0)));
                }
                let image = egui::ColorImage::new([n + 1, 1], pixels);
                let texture = ui.ctx().load_texture_cached(
                    "picker_hue_grad",
                    id.with("hue"),
                    image,
                    egui::TextureOptions::LINEAR,
                );
                ui.painter().image(
                    texture.id(),
                    hue_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.painter().rect_stroke(
                    hue_rect,
                    3_u8,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    StrokeKind::Inside,
                );

                // Indicador na barra de matiz
                let hx = hue_rect.left() + hsva[0] * hue_rect.width();
                ui.painter().line_segment(
                    [pos2(hx, hue_rect.top()), pos2(hx, hue_rect.bottom())],
                    egui::Stroke::new(2.5, Color32::WHITE),
                );
                ui.painter().line_segment(
                    [pos2(hx, hue_rect.top()), pos2(hx, hue_rect.bottom())],
                    egui::Stroke::new(1.0, Color32::BLACK),
                );
            }

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(3.0);

            // Linha com Botões "U8" e "📋" centralizados verticalmente + valores "R, G, B"
            ui.horizontal(|ui| {
                let btn_u8 = render_centered_emoji_button(ui, "", "U8");
                btn_u8.on_hover_text(loc.t("color_u8_tooltip"));

                let btn_copy = render_centered_emoji_button(ui, "📋", "");
                if btn_copy.on_hover_text(loc.t("color_copy_tooltip")).clicked() {
                    ui.ctx().copy_text(format!("{} (R {}, G {}, B {})", html_hex, r, g, b));
                }

                let mut cur_r = r;
                let mut cur_g = g;
                let mut cur_b = b;

                ui.label(RichText::new("R").strong().size(11.5));
                let resp_r = ui.add(egui::DragValue::new(&mut cur_r).range(0..=255));
                ui.label(RichText::new("G").strong().size(11.5));
                let resp_g = ui.add(egui::DragValue::new(&mut cur_g).range(0..=255));
                ui.label(RichText::new("B").strong().size(11.5));
                let resp_b = ui.add(egui::DragValue::new(&mut cur_b).range(0..=255));

                if resp_r.changed() || resp_g.changed() || resp_b.changed() {
                    let new_c = Color32::from_rgb(cur_r, cur_g, cur_b);
                    *color = new_c;
                    let new_h = egui::ecolor::Hsva::from(new_c);
                    hsva = [new_h.h, new_h.s, new_h.v];
                    changed = true;
                }
            });

            ui.add_space(4.0);

            // Saída e entrada editável de "HTML: #00ff00"
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("HTML:")
                        .strong()
                        .monospace()
                        .size(12.0)
                        .color(ui.visuals().text_color()),
                );

                let [cur_r, cur_g, cur_b, _] = color.to_srgba_unmultiplied();
                let current_hex = format!("#{:02x}{:02x}{:02x}", cur_r, cur_g, cur_b);

                let edit_resp = ui.add(
                    egui::TextEdit::singleline(&mut hex_buf)
                        .desired_width(75.0)
                        .char_limit(9)
                        .font(egui::TextStyle::Monospace),
                );

                if edit_resp.changed() {
                    if let Some(parsed) = parse_hex_color(&hex_buf) {
                        *color = parsed;
                        let new_h = egui::ecolor::Hsva::from(parsed);
                        hsva = [new_h.h, new_h.s, new_h.v];
                        changed = true;
                    }
                } else if edit_resp.lost_focus() {
                    hex_buf = current_hex;
                } else if !edit_resp.has_focus() {
                    if parse_hex_color(&hex_buf) != Some(*color) {
                        hex_buf = current_hex;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if render_centered_emoji_button(ui, "📋", loc.t("copy_color_html"))
                        .on_hover_text(loc.t("color_copy_tooltip"))
                        .clicked()
                    {
                        ui.ctx().copy_text(hex_buf.clone());
                    }
                });
            });

            // Salva o estado HSV e o buffer de texto no cache temporário
            ui.ctx().data_mut(|d| {
                d.insert_temp(hsva_id, hsva);
                d.insert_temp(hex_id, hex_buf);
            });
        });

    if changed {
        button_response.mark_changed();
    }

    button_response
}

/// Helper para botões de menus de contexto com emojis perfeitamente centralizados
pub fn render_centered_emoji_menu_button(
    ui: &mut Ui,
    emoji: &str,
    text: impl AsRef<str>,
) -> Response {
    let text = text.as_ref();
    let font_id = FontId::proportional(12.0);
    let emoji_font_id = FontId::proportional(12.5);

    let emoji_galley = ui.painter().layout_no_wrap(emoji.to_string(), emoji_font_id, Color32::PLACEHOLDER);
    let text_galley = if !text.is_empty() {
        Some(ui.painter().layout_no_wrap(text.to_string(), font_id, Color32::PLACEHOLDER))
    } else {
        None
    };

    let padding = Vec2::new(8.0, 4.0);
    let spacing = if text.is_empty() { 0.0 } else { 8.0 };
    let text_width = text_galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let text_height = text_galley.as_ref().map(|g| g.size().y).unwrap_or(0.0);

    let min_width = ui.available_width().max(padding.x * 2.0 + emoji_galley.size().x + spacing + text_width);
    let desired_height = (emoji_galley.size().y.max(text_height) + padding.y * 2.0).max(24.0);

    let (rect, response) = ui.allocate_exact_size(Vec2::new(min_width, desired_height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        if response.hovered() {
            ui.painter().rect_filled(rect, 3_u8, visuals.bg_fill);
            ui.painter().rect_stroke(rect, 3_u8, Stroke::new(1.0, visuals.bg_stroke.color), egui::StrokeKind::Inside);
        }

        let center_y = rect.center().y;

        let emoji_w = emoji_galley.size().x;
        let emoji_visual_y = if emoji_galley.mesh_bounds.is_positive() {
            emoji_galley.mesh_bounds.center().y
        } else {
            emoji_galley.rect.center().y
        };

        let emoji_x = rect.left() + padding.x;
        let emoji_pos = Pos2::new(emoji_x, center_y - emoji_visual_y);
        ui.painter().galley(emoji_pos, emoji_galley, visuals.text_color());

        if let Some(tg) = text_galley {
            let text_visual_y = if tg.mesh_bounds.is_positive() {
                tg.mesh_bounds.center().y
            } else {
                tg.rect.center().y
            };
            let text_x = emoji_x + emoji_w + spacing;
            let text_pos = Pos2::new(text_x, center_y - text_visual_y);
            ui.painter().galley(text_pos, tg, visuals.text_color());
        }
    }

    response
}

/// Configura os parâmetros visuais e estilos dinâmicos de tema para corresponder
/// com fidelidade à transição de tema do Windows 11 (Claro <-> Escuro), garantindo
/// alto contraste tipográfico e legibilidade absoluta em todos os botões, rótulos e widgets,
/// além de barras de rolagem personalizadas e consistentes.
pub fn aplicar_tema_visuals(ctx: &egui::Context, dark_mode: bool) {
    let visuals = if dark_mode {
        let mut v = egui::Visuals::dark();
        v.override_text_color = None;

        // Track/trilho da barra de scroll (vertical e horizontal) em tom escuro do Windows 11
        v.extreme_bg_color = Color32::from_rgb(20, 22, 26);
        v.faint_bg_color = Color32::from_rgb(28, 30, 36);

        // Botões e widgets: fundo elegante, bordas sutis e tipografia clara com contraste nítido
        v.widgets.inactive.bg_fill = Color32::from_rgb(46, 50, 60);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(68, 74, 88));
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(235, 238, 245));

        v.widgets.hovered.bg_fill = Color32::from_rgb(60, 66, 80);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(95, 105, 126));
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        v.widgets.active.bg_fill = Color32::from_rgb(76, 84, 102);
        v.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(115, 128, 154));
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        v.widgets.open.bg_fill = Color32::from_rgb(52, 58, 70);
        v.widgets.open.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 90, 110));
        v.widgets.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 224, 234));
        v
    } else {
        let mut v = egui::Visuals::light();
        v.override_text_color = None;

        // Track/trilho da barra de scroll (vertical e horizontal) em tom claro limpo do Windows 11
        v.extreme_bg_color = Color32::from_rgb(240, 242, 246);
        v.faint_bg_color = Color32::from_rgb(248, 249, 252);

        // Botões e widgets: fundo suave e tipografia escura com contraste nítido
        v.widgets.inactive.bg_fill = Color32::from_rgb(238, 241, 246);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(205, 212, 222));
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(22, 26, 34));

        v.widgets.hovered.bg_fill = Color32::from_rgb(225, 232, 242);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(170, 180, 196));
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(8, 12, 18));

        v.widgets.active.bg_fill = Color32::from_rgb(210, 218, 230);
        v.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 152, 172));
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::BLACK);

        v.widgets.open.bg_fill = Color32::from_rgb(230, 235, 244);
        v.widgets.open.bg_stroke = Stroke::new(1.0, Color32::from_rgb(180, 190, 206));
        v.widgets.open.fg_stroke = Stroke::new(1.0, Color32::BLACK);

        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(28, 32, 40));
        v
    };
    ctx.set_visuals(visuals);

    // Configuração das barras de rolagem (verticais e horizontais)
    let configure_scroll = |s: &mut egui::Style| {
        s.spacing.scroll.foreground_color = false;
        s.spacing.scroll.floating = false;
        s.spacing.scroll.bar_width = 10.0;
        s.spacing.scroll.handle_min_length = 24.0;
        s.spacing.scroll.dormant_background_opacity = 1.0;
        s.spacing.scroll.active_background_opacity = 1.0;
        s.spacing.scroll.interact_background_opacity = 1.0;
        s.spacing.scroll.dormant_handle_opacity = 0.80;
        s.spacing.scroll.active_handle_opacity = 0.95;
        s.spacing.scroll.interact_handle_opacity = 1.0;
    };
    ctx.style_mut_of(egui::Theme::Dark, configure_scroll);
    ctx.style_mut_of(egui::Theme::Light, configure_scroll);
}

// ============================================================================
// GUARDIÃO DE PERSISTÊNCIA HOVER DE ÁRVORES DE MENU (Menus, Submenus e Contexto)
// ============================================================================
// Mantém qualquer menu, submenu, menu de contexto ou dropdown aberto mesmo após
// cliques internos repetidos, fechando a árvore de menus exclusivamente quando
// o ponteiro do mouse deixa a região geométrica ativa do menu/submenu aberto
// (ou quando o usuário pressiona Escape).

#[derive(Clone, Debug, Default)]
struct MenuHoverState {
    rects_this_pass: Vec<egui::Rect>,
    any_sizing_pass: bool,
    last_open_root_popup_id: Option<egui::Id>,
    opened_pass_nr: Option<u64>,
    pointer_entered: bool,
    initial_pointer_pos: Option<Pos2>,
}

fn menu_hover_state_id() -> egui::Id {
    egui::Id::unique("linceu_menu_hover_guard_state")
}

/// Configuração padrão para menus e submenus persistentes (ignora cliques para fechamento
/// e elimina frestas entre menu pai e submenu).
pub fn config_menu_persistente() -> egui::containers::menu::MenuConfig {
    egui::containers::menu::MenuConfig::new()
        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
        .submenu_gap(0.0)
}

/// Inicializa a coleta de retângulos de menus abertos no início do frame da GUI.
pub fn iniciar_rastreamento_menus_frame(ctx: &egui::Context) {
    let any_open = egui::Popup::is_any_open(ctx);
    let id = menu_hover_state_id();
    ctx.data_mut(|d| {
        let mut st = d.get_temp::<MenuHoverState>(id).unwrap_or_default();
        st.rects_this_pass.clear();
        st.any_sizing_pass = false;
        if !any_open {
            st.last_open_root_popup_id = None;
            st.opened_pass_nr = None;
            st.pointer_entered = false;
            st.initial_pointer_pos = None;
        }
        d.insert_temp(id, st);
    });
}

/// Registra um retângulo pertencente à árvore de menu atualmente aberta neste frame.
pub fn registrar_retangulo_menu_ativo(
    ctx: &egui::Context,
    rect: egui::Rect,
    is_sizing_pass: bool,
) {
    let id = menu_hover_state_id();
    ctx.data_mut(|d| {
        let mut st = d.get_temp::<MenuHoverState>(id).unwrap_or_default();
        let invalid_size = !rect.is_finite() || rect.width() <= 1.0 || rect.height() <= 1.0;
        st.any_sizing_pass |= is_sizing_pass || invalid_size;
        if rect.is_finite() && rect.width() > 0.0 && rect.height() > 0.0 {
            st.rects_this_pass.push(rect);
        }
        d.insert_temp(id, st);
    });
}

/// Renderiza um botão de menu (na `MenuBar` raiz) ou de submenu (aninhado em qualquer nível)
/// que permanece aberto ao clicar em suas opções internas e registra automaticamente sua área.
pub fn menu_persistente<'a, R>(
    ui: &mut Ui,
    atoms: impl egui::IntoAtoms<'a>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Response {
    let cfg = config_menu_persistente();
    if egui::containers::menu::is_in_menu(ui) {
        let (btn_resp, popup_opt) = egui::containers::menu::SubMenuButton::new(atoms)
            .config(cfg)
            .ui(ui, |ui| {
                let sizing = ui.is_sizing_pass();
                let out = add_contents(ui);
                (sizing, out)
            });
        if let Some(popup_inner) = popup_opt {
            registrar_retangulo_menu_ativo(
                ui.ctx(),
                popup_inner.response.rect,
                popup_inner.inner.0,
            );
        }
        btn_resp
    } else {
        let state_id = menu_hover_state_id();
        let prev_root = ui
            .ctx()
            .data(|d| d.get_temp::<MenuHoverState>(state_id))
            .and_then(|s| s.last_open_root_popup_id);

        let (btn_resp, popup_opt) = egui::containers::menu::MenuButton::new(atoms)
            .config(cfg)
            .ui(ui, |ui| {
                let sizing = ui.is_sizing_pass();
                let out = add_contents(ui);
                (sizing, out)
            });

        let popup_id = egui::Popup::default_response_id(&btn_resp);
        if let Some(popup_inner) = popup_opt {
            ui.ctx().data_mut(|d| {
                let mut st = d.get_temp::<MenuHoverState>(state_id).unwrap_or_default();
                if st.last_open_root_popup_id != Some(popup_id) {
                    st.last_open_root_popup_id = Some(popup_id);
                    st.opened_pass_nr = None;
                }
                d.insert_temp(state_id, st);
            });
            registrar_retangulo_menu_ativo(ui.ctx(), btn_resp.rect, false);
            registrar_retangulo_menu_ativo(
                ui.ctx(),
                popup_inner.response.rect,
                popup_inner.inner.0,
            );
        } else if prev_root == Some(popup_id) && btn_resp.clicked() {
            // Impede que clicar novamente no cabeçalho do menu aberto o feche enquanto o cursor está sobre ele
            egui::Popup::open_id(ui.ctx(), popup_id);
            registrar_retangulo_menu_ativo(ui.ctx(), btn_resp.rect, true);
            ui.ctx().request_repaint();
        }
        btn_resp
    }
}

/// Renderiza um menu de contexto (botão direito) que permanece aberto após cliques em suas opções
/// e só fecha quando o ponteiro do mouse sai da área do menu.
pub fn menu_contexto_persistente(
    response: &Response,
    add_contents: impl FnOnce(&mut Ui),
) {
    let popup_id = egui::Popup::default_response_id(response);
    let was_open_before = egui::Popup::is_id_open(&response.ctx, popup_id);
    let cfg = config_menu_persistente();

    let inner = egui::Popup::context_menu(response)
        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
        .info(
            egui::UiStackInfo::new(egui::UiKind::Menu)
                .with_tag_value(egui::containers::menu::MenuConfig::MENU_CONFIG_TAG, cfg),
        )
        .show(|ui| {
            let sizing = ui.is_sizing_pass();
            add_contents(ui);
            sizing
        });

    if let Some(popup_inner) = inner {
        let state_id = menu_hover_state_id();
        response.ctx.data_mut(|d| {
            let mut st = d.get_temp::<MenuHoverState>(state_id).unwrap_or_default();
            if st.last_open_root_popup_id != Some(popup_id) {
                st.last_open_root_popup_id = Some(popup_id);
                st.opened_pass_nr = None;
                st.pointer_entered = false;
            }
            d.insert_temp(state_id, st);
        });
        registrar_retangulo_menu_ativo(
            &response.ctx,
            popup_inner.response.rect,
            popup_inner.inner || !was_open_before,
        );
    }
}

/// Registra um `ComboBox` configurado com `PopupCloseBehavior::IgnoreClicks` no guardião de hover.
pub fn combo_persistente_registrar(ctx: &egui::Context, combo_resp: &Response) {
    let popup_id = combo_resp.id.with("popup");
    let state_id = menu_hover_state_id();
    let prev_root = ctx
        .data(|d| d.get_temp::<MenuHoverState>(state_id))
        .and_then(|s| s.last_open_root_popup_id);

    if egui::ComboBox::is_open(ctx, combo_resp.id) {
        ctx.data_mut(|d| {
            let mut st = d.get_temp::<MenuHoverState>(state_id).unwrap_or_default();
            if st.last_open_root_popup_id != Some(popup_id) {
                st.last_open_root_popup_id = Some(popup_id);
                st.opened_pass_nr = None;
            }
            d.insert_temp(state_id, st);
        });
        registrar_retangulo_menu_ativo(ctx, combo_resp.rect, false);
        let fg_layer = egui::LayerId::new(egui::Order::Foreground, popup_id);
        let (area_rect_opt, visible_last) =
            ctx.memory(|m| (m.area_rect(popup_id), m.areas().visible_last_frame(&fg_layer)));
        if let Some(r) = area_rect_opt {
            registrar_retangulo_menu_ativo(ctx, r, !visible_last);
        } else {
            registrar_retangulo_menu_ativo(ctx, combo_resp.rect, true);
        }
    } else if prev_root == Some(popup_id) && combo_resp.clicked() {
        egui::Popup::open_id(ctx, popup_id);
        registrar_retangulo_menu_ativo(ctx, combo_resp.rect, true);
        ctx.request_repaint();
    }
}

fn fechar_arvore_menus(ctx: &egui::Context, st: &mut MenuHoverState) {
    egui::Popup::close_all(ctx);
    ctx.data_mut(|d| d.remove_by_type::<egui::containers::menu::MenuState>());
    *st = MenuHoverState::default();
    ctx.request_repaint();
}

/// Avalia ao final do frame se o ponteiro permanece sobre qualquer região da árvore de menu aberta;
/// caso tenha saído da região ativa, fecha imediatamente toda a árvore de menu e submenus.
pub fn finalizar_rastreamento_menus_frame(ctx: &egui::Context) {
    if !egui::Popup::is_any_open(ctx) {
        return;
    }

    let state_id = menu_hover_state_id();
    let mut st = ctx
        .data(|d| d.get_temp::<MenuHoverState>(state_id))
        .unwrap_or_default();

    // Se nenhum menu rastreado registrou retângulos (ex.: color picker nativo de diálogo), não interfere
    if st.rects_this_pass.is_empty() {
        return;
    }

    const SEAM_TOLERANCE: f32 = 6.0;
    let pass_nr = ctx.cumulative_pass_nr();
    let hover_pos = ctx.pointer_hover_pos();

    if st.opened_pass_nr.is_none() {
        st.opened_pass_nr = Some(pass_nr);
        st.initial_pointer_pos = hover_pos;
    }

    let inside_any = hover_pos.is_some_and(|pos| {
        st.rects_this_pass
            .iter()
            .any(|r| r.expand(SEAM_TOLERANCE).contains(pos))
    });

    if inside_any {
        st.pointer_entered = true;
    }

    // No passe inicial de dimensionamento (sizing pass), aguarda o posicionamento definitivo no próximo frame
    if st.any_sizing_pass || st.opened_pass_nr == Some(pass_nr) {
        ctx.data_mut(|d| d.insert_temp(state_id, st));
        ctx.request_repaint();
        return;
    }

    // Enquanto botão do mouse estiver pressionado (arrasto de scrollbar ou seleção em TextEdit interno), mantém aberto
    if ctx.input(|i| i.pointer.any_down()) {
        ctx.data_mut(|d| d.insert_temp(state_id, st));
        return;
    }

    match hover_pos {
        Some(pos) => {
            if inside_any {
                ctx.data_mut(|d| d.insert_temp(state_id, st));
                return;
            }

            let moving_towards_any = ctx.input(|i| {
                st.rects_this_pass
                    .iter()
                    .any(|r| i.pointer.is_moving_towards_rect(r))
            });
            if moving_towards_any {
                ctx.data_mut(|d| d.insert_temp(state_id, st));
                ctx.request_repaint();
                return;
            }

            if st.pointer_entered {
                fechar_arvore_menus(ctx, &mut st);
            } else {
                let clicked_any = ctx.input(|i| i.pointer.any_click());
                let moved_away = st
                    .initial_pointer_pos
                    .is_some_and(|init| init.distance(pos) > 10.0);
                if clicked_any || moved_away {
                    fechar_arvore_menus(ctx, &mut st);
                }
            }
        }
        None => {
            if st.pointer_entered {
                fechar_arvore_menus(ctx, &mut st);
            }
        }
    }

    ctx.data_mut(|d| d.insert_temp(state_id, st));
}

#[cfg(test)]
mod tests {
    use egui::{Color32, Context, FontId};

    #[test]
    fn test_emoji_and_text_galley_metrics() {
        let ctx = Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let emoji = "📁";
            let text = "Selecionar Diretório";
            let emoji_galley = ui.painter().layout_no_wrap(emoji.to_string(), FontId::proportional(12.5), Color32::PLACEHOLDER);
            let text_galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(11.5), Color32::PLACEHOLDER);

            assert!(emoji_galley.mesh_bounds.is_positive());
            assert!(text_galley.mesh_bounds.is_positive());
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_aplicar_tema_visuals_dark_and_light() {
        use super::aplicar_tema_visuals;
        let ctx = Context::default();
        aplicar_tema_visuals(&ctx, true);
        let mut out1 = ctx.run_ui(Default::default(), |ui| {
            assert!(ui.visuals().dark_mode);
            assert_eq!(ui.visuals().extreme_bg_color, Color32::from_rgb(20, 22, 26));
            assert_ne!(ui.visuals().widgets.inactive.bg_fill, ui.visuals().widgets.inactive.fg_stroke.color);
            assert_eq!(ui.visuals().widgets.inactive.fg_stroke.color, Color32::from_rgb(235, 238, 245));
        });
        out1.textures_delta.clear();

        aplicar_tema_visuals(&ctx, false);
        let mut out2 = ctx.run_ui(Default::default(), |ui| {
            assert!(!ui.visuals().dark_mode);
            assert_eq!(ui.visuals().extreme_bg_color, Color32::from_rgb(240, 242, 246));
            assert_ne!(ui.visuals().widgets.inactive.bg_fill, ui.visuals().widgets.inactive.fg_stroke.color);
            assert_eq!(ui.visuals().widgets.inactive.fg_stroke.color, Color32::from_rgb(22, 26, 34));
        });
        out2.textures_delta.clear();
    }

    #[test]
    fn test_menu_persistente_mantem_aberto_ao_clicar_e_fecha_ao_sair_da_regiao() {
        use super::{
            config_menu_persistente, finalizar_rastreamento_menus_frame,
            iniciar_rastreamento_menus_frame, menu_persistente,
        };
        use egui::{Event, MenuBar, Modifiers, PointerButton, Pos2, RawInput, Rect};

        #[derive(Default)]
        struct Probe {
            cliques_opcao_direta: usize,
            cliques_submenu: usize,
            btn_menu_pos: Pos2,
            item_direto_pos: Pos2,
            btn_submenu_pos: Pos2,
            item_submenu_pos: Pos2,
        }

        let ctx = Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0)));
        let mut p = Probe {
            btn_menu_pos: Pos2::new(30.0, 12.0),
            ..Default::default()
        };

        fn rodar_frame(ctx: &Context, screen: Option<Rect>, p: &mut Probe, eventos: Vec<Event>) {
            let raw = RawInput {
                screen_rect: screen,
                events: eventos,
                ..Default::default()
            };
            let mut out = ctx.run_ui(raw, |ui| {
                iniciar_rastreamento_menus_frame(ui.ctx());
                egui::Panel::top("test_top_bar").show(ui, |ui| {
                    MenuBar::new().config(config_menu_persistente()).ui(ui, |ui| {
                        let r_root = menu_persistente(ui, "Configurações", |ui| {
                            ui.set_min_width(180.0);
                            let r_opt = ui.button("Opção Direta");
                            p.item_direto_pos = r_opt.rect.center();
                            if r_opt.clicked() {
                                p.cliques_opcao_direta += 1;
                            }
                            let r_sub = menu_persistente(ui, "Submenu Fixar", |ui| {
                                ui.set_min_width(160.0);
                                let r_sub_opt = ui.button("Fixar Coluna #1");
                                p.item_submenu_pos = r_sub_opt.rect.center();
                                if r_sub_opt.clicked() {
                                    p.cliques_submenu += 1;
                                }
                            });
                            p.btn_submenu_pos = r_sub.rect.center();
                        });
                        p.btn_menu_pos = r_root.rect.center();
                    });
                });
                finalizar_rastreamento_menus_frame(ui.ctx());
            });
            out.textures_delta.clear();
        }

        // Frame 0: layout inicial
        rodar_frame(&ctx, screen, &mut p, vec![]);
        assert!(!egui::Popup::is_any_open(&ctx));

        // Frame 1 & 2: move e clica no botão "Configurações" para abrir o menu raiz
        let pos_btn = p.btn_menu_pos;
        rodar_frame(
            &ctx,
            screen,
            &mut p,
            vec![
                Event::PointerMoved(pos_btn),
                Event::PointerButton {
                    pos: pos_btn,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        rodar_frame(
            &ctx,
            screen,
            &mut p,
            vec![Event::PointerButton {
                pos: pos_btn,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_btn)]);
        assert!(egui::Popup::is_any_open(&ctx), "Menu deve abrir ao clicar");

        // Move para "Opção Direta" dentro do menu e clica DUAS vezes seguidas: o menu NÃO deve fechar!
        let pos_opt = p.item_direto_pos;
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_opt)]);
        for esperado in 1..=2 {
            rodar_frame(
                &ctx,
                screen,
                &mut p,
                vec![Event::PointerButton {
                    pos: pos_opt,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                }],
            );
            rodar_frame(
                &ctx,
                screen,
                &mut p,
                vec![Event::PointerButton {
                    pos: pos_opt,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_opt)]);
            assert_eq!(p.cliques_opcao_direta, esperado);
            assert!(
                egui::Popup::is_any_open(&ctx),
                "Menu deve permanecer aberto após clique #{esperado} na opção direta"
            );
        }

        // Move para "Submenu Fixar" para abrir o submenu e clica DUAS vezes em "Fixar Coluna #1":
        // tanto o menu raiz quanto o submenu devem permanecer abertos!
        let pos_sub_btn = p.btn_submenu_pos;
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_sub_btn)]);
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_sub_btn)]);
        assert_ne!(p.item_submenu_pos, Pos2::ZERO, "Submenu deve abrir por hover");

        let pos_sub_opt = p.item_submenu_pos;
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_sub_opt)]);
        for esperado in 1..=2 {
            rodar_frame(
                &ctx,
                screen,
                &mut p,
                vec![Event::PointerButton {
                    pos: pos_sub_opt,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                }],
            );
            rodar_frame(
                &ctx,
                screen,
                &mut p,
                vec![Event::PointerButton {
                    pos: pos_sub_opt,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_sub_opt)]);
            assert_eq!(p.cliques_submenu, esperado);
            assert!(
                egui::Popup::is_any_open(&ctx),
                "Árvore de menu + submenu deve permanecer aberta após clique #{esperado}"
            );
        }

        // Retira o ponteiro do mouse de cima da região do menu e submenu (ex.: área livre (700, 500))
        // SEM precisar clicar: a árvore de menu deve fechar automaticamente!
        let pos_fora = Pos2::new(700.0, 500.0);
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_fora)]);
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_fora)]);
        assert!(
            !egui::Popup::is_any_open(&ctx),
            "Árvore de menu deve fechar automaticamente quando o ponteiro sai da região do menu aberto"
        );
    }

    #[test]
    fn test_menu_contexto_persistente_mantem_aberto_ao_clicar_e_fecha_ao_sair() {
        use super::{
            finalizar_rastreamento_menus_frame, iniciar_rastreamento_menus_frame,
            menu_contexto_persistente,
        };
        use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

        #[derive(Default)]
        struct CtxProbe {
            cliques_ctx: usize,
            celula_pos: Pos2,
            opcao_ctx_pos: Pos2,
        }

        let ctx = Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0)));
        let mut p = CtxProbe::default();

        fn rodar_frame(ctx: &Context, screen: Option<Rect>, p: &mut CtxProbe, eventos: Vec<Event>) {
            let raw = RawInput {
                screen_rect: screen,
                events: eventos,
                ..Default::default()
            };
            let mut out = ctx.run_ui(raw, |ui| {
                iniciar_rastreamento_menus_frame(ui.ctx());
                egui::CentralPanel::default().show(ui, |ui| {
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(160.0, 32.0), egui::Sense::click());
                    p.celula_pos = rect.center();
                    menu_contexto_persistente(&resp, |ui| {
                        ui.set_min_width(180.0);
                        let b = ui.button("📌 Fixar Linha");
                        p.opcao_ctx_pos = b.rect.center();
                        if b.clicked() {
                            p.cliques_ctx += 1;
                        }
                    });
                });
                finalizar_rastreamento_menus_frame(ui.ctx());
            });
            out.textures_delta.clear();
        }

        rodar_frame(&ctx, screen, &mut p, vec![]);
        let pos_cel = p.celula_pos;

        // Botão direito abre o menu de contexto
        rodar_frame(
            &ctx,
            screen,
            &mut p,
            vec![
                Event::PointerMoved(pos_cel),
                Event::PointerButton {
                    pos: pos_cel,
                    button: PointerButton::Secondary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        rodar_frame(
            &ctx,
            screen,
            &mut p,
            vec![Event::PointerButton {
                pos: pos_cel,
                button: PointerButton::Secondary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_cel)]);
        assert!(egui::Popup::is_any_open(&ctx), "Menu de contexto deve abrir");

        // Clica duas vezes na opção dentro do menu de contexto: permanece aberto!
        let pos_opt = p.opcao_ctx_pos;
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_opt)]);
        for esperado in 1..=2 {
            rodar_frame(
                &ctx,
                screen,
                &mut p,
                vec![Event::PointerButton {
                    pos: pos_opt,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                }],
            );
            rodar_frame(
                &ctx,
                screen,
                &mut p,
                vec![Event::PointerButton {
                    pos: pos_opt,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_opt)]);
            assert_eq!(p.cliques_ctx, esperado);
            assert!(egui::Popup::is_any_open(&ctx));
        }

        // Retira o ponteiro da área do menu de contexto: fecha automaticamente!
        let pos_fora = Pos2::new(650.0, 480.0);
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_fora)]);
        rodar_frame(&ctx, screen, &mut p, vec![Event::PointerMoved(pos_fora)]);
        assert!(!egui::Popup::is_any_open(&ctx));
    }

    #[test]
    fn test_custom_color_picker_popup_opens_on_click() {
        use super::*;
        use egui::*;
        let ctx = Context::default();
        let loc = crate::locale::Localizer::new();
        let mut color = Color32::from_rgb(46, 125, 50);

        let mut btn_pos = Pos2::ZERO;
        let mut popup_was_open = false;

        let screen = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));

        let raw = RawInput {
            screen_rect: screen,
            ..Default::default()
        };
        let mut out = ctx.run_ui(raw, |ui| {
            let resp = render_custom_color_edit_button(
                ui,
                Id::unique("test_color_btn"),
                &mut color,
                &loc,
            );
            btn_pos = resp.rect.center();
        });
        out.textures_delta.clear();

        let raw_click = RawInput {
            screen_rect: screen,
            events: vec![
                Event::PointerMoved(btn_pos),
                Event::PointerButton {
                    pos: btn_pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
                Event::PointerButton {
                    pos: btn_pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        };

        let mut out_click = ctx.run_ui(raw_click, |ui| {
            let _ = render_custom_color_edit_button(
                ui,
                Id::unique("test_color_btn"),
                &mut color,
                &loc,
            );
            let popup_id = Id::unique("test_color_btn").with("custom_color_picker_popup");
            popup_was_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        });
        out_click.textures_delta.clear();

        assert!(popup_was_open, "Popup da paleta de cores DEVE estar aberto após clique!");

        // Frame 2: Sem novos cliques, o popup permanece aberto
        let raw_hold = RawInput {
            screen_rect: screen,
            ..Default::default()
        };
        let mut out_hold = ctx.run_ui(raw_hold, |ui| {
            let _ = render_custom_color_edit_button(
                ui,
                Id::unique("test_color_btn"),
                &mut color,
                &loc,
            );
            let popup_id = Id::unique("test_color_btn").with("custom_color_picker_popup");
            assert!(egui::Popup::is_id_open(ui.ctx(), popup_id), "Popup deve continuar aberto nos frames seguintes!");
        });
        out_hold.textures_delta.clear();

        // Frame 3: Fechando explicitamente com Popup::close_id (como no botão ❌)
        let popup_id = Id::unique("test_color_btn").with("custom_color_picker_popup");
        egui::Popup::close_id(&ctx, popup_id);

        let mut out_closed = ctx.run_ui(RawInput::default(), |ui| {
            let _ = render_custom_color_edit_button(
                ui,
                Id::unique("test_color_btn"),
                &mut color,
                &loc,
            );
            assert!(!egui::Popup::is_id_open(ui.ctx(), popup_id), "Popup deve estar fechado após fechar!");
        });
        out_closed.textures_delta.clear();
    }

    #[test]
    fn test_parse_hex_color() {
        use super::*;
        assert_eq!(parse_hex_color("#00ff00"), Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#00FF00"), Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("00ff00"), Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#0f0"), Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("0f0"), Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#ff5733"), Some(Color32::from_rgb(255, 87, 51)));
        assert_eq!(parse_hex_color("#00ff00ff"), Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("  #123456  "), Some(Color32::from_rgb(0x12, 0x34, 0x56)));

        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#12"), None);
        assert_eq!(parse_hex_color("#12345"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn test_custom_color_picker_html_edit() {
        use super::*;
        use egui::*;
        let ctx = Context::default();
        let loc = crate::locale::Localizer::new();
        let mut color = Color32::from_rgb(0, 255, 0);

        let btn_id = Id::unique("test_html_edit_btn");
        let popup_id = btn_id.with("custom_color_picker_popup");
        let hex_id = btn_id.with("html_hex_edit_buf");

        Popup::open_id(&ctx, popup_id);

        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let _ = render_custom_color_edit_button(ui, btn_id, &mut color, &loc);
        });
        out.textures_delta.clear();

        let buf = ctx.data(|d| d.get_temp::<String>(hex_id));
        assert_eq!(buf, Some("#00ff00".to_string()));

        // Simula que outro controle mudou a cor para azul
        color = Color32::from_rgb(0, 0, 255);
        let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
            let _ = render_custom_color_edit_button(ui, btn_id, &mut color, &loc);
        });
        out2.textures_delta.clear();

        let buf2 = ctx.data(|d| d.get_temp::<String>(hex_id));
        assert_eq!(buf2, Some("#0000ff".to_string()));
    }
}

