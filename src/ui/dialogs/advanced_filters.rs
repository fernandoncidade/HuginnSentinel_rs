use egui::{Color32, RichText, ScrollArea, TextEdit, Vec2};

use crate::core::types::OperationType;
use crate::locale::Localizer;
use crate::services::GerenciadorTabela;
use crate::tools::filters::{ContadoresEventos, FiltroEstado};

pub struct AdvancedFiltersDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
}

impl Default for AdvancedFiltersDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
        }
    }
}

impl AdvancedFiltersDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn mostrar(
        &mut self,
        ctx: &egui::Context,
        loc: &Localizer,
        filtros: &mut FiltroEstado,
        filtros_sujos: &mut bool,
        contadores: &ContadoresEventos,
        gerenciador_tabela: &GerenciadorTabela,
    ) {
        if !self.aberto {
            return;
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        let mut alterado = false;

        super::show_modeless_dialog(
            ctx,
            "dialog_advanced_filters",
            format!("🔍 {}", loc.t("advanced_filters")),
            [650.0, 520.0],
            [480.0, 360.0],
            &mut state,
            |ui, fechar| {
                let footer_h = 36.0;
                let scroll_h = (ui.available_height() - footer_h).max(120.0);

                ScrollArea::vertical()
                    .id_salt("advanced_filters_scroll")
                    .max_height(scroll_h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                    // 1. Grupo de Operações (operation_filter)
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(
                                RichText::new(loc.t("operation_filter"))
                                    .color(Color32::from_rgb(33, 150, 243)),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if crate::ui::widgets::render_centered_emoji_button(
                                        ui,
                                        "",
                                        loc.t("unselect_all"),
                                    )
                                    .clicked()
                                    {
                                        filtros.op_movido = false;
                                        filtros.op_renomeado = false;
                                        filtros.op_adicionado = false;
                                        filtros.op_excluido = false;
                                        filtros.op_modificado = false;
                                        filtros.op_escaneado = false;
                                        alterado = true;
                                    }
                                    if crate::ui::widgets::render_centered_emoji_button(
                                        ui,
                                        "",
                                        loc.t("select_all"),
                                    )
                                    .clicked()
                                    {
                                        filtros.op_movido = true;
                                        filtros.op_renomeado = true;
                                        filtros.op_adicionado = true;
                                        filtros.op_excluido = true;
                                        filtros.op_modificado = true;
                                        filtros.op_escaneado = true;
                                        alterado = true;
                                    }
                                },
                            );
                        });
                        ui.add_space(6.0);

                        let ops = [
                            (
                                OperationType::Movido,
                                &mut filtros.op_movido,
                                loc.t("op_moved"),
                                contadores.movidos,
                            ),
                            (
                                OperationType::Renomeado,
                                &mut filtros.op_renomeado,
                                loc.t("op_renamed"),
                                contadores.renomeados,
                            ),
                            (
                                OperationType::Adicionado,
                                &mut filtros.op_adicionado,
                                loc.t("op_added"),
                                contadores.adicionados,
                            ),
                            (
                                OperationType::Excluido,
                                &mut filtros.op_excluido,
                                loc.t("op_deleted"),
                                contadores.excluidos,
                            ),
                            (
                                OperationType::Modificado,
                                &mut filtros.op_modificado,
                                loc.t("op_modified"),
                                contadores.modificados,
                            ),
                            (
                                OperationType::Escaneado,
                                &mut filtros.op_escaneado,
                                loc.t("op_scanned"),
                                contadores.escaneados,
                            ),
                        ];

                        ui.columns(2, |cols| {
                            for (i, (op, checked, rotulo, count)) in ops.into_iter().enumerate() {
                                let target_col = &mut cols[i % 2];
                                let (bg, _fg) = gerenciador_tabela.obter_cores_operacao(op);
                                target_col.horizontal(|ui| {
                                    if ui.checkbox(checked, "").changed() {
                                        alterado = true;
                                    }
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(14.0, 14.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 3_u8, bg);
                                    ui.label(format!("{} ({})", rotulo, count));
                                });
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Grupo de Busca Rápida (search)
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(loc.t("search")).color(Color32::from_rgb(76, 175, 80)),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            let hint_busca = loc.t("search_placeholder");
                            if ui
                                .add(
                                    TextEdit::singleline(&mut filtros.texto_busca)
                                        .hint_text(hint_busca)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                alterado = true;
                            }
                            if !filtros.texto_busca.is_empty() && ui.small_button("✖").clicked() {
                                filtros.texto_busca.clear();
                                alterado = true;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 3. Grupo de Filtro de Extensão (extension_filter)
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(loc.t("extension_filter"))
                                .color(Color32::from_rgb(255, 152, 0)),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("📄");
                            let hint_ext = "pdf, txt, doc, docx, exe, dll, zip...";
                            if ui
                                .add(
                                    TextEdit::singleline(&mut filtros.filtro_extensao)
                                        .hint_text(hint_ext)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                alterado = true;
                            }
                            if !filtros.filtro_extensao.is_empty()
                                && crate::ui::widgets::render_centered_emoji_button(ui, "✖", "").clicked()
                            {
                                filtros.filtro_extensao.clear();
                                alterado = true;
                            }
                        });

                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(loc.t("chips_label")).size(11.0).color(ui.visuals().weak_text_color()));
                            let preset_exts = [
                                "exe", "dll", "pdf", "docx", "txt", "zip", "ps1", "bat", "jpg",
                                "png",
                            ];
                            for &ext in &preset_exts {
                                let present = filtros
                                    .filtro_extensao
                                    .split(',')
                                    .any(|s| s.trim().eq_ignore_ascii_case(ext));
                                if ui.selectable_label(present, format!(".{}", ext)).clicked() {
                                    let mut current: Vec<String> = filtros
                                        .filtro_extensao
                                        .split(',')
                                        .map(|s| s.trim().to_lowercase())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    if present {
                                        current.retain(|s| s != ext);
                                    } else {
                                        current.push(ext.to_string());
                                    }
                                    filtros.filtro_extensao = current.join(", ");
                                    alterado = true;
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 4. Grupo de Nível de Risco (risk_level)
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(loc.t("risk_level"))
                                .color(Color32::from_rgb(233, 30, 99)),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let niveis = ["Todos", "Crítico", "Alto", "Médio", "Baixo"];
                            for &n in &niveis {
                                let rotulo = loc.traduzir_risco(n);
                                if ui
                                    .selectable_value(
                                        &mut filtros.filtro_risco,
                                        n.to_string(),
                                        rotulo,
                                    )
                                    .changed()
                                {
                                    alterado = true;
                                }
                            }
                        });

                        if contadores.criticos > 0 || contadores.altos > 0 {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if contadores.criticos > 0 {
                                    let txt = format!(
                                        "⚠️ {} {}",
                                        contadores.criticos,
                                        loc.t("risk_critical")
                                    );
                                    ui.colored_label(Color32::from_rgb(244, 67, 54), txt);
                                }
                                if contadores.altos > 0 {
                                    let txt =
                                        format!("🔸 {} {}", contadores.altos, loc.t("risk_high"));
                                    ui.colored_label(Color32::from_rgb(255, 152, 0), txt);
                                }
                            });
                        }
                    });

                    ui.add_space(8.0);

                    // 5. Grupo de Filtro por Data e Hora com as 6 Flags de Ignorar (date_filter)
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(loc.t("date_filter"))
                                .color(Color32::from_rgb(156, 39, 176)),
                        );
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new(loc.t("start_date")).strong());
                            let hint_data = loc.t("date_format_hint");
                            if ui
                                .add(
                                    TextEdit::singleline(&mut filtros.data_inicio)
                                        .hint_text(&hint_data),
                                )
                                .changed()
                            {
                                alterado = true;
                            }

                            ui.label(RichText::new(loc.t("end_date")).strong());
                            if ui
                                .add(
                                    TextEdit::singleline(&mut filtros.data_fim)
                                        .hint_text(&hint_data),
                                )
                                .changed()
                            {
                                alterado = true;
                            }
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(loc.t("presets_label"))
                                    .size(11.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                            if crate::ui::widgets::render_centered_emoji_button(
                                ui,
                                "",
                                loc.t("preset_today"),
                            )
                            .clicked()
                            {
                                let hoje =
                                    chrono::Local::now().format("%Y-%m-%d 00:00:00").to_string();
                                filtros.data_inicio = hoje;
                                alterado = true;
                            }
                            if crate::ui::widgets::render_centered_emoji_button(
                                ui,
                                "",
                                loc.t("preset_7_days"),
                            )
                            .clicked()
                            {
                                let d7 = (chrono::Local::now() - chrono::Duration::days(7))
                                    .format("%Y-%m-%d 00:00:00")
                                    .to_string();
                                filtros.data_inicio = d7;
                                alterado = true;
                            }
                            if crate::ui::widgets::render_centered_emoji_button(
                                ui,
                                "",
                                loc.t("preset_30_days"),
                            )
                            .clicked()
                            {
                                let d30 = (chrono::Local::now() - chrono::Duration::days(30))
                                    .format("%Y-%m-%d 00:00:00")
                                    .to_string();
                                filtros.data_inicio = d30;
                                alterado = true;
                            }
                            if crate::ui::widgets::render_centered_emoji_button(
                                ui,
                                "",
                                loc.t("clear"),
                            )
                            .clicked()
                            {
                                filtros.data_inicio.clear();
                                filtros.data_fim.clear();
                                alterado = true;
                            }
                        });

                        ui.separator();
                        ui.label(
                            RichText::new(loc.t("date_exception_options"))
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(2.0);

                        if ui
                            .checkbox(&mut filtros.ignorar_data_mover, loc.t("ignore_move_filter"))
                            .changed()
                        {
                            alterado = true;
                        }
                        if ui
                            .checkbox(
                                &mut filtros.ignorar_data_renomeados,
                                loc.t("ignore_rename_filter"),
                            )
                            .changed()
                        {
                            alterado = true;
                        }
                        if ui
                            .checkbox(
                                &mut filtros.ignorar_data_adicionados,
                                loc.t("ignore_add_filter"),
                            )
                            .changed()
                        {
                            alterado = true;
                        }
                        if ui
                            .checkbox(
                                &mut filtros.ignorar_data_excluidos,
                                loc.t("ignore_delete_filter"),
                            )
                            .changed()
                        {
                            alterado = true;
                        }
                        if ui
                            .checkbox(
                                &mut filtros.ignorar_data_modificados,
                                loc.t("ignore_modified_filter"),
                            )
                            .changed()
                        {
                            alterado = true;
                        }
                        if ui
                            .checkbox(
                                &mut filtros.ignorar_data_escaneados,
                                loc.t("ignore_scanned_filter"),
                            )
                            .changed()
                        {
                            alterado = true;
                        }
                    });

                    ui.add_space(8.0);
                });

                ui.separator();
                ui.add_space(4.0);

                // Rodapé fixo com Ações (Sempre visível no canto inferior da janela)
                ui.horizontal(|ui| {
                    let total_operacoes = contadores.total_operacoes_ativas(filtros);
                    let total_str = total_operacoes.to_string();
                    ui.label(RichText::new(loc.t_args("total_events", &[&total_str])).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if crate::ui::widgets::render_centered_emoji_button_sized(
                            ui,
                            "",
                            loc.t("close"),
                            egui::vec2(85.0, 24.0),
                        )
                        .clicked()
                        {
                            *fechar = true;
                        }
                        if crate::ui::widgets::render_centered_emoji_button_sized(
                            ui,
                            "🔄",
                            loc.t("clear_filters"),
                            egui::vec2(140.0, 24.0),
                        )
                        .clicked()
                        {
                            filtros.resetar();
                            alterado = true;
                        }
                    });
                });
            },
        );

        self.aberto = state.aberto;
        self.elevar_z_order = state.elevar_z_order;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
        if alterado {
            *filtros_sujos = true;
        }
    }
}
