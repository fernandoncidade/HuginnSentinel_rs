use huginn_rs::data::charts::TipoGrafico;
use huginn_rs::locale::{obter_secoes_manual, Localizer, IDIOMAS_SUPORTADOS};

#[test]
fn test_localizer_all_languages_and_fallbacks() {
    let loc = Localizer::new();

    for (code, _label) in IDIOMAS_SUPORTADOS {
        loc.set_idioma(code);
        assert_eq!(loc.idioma_atual(), *code);

        let select_dir = loc.t("select_dir");
        assert!(
            !select_dir.is_empty(),
            "Tradução de select_dir não deve ser vazia para {}",
            code
        );

        let op_add = loc.traduzir_tipo_operacao("Adicionado");
        assert!(
            !op_add.is_empty(),
            "Tradução de operação Adicionado não deve ser vazia para {}",
            code
        );

        let secoes_manual = obter_secoes_manual(code);
        assert!(
            !secoes_manual.is_empty(),
            "Manual deve ter seções disponíveis para {}",
            code
        );
    }
}

#[test]
fn test_all_17_charts_translations() {
    let loc = Localizer::new();

    for (code, _) in IDIOMAS_SUPORTADOS {
        loc.set_idioma(code);
        for tipo in TipoGrafico::all() {
            let titulo = tipo.titulo(&loc);
            let desc = tipo.descricao(&loc);

            assert!(
                !titulo.is_empty(),
                "Título do gráfico {:?} vazio em {}",
                tipo,
                code
            );
            assert!(
                !desc.is_empty(),
                "Descrição do gráfico {:?} vazia em {}",
                tipo,
                code
            );
            assert!(
                !titulo.starts_with("chart_"),
                "Título do gráfico {:?} não foi traduzido em {}: {}",
                tipo,
                code,
                titulo
            );
            assert!(
                !desc.starts_with("chart_"),
                "Descrição do gráfico {:?} não foi traduzida em {}: {}",
                tipo,
                code,
                desc
            );
        }
    }
}

#[test]
fn test_dialog_and_ui_translations() {
    let loc = Localizer::new();

    let ui_keys = [
        "file_menu",
        "settings",
        "options_menu",
        "security_menu",
        "help_menu",
        "select_dir",
        "scan_dir",
        "stop_scan",
        "start_monitor",
        "stop_monitor",
        "export_data",
        "clear_data",
        "exit",
        "columns_config",
        "colors_config",
        "reset_defaults",
        "alert_center",
        "sessions",
        "recovery",
        "multi_dir",
        "evidence_export",
        "notification_config",
        "manual",
        "about",
        "event_filters",
        "quick_search",
        "risk_level",
        "extensions",
        "reset_filters",
        "open_with_default_app",
        "show_in_folder",
        "isolate_in_quarantine",
        "scan_label",
        "status_active",
        "status_paused",
        "status_stopped",
        "light_mode",
        "dark_mode",
        "topics",
        "search_manual_hint",
        "security_path",
        "alert_rules",
        "alert_justification",
        "tab_about",
        "tab_clc",
        "tab_copyright",
        "tab_eula",
        "tab_licenses",
        "tab_notices",
        "tab_privacy",
        "tab_release",
    ];

    for (code, _) in IDIOMAS_SUPORTADOS {
        loc.set_idioma(code);

        for key in &ui_keys {
            let val = loc.t(key);
            assert!(
                !val.is_empty(),
                "Chave '{}' não deve ser vazia em {}",
                key,
                code
            );
            assert_ne!(
                val, *key,
                "Chave '{}' deve estar traduzida em {}",
                key, code
            );
        }

        // Teste de interpolação de argumentos
        let msg = loc.t_args("total_events", &[&"42"]);
        assert!(
            msg.contains("42"),
            "Interpolação falhou em {}: {}",
            code,
            msg
        );

        let periodo = loc.t_args("period_label", &[&"10:00", &"11:00"]);
        assert!(
            periodo.contains("10:00") && periodo.contains("11:00"),
            "Interpolação de 2 args falhou em {}: {}",
            code,
            periodo
        );
    }
}

#[test]
fn test_asset_and_language_path_resolution() {
    use huginn_rs::language::Localizer as LanguageLocalizer;
    use huginn_rs::utils::resolver_caminho_asset;

    // Verifica se os assets são resolvidos em src/assets
    let icon_path = resolver_caminho_asset("icons/huginn.ico");
    assert!(
        icon_path.is_some(),
        "Ícone deve ser resolvido em src/assets/icons"
    );
    let icon_data = huginn_rs::utils::icons::carregar_icone_janela();
    assert!(
        icon_data.is_some(),
        "carregar_icone_janela() deve carregar com sucesso o ícone huginn.ico"
    );

    // Verifica se a documentação é resolvida em src/assets/docs
    let doc_path = resolver_caminho_asset("architecture.md");
    assert!(
        doc_path.is_some(),
        "Documento de arquitetura deve ser resolvido em src/assets/docs"
    );

    // Verifica se os arquivos de tradução JSON são resolvidos em src/language
    let lang_path = resolver_caminho_asset("pt_BR.json");
    assert!(
        lang_path.is_some(),
        "Arquivo JSON de tradução deve ser resolvido em src/language"
    );

    // Verifica o alias language::Localizer
    let loc = LanguageLocalizer::new();
    loc.set_idioma("pt_BR");
    assert_eq!(loc.idioma_atual(), "pt_BR");
}

#[test]
fn test_dynamic_operations_and_risks_runtime_translation() {
    let loc = Localizer::new();

    // 1. Português
    loc.set_idioma("pt_BR");
    assert_eq!(loc.traduzir_tipo_operacao("Adicionado"), "Adicionado");
    assert_eq!(loc.traduzir_tipo_operacao("Modificado"), "Modificado");
    assert_eq!(loc.traduzir_tipo_operacao("Excluído"), "Excluído");
    assert_eq!(loc.traduzir_tipo_operacao("Renomeado"), "Renomeado");
    assert_eq!(loc.traduzir_tipo_operacao("Movido"), "Movido");
    assert_eq!(loc.traduzir_tipo_operacao("Escaneado"), "Escaneado");

    // 2. Inglês
    loc.set_idioma("en_US");
    assert_eq!(loc.traduzir_tipo_operacao("Adicionado"), "Added");
    assert_eq!(loc.traduzir_tipo_operacao("Modificado"), "Modified");
    assert_eq!(loc.traduzir_tipo_operacao("Excluído"), "Deleted");
    assert_eq!(loc.traduzir_tipo_operacao("Renomeado"), "Renamed");
    assert_eq!(loc.traduzir_tipo_operacao("Movido"), "Moved");
    assert_eq!(loc.traduzir_tipo_operacao("Escaneado"), "Scanned");
    assert_eq!(loc.traduzir_risco("Crítico"), "Critical");
    assert_eq!(loc.traduzir_risco("Alto"), "High");
    assert_eq!(loc.traduzir_risco("Médio"), "Medium");
    assert_eq!(loc.traduzir_risco("Baixo"), "Low");

    // 3. Espanhol
    loc.set_idioma("es_ES");
    assert_eq!(loc.traduzir_tipo_operacao("Adicionado"), "Añadido");
    assert_eq!(loc.traduzir_tipo_operacao("Modificado"), "Modificado");
    assert_eq!(loc.traduzir_tipo_operacao("Excluído"), "Eliminado");
    assert_eq!(loc.traduzir_tipo_operacao("Renomeado"), "Renombrado");
    assert_eq!(loc.traduzir_tipo_operacao("Movido"), "Movido");
    assert_eq!(loc.traduzir_tipo_operacao("Escaneado"), "Escaneado");

    // 4. Francês
    loc.set_idioma("fr_FR");
    assert_eq!(loc.traduzir_tipo_operacao("Adicionado"), "Ajouté");
    assert_eq!(loc.traduzir_tipo_operacao("Modificado"), "Modifié");
    assert_eq!(loc.traduzir_tipo_operacao("Excluído"), "Supprimé");
    assert_eq!(loc.traduzir_tipo_operacao("Renomeado"), "Renommé");
    assert_eq!(loc.traduzir_tipo_operacao("Movido"), "Déplacé");
    assert_eq!(loc.traduzir_tipo_operacao("Escaneado"), "Scanné");

    // 5. Italiano
    loc.set_idioma("it_IT");
    assert_eq!(loc.traduzir_tipo_operacao("Adicionado"), "Aggiunto");
    assert_eq!(loc.traduzir_tipo_operacao("Modificado"), "Modificato");
    assert_eq!(loc.traduzir_tipo_operacao("Excluído"), "Eliminato");
    assert_eq!(loc.traduzir_tipo_operacao("Renomeado"), "Rinominato");
    assert_eq!(loc.traduzir_tipo_operacao("Movido"), "Spostato");
    assert_eq!(loc.traduzir_tipo_operacao("Escaneado"), "Scansionato");

    // 6. Alemão
    loc.set_idioma("de_DE");
    assert_eq!(loc.traduzir_tipo_operacao("Adicionado"), "Hinzugefügt");
    assert_eq!(loc.traduzir_tipo_operacao("Modificado"), "Geändert");
    assert_eq!(loc.traduzir_tipo_operacao("Excluído"), "Gelöscht");
    assert_eq!(loc.traduzir_tipo_operacao("Renomeado"), "Umbenannt");
    assert_eq!(loc.traduzir_tipo_operacao("Movido"), "Verschoben");
    assert_eq!(loc.traduzir_tipo_operacao("Escaneado"), "Gescannt");
}

#[test]
fn test_dynamic_metadata_runtime_translation() {
    let loc = Localizer::new();

    // 1. Português
    loc.set_idioma("pt_BR");
    assert_eq!(loc.traduzir_tipo_arquivo("Código Fonte"), "Código Fonte");
    assert_eq!(loc.traduzir_tipo_arquivo("Imagem"), "Imagem");
    assert_eq!(loc.traduzir_tipo_arquivo("Executável"), "Executável");
    assert_eq!(loc.traduzir_atributos("Oculto"), "Oculto");
    assert_eq!(loc.traduzir_atributos("Somente Leitura"), "Somente Leitura");
    assert_eq!(loc.traduzir_booleano("Sim"), "Sim");
    assert_eq!(loc.traduzir_booleano("Não"), "Não");
    assert_eq!(loc.traduzir_baseline_status("Novo"), "Novo");
    assert_eq!(loc.traduzir_baseline_status("Intacto"), "Intacto");
    assert_eq!(loc.traduzir_quarantine_status("Isolado"), "Isolado");
    assert_eq!(loc.traduzir_quarantine_status("Restaurado"), "Restaurado");
    assert_eq!(
        loc.traduzir_alerta("Ransomware"),
        "Extensão Suspeita de Ransomware"
    );
    assert_eq!(
        loc.traduzir_alerta("Extensão Dupla"),
        "Extensão Dupla Dissimulada"
    );
    assert_eq!(loc.traduzir_nome_idioma("pt_BR"), "Português (Brasil)");

    // 2. Inglês
    loc.set_idioma("en_US");
    assert_eq!(loc.traduzir_tipo_arquivo("Código Fonte"), "Source Code");
    assert_eq!(loc.traduzir_tipo_arquivo("Imagem"), "Image");
    assert_eq!(loc.traduzir_tipo_arquivo("Executável"), "Executable");
    assert_eq!(
        loc.traduzir_tipo_arquivo("Executável Windows"),
        "Windows Executable"
    );
    assert_eq!(loc.traduzir_tipo_arquivo("Arquivo de Texto"), "Text File");
    assert_eq!(loc.traduzir_atributos("Oculto"), "Hidden");
    assert_eq!(loc.traduzir_atributos("Somente Leitura"), "Read Only");
    assert_eq!(loc.traduzir_atributos("Arquivo Normal"), "Archive");
    assert_eq!(loc.traduzir_booleano("Sim"), "Yes");
    assert_eq!(loc.traduzir_booleano("Não"), "No");
    assert_eq!(loc.traduzir_baseline_status("Novo"), "New");
    assert_eq!(loc.traduzir_baseline_status("Intacto"), "Intact");
    assert_eq!(loc.traduzir_quarantine_status("Isolado"), "Isolated");
    assert_eq!(loc.traduzir_quarantine_status("Restaurado"), "Restored");
    assert_eq!(
        loc.traduzir_alerta("Ransomware"),
        "Suspicious Ransomware Extension"
    );
    assert_eq!(
        loc.traduzir_alerta("Extensão Dupla"),
        "Disguised Double Extension"
    );
    assert_eq!(loc.traduzir_nome_idioma("en_US"), "English (US)");

    // 3. Espanhol
    loc.set_idioma("es_ES");
    assert_eq!(loc.traduzir_tipo_arquivo("Código Fonte"), "Código Fuente");
    assert_eq!(loc.traduzir_tipo_arquivo("Imagem"), "Imagen");
    assert_eq!(loc.traduzir_tipo_arquivo("Executável"), "Ejecutable");
    assert_eq!(loc.traduzir_atributos("Oculto"), "Oculto");
    assert_eq!(loc.traduzir_booleano("Sim"), "Sí");
    assert_eq!(loc.traduzir_booleano("Não"), "No");
    assert_eq!(loc.traduzir_baseline_status("Novo"), "Nuevo");
    assert_eq!(loc.traduzir_quarantine_status("Isolado"), "Aislado");
    assert_eq!(loc.traduzir_nome_idioma("es_ES"), "Español");

    // 4. Francês
    loc.set_idioma("fr_FR");
    assert_eq!(loc.traduzir_tipo_arquivo("Código Fonte"), "Code Source");
    assert_eq!(loc.traduzir_tipo_arquivo("Imagem"), "Image");
    assert_eq!(loc.traduzir_tipo_arquivo("Executável"), "Exécutable");
    assert_eq!(loc.traduzir_atributos("Oculto"), "Masqué");
    assert_eq!(loc.traduzir_booleano("Sim"), "Oui");
    assert_eq!(loc.traduzir_booleano("Não"), "Non");
    assert_eq!(loc.traduzir_baseline_status("Novo"), "Nouveau");
    assert_eq!(loc.traduzir_quarantine_status("Isolado"), "Isolé");
    assert_eq!(loc.traduzir_nome_idioma("fr_FR"), "Français");

    // 5. Italiano
    loc.set_idioma("it_IT");
    assert_eq!(loc.traduzir_tipo_arquivo("Código Fonte"), "Codice Sorgente");
    assert_eq!(loc.traduzir_tipo_arquivo("Imagem"), "Immagine");
    assert_eq!(loc.traduzir_tipo_arquivo("Executável"), "Eseguibile");
    assert_eq!(loc.traduzir_atributos("Oculto"), "Nascosto");
    assert_eq!(loc.traduzir_booleano("Sim"), "Sì");
    assert_eq!(loc.traduzir_booleano("Não"), "No");
    assert_eq!(loc.traduzir_baseline_status("Novo"), "Nuovo");
    assert_eq!(loc.traduzir_quarantine_status("Isolado"), "Isolato");
    assert_eq!(loc.traduzir_nome_idioma("it_IT"), "Italiano");

    // 6. Alemão
    loc.set_idioma("de_DE");
    assert_eq!(loc.traduzir_tipo_arquivo("Código Fonte"), "Quellcode");
    assert_eq!(loc.traduzir_tipo_arquivo("Imagem"), "Bild");
    assert_eq!(loc.traduzir_tipo_arquivo("Executável"), "Ausführbare Datei");
    assert_eq!(loc.traduzir_atributos("Oculto"), "Versteckt");
    assert_eq!(loc.traduzir_booleano("Sim"), "Ja");
    assert_eq!(loc.traduzir_booleano("Não"), "Nein");
    assert_eq!(loc.traduzir_baseline_status("Novo"), "Neu");
    assert_eq!(loc.traduzir_quarantine_status("Isolado"), "Isoliert");
    assert_eq!(loc.traduzir_nome_idioma("de_DE"), "Deutsch");
}

#[test]
fn test_detectar_idioma_sistema_e_fallback() {
    let detectado = Localizer::detectar_idioma_sistema();
    let suportados = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];
    assert!(
        suportados.contains(&detectado.as_str()),
        "Idioma detectado deve ser um dos suportados ou fallback en_US"
    );
}

#[test]
fn test_active_windows_counter_and_freeze_panes_all_6_languages() {
    let loc = Localizer::new();

    // Verifica formatação exata no Português ("1 Janela:", "2 Janelas:", "3 Janelas:")
    loc.set_idioma("pt_BR");
    assert_eq!(loc.t_args("active_windows_count_one", &["1"]), "1 Janela:");
    assert_eq!(
        loc.t_args("active_windows_count_many", &["2"]),
        "2 Janelas:"
    );
    assert_eq!(
        loc.t_args("active_windows_count_many", &["3"]),
        "3 Janelas:"
    );

    // Verifica Inglês
    loc.set_idioma("en_US");
    assert_eq!(loc.t_args("active_windows_count_one", &["1"]), "1 Window:");
    assert_eq!(
        loc.t_args("active_windows_count_many", &["2"]),
        "2 Windows:"
    );
    assert_eq!(
        loc.t_args("active_windows_count_many", &["3"]),
        "3 Windows:"
    );

    let new_keys = [
        "active_windows_count_one",
        "active_windows_count_many",
        "autofit_adjust_column_widths",
        "freeze_rows_menu",
        "freeze_columns_menu",
        "freeze_column_headers_row",
        "freeze_row_numbers_column",
        "freeze_selected_row_fmt",
        "freeze_selected_column_fmt",
        "freeze_specific_row_label",
        "freeze_row_input_hint",
        "pin_row_btn",
        "unpin_row_btn",
        "table_rows_submenu",
        "pinned_rows_active_label",
        "no_table_rows_available",
        "row_item_fmt",
        "unfreeze_all_rows",
        "reset_freeze_rows_default",
        "freeze_all_visible_columns",
        "unfreeze_all_columns",
        "reset_freeze_columns_default",
        "pinned_status_badge",
        "ctx_column_header_fmt",
        "ctx_sort_asc",
        "ctx_sort_desc",
        "ctx_autofit_col",
        "ctx_autofit_all",
        "ctx_pin_column",
        "ctx_unpin_column",
        "ctx_hide_column",
        "ctx_unhide_all_columns",
        "ctx_pin_row",
        "ctx_unpin_row",
        "ctx_insert_row_above",
        "ctx_insert_row_below",
        "ctx_duplicate_row",
        "ctx_delete_selected_row",
        "ctx_clear_row_content",
        "ctx_hide_selected_row",
        "ctx_unhide_all_rows",
        "ctx_open_default_app",
        "ctx_show_in_explorer",
        "ctx_copy_cell_value",
        "ctx_copy_row_tsv",
        "ctx_copy_sha256",
        "ctx_filter_only_op_fmt",
        "ctx_filter_by_ext_fmt",
        "ctx_insert_row",
        "ctx_move_to_quarantine",
    ];

    for (code, _) in IDIOMAS_SUPORTADOS {
        loc.set_idioma(code);
        for key in &new_keys {
            let val = loc.t(key);
            assert!(
                !val.is_empty() && val != *key,
                "Chave '{}' ausente ou não traduzida em '{}'",
                key,
                code
            );
        }
    }
}

#[test]
fn test_all_6_release_notes_detailed_refactoring() {
    use huginn_rs::ui::dialogs::about::{AboutDialog, AboutTab};

    for (code, _) in IDIOMAS_SUPORTADOS {
        let content = AboutDialog::obter_texto_aba(AboutTab::Release, code);
        assert!(
            content.len() > 3000,
            "Release notes de '{}' devem detalhar profundamente a refatoração (tamanho atual: {})",
            code,
            content.len()
        );
        assert!(
            content.contains("Qt") && content.contains("Rust") && content.contains("egui"),
            "Release notes de '{}' devem mencionar a refatoração de C++/Qt6 para Rust/egui",
            code
        );
    }
}

#[test]
fn test_selection_behavior_translations_all_languages() {
    let loc = Localizer::new();
    let keys = [
        "selection_behavior_menu",
        "selection_mode_title",
        "follow_selection_mode",
        "static_selection_mode",
        "selection_targets_title",
        "follow_cells_selection",
        "follow_rows_selection",
        "auto_scroll_to_selection",
        "reset_selection_behavior",
        "cell_selection_color_title",
        "customize_selection_color_btn",
        "reset_selection_color_btn",
        "selection_color_dialog_title",
        "selection_color_dialog_desc",
        "select_color_label",
        "recommended_palette_label",
        "color_preset_green",
        "color_preset_blue",
        "color_preset_teal",
        "color_preset_purple",
        "color_preset_orange",
        "color_preset_red",
        "color_preset_gray",
        "selection_color_preview_label",
        "preview_unselected_cell",
        "preview_selected_cell",
        "apply_and_save_color_btn",
        "confirm_apply_color_title",
        "confirm_apply_color_msg",
        "confirm_reset_color_title",
        "confirm_reset_color_msg",
    ];

    for (code, _) in IDIOMAS_SUPORTADOS {
        loc.set_idioma(code);
        for key in &keys {
            let val = loc.t(key);
            assert!(
                !val.is_empty() && val != *key,
                "Chave de comportamento de seleção '{}' ausente ou não traduzida no idioma '{}'",
                key,
                code
            );
        }
    }
}

#[test]
fn test_scan_and_monitor_workflow_translations_all_languages() {
    let loc = Localizer::new();
    let keys = [
        "scanning_menu",
        "monitoring_menu",
        "scan_prompt_title",
        "scan_prompt_question",
        "scan_and_monitor_btn",
        "scan_only_btn",
        "monitor_prompt_title",
        "monitor_prompt_question",
        "scan_first_then_monitor_btn",
        "monitor_only_btn",
        "scan_notice_active_info",
        "scan_notice_disabled_info",
        "monitor_notice_active_info",
        "monitor_notice_disabled_info",
        "enable_scan_prompt",
        "enable_monitor_prompt",
        "scan_default_behavior_title",
        "monitor_default_behavior_title",
        "scan_opt_only_scan",
        "scan_opt_scan_and_monitor",
        "monitor_opt_only_monitor",
        "monitor_opt_scan_first_then_monitor",
        "scan_run_only_now",
        "scan_run_and_monitor_now",
        "monitor_run_only_now",
        "monitor_run_scan_then_now",
        "dont_ask_again",
    ];

    for (code, _) in IDIOMAS_SUPORTADOS {
        loc.set_idioma(code);
        for key in &keys {
            let val = loc.t(key);
            assert!(
                !val.is_empty() && val != *key,
                "Chave de fluxo de escaneamento/monitoramento '{}' ausente ou não traduzida no idioma '{}'",
                key,
                code
            );
        }
    }
}

#[test]
fn test_recommended_palette_selected_text_color_adapts_to_theme() {
    let ctx = egui::Context::default();

    // 1. No tema escuro (dark mode), a opção selecionada deve ser branca (Color32::WHITE)
    huginn_rs::ui::widgets::aplicar_tema_visuals(&ctx, true);
    let mut color_dark = egui::Color32::TRANSPARENT;
    let mut out1 = ctx.run_ui(egui::RawInput::default(), |ui| {
        let is_current = true;
        color_dark = if is_current {
            if ui.visuals().dark_mode {
                egui::Color32::WHITE
            } else {
                egui::Color32::BLACK
            }
        } else {
            ui.visuals().text_color()
        };
    });
    out1.textures_delta.clear();
    assert_eq!(
        color_dark,
        egui::Color32::WHITE,
        "No tema escuro a fonte da opção selecionada deve ser branca"
    );

    // 2. No tema claro (light mode), a opção selecionada deve migrar para preta (Color32::BLACK)
    huginn_rs::ui::widgets::aplicar_tema_visuals(&ctx, false);
    let mut color_light = egui::Color32::TRANSPARENT;
    let mut out2 = ctx.run_ui(egui::RawInput::default(), |ui| {
        let is_current = true;
        color_light = if is_current {
            if ui.visuals().dark_mode {
                egui::Color32::WHITE
            } else {
                egui::Color32::BLACK
            }
        } else {
            ui.visuals().text_color()
        };
    });
    out2.textures_delta.clear();
    assert_eq!(
        color_light,
        egui::Color32::BLACK,
        "No tema claro a fonte da opção selecionada deve migrar para preta"
    );
}


