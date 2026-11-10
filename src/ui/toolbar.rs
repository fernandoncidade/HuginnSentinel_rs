use egui::{Color32, RichText, Ui};

use super::app::HuginnApp;

pub fn renderizar_toolbar(app: &mut HuginnApp, ui: &mut Ui) {
    ui.add_space(3.0);

    let moldura = egui::Frame::NONE
        .fill(ui.visuals().widgets.noninteractive.bg_fill)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(5_u8)
        .inner_margin(egui::Margin::symmetric(8, 4));

    moldura.show(ui, |ui| {
        let toolbar_height = 28.0;
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), toolbar_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if super::widgets::render_centered_emoji_button(ui, "📁", app.loc.t("select_dir")).clicked() {
                    app.selecionar_diretorio();
                }

                if app.scanner.is_some() {
                    if super::widgets::render_colored_emoji_button(
                        ui,
                        "⏹",
                        app.loc.t("stop_scan"),
                        Color32::from_rgb(244, 67, 54),
                    )
                    .clicked()
                    {
                        app.parar_escaneamento();
                    }
                } else {
                    if super::widgets::render_centered_emoji_button(ui, "🔍", app.loc.t("scan_dir")).clicked() {
                        app.acionar_botao_escanear();
                    }
                }

                ui.separator();

                if app.observador.is_some() {
                    if super::widgets::render_colored_emoji_button(
                        ui,
                        "⏹",
                        app.loc.t("stop_monitor"),
                        Color32::from_rgb(244, 67, 54),
                    )
                    .clicked()
                    {
                        app.parar_monitoramento();
                    }
                } else {
                    if super::widgets::render_colored_emoji_button(
                        ui,
                        "▶",
                        app.loc.t("start_monitor"),
                        Color32::from_rgb(76, 175, 80),
                    )
                    .clicked()
                    {
                        app.acionar_botao_monitorar();
                    }
                }

                let is_paused = app.is_pausado();
                let pause_resp = if is_paused {
                    super::widgets::render_centered_emoji_button(ui, "⏯", app.loc.t("resume_monitor"))
                } else {
                    super::widgets::render_centered_emoji_button(ui, "⏸", app.loc.t("pause_monitor"))
                };
                if pause_resp.clicked() {
                    app.alternar_pausa();
                }

                ui.separator();

                if super::widgets::render_centered_emoji_button(ui, "💾", app.loc.t("export_data")).clicked() {
                    app.exportar_dados();
                }

                if super::widgets::render_centered_emoji_button(ui, "🗑", app.loc.t("clear_data")).clicked() {
                    app.limpar_dados();
                }

                ui.separator();

                if super::widgets::render_colored_emoji_button(
                    ui,
                    "📊",
                    app.loc.t("statistics"),
                    Color32::from_rgb(33, 150, 243),
                )
                .clicked()
                {
                    app.abrir_dialogo_estatisticas();
                }

                // Posição restaurada no eixo horizontal: pasta ativa à direita
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref dir) = app.diretorio_ativo {
                        let lbl = app.loc.t_args("active_folder", &[&dir.to_string_lossy()]);
                        ui.label(RichText::new(lbl).size(11.5).color(ui.visuals().weak_text_color()));
                    } else {
                        let lbl = app.loc.t("no_folder_selected");
                        ui.label(RichText::new(lbl).size(11.5).color(ui.visuals().weak_text_color()));
                    }
                });
            },
        );
    });

    ui.add_space(2.0);
}
