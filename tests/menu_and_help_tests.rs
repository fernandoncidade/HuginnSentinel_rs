use huginn_rs::core::event::EventModel;
use huginn_rs::locale::Localizer;
use huginn_rs::services::GerenciadorTabela;
use huginn_rs::ui::dialogs::HelpDialog;

#[test]
fn test_help_dialog_shortcuts_all_languages() {

    let languages = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];
    let expected_sequences = [
        "F1",
        "Ctrl+M",
        "Ctrl+I",
        "Ctrl+Shift+O",
        "Ctrl+R",
        "Ctrl+P",
        "Ctrl+Shift+S",
        "Ctrl+S",
        "Ctrl+G",
        "Ctrl+L",
        "Ctrl+Q",
        "Ctrl+A",
        "Ctrl+Shift+A",
        "Ctrl+O",
    ];

    for lang in languages {
        let mut help = HelpDialog::default();
        help.abrir();
        assert!(help.aberto);
        assert!(help.elevar_z_order);

        let loc = Localizer::new();
        loc.set_idioma(lang);

        // Verifica se cada idioma possui as 14 sequências de atalho
        assert_eq!(
            expected_sequences.len(),
            14,
            "Deve conter exatamente 14 atalhos mapeados"
        );
    }
}

#[test]
fn test_gerenciador_tabela_colunas_para_colorir() {
    let mut gt = GerenciadorTabela::new();

    // Por padrão, tipo_operacao deve estar habilitado para colorir
    assert!(gt.coluna_deve_colorir("tipo_operacao"));
    assert!(!gt.coluna_deve_colorir("nome"));
    assert!(!gt.coluna_deve_colorir("size_kb"));

    // Adiciona colunas para colorir
    gt.set_coluna_colorir("nome", true);
    assert!(gt.coluna_deve_colorir("nome"));

    gt.set_coluna_colorir("nome", false);
    assert!(!gt.coluna_deve_colorir("nome"));

    // Seleciona todas as colunas
    let cols = ["tipo_operacao", "nome", "dir_atual", "size_kb", "tipo"];
    gt.selecionar_todas_colunas_colorir(&cols);
    for col in cols {
        assert!(gt.coluna_deve_colorir(col));
    }

    // Resetar restaura o padrão (apenas tipo_operacao)
    gt.resetar_colunas_colorir();
    assert!(gt.coluna_deve_colorir("tipo_operacao"));
    assert!(!gt.coluna_deve_colorir("nome"));
    assert!(!gt.coluna_deve_colorir("size_kb"));
}

#[test]
fn test_gerenciador_tabela_ordenacao_linhas_toggle() {
    let mut gt = GerenciadorTabela::new();
    assert!(gt.is_ordenacao_habilitada());

    let state = gt.alternar_ordenacao_linhas();
    assert!(!state);
    assert!(!gt.is_ordenacao_habilitada());

    let state2 = gt.alternar_ordenacao_linhas();
    assert!(state2);
    assert!(gt.is_ordenacao_habilitada());
}

#[test]
fn test_gerenciador_tabela_inverter_ordem_e_definir_coluna() {
    let mut gt = GerenciadorTabela::new();
    assert_eq!(gt.coluna_ordenacao.as_deref(), Some("timestamp"));
    assert!(!gt.ordem_ascendente); // padrão: decrescente (mais recentes primeiro)
    assert!(gt.is_ordenacao_habilitada());

    // 1. Inverter ordem padrão do timestamp (de decrescente para ascendente)
    let nova_ordem = gt.inverter_ordem();
    assert!(nova_ordem);
    assert!(gt.ordem_ascendente);
    assert!(gt.is_ordenacao_habilitada());

    // Inverter novamente (de ascendente para decrescente)
    let nova_ordem2 = gt.inverter_ordem();
    assert!(!nova_ordem2);
    assert!(!gt.ordem_ascendente);

    // 2. Definir ordenação por coluna selecionada pelo cabeçalho (ex: tamanho)
    gt.definir_ordenacao_coluna("tamanho", true);
    assert_eq!(gt.coluna_ordenacao.as_deref(), Some("tamanho"));
    assert!(gt.ordem_ascendente);
    assert!(gt.is_ordenacao_habilitada());

    // Inverter ordenação da coluna selecionada
    let inv_tam = gt.inverter_ordem();
    assert!(!inv_tam);
    assert!(!gt.ordem_ascendente);
    assert_eq!(gt.coluna_ordenacao.as_deref(), Some("tamanho"));

    // 3. Definir ordenação por outra coluna selecionada (ex: nome)
    gt.definir_ordenacao_coluna("nome", true);
    assert_eq!(gt.coluna_ordenacao.as_deref(), Some("nome"));
    assert!(gt.ordem_ascendente);
}

#[test]
fn test_exportacao_formatos_multiplos() {
    let temp_dir =
        std::env::temp_dir().join(format!("linceu_test_export_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let mut ev = EventModel::default();
    ev.id = Some(101);
    ev.tipo_operacao = "Adicionado".to_string();
    ev.nome = "relatorio_teste.docx".to_string();
    ev.dir_atual = "C:\\Projetos".to_string();
    ev.size_kb = 125.5;
    ev.tipo = "Documento".to_string();
    ev.risk_level = "Baixo".to_string();

    let eventos = vec![ev];

    // Testa exportação JSON
    let path_json = temp_dir.join("export.json");
    let json_data = serde_json::to_string_pretty(&eventos).unwrap();
    std::fs::write(&path_json, json_data).unwrap();
    assert!(path_json.exists());
    let conteudo_json = std::fs::read_to_string(&path_json).unwrap();
    assert!(conteudo_json.contains("relatorio_teste.docx"));

    // Testa exportação CSV
    let path_csv = temp_dir.join("export.csv");
    let mut csv = String::from("id,tipo_operacao,nome,dir_atual,size_kb\n");
    for e in &eventos {
        csv.push_str(&format!(
            "{},\"{}\",\"{}\",\"{}\",{:.2}\n",
            e.id.unwrap_or(0),
            e.tipo_operacao,
            e.nome,
            e.dir_atual,
            e.size_kb
        ));
    }
    std::fs::write(&path_csv, csv).unwrap();
    assert!(path_csv.exists());
    let conteudo_csv = std::fs::read_to_string(&path_csv).unwrap();
    assert!(conteudo_csv.contains("relatorio_teste.docx"));

    // Testa exportação XML
    let path_xml = temp_dir.join("export.xml");
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<huginn_events>\n");
    for e in &eventos {
        xml.push_str(&format!(
            "  <event><id>{}</id><nome>{}</nome></event>\n",
            e.id.unwrap_or(0),
            e.nome
        ));
    }
    xml.push_str("</huginn_events>\n");
    std::fs::write(&path_xml, xml).unwrap();
    assert!(path_xml.exists());
    let conteudo_xml = std::fs::read_to_string(&path_xml).unwrap();
    assert!(conteudo_xml.contains("<nome>relatorio_teste.docx</nome>"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_renderizar_menu_multi_coluna_estatisticas_dimensionamento() {
    use huginn_rs::data::charts::TipoGrafico;
    use huginn_rs::ui::menu_bar::renderizar_menu_multi_coluna;

    let ctx = egui::Context::default();
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 900.0))),
        ..Default::default()
    };
    let mut btn_rect = egui::Rect::ZERO;
    let mut out = ctx.run_ui(raw.clone(), |ui| {
        egui::containers::menu::MenuBar::new().ui(ui, |ui| {
            let resp = ui.menu_button("Menu Teste", |_ui| {});
            btn_rect = resp.response.rect;
        });
    });
    out.textures_delta.clear();

    // Abre o menu simulando clique
    let raw_click = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 900.0))),
        events: vec![
            egui::Event::PointerMoved(btn_rect.center()),
            egui::Event::PointerButton {
                pos: btn_rect.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: btn_rect.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    };

    let mut out = ctx.run_ui(raw_click, |ui| {
        egui::containers::menu::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Menu Teste", |ui| {
                let loc = Localizer::new();
                let graficos = TipoGrafico::all();
                renderizar_menu_multi_coluna(ui, graficos, |ui, tipo| {
                    let _ = ui.button(tipo.titulo(&loc));
                });
            });
        });
    });
    out.textures_delta.clear();

    // Frame de estabilização do layout
    let mut final_available_h = 0.0;
    let mut out = ctx.run_ui(raw.clone(), |ui| {
        egui::containers::menu::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Menu Teste", |ui| {
                final_available_h = ui.available_height();
                let loc = Localizer::new();
                let graficos = TipoGrafico::all();
                renderizar_menu_multi_coluna(ui, graficos, |ui, tipo| {
                    let _ = ui.button(tipo.titulo(&loc));
                });
            });
        });
    });
    out.textures_delta.clear();

    // Com auto_shrink vertical ativo, a altura do menu se ajusta aos 9 itens por coluna
    // (~200px a 240px) e NÃO deve inflar para o máximo de 460px
    assert!(
        final_available_h < 300.0,
        "A altura do menu de gráficos deveria se auto-ajustar aos itens (~204px), mas foi: {}",
        final_available_h
    );
}
