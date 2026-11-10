use egui::{Button, MenuBar, ScrollArea, Ui, Vec2};

use super::app::HuginnApp;
use super::widgets::{
    config_menu_persistente, menu_persistente, registrar_retangulo_menu_ativo,
};
use crate::core::types::OperationType;
use crate::data::charts::TipoGrafico;
use crate::locale::IDIOMAS_SUPORTADOS;

pub fn renderizar_menu_bar(app: &mut HuginnApp, ui: &mut Ui) {
    MenuBar::new()
        .config(config_menu_persistente())
        .ui(ui, |ui| {
            // ==========================================
            // 1. MENU ARQUIVO (gmui_05_configurar_menu_arquivo)
            // ==========================================
            let resp_arquivo = menu_persistente(ui, app.loc.t("file_menu"), |ui| {
                ui.set_min_width(240.0);
                if ui
                    .add(
                        Button::new(format!("📁 {}", app.loc.t("select_dir")))
                            .shortcut_text("Ctrl+Shift+O"),
                    )
                    .clicked()
                {
                    app.selecionar_diretorio();
                }

                ui.separator();

                if ui
                    .add(
                        Button::new(format!("▶ {}", app.loc.t("start_stop")))
                            .shortcut_text("Ctrl+R"),
                    )
                    .clicked()
                {
                    app.alternar_analise_diretorio();
                }

                if ui
                    .add(
                        Button::new(format!("⏸ {}", app.loc.t("pause_analysis")))
                            .shortcut_text("Ctrl+P"),
                    )
                    .clicked()
                {
                    app.alternar_pausa();
                }

                ui.separator();

                if ui
                    .add(
                        Button::new(format!("💾 {}", app.loc.t("save_as")))
                            .shortcut_text("Ctrl+Shift+S"),
                    )
                    .clicked()
                {
                    app.abrir_salvar_como();
                }

                if ui
                    .add(Button::new(format!("💾 {}", app.loc.t("save"))).shortcut_text("Ctrl+S"))
                    .clicked()
                {
                    app.salvar_dados();
                }

                ui.separator();

                if ui
                    .add(
                        Button::new(format!("📊 {}", app.loc.t("statistics")))
                            .shortcut_text("Ctrl+G"),
                    )
                    .clicked()
                {
                    app.abrir_dialogo_estatisticas();
                }

                if ui
                    .add(
                        Button::new(format!("🗑️ {}", app.loc.t("clear_data")))
                            .shortcut_text("Ctrl+L"),
                    )
                    .clicked()
                {
                    app.limpar_dados();
                }

                if ui
                    .button(format!("🗄️ {}", app.loc.t("clear_events_database")))
                    .clicked()
                {
                    app.abrir_dialogo_limpeza_alvo(crate::ui::dialogs::cleanup::LimpezaAlvo::BancoDados);
                }

                ui.separator();

                if ui
                    .add(Button::new(format!("❌ {}", app.loc.t("exit"))).shortcut_text("Ctrl+Q"))
                    .clicked()
                {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            });

            // ==========================================
            // 2. MENU CONFIGURAÇÕES (gmui_06_configurar_menu_configuracoes)
            // ==========================================
            let resp_config = menu_persistente(ui, app.loc.t("settings"), |ui| {
                ui.set_min_width(240.0);

                // 2.0.1 Submenu Escaneamento
                menu_persistente(ui, format!("🔍 {}", app.loc.t("scanning_menu")), |ui| {
                    ui.set_min_width(340.0);

                    // Mensagem informativa de status
                    let info_text = if app.scan_monitor_config.perguntar_ao_escanear {
                        app.loc.t("scan_notice_active_info")
                    } else {
                        app.loc.t("scan_notice_disabled_info")
                    };
                    ui.label(
                        egui::RichText::new(format!("ℹ️ {}", info_text))
                            .size(11.5)
                            .italics(),
                    );

                    ui.separator();

                    let mut mudou = false;
                    let mut perguntar = app.scan_monitor_config.perguntar_ao_escanear;
                    if ui
                        .checkbox(&mut perguntar, app.loc.t("enable_scan_prompt"))
                        .changed()
                    {
                        app.scan_monitor_config.perguntar_ao_escanear = perguntar;
                        mudou = true;
                    }

                    ui.separator();

                    ui.label(
                        egui::RichText::new(app.loc.t("scan_default_behavior_title"))
                            .strong(),
                    );

                    let mut acao = app.scan_monitor_config.acao_escaneamento;
                    if ui
                        .radio_value(
                            &mut acao,
                            crate::core::ScanActionOption::ApenasEscanear,
                            app.loc.t("scan_opt_only_scan"),
                        )
                        .changed()
                    {
                        app.scan_monitor_config.acao_escaneamento = acao;
                        mudou = true;
                    }

                    if ui
                        .radio_value(
                            &mut acao,
                            crate::core::ScanActionOption::EscanearEMonitorar,
                            app.loc.t("scan_opt_scan_and_monitor"),
                        )
                        .changed()
                    {
                        app.scan_monitor_config.acao_escaneamento = acao;
                        mudou = true;
                    }

                    ui.separator();

                    if ui
                        .button(format!("🔍 {}", app.loc.t("scan_run_only_now")))
                        .clicked()
                    {
                        app.executar_apenas_escaneamento();
                    }

                    if ui
                        .button(format!("▶ {}", app.loc.t("scan_run_and_monitor_now")))
                        .clicked()
                    {
                        app.executar_escanear_e_monitorar();
                    }

                    if mudou {
                        app.salvar_configuracao_scan_monitor();
                    }
                });

                // 2.0.2 Submenu Monitoramento
                menu_persistente(ui, format!("▶ {}", app.loc.t("monitoring_menu")), |ui| {
                    ui.set_min_width(340.0);

                    // Mensagem informativa de status
                    let info_text = if app.scan_monitor_config.perguntar_ao_monitorar {
                        app.loc.t("monitor_notice_active_info")
                    } else {
                        app.loc.t("monitor_notice_disabled_info")
                    };
                    ui.label(
                        egui::RichText::new(format!("ℹ️ {}", info_text))
                            .size(11.5)
                            .italics(),
                    );

                    ui.separator();

                    let mut mudou = false;
                    let mut perguntar = app.scan_monitor_config.perguntar_ao_monitorar;
                    if ui
                        .checkbox(&mut perguntar, app.loc.t("enable_monitor_prompt"))
                        .changed()
                    {
                        app.scan_monitor_config.perguntar_ao_monitorar = perguntar;
                        mudou = true;
                    }

                    ui.separator();

                    ui.label(
                        egui::RichText::new(app.loc.t("monitor_default_behavior_title"))
                            .strong(),
                    );

                    let mut acao = app.scan_monitor_config.acao_monitoramento;
                    if ui
                        .radio_value(
                            &mut acao,
                            crate::core::MonitorActionOption::ApenasMonitorar,
                            app.loc.t("monitor_opt_only_monitor"),
                        )
                        .changed()
                    {
                        app.scan_monitor_config.acao_monitoramento = acao;
                        mudou = true;
                    }

                    if ui
                        .radio_value(
                            &mut acao,
                            crate::core::MonitorActionOption::EscanearEDepoisMonitorar,
                            app.loc.t("monitor_opt_scan_first_then_monitor"),
                        )
                        .changed()
                    {
                        app.scan_monitor_config.acao_monitoramento = acao;
                        mudou = true;
                    }

                    ui.separator();

                    if ui
                        .button(format!("▶ {}", app.loc.t("monitor_run_only_now")))
                        .clicked()
                    {
                        app.executar_apenas_monitoramento();
                    }

                    if ui
                        .button(format!("🔍 {}", app.loc.t("monitor_run_scan_then_now")))
                        .clicked()
                    {
                        app.executar_escanear_e_depois_monitorar();
                    }

                    if mudou {
                        app.salvar_configuracao_scan_monitor();
                    }
                });

                ui.separator();

                // 2.1 Submenu Filtros
                menu_persistente(ui, format!("🔍 {}", app.loc.t("filters")), |ui| {
                    ui.set_min_width(220.0);
                    if ui
                        .checkbox(&mut app.filtros.op_movido, app.loc.t("op_moved"))
                        .changed()
                    {
                        app.filtros_sujos = true;
                    }
                    if ui
                        .checkbox(&mut app.filtros.op_renomeado, app.loc.t("op_renamed"))
                        .changed()
                    {
                        app.filtros_sujos = true;
                    }
                    if ui
                        .checkbox(&mut app.filtros.op_adicionado, app.loc.t("op_added"))
                        .changed()
                    {
                        app.filtros_sujos = true;
                    }
                    if ui
                        .checkbox(&mut app.filtros.op_excluido, app.loc.t("op_deleted"))
                        .changed()
                    {
                        app.filtros_sujos = true;
                    }
                    if ui
                        .checkbox(&mut app.filtros.op_modificado, app.loc.t("op_modified"))
                        .changed()
                    {
                        app.filtros_sujos = true;
                    }
                    if ui
                        .checkbox(&mut app.filtros.op_escaneado, app.loc.t("op_scanned"))
                        .changed()
                    {
                        app.filtros_sujos = true;
                    }

                    ui.separator();

                    if ui.button(app.loc.t("advanced_filters")).clicked() {
                        app.abrir_dialogo_filtros_avancados();
                    }
                });

                // 2.2 Submenu Configurar Colunas (gmui_11_criar_submenu_colunas)
                menu_persistente(
                    ui,
                    format!("📑 {}", app.loc.t("configure_columns")),
                    |ui| {
                        ui.set_min_width(240.0);
                        menu_persistente(ui, format!("📑 {}", app.loc.t("columns")), |ui| {
                            let colunas = app.gerenciador_colunas.colunas().to_vec();
                            renderizar_menu_multi_coluna(ui, &colunas, |ui, col| {
                                let mut visivel = col.visivel;
                                let nome = app.loc.get_text(col.translation_key);
                                if ui.checkbox(&mut visivel, nome).changed() {
                                    app.gerenciador_colunas.set_visibilidade(col.key, visivel);
                                    app.gerenciador_colunas.salvar_configuracoes();
                                    app.filtros_sujos = true;
                                }
                            });
                        });

                        if ui
                            .button(format!("📑 {}", app.loc.t("open_column_manager_dialog")))
                            .clicked()
                        {
                            app.abrir_dialogo_colunas();
                        }

                        ui.separator();

                        if ui
                            .button(format!("↔️ {}", app.loc.t("autofit_adjust_column_widths")))
                            .clicked()
                        {
                            app.gerenciador_colunas
                                .autofit_todas_colunas_ao_cabecalho(&app.loc);
                        }

                        if ui
                            .add(
                                Button::new(app.loc.t("select_all_columns"))
                                    .shortcut_text("Ctrl+A"),
                            )
                            .clicked()
                        {
                            app.selecionar_todas_colunas();
                        }

                        if ui.button(app.loc.t("reset_columns")).clicked() {
                            app.resetar_colunas();
                        }

                        ui.separator();

                        let mut ordenacao_ativa = app.gerenciador_tabela.is_ordenacao_habilitada();
                        if ui
                            .checkbox(
                                &mut ordenacao_ativa,
                                format!("{} (Ctrl+O)", app.loc.t("sort_rows")),
                            )
                            .changed()
                        {
                            app.alternar_ordenacao_linhas();
                        }
                    },
                );

                // 2.3 Submenu Colorir Colunas (gmui_13_criar_submenu_colunas_coloridas)
                menu_persistente(ui, format!("🎨 {}", app.loc.t("color_columns")), |ui| {
                    ui.set_min_width(240.0);
                    menu_persistente(ui, format!("📑 {}", app.loc.t("columns")), |ui| {
                        let colunas = app.gerenciador_colunas.colunas().to_vec();
                        renderizar_menu_multi_coluna(ui, &colunas, |ui, col| {
                            let mut colorir = app.gerenciador_tabela.coluna_deve_colorir(col.key);
                            let nome = app.loc.get_text(col.translation_key);
                            if ui.checkbox(&mut colorir, nome).changed() {
                                app.gerenciador_tabela.set_coluna_colorir(col.key, colorir);
                            }
                        });
                    });

                    if ui
                        .button(format!("🎨 {}", app.loc.t("open_color_manager_dialog")))
                        .clicked()
                    {
                        app.abrir_dialogo_cores();
                    }

                    ui.separator();

                    if ui
                        .add(
                            Button::new(app.loc.t("select_all_columns"))
                                .shortcut_text("Ctrl+Shift+A"),
                        )
                        .clicked()
                    {
                        app.selecionar_todas_cores_colunas();
                    }

                    if ui.button(app.loc.t("reset_column_colors")).clicked() {
                        app.resetar_cores_colunas();
                    }
                });

                // 2.3.1 Submenu Fixar Linhas (Congelamento arbitrário de linhas na rolagem vertical)
                menu_persistente(
                    ui,
                    format!("📌 {}", app.loc.t("freeze_rows_menu")),
                    |ui| {
                        ui.set_min_width(290.0);

                        // Opção para a 1ª linha (onde estão os cabeçalhos das colunas)
                        let mut fixar_cabecalho = app.spreadsheet_grid.layout.freeze_header_row;
                        if ui
                            .checkbox(
                                &mut fixar_cabecalho,
                                app.loc.t("freeze_column_headers_row"),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.layout.freeze_header_row = fixar_cabecalho;
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }

                        let total_linhas = app.indices_filtrados.len();
                        let linha_ativa = app
                            .spreadsheet_grid
                            .selection
                            .active_cell
                            .map(|c| c.row)
                            .or(app.linha_selecionada)
                            .filter(|&r| r < total_linhas);

                        if let Some(r_idx) = linha_ativa {
                            let row_num_str = (r_idx + 1).to_string();
                            let mut fixa_sel = app.spreadsheet_grid.layout.is_row_pinned(r_idx);
                            if ui
                                .checkbox(
                                    &mut fixa_sel,
                                    app.loc.t_args("freeze_selected_row_fmt", &[&row_num_str]),
                                )
                                .changed()
                            {
                                app.spreadsheet_grid.layout.set_row_pinned(r_idx, fixa_sel);
                                app.spreadsheet_grid
                                    .layout
                                    .salvar_configuracao_painel_fixo();
                            }
                        }

                        ui.separator();

                        // Submenu visual multi-coluna para selecionar diretamente linhas da tabela
                        menu_persistente(
                            ui,
                            format!("📑 {}", app.loc.t("table_rows_submenu")),
                            |ui| {
                                if total_linhas == 0 {
                                    ui.set_min_width(220.0);
                                    ui.label(app.loc.t("no_table_rows_available"));
                                } else {
                                    let mut candidatos: std::collections::BTreeSet<usize> =
                                        (0..total_linhas.min(54)).collect();
                                    for &pr in &app.spreadsheet_grid.layout.pinned_rows {
                                        if pr < total_linhas {
                                            candidatos.insert(pr);
                                        }
                                    }
                                    let lista_linhas: Vec<(usize, String)> = candidatos
                                        .into_iter()
                                        .map(|r_idx| {
                                            let num_s = (r_idx + 1).to_string();
                                            let base_lbl =
                                                app.loc.t_args("row_item_fmt", &[&num_s]);
                                            let preview = app
                                                .indices_filtrados
                                                .get(r_idx)
                                                .and_then(|&ev_i| app.eventos.get(ev_i))
                                                .map(|ev| {
                                                    let mut n = ev.nome.clone();
                                                    if n.chars().count() > 20 {
                                                        n = format!(
                                                            "{}…",
                                                            n.chars().take(19).collect::<String>()
                                                        );
                                                    }
                                                    format!("{} — {}", base_lbl, n)
                                                })
                                                .unwrap_or(base_lbl);
                                            (r_idx, preview)
                                        })
                                        .collect();

                                    renderizar_menu_multi_coluna(
                                        ui,
                                        &lista_linhas,
                                        |ui, (r_idx, label)| {
                                            let mut pinned =
                                                app.spreadsheet_grid.layout.is_row_pinned(*r_idx);
                                            if ui.checkbox(&mut pinned, label.as_str()).changed() {
                                                app.spreadsheet_grid
                                                    .layout
                                                    .set_row_pinned(*r_idx, pinned);
                                                app.spreadsheet_grid
                                                    .layout
                                                    .salvar_configuracao_painel_fixo();
                                            }
                                        },
                                    );
                                }
                            },
                        );

                        ui.separator();

                        // Entrada numérica para fixar ou desafixar qualquer linha arbitrária (1..=N ou intervalos)
                        ui.label(app.loc.t("freeze_specific_row_label"));
                        let max_opt = if total_linhas > 0 {
                            Some(total_linhas)
                        } else {
                            None
                        };
                        ui.horizontal(|ui| {
                            let resp_edit = ui.add(
                                egui::TextEdit::singleline(&mut app.spreadsheet_grid.row_pin_input)
                                    .desired_width(105.0)
                                    .hint_text(app.loc.t("freeze_row_input_hint")),
                            );
                            let enter_pressed = resp_edit.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));

                            if ui
                                .button(format!("📌 {}", app.loc.t("pin_row_btn")))
                                .clicked()
                                || enter_pressed
                            {
                                let rows = crate::ui::spreadsheet::GridLayout::parse_row_numbers_spec(
                                    &app.spreadsheet_grid.row_pin_input,
                                    max_opt,
                                );
                                if !rows.is_empty() {
                                    for r in rows {
                                        app.spreadsheet_grid.layout.set_row_pinned(r, true);
                                    }
                                    app.spreadsheet_grid
                                        .layout
                                        .salvar_configuracao_painel_fixo();
                                    app.spreadsheet_grid.row_pin_input.clear();
                                }
                            }

                            if ui
                                .button(format!("🔓 {}", app.loc.t("unpin_row_btn")))
                                .clicked()
                            {
                                let rows = crate::ui::spreadsheet::GridLayout::parse_row_numbers_spec(
                                    &app.spreadsheet_grid.row_pin_input,
                                    None,
                                );
                                if !rows.is_empty() {
                                    for r in rows {
                                        app.spreadsheet_grid.layout.set_row_pinned(r, false);
                                    }
                                    app.spreadsheet_grid
                                        .layout
                                        .salvar_configuracao_painel_fixo();
                                    app.spreadsheet_grid.row_pin_input.clear();
                                }
                            }
                        });

                        // Lista rápida de linhas de dados atualmente fixadas
                        let pinned_ativas: Vec<usize> = app
                            .spreadsheet_grid
                            .layout
                            .pinned_rows
                            .iter()
                            .copied()
                            .collect();
                        if !pinned_ativas.is_empty() {
                            ui.separator();
                            let qtd_s = pinned_ativas.len().to_string();
                            ui.label(app.loc.t_args("pinned_rows_active_label", &[&qtd_s]));
                            ui.horizontal_wrapped(|ui| {
                                for r_idx in pinned_ativas.iter().take(12) {
                                    let n_str = (*r_idx + 1).to_string();
                                    let badge = format!("✖ #{}", n_str);
                                    if ui.small_button(badge).clicked() {
                                        app.spreadsheet_grid.layout.set_row_pinned(*r_idx, false);
                                        app.spreadsheet_grid
                                            .layout
                                            .salvar_configuracao_painel_fixo();
                                    }
                                }
                            });
                        }

                        ui.separator();

                        if ui
                            .button(format!("🔓 {}", app.loc.t("unfreeze_all_rows")))
                            .clicked()
                        {
                            app.spreadsheet_grid.layout.unfreeze_all_rows();
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }

                        if ui
                            .button(format!("🔄 {}", app.loc.t("reset_freeze_rows_default")))
                            .clicked()
                        {
                            app.spreadsheet_grid.layout.reset_freeze_rows_default();
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }
                    },
                );

                // 2.3.2 Submenu Fixar Colunas (Congelamento arbitrário de colunas na rolagem horizontal)
                menu_persistente(
                    ui,
                    format!("📌 {}", app.loc.t("freeze_columns_menu")),
                    |ui| {
                        ui.set_min_width(270.0);

                        // Opção para a 1ª coluna (numeração sequencial de linhas #)
                        let mut fixar_num_linhas = app.spreadsheet_grid.layout.freeze_row_headers;
                        if ui
                            .checkbox(
                                &mut fixar_num_linhas,
                                app.loc.t("freeze_row_numbers_column"),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.layout.freeze_row_headers = fixar_num_linhas;
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }

                        // Ação rápida para a coluna atualmente selecionada na planilha (respeitando a ordem visual)
                        let cols_base: Vec<_> = app
                            .gerenciador_colunas
                            .colunas_visiveis()
                            .into_iter()
                            .filter(|c| !app.spreadsheet_grid.manager.is_col_hidden(c.key))
                            .cloned()
                            .collect();
                        let mut cols_fixas_vis = Vec::new();
                        let mut cols_livres_vis = Vec::new();
                        for (orig_idx, c_info) in cols_base.iter().cloned().enumerate() {
                            if app
                                .spreadsheet_grid
                                .layout
                                .is_col_effectively_pinned(orig_idx, c_info.key)
                            {
                                cols_fixas_vis.push((orig_idx, c_info));
                            } else {
                                cols_livres_vis.push((orig_idx, c_info));
                            }
                        }
                        let mut cols_visiveis_ord = cols_fixas_vis;
                        cols_visiveis_ord.extend(cols_livres_vis);

                        if let Some(ac) = app.spreadsheet_grid.selection.active_cell {
                            if let Some((orig_idx, col_sel)) = cols_visiveis_ord.get(ac.col) {
                                let col_nome = app.loc.get_text(col_sel.translation_key);
                                let mut fixa_col = app
                                    .spreadsheet_grid
                                    .layout
                                    .is_col_effectively_pinned(*orig_idx, col_sel.key);
                                if ui
                                    .checkbox(
                                        &mut fixa_col,
                                        app.loc.t_args("freeze_selected_column_fmt", &[&col_nome]),
                                    )
                                    .changed()
                                {
                                    if !fixa_col && *orig_idx < app.spreadsheet_grid.layout.frozen_cols
                                    {
                                        for (idx, c_info) in cols_base
                                            .iter()
                                            .enumerate()
                                            .take(app.spreadsheet_grid.layout.frozen_cols)
                                        {
                                            if idx != *orig_idx {
                                                app.spreadsheet_grid
                                                    .layout
                                                    .set_col_pinned(c_info.key, true);
                                            }
                                        }
                                        app.spreadsheet_grid.layout.frozen_cols = 0;
                                    }
                                    app.spreadsheet_grid
                                        .layout
                                        .set_col_pinned(col_sel.key, fixa_col);
                                    app.spreadsheet_grid
                                        .layout
                                        .salvar_configuracao_painel_fixo();
                                }
                            }
                        }

                        ui.separator();

                        // Submenu multi-coluna listando todas as 58 colunas configuráveis da tabela
                        menu_persistente(ui, format!("📑 {}", app.loc.t("columns")), |ui| {
                            let colunas = app.gerenciador_colunas.colunas().to_vec();
                            renderizar_menu_multi_coluna(ui, &colunas, |ui, col| {
                                let mut fixa = app.spreadsheet_grid.layout.is_col_pinned(col.key);
                                let nome = app.loc.get_text(col.translation_key);
                                if ui.checkbox(&mut fixa, nome).changed() {
                                    app.spreadsheet_grid.layout.set_col_pinned(col.key, fixa);
                                    if !col.visivel && fixa {
                                        app.gerenciador_colunas.set_visibilidade(col.key, true);
                                        app.gerenciador_colunas.salvar_configuracoes();
                                    }
                                    app.spreadsheet_grid
                                        .layout
                                        .salvar_configuracao_painel_fixo();
                                }
                            });
                        });

                        if ui
                            .button(format!("📑 {}", app.loc.t("open_column_manager_dialog")))
                            .clicked()
                        {
                            app.abrir_dialogo_colunas();
                        }

                        ui.separator();

                        if ui
                            .button(format!("📌 {}", app.loc.t("freeze_all_visible_columns")))
                            .clicked()
                        {
                            for c in &cols_base {
                                app.spreadsheet_grid.layout.set_col_pinned(c.key, true);
                            }
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }

                        if ui
                            .button(format!("🔓 {}", app.loc.t("unfreeze_all_columns")))
                            .clicked()
                        {
                            app.spreadsheet_grid.layout.unfreeze_all_cols();
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }

                        if ui
                            .button(format!("🔄 {}", app.loc.t("reset_freeze_columns_default")))
                            .clicked()
                        {
                            app.spreadsheet_grid.layout.reset_freeze_cols_default();
                            app.spreadsheet_grid
                                .layout
                                .salvar_configuracao_painel_fixo();
                        }
                    },
                );

                // 2.3.3 Submenu Comportamento da Seleção
                menu_persistente(
                    ui,
                    format!("🎯 {}", app.loc.t("selection_behavior_menu")),
                    |ui| {
                        ui.set_min_width(280.0);

                        ui.label(
                            egui::RichText::new(app.loc.t("selection_mode_title"))
                                .strong(),
                        );

                        let mut mudou = false;
                        let mut modo_acompanhar =
                            app.spreadsheet_grid.selection_config.acompanhar_selecao;
                        if ui
                            .radio_value(
                                &mut modo_acompanhar,
                                true,
                                app.loc.t("follow_selection_mode"),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.selection_config.acompanhar_selecao = true;
                            mudou = true;
                        }
                        if ui
                            .radio_value(
                                &mut modo_acompanhar,
                                false,
                                app.loc.t("static_selection_mode"),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.selection_config.acompanhar_selecao = false;
                            mudou = true;
                        }

                        ui.separator();

                        ui.label(
                            egui::RichText::new(app.loc.t("selection_targets_title"))
                                .strong(),
                        );

                        let mut celulas = app.spreadsheet_grid.selection_config.acompanhar_celulas;
                        if ui
                            .add_enabled(
                                modo_acompanhar,
                                egui::Checkbox::new(&mut celulas, app.loc.t("follow_cells_selection")),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.selection_config.acompanhar_celulas = celulas;
                            mudou = true;
                        }

                        let mut linhas = app.spreadsheet_grid.selection_config.acompanhar_linhas;
                        if ui
                            .add_enabled(
                                modo_acompanhar,
                                egui::Checkbox::new(&mut linhas, app.loc.t("follow_rows_selection")),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.selection_config.acompanhar_linhas = linhas;
                            mudou = true;
                        }

                        let mut auto_scroll = app.spreadsheet_grid.selection_config.auto_scroll;
                        if ui
                            .add_enabled(
                                modo_acompanhar,
                                egui::Checkbox::new(&mut auto_scroll, app.loc.t("auto_scroll_to_selection")),
                            )
                            .changed()
                        {
                            app.spreadsheet_grid.selection_config.auto_scroll = auto_scroll;
                            mudou = true;
                        }

                        ui.separator();

                        ui.label(
                            egui::RichText::new(app.loc.t("cell_selection_color_title"))
                                .strong(),
                        );

                        // Amostra visual da cor atual + código Hexadecimal
                        ui.horizontal(|ui| {
                            let cor_atual = app.spreadsheet_grid.selection_config.cor_selecao;
                            let cor_color32 = egui::Color32::from_rgb(cor_atual[0], cor_atual[1], cor_atual[2]);
                            let (rect, resp) = ui.allocate_exact_size(egui::Vec2::new(20.0, 16.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2_u8, cor_color32);
                            let border_stroke = if resp.hovered() {
                                egui::Stroke::new(1.8, if ui.visuals().dark_mode {
                                    egui::Color32::from_rgb(100, 190, 255)
                                } else {
                                    egui::Color32::from_rgb(20, 110, 230)
                                })
                            } else {
                                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(100))
                            };
                            ui.painter().rect_stroke(
                                rect,
                                2_u8,
                                border_stroke,
                                egui::StrokeKind::Inside,
                            );

                            let hex_code = format!("#{:02X}{:02X}{:02X}", cor_atual[0], cor_atual[1], cor_atual[2]);
                            ui.monospace(hex_code);
                        });

                        if ui
                            .button(format!("🎨 {}", app.loc.t("customize_selection_color_btn")))
                            .clicked()
                        {
                            app.cor_selecao_temp = app.spreadsheet_grid.selection_config.cor_selecao;
                            app.dialogo_cor_selecao_aberto = true;
                        }

                        if ui
                            .button(format!("🔄 {}", app.loc.t("reset_selection_color_btn")))
                            .clicked()
                        {
                            app.prompt_confirmar_restaurar_cor = true;
                        }

                        ui.separator();

                        if ui
                            .button(format!("🔄 {}", app.loc.t("reset_selection_behavior")))
                            .clicked()
                        {
                            app.spreadsheet_grid.selection_config =
                                crate::ui::spreadsheet::SelectionBehaviorConfig::default();
                            mudou = true;
                        }

                        if mudou {
                            app.spreadsheet_grid.salvar_configuracao_selecao();
                        }
                    },
                );

                // 2.4 Submenu Configurar Cores (gmui_08_criar_submenu_cores)
                menu_persistente(
                    ui,
                    format!("🎨 {}", app.loc.t("configure_colors")),
                    |ui| {
                        ui.set_min_width(230.0);
                        menu_persistente(
                            ui,
                            format!("🏷️ {}", app.loc.t("operation_colors")),
                            |ui| {
                                ui.set_min_width(230.0);
                                let ops = [
                                    OperationType::Renomeado,
                                    OperationType::Adicionado,
                                    OperationType::Excluido,
                                    OperationType::Modificado,
                                    OperationType::Movido,
                                    OperationType::Escaneado,
                                ];
                                for op in ops {
                                    let (bg, _fg) =
                                        app.gerenciador_tabela.obter_cores_operacao(op);
                                    let label = app.loc.traduzir_tipo_operacao(op.as_str());
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            Vec2::new(14.0, 14.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().rect_filled(rect, 2_u8, bg);
                                        if ui.button(label).clicked() {
                                            app.abrir_dialogo_cores();
                                        }
                                    });
                                }
                            },
                        );

                        ui.separator();

                        if ui.button(app.loc.t("reset_colors")).clicked() {
                            app.gerenciador_tabela.resetar_cores_padrao();
                        }
                    },
                );

                // 2.5 Submenu Opções de Exportação (gmui_16_criar_submenu_exportacao)
                menu_persistente(
                    ui,
                    format!("💾 {}", app.loc.t("export_options")),
                    |ui| {
                        ui.set_min_width(240.0);
                        ui.checkbox(
                            &mut app.export_colunas_ativas,
                            app.loc.t("export_active_columns"),
                        );
                        ui.checkbox(
                            &mut app.export_filtros_ativos,
                            app.loc.t("export_active_filters"),
                        );
                        ui.checkbox(&mut app.export_selecao, app.loc.t("export_selected_data"));

                        ui.separator();

                        if ui.button(app.loc.t("reset_export_options")).clicked() {
                            app.resetar_opcoes_exportacao();
                        }
                    },
                );

                // 2.6 Submenu Estatísticas (gmui_06: criar_submenu_estatisticas)
                menu_persistente(ui, format!("📊 {}", app.loc.t("statistics")), |ui| {
                    ui.set_min_width(220.0);
                    menu_persistente(
                        ui,
                        format!("📈 {}", app.loc.t("plot_graphs")),
                        |ui| {
                            let graficos = TipoGrafico::all();
                            renderizar_menu_multi_coluna(ui, graficos, |ui, tipo| {
                                let titulo = tipo.titulo(&app.loc);
                                if ui.button(titulo).clicked() {
                                    app.abrir_dialogo_estatisticas_com_grafico(*tipo);
                                }
                            });
                        },
                    );
                });

                // 2.7 Submenu Segurança (secui_07_configurar_menu_seguranca)
                menu_persistente(
                    ui,
                    format!("🛡️ {}", app.loc.t("security_menu")),
                    |ui| {
                        ui.set_min_width(250.0);
                        if ui
                            .button(format!(
                                "📁 {}",
                                app.loc.t("security_manage_directories")
                            ))
                            .clicked()
                        {
                            app.abrir_dialogo_multi_dir();
                        }
                        if ui
                            .button(format!("🛡️ {}", app.loc.t("security_alert_center")))
                            .clicked()
                        {
                            app.abrir_dialogo_alertas();
                        }
                        if ui
                            .button(format!("⏱️ {}", app.loc.t("security_session_center")))
                            .clicked()
                        {
                            app.abrir_dialogo_sessoes();
                        }
                        if ui
                            .button(format!("🔒 {}", app.loc.t("security_recovery_center")))
                            .clicked()
                        {
                            app.abrir_dialogo_recuperacao();
                        }
                        if ui
                            .button(format!("📦 {}", app.loc.t("security_evidence_center")))
                            .clicked()
                        {
                            app.abrir_dialogo_evidencias();
                        }

                        ui.separator();

                        menu_persistente(
                            ui,
                            format!("🔔 {}", app.loc.t("security_notifications_menu")),
                            |ui| {
                                ui.set_min_width(260.0);
                                let check_alerts =
                                    if app.dialog_notify_config.alerts_popup.config.enabled {
                                        "✔ "
                                    } else {
                                        ""
                                    };
                                if ui
                                    .button(format!(
                                        "{check_alerts}⚠️ {}",
                                        app.loc.t("security_alert_popup_notifications")
                                    ))
                                    .clicked()
                                {
                                    app.abrir_dialogo_popup_alertas();
                                }

                                let check_sessions =
                                    if app.dialog_notify_config.sessions_popup.config.enabled {
                                        "✔ "
                                    } else {
                                        ""
                                    };
                                if ui
                                    .button(format!(
                                        "{check_sessions}⏱️ {}",
                                        app.loc.t("security_session_popup_notifications")
                                    ))
                                    .clicked()
                                {
                                    app.abrir_dialogo_popup_sessoes();
                                }

                                let check_quar =
                                    if app.dialog_notify_config.quarantine_popup.config.enabled {
                                        "✔ "
                                    } else {
                                        ""
                                    };
                                if ui
                                    .button(format!(
                                        "{check_quar}🔒 {}",
                                        app.loc.t("security_quarantine_popup_notifications")
                                    ))
                                    .clicked()
                                {
                                    app.abrir_dialogo_popup_quarentena();
                                }

                                let check_sum =
                                    if app.dialog_notify_config.summary_popup.config.enabled {
                                        "✔ "
                                    } else {
                                        ""
                                    };
                                if ui
                                    .button(format!(
                                        "{check_sum}📢 {}",
                                        app.loc.t("security_summary_notifications")
                                    ))
                                    .clicked()
                                {
                                    app.abrir_dialogo_notificacoes_resumo();
                                }
                            },
                        );

                        ui.separator();

                        if ui
                            .button(format!("🗑️ {}", app.loc.t("security_clear_database")))
                            .clicked()
                        {
                            app.limpar_banco_seguranca();
                        }
                    },
                );
            });

            // ==========================================
            // 3. MENU OPÇÕES (gmui_07_configurar_menu_opcoes)
            // ==========================================
            let resp_opcoes = menu_persistente(ui, app.loc.t("options_menu"), |ui| {
                ui.set_min_width(240.0);

                // 3.1 Submenu Idiomas (gmui_18_criar_submenu_idiomas)
                menu_persistente(ui, format!("🌐 {}", app.loc.t("language")), |ui| {
                    ui.set_min_width(200.0);
                    for (code, _default_label) in IDIOMAS_SUPORTADOS {
                        let flag = match *code {
                            "pt_BR" => "🇧🇷",
                            "en_US" => "🇺🇸",
                            "es_ES" => "🇪🇸",
                            "fr_FR" => "🇫🇷",
                            "it_IT" => "🇮🇹",
                            "de_DE" => "🇩🇪",
                            _ => "🌐",
                        };
                        let label = format!("{} {}", flag, app.loc.traduzir_nome_idioma(code));
                        let ativo = app.loc.idioma_atual() == *code;
                        if ui.selectable_label(ativo, label).clicked() {
                            app.loc.set_idioma(code);
                            app.loc.salvar_preferencia_idioma(code);
                            app.gerenciador_colunas
                                .autofit_todas_colunas_ao_cabecalho(&app.loc);
                            ui.ctx().request_repaint();
                        }
                    }
                });

                ui.separator();

                // 3.2 Manual de Utilização (Ctrl+M)
                if ui
                    .add(
                        Button::new(format!("📖 {}", app.loc.t("manual")))
                            .shortcut_text("Ctrl+M"),
                    )
                    .clicked()
                {
                    app.abrir_dialogo_manual();
                }

                ui.separator();

                // 3.3 Sobre (Ctrl+I)
                if ui
                    .add(Button::new(format!("ℹ️ {}", app.loc.t("about"))).shortcut_text("Ctrl+I"))
                    .clicked()
                {
                    app.abrir_dialogo_sobre();
                }

                // 3.4 Ajuda (F1)
                if ui
                    .add(Button::new(format!("❓ {}", app.loc.t("help"))).shortcut_text("F1"))
                    .clicked()
                {
                    app.abrir_dialogo_ajuda();
                }
            });

            // Quando qualquer menu da barra superior estiver aberto, registra a faixa combinada
            // dos 3 botões ("Arquivo", "Configurações", "Opções") para permitir alternância fluida
            // entre menus adjacentes sem fechar nos poucos pixels entre os botões, fechando caso
            // o ponteiro vá para a área vazia à direita da barra ou saia dos menus.
            let any_bar_open = egui::Popup::is_id_open(
                ui.ctx(),
                egui::Popup::default_response_id(&resp_arquivo),
            ) || egui::Popup::is_id_open(
                ui.ctx(),
                egui::Popup::default_response_id(&resp_config),
            ) || egui::Popup::is_id_open(
                ui.ctx(),
                egui::Popup::default_response_id(&resp_opcoes),
            );
            if any_bar_open {
                let bar_buttons_span = resp_arquivo
                    .rect
                    .union(resp_config.rect)
                    .union(resp_opcoes.rect);
                registrar_retangulo_menu_ativo(ui.ctx(), bar_buttons_span, false);
            }
        });
}

/// Helper para dividir listas extensas de menu em colunas adaptativas com rolagem vertical suave,
/// garantindo que o submenu nunca ultrapasse as dimensões da tela nem sobreponha o menu pai.
pub fn renderizar_menu_multi_coluna<T, F>(ui: &mut Ui, itens: &[T], mut render_item: F)
where
    F: FnMut(&mut Ui, &T),
{
    if itens.is_empty() {
        return;
    }

    let screen_rect = crate::ui::dialogs::modeless::screen_rect(ui.ctx());
    let col_w = 240.0;

    // Espaço horizontal disponível entre a abertura do submenu e a borda direita da janela
    let available_w = (screen_rect.right() - ui.cursor().left() - 30.0).max(col_w);

    // Se couberem 2 colunas com folga sem estourar a tela à direita, usa 2 colunas; senão 1 coluna
    let num_cols = if available_w >= (col_w * 2.0 + 20.0) && itens.len() > 14 {
        2
    } else {
        1
    };

    let max_h = (screen_rect.height() - 140.0).clamp(240.0, 460.0);

    ScrollArea::vertical()
        .id_salt(ui.scope_id().with("menu_multi_col_scroll"))
        .max_height(max_h)
        .auto_shrink([true, true])
        .show(ui, |ui| {
            if num_cols == 1 {
                ui.vertical(|ui| {
                    ui.set_min_width(col_w);
                    for item in itens {
                        render_item(ui, item);
                    }
                });
            } else {
                let itens_por_col = (itens.len() + 1) / 2;
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(col_w);
                        for item in &itens[..itens_por_col] {
                            render_item(ui, item);
                        }
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.set_width(col_w);
                        for item in &itens[itens_por_col..] {
                            render_item(ui, item);
                        }
                    });
                });
            }
        });
}
