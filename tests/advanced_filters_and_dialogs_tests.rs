use huginn_rs::core::event::EventModel;
use huginn_rs::locale::{obter_secoes_manual, Localizer};
use huginn_rs::tools::filters::FiltroEstado;
use huginn_rs::ui::dialogs::about::{AboutDialog, AboutTab};
use huginn_rs::ui::dialogs::AdvancedFiltersDialog;
use huginn_rs::ui::dialogs::ManualDialog;

#[test]
fn test_advanced_filters_date_ignore_flags() {
    let mut filtro = FiltroEstado::default();
    filtro.data_inicio = "2026-05-01 00:00:00".to_string();
    filtro.data_fim = "2026-05-31 23:59:59".to_string();

    // Evento fora do intervalo (abril de 2026)
    let mut ev_movido = EventModel::default();
    ev_movido.tipo_operacao = "Movido".to_string();
    ev_movido.timestamp = "2026-04-10 12:00:00".to_string();

    // Com ignorar_data_mover = true (padrão), o evento movido DEVE passar mesmo fora da data
    assert!(filtro.ignorar_data_mover);
    assert!(
        filtro.corresponde(&ev_movido),
        "Movido deve passar quando ignorar_data_mover é true"
    );

    // Quando desativa ignorar_data_mover, o filtro de data passa a ser exigido
    filtro.ignorar_data_mover = false;
    assert!(
        !filtro.corresponde(&ev_movido),
        "Movido NÃO deve passar quando ignorar_data_mover é false"
    );

    // Testa para renomeado
    let mut ev_renomeado = EventModel::default();
    ev_renomeado.tipo_operacao = "Renomeado".to_string();
    ev_renomeado.timestamp = "2026-04-10 12:00:00".to_string();

    assert!(filtro.ignorar_data_renomeados);
    assert!(filtro.corresponde(&ev_renomeado));

    filtro.ignorar_data_renomeados = false;
    assert!(!filtro.corresponde(&ev_renomeado));

    // Testa para adicionado
    let mut ev_add = EventModel::default();
    ev_add.tipo_operacao = "Adicionado".to_string();
    ev_add.timestamp = "2026-04-10 12:00:00".to_string();

    assert!(filtro.ignorar_data_adicionados);
    assert!(filtro.corresponde(&ev_add));

    filtro.ignorar_data_adicionados = false;
    assert!(!filtro.corresponde(&ev_add));

    // Testa para excluído
    let mut ev_del = EventModel::default();
    ev_del.tipo_operacao = "Excluído".to_string();
    ev_del.timestamp = "2026-04-10 12:00:00".to_string();

    assert!(filtro.ignorar_data_excluidos);
    assert!(filtro.corresponde(&ev_del));

    filtro.ignorar_data_excluidos = false;
    assert!(!filtro.corresponde(&ev_del));

    // Testa para modificado
    let mut ev_mod = EventModel::default();
    ev_mod.tipo_operacao = "Modificado".to_string();
    ev_mod.timestamp = "2026-04-10 12:00:00".to_string();

    assert!(filtro.ignorar_data_modificados);
    assert!(filtro.corresponde(&ev_mod));

    filtro.ignorar_data_modificados = false;
    assert!(!filtro.corresponde(&ev_mod));

    // Testa para escaneado
    let mut ev_scan = EventModel::default();
    ev_scan.tipo_operacao = "Escaneado".to_string();
    ev_scan.timestamp = "2026-04-10 12:00:00".to_string();

    assert!(filtro.ignorar_data_escaneados);
    assert!(filtro.corresponde(&ev_scan));

    filtro.ignorar_data_escaneados = false;
    assert!(!filtro.corresponde(&ev_scan));
}

#[test]
fn test_manual_dialog_sections_all_languages() {
    let idiomas = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];
    for lang in idiomas {
        let secoes = obter_secoes_manual(lang);
        assert!(
            !secoes.is_empty(),
            "Idioma {} deve conter seções no manual",
            lang
        );
        for sec in &secoes {
            assert!(!sec.id.is_empty(), "ID da seção não pode ser vazio");
            assert!(!sec.title.is_empty(), "Título da seção não pode ser vazio");
            assert!(!sec.paragraphs.is_empty(), "Seção deve conter parágrafos");
        }
    }

    let mut manual_dlg = ManualDialog::default();
    assert!(!manual_dlg.aberto);
    manual_dlg.abrir();
    assert!(manual_dlg.aberto);
    assert!(manual_dlg.elevar_z_order);
}

#[test]
fn test_about_dialog_all_tabs_and_history() {
    let mut about_dlg = AboutDialog::default();
    assert!(!about_dlg.aberto);
    about_dlg.abrir();
    assert!(about_dlg.aberto);
    assert!(about_dlg.elevar_z_order);

    let abas = [
        AboutTab::Sobre,
        AboutTab::Historico,
        AboutTab::Clc,
        AboutTab::Copyright,
        AboutTab::Eula,
        AboutTab::Licencas,
        AboutTab::Avisos,
        AboutTab::Privacidade,
        AboutTab::Release,
    ];

    for tab in abas {
        let texto = AboutDialog::obter_texto_aba(tab, "pt_BR");
        assert!(
            !texto.trim().is_empty(),
            "Aba {:?} deve conter texto não vazio",
            tab
        );
    }

    // Verifica que a aba de Histórico contém menção ao nome Linceu e Lighthouse
    let historico_pt = AboutDialog::obter_texto_aba(AboutTab::Historico, "pt_BR");
    assert!(
        historico_pt.contains("Linceu"),
        "Histórico deve mencionar Linceu"
    );
    assert!(
        historico_pt.contains("Lighthouse"),
        "Histórico deve mencionar Lighthouse"
    );

    let licenca_pt = AboutDialog::obter_texto_aba(AboutTab::Licencas, "pt_BR");
    assert!(
        licenca_pt.to_lowercase().contains("mit") || licenca_pt.contains("Licença"),
        "Aba licenças deve conter dados de licença"
    );
}

#[test]
fn test_advanced_filters_dialog_initialization() {
    let mut dlg = AdvancedFiltersDialog::default();
    assert!(!dlg.aberto);
    dlg.abrir();
    assert!(dlg.aberto);
    assert!(dlg.elevar_z_order);
}

#[test]
fn test_translations_filter_keys_all_languages() {
    let loc = Localizer::new();
    let chaves = [
        "advanced_filters",
        "operation_filter",
        "extension_filter",
        "date_filter",
        "clear_filters",
        "ignore_move_filter",
        "ignore_rename_filter",
        "ignore_add_filter",
        "ignore_delete_filter",
        "ignore_modified_filter",
        "ignore_scanned_filter",
        "topics",
        "find",
        "tab_about",
        "tab_history",
        "tab_clc",
        "tab_licenses",
        "tab_privacy",
        "tab_release",
    ];

    let idiomas = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];
    for lang in idiomas {
        loc.set_idioma(lang);
        for &chave in &chaves {
            let t = loc.t(chave);
            assert_ne!(
                t, chave,
                "Chave '{}' deve estar traduzida em '{}'",
                chave, lang
            );
            assert!(
                !t.is_empty(),
                "Tradução de '{}' em '{}' não pode ser vazia",
                chave,
                lang
            );
        }
    }
}

#[test]
fn test_modeless_dialogs_state_and_z_order() {
    use huginn_rs::ui::dialogs::{
        AlertCenterDialog, ColorDialog, ColumnDialog, EvidenceDialog, MultiDirectoryDialog,
        NotificationConfigDialog, RecoveryDialog, SessionCenterDialog,
    };

    let mut col_dlg = ColumnDialog::default();
    assert!(!col_dlg.aberto);
    assert!(!col_dlg.minimizado);
    assert!(!col_dlg.maximizado);
    col_dlg.abrir();
    assert!(col_dlg.aberto);
    assert!(col_dlg.elevar_z_order);
    assert!(!col_dlg.minimizado);

    let mut color_dlg = ColorDialog::default();
    color_dlg.abrir();
    assert!(color_dlg.aberto);
    assert!(color_dlg.elevar_z_order);

    let mut alert_dlg = AlertCenterDialog::default();
    alert_dlg.abrir();
    assert!(alert_dlg.aberto);
    assert!(alert_dlg.elevar_z_order);

    let mut session_dlg = SessionCenterDialog::default();
    session_dlg.abrir();
    assert!(session_dlg.aberto);
    assert!(session_dlg.elevar_z_order);

    let mut rec_dlg = RecoveryDialog::default();
    rec_dlg.abrir();
    assert!(rec_dlg.aberto);
    assert!(rec_dlg.elevar_z_order);

    let mut multi_dlg = MultiDirectoryDialog::default();
    multi_dlg.abrir();
    assert!(multi_dlg.aberto);
    assert!(multi_dlg.elevar_z_order);

    let mut evi_dlg = EvidenceDialog::default();
    evi_dlg.abrir();
    assert!(evi_dlg.aberto);
    assert!(evi_dlg.elevar_z_order);

    let mut notif_dlg = NotificationConfigDialog::default();
    notif_dlg.abrir();
    assert!(notif_dlg.alerts_popup.aberto);
    assert!(notif_dlg.alerts_popup.elevar_z_order);
    notif_dlg.abrir_sessions();
    assert!(notif_dlg.sessions_popup.aberto);
    notif_dlg.abrir_quarantine();
    assert!(notif_dlg.quarantine_popup.aberto);
    notif_dlg.abrir_summary();
    assert!(notif_dlg.summary_popup.aberto);
}

#[test]
fn test_table_column_widths_resizing_and_persistence() {
    use huginn_rs::services::gerenciador_colunas::GerenciadorColunas;

    let mut gc = GerenciadorColunas::new();
    let colunas_originais = gc.colunas().to_vec();
    assert!(!colunas_originais.is_empty());

    // Altera largura de 'nome'
    gc.set_largura("nome", 350.0);
    let nome_col = gc.colunas().iter().find(|c| c.key == "nome").unwrap();
    assert_eq!(nome_col.largura_padrao, 350.0);

    // Clamping mínimo e máximo
    gc.set_largura("nome", 10.0); // Mínimo é 50.0
    let nome_col = gc.colunas().iter().find(|c| c.key == "nome").unwrap();
    assert_eq!(nome_col.largura_padrao, 50.0);

    gc.set_largura("nome", 5000.0); // Máximo é 1200.0
    let nome_col = gc.colunas().iter().find(|c| c.key == "nome").unwrap();
    assert_eq!(nome_col.largura_padrao, 1200.0);

    // Resetar larguras
    gc.resetar_larguras();
    let nome_col = gc.colunas().iter().find(|c| c.key == "nome").unwrap();
    assert_eq!(nome_col.largura_padrao, 200.0);
}

#[test]
fn test_cleanup_dialog_lifecycle_and_steps() {
    use huginn_rs::ui::dialogs::cleanup::{CleanupDialog, CleanupStep, LimpezaAlvo};

    let mut dlg = CleanupDialog::default();
    assert!(!dlg.aberto);
    assert_eq!(dlg.step, CleanupStep::SelecionarAlvo);

    dlg.abrir_selecao();
    assert!(dlg.aberto);
    assert!(dlg.elevar_z_order);
    assert_eq!(dlg.step, CleanupStep::SelecionarAlvo);

    dlg.processar_resultado(LimpezaAlvo::BancoDados, Ok(()));
    assert_eq!(dlg.step, CleanupStep::Sucesso(LimpezaAlvo::BancoDados));

    dlg.processar_resultado(LimpezaAlvo::Ambos, Err("Erro de teste".to_string()));
    assert_eq!(dlg.step, CleanupStep::Erro("Erro de teste".to_string()));

    dlg.abrir_alvo(LimpezaAlvo::BancoSeguranca);
    assert!(dlg.aberto);
    assert!(dlg.aberto_direto);
    assert_eq!(dlg.step, CleanupStep::Confirmacao(LimpezaAlvo::BancoSeguranca));

    dlg.fechar();
    assert!(!dlg.aberto);
    assert!(!dlg.aberto_direto);
    assert_eq!(dlg.step, CleanupStep::SelecionarAlvo);

    let ctx = egui::Context::default();
    let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
    out.textures_delta.clear();
    let idiomas = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];
    let loc = huginn_rs::locale::Localizer::new();
    for lang in idiomas {
        loc.set_idioma(lang);
        let (btn_w, win_w, win_h) = CleanupDialog::calcular_geometria(&ctx, &loc, &CleanupStep::SelecionarAlvo);
        assert!(btn_w >= 200.0, "btn_w deve ter tamanho suficiente em {}: {}", lang, btn_w);
        assert!(win_w >= btn_w, "win_w deve acompanhar btn_w em {}: {} vs {}", lang, win_w, btn_w);
        assert!(win_w >= 340.0, "win_w deve respeitar mínimo do Windows 11 em {}: {}", lang, win_w);
        assert_eq!(win_h, 226.0);

        let (_, _, h_confirm_ambos) = CleanupDialog::calcular_geometria(&ctx, &loc, &CleanupStep::Confirmacao(LimpezaAlvo::Ambos));
        assert_eq!(h_confirm_ambos, 198.0);

        let (_, _, h_confirm_db) = CleanupDialog::calcular_geometria(&ctx, &loc, &CleanupStep::Confirmacao(LimpezaAlvo::BancoDados));
        assert_eq!(h_confirm_db, 168.0);

        let (_, _, h_sucesso) = CleanupDialog::calcular_geometria(&ctx, &loc, &CleanupStep::Sucesso(LimpezaAlvo::Ambos));
        assert_eq!(h_sucesso, 126.0);

        let (_, _, h_erro) = CleanupDialog::calcular_geometria(&ctx, &loc, &CleanupStep::Erro("teste".to_string()));
        assert_eq!(h_erro, 132.0);
    }

    // Testa a renderização com mostrar em cada passo e alvo
    for lang in ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"] {
        loc.set_idioma(lang);
        for alvo in [LimpezaAlvo::Ambos, LimpezaAlvo::BancoDados, LimpezaAlvo::TabelaUI, LimpezaAlvo::BancoSeguranca] {
            let mut dlg = CleanupDialog::default();
            dlg.abrir_alvo(alvo);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                dlg.mostrar(ctx, &loc, |_| {});
            });
            out.textures_delta.clear();

            dlg.processar_resultado(alvo, Ok(()));
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                dlg.mostrar(ctx, &loc, |_| {});
            });
            out.textures_delta.clear();
        }
    }
}
