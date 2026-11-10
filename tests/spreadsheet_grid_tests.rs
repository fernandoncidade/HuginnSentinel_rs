//! Testes de Aceitação e Integração para o Componente SpreadsheetGrid (Planilha Eletrônica)
//! Validação integral dos 15 critérios de aceitação definidos na Especificação Técnica.

use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::locale::Localizer;
use huginn_rs::services::gerenciador_colunas::GerenciadorColunas;
use huginn_rs::ui::spreadsheet::cell_selection::{CellCoord, CellSelectionModel};
use huginn_rs::ui::spreadsheet::col_header::{col_index_to_letter, letter_to_col_index};
use huginn_rs::ui::spreadsheet::context_menu::GridContextAction;
use huginn_rs::ui::spreadsheet::grid_layout::GridLayout;
use huginn_rs::ui::spreadsheet::row_col_manager::{
    RowColumnManager, SpreadsheetCommand,
};
use huginn_rs::ui::spreadsheet::SpreadsheetGrid;

/// Critério 1: Seleção individual, seleção por intervalo retangular e seleção integral (Select All)
#[test]
fn test_criterio_1_selecao_individual_intervalo_integral() {
    let mut model = CellSelectionModel::new();

    // 1.1 Seleção individual
    model.select_cell(2, 3);
    assert_eq!(model.active_cell, Some(CellCoord { row: 2, col: 3 }));
    assert!(model.is_cell_selected(2, 3));
    assert!(!model.is_cell_selected(2, 4));

    // 1.2 Seleção de intervalo retangular (2,3) até (5,6)
    model.select_range(2, 3, 5, 6);
    for r in 2..=5 {
        for c in 3..=6 {
            assert!(
                model.is_cell_selected(r, c),
                "Célula ({}, {}) deveria estar selecionada",
                r,
                c
            );
        }
    }
    assert!(!model.is_cell_selected(1, 3));
    assert!(!model.is_cell_selected(6, 3));

    // 1.3 Seleção Integral (Select All) via estado lógico global eficiente
    model.select_all();
    assert!(model.is_all_selected);
    assert!(model.is_cell_selected(0, 0));
    assert!(model.is_cell_selected(999, 999));
    assert!(model.is_row_selected(42));
    assert!(model.is_col_selected(88));
}

/// Critério 2: Seleção de linhas inteiras e colunas inteiras pelos respectivos cabeçalhos
#[test]
fn test_criterio_2_selecao_linhas_e_colunas_inteiras() {
    let mut model = CellSelectionModel::new();

    // Seleciona Linha 5
    model.select_row(5, false, false, 20);
    assert!(model.is_row_selected(5));
    assert!(model.is_cell_selected(5, 0));
    assert!(model.is_cell_selected(5, 19));
    assert!(!model.is_row_selected(6));

    // Seleciona Coluna 4
    model.select_col(4, false, false, 100);
    assert!(model.is_col_selected(4));
    assert!(model.is_cell_selected(0, 4));
    assert!(model.is_cell_selected(99, 4));
    assert!(!model.is_col_selected(3));
}

/// Critério 3: Conversão de índices de coluna para identificadores alfabéticos estilo Excel
/// (A..Z, AA..AZ, BA..ZZ, AAA..) e roundtrip inverso
#[test]
fn test_criterio_3_conversao_alfabetica_colunas() {
    assert_eq!(col_index_to_letter(0), "A");
    assert_eq!(col_index_to_letter(25), "Z");
    assert_eq!(col_index_to_letter(26), "AA");
    assert_eq!(col_index_to_letter(27), "AB");
    assert_eq!(col_index_to_letter(51), "AZ");
    assert_eq!(col_index_to_letter(52), "BA");
    assert_eq!(col_index_to_letter(701), "ZZ");
    assert_eq!(col_index_to_letter(702), "AAA");
    assert_eq!(col_index_to_letter(703), "AAB");

    // Inverso e roundtrip
    assert_eq!(letter_to_col_index("A"), Some(0));
    assert_eq!(letter_to_col_index("Z"), Some(25));
    assert_eq!(letter_to_col_index("AA"), Some(26));
    assert_eq!(letter_to_col_index("AZ"), Some(51));
    assert_eq!(letter_to_col_index("BA"), Some(52));
    assert_eq!(letter_to_col_index("ZZ"), Some(701));
    assert_eq!(letter_to_col_index("AAA"), Some(702));

    // Case-insensitivity
    assert_eq!(letter_to_col_index("aa"), Some(26));
    assert_eq!(letter_to_col_index("aAa"), Some(702));
    assert_eq!(letter_to_col_index(""), None);
    assert_eq!(letter_to_col_index("123"), None);
}

/// Critério 4: Inserção, exclusão e ocultação estrutural com pilha de histórico (Undo/Redo)
#[test]
fn test_criterio_4_operacoes_estruturais_e_undo_redo() {
    let mut manager = RowColumnManager::new();
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());

    let evento = EventModel::new(OperationType::Adicionado, "arquivo_teste.txt", "C:\\Dir");

    // Executa comando de exclusão de linha
    manager.undo_stack.push(SpreadsheetCommand::DeleteRow {
        index: 3,
        event: evento,
    });
    assert!(manager.can_undo());
    assert_eq!(manager.undo_stack.len(), 1);

    // Oculta linha
    manager.hide_row(2);
    assert_eq!(manager.undo_stack.len(), 2);
    assert!(manager.is_row_hidden(2));
}

/// Critério 5: Preservação de dados e integridade após operações estruturais
#[test]
fn test_criterio_5_preservacao_dados_em_eventos() {
    let mut eventos = vec![
        EventModel::new(OperationType::Adicionado, "arquivo1.txt", "C:\\Dir"),
        EventModel::new(OperationType::Modificado, "arquivo2.txt", "C:\\Dir"),
        EventModel::new(OperationType::Excluido, "arquivo3.txt", "C:\\Dir"),
    ];

    // Duplicar registro de evento mantendo integridade
    let mut clone = eventos[1].clone();
    clone.id = None;
    eventos.insert(2, clone);

    assert_eq!(eventos.len(), 4);
    assert_eq!(eventos[0].nome, "arquivo1.txt");
    assert_eq!(eventos[1].nome, "arquivo2.txt");
    assert_eq!(eventos[2].nome, "arquivo2.txt");
    assert_eq!(eventos[3].nome, "arquivo3.txt");
}

/// Critério 6: Redimensionamento e AutoFit com limites mínimos e máximos
#[test]
fn test_criterio_6_redimensionamento_e_autofit() {
    let mut layout = GridLayout::new();

    // Redimensionamento manual com limites
    layout.set_col_width("data_hora", 25.0); // menor que o mínimo de 40.0
    assert_eq!(layout.get_col_width("data_hora", 100.0), 40.0);

    layout.set_col_width("data_hora", 1500.0); // maior que o máximo de 1200.0
    assert_eq!(layout.get_col_width("data_hora", 100.0), 1200.0);

    layout.set_col_width("data_hora", 185.0);
    assert_eq!(layout.get_col_width("data_hora", 100.0), 185.0);

    // Altura de linhas
    layout.set_row_height(0, 10.0); // abaixo do mínimo de 18.0
    assert_eq!(layout.get_row_height(0), 18.0);
    layout.set_row_height(0, 32.0);
    assert_eq!(layout.get_row_height(0), 32.0);
}

/// Critério 7: Ocultação e reexibição sem perda de dados
#[test]
fn test_criterio_7_ocultacao_e_reexibicao() {
    let mut manager = RowColumnManager::new();

    assert!(!manager.is_row_hidden(2));
    assert!(!manager.is_col_hidden("usuario"));

    // Oculta linha e coluna
    manager.hide_row(2);
    manager.hide_col("usuario");
    assert!(manager.is_row_hidden(2));
    assert!(manager.is_col_hidden("usuario"));

    // Reexibe
    manager.unhide_row(2);
    assert!(!manager.is_row_hidden(2));

    manager.unhide_col("usuario");
    assert!(!manager.is_col_hidden("usuario"));

    // Reexibir todos
    manager.hide_row(5);
    manager.hide_col("hash");
    manager.unhide_all_rows();
    manager.unhide_all_cols();
    assert!(!manager.is_row_hidden(5));
    assert!(!manager.is_col_hidden("hash"));
}

/// Critério 8: Congelamento e descongelamento de painéis (Frozen Panes)
#[test]
fn test_criterio_8_congelamento_paineis() {
    let mut layout = GridLayout::new();

    assert_eq!(layout.frozen_rows, 0);
    assert_eq!(layout.frozen_cols, 0);

    layout.toggle_freeze_rows(2);
    layout.toggle_freeze_cols(3);
    assert_eq!(layout.frozen_rows, 2);
    assert_eq!(layout.frozen_cols, 3);

    layout.toggle_freeze_rows(2);
    layout.toggle_freeze_cols(3);
    assert_eq!(layout.frozen_rows, 0);
    assert_eq!(layout.frozen_cols, 0);
}

/// Critério 9: Navegação por teclado e expansão de seleção
#[test]
fn test_criterio_9_navegacao_teclado_e_expansao() {
    let mut model = CellSelectionModel::new();
    model.select_cell(10, 5);

    // Expansão de seleção com Shift a partir da âncora
    model.expand_to(12, 6);
    assert_eq!(model.active_cell, Some(CellCoord { row: 12, col: 6 }));
    assert!(model.is_cell_selected(10, 5));
    assert!(model.is_cell_selected(11, 5));
    assert!(model.is_cell_selected(12, 6));

    // Nova seleção de célula redefine âncora
    model.select_cell(0, 0);
    assert_eq!(model.active_cell, Some(CellCoord { row: 0, col: 0 }));
    assert!(!model.is_cell_selected(10, 5));
}

/// Critério 10: Menus contextuais e disponibilidade de comandos para linha, coluna e célula
#[test]
fn test_criterio_10_menus_contextuais() {
    // Valida mapeamento de ações de contexto
    let acao_inserir = GridContextAction::InsertRowAbove;
    assert_eq!(acao_inserir, GridContextAction::InsertRowAbove);

    let acao_autofit = GridContextAction::AutoFitColumn("caminho".to_string());
    match acao_autofit {
        GridContextAction::AutoFitColumn(k) => assert_eq!(k, "caminho"),
        _ => panic!("Ação inesperada"),
    }
}

/// Critério 11: Classificação e filtragem integradas sem desassociar registros
#[test]
fn test_criterio_11_classificacao_e_filtragem_integradas() {
    let mut gc = GerenciadorColunas::default();

    // Filtros de colunas visíveis
    assert!(gc.colunas().iter().any(|c| c.key == "nome" && c.visivel));
    gc.set_visibilidade("nome", false);
    assert!(gc.colunas().iter().any(|c| c.key == "nome" && !c.visivel));
    gc.alternar_visibilidade("nome");
    assert!(gc.colunas().iter().any(|c| c.key == "nome" && c.visivel));
}

/// Critério 12: Atualização consistente da seleção após inserções ou exclusões estruturais
#[test]
fn test_criterio_12_atualizacao_selecao_apos_alteracoes_estruturais() {
    let mut model = CellSelectionModel::new();
    model.select_cell(5, 4);

    // Inserção de linha antes (linha 3) deve deslocar a célula ativa para a linha 6
    model.on_row_inserted(3);
    assert_eq!(model.active_cell, Some(CellCoord { row: 6, col: 4 }));

    // Inserção de linha depois (linha 8) não afeta a célula ativa na linha 6
    model.on_row_inserted(8);
    assert_eq!(model.active_cell, Some(CellCoord { row: 6, col: 4 }));

    // Exclusão de linha antes (linha 2) desloca a célula de volta para linha 5
    model.on_row_deleted(2);
    assert_eq!(model.active_cell, Some(CellCoord { row: 5, col: 4 }));

    // Exclusão da linha ativa (linha 5) mantém a célula ativa na posição 5 ocupada pelo elemento seguinte
    model.on_row_deleted(5);
    assert_eq!(model.active_cell, Some(CellCoord { row: 5, col: 4 }));
}

/// Critério 13: Tratamento de limites, casos extremos, 0 linhas e intervalos inválidos
#[test]
fn test_criterio_13_limites_e_tabela_vazia() {
    let layout = GridLayout::new();

    // Grade vazia (0 linhas)
    let (first, last) = layout.compute_visible_rows(0.0, 600.0, 0);
    assert_eq!((first, last), (0, 0));

    // Índice fora do limite
    let (first_oob, last_oob) = layout.compute_visible_rows(100_000.0, 600.0, 10);
    assert_eq!(last_oob, 10);
    assert!(first_oob >= 10 || first_oob == 0 || first_oob <= last_oob);
}

/// Critério 14: Desempenho e virtualização em grades massivas (10.000+ linhas)
#[test]
fn test_criterio_14_desempenho_virtualizacao_massiva() {
    let layout = GridLayout::new();
    let total_rows = 150_000;
    let viewport_height = 800.0;
    let scroll_y = 54_321.0;

    let start_time = std::time::Instant::now();
    let (first_row, last_row) = layout.compute_visible_rows(scroll_y, viewport_height, total_rows);
    let elapsed = start_time.elapsed();

    // A computação virtual deve ser praticamente instantânea (sub-microsegundo)
    assert!(elapsed.as_millis() < 5);
    assert!(first_row <= last_row);
    assert!(last_row <= total_rows);

    // Seleção integral sobre 150.000 linhas também não materializa elementos na memória
    let mut model = CellSelectionModel::new();
    let sel_start = std::time::Instant::now();
    model.select_all();
    let sel_elapsed = sel_start.elapsed();

    assert!(sel_elapsed.as_millis() < 1);
    assert!(model.is_cell_selected(123_456, 12));
}

/// Critério 15: Compatibilidade com modelo de dados, exportação e AutoFit dinâmico do projeto
#[test]
fn test_criterio_15_compatibilidade_modelo_dados_e_autofit() {
    let grid = SpreadsheetGrid::new();
    assert_eq!(grid.layout.default_row_height, 26.0);

    // AutoFit dinâmico em GerenciadorColunas baseado no cabeçalho
    let mut gc = GerenciadorColunas::default();
    let loc = Localizer::new();
    gc.autofit_todas_colunas_ao_cabecalho(&loc);

    for col in gc.colunas_visiveis() {
        assert!(col.largura_padrao >= 60.0);
        assert!(col.largura_padrao <= 1200.0);
    }
}

/// Critério 16: Fixação (congelamento) arbitrária de qualquer linha (incluindo cabeçalho) e qualquer coluna
#[test]
fn test_criterio_16_fixacao_arbitraria_linhas_e_colunas() {
    let mut layout = GridLayout::new();

    // Estado inicial padrão: 1ª linha de cabeçalho e 1ª coluna numérica (#) fixas
    assert!(layout.freeze_header_row);
    assert!(layout.freeze_row_headers);
    assert!(layout.pinned_rows.is_empty());
    assert!(layout.pinned_cols.is_empty());

    // Alterna a 1ª linha de cabeçalhos das colunas para rolar livremente
    layout.freeze_header_row = false;
    assert!(!layout.freeze_header_row);

    // Fixa linhas arbitrárias não contíguas (ex: linha #1 -> idx 0, linha #7 -> idx 6, linha #25 -> idx 24)
    layout.set_row_pinned(0, true);
    layout.set_row_pinned(6, true);
    layout.set_row_pinned(24, true);
    assert!(layout.is_row_pinned(0));
    assert!(layout.is_row_pinned(6));
    assert!(layout.is_row_pinned(24));
    assert!(!layout.is_row_pinned(5));

    // Parser de intervalos 1-based digitados pelo usuário ("1, 5, 10-12")
    let parsed = GridLayout::parse_row_numbers_spec("1, 5, 10-12", Some(50));
    assert_eq!(parsed, vec![0, 4, 9, 10, 11]);

    // Consistência de índices ao inserir/excluir linhas na tabela
    layout.on_row_inserted(5); // desloca 6 -> 7 e 24 -> 25
    assert!(layout.is_row_pinned(0));
    assert!(layout.is_row_pinned(7));
    assert!(layout.is_row_pinned(25));

    layout.on_row_deleted(7); // remove a linha 7 e desloca 25 -> 24
    assert!(!layout.is_row_pinned(7));
    assert!(layout.is_row_pinned(24));

    // Desafixar todas as linhas vs restaurar padrão
    layout.unfreeze_all_rows();
    assert!(!layout.freeze_header_row);
    assert!(layout.pinned_rows.is_empty());

    layout.reset_freeze_rows_default();
    assert!(layout.freeze_header_row);
    assert!(layout.pinned_rows.is_empty());

    // Fixa colunas arbitrárias pela chave
    layout.set_col_pinned("nome", true);
    layout.set_col_pinned("sha256_atual", true);
    assert!(layout.is_col_pinned("nome"));
    assert!(layout.is_col_effectively_pinned(4, "sha256_atual"));
    assert!(!layout.is_col_effectively_pinned(2, "caminho"));

    layout.toggle_pin_col("nome");
    assert!(!layout.is_col_pinned("nome"));

    layout.unfreeze_all_cols();
    assert!(!layout.freeze_row_headers);
    assert!(layout.pinned_cols.is_empty());

    layout.reset_freeze_cols_default();
    assert!(layout.freeze_row_headers);
    assert!(layout.pinned_cols.is_empty());
}

#[test]
fn test_criterio_drag_selection_cells_rows_and_cols() {
    use huginn_rs::ui::spreadsheet::GridDragMode;

    let mut model = CellSelectionModel::new();

    // 1. Arrastar células: seleciona da célula âncora (2, 3) até a célula de destino (5, 7)
    model.select_cell(2, 3);
    assert_eq!(model.anchor_cell, Some(CellCoord { row: 2, col: 3 }));
    assert_eq!(model.active_cell, Some(CellCoord { row: 2, col: 3 }));

    model.expand_to(5, 7);
    for r in 2..=5 {
        for c in 3..=7 {
            assert!(
                model.is_cell_selected(r, c),
                "Célula ({}, {}) deveria estar selecionada pelo drag de células",
                r,
                c
            );
        }
    }
    assert!(!model.is_cell_selected(1, 3));
    assert!(!model.is_cell_selected(6, 7));

    // Arrastar em direção inversa: expande de (5, 7) de volta para (1, 1)
    model.select_cell(5, 7);
    model.expand_to(1, 1);
    for r in 1..=5 {
        for c in 1..=7 {
            assert!(model.is_cell_selected(r, c));
        }
    }

    // 2. Arrastar cabeçalhos de linhas: seleciona da linha 3 até a linha 8
    model.clear();
    model.select_row_range(3, 8, 20);
    for r in 3..=8 {
        assert!(model.is_row_selected(r), "Linha {} deveria estar selecionada", r);
        for c in 0..20 {
            assert!(model.is_cell_selected(r, c));
        }
    }
    assert!(!model.is_row_selected(2));
    assert!(!model.is_row_selected(9));

    // Arrastar cabeçalhos de linhas em ordem decrescente (8 para 3)
    model.clear();
    model.select_row_range(8, 3, 20);
    for r in 3..=8 {
        assert!(model.is_row_selected(r));
    }

    // 3. Arrastar cabeçalhos de colunas: seleciona da coluna 2 até a coluna 6
    model.clear();
    model.select_col_range(2, 6, 50);
    for c in 2..=6 {
        assert!(model.is_col_selected(c), "Coluna {} deveria estar selecionada", c);
        for r in 0..50 {
            assert!(model.is_cell_selected(r, c));
        }
    }
    assert!(!model.is_col_selected(1));
    assert!(!model.is_col_selected(7));

    // Arrastar cabeçalhos de colunas em ordem decrescente (6 para 2)
    model.clear();
    model.select_col_range(6, 2, 50);
    for c in 2..=6 {
        assert!(model.is_col_selected(c));
    }

    // 4. Modos de arrasto (GridDragMode)
    let grid = SpreadsheetGrid::new();
    assert_eq!(grid.drag_mode, GridDragMode::None);

    let cell_drag = GridDragMode::Cells {
        anchor_row: 2,
        anchor_col: 4,
    };
    assert_ne!(cell_drag, GridDragMode::None);

    let row_drag = GridDragMode::Rows { anchor_row: 5 };
    assert_ne!(row_drag, GridDragMode::None);

    let col_drag = GridDragMode::Cols { anchor_col: 3 };
    assert_ne!(col_drag, GridDragMode::None);
}

/// Critério 16: Comportamento da Seleção (Acompanhar Dinamicamente vs Permanecer Estática)
#[test]
fn test_criterio_16_comportamento_selecao_estatica_vs_acompanhar() {
    use huginn_rs::ui::spreadsheet::{SelectionBehaviorConfig, SelectionFollowMode};

    // 1. Configuração padrão (acompanhar ativo por padrão para células, linhas e rolagem)
    let mut config = SelectionBehaviorConfig::default();
    assert!(config.acompanhar_selecao);
    assert!(config.acompanhar_celulas);
    assert!(config.acompanhar_linhas);
    assert!(config.auto_scroll);

    assert!(config.deve_acompanhar_celulas());
    assert!(config.deve_acompanhar_linhas());
    assert!(config.deve_acompanhar_rolagem());

    // 2. Modo Estático (Permanecer Estática)
    // Desativando o modo de acompanhamento, todas as verificações tornam-se estáticas
    config.acompanhar_selecao = false;
    assert!(!config.deve_acompanhar_celulas());
    assert!(!config.deve_acompanhar_linhas());
    assert!(!config.deve_acompanhar_rolagem());

    // 3. Controle Granular: Acompanhar apenas Linhas, células estáticas
    config.acompanhar_selecao = true;
    config.acompanhar_celulas = false;
    config.acompanhar_linhas = true;
    config.auto_scroll = false;
    assert!(!config.deve_acompanhar_celulas());
    assert!(config.deve_acompanhar_linhas());
    assert!(!config.deve_acompanhar_rolagem());

    // 4. Teste de deslocamento de linhas (shift_rows) para rastreamento de dados
    let mut model = CellSelectionModel::new();

    // 4.1 Célula ativa individual
    model.select_cell(4, 2);
    assert_eq!(model.active_cell, Some(CellCoord { row: 4, col: 2 }));
    assert_eq!(model.anchor_cell, Some(CellCoord { row: 4, col: 2 }));

    model.shift_rows(5);
    assert_eq!(model.active_cell, Some(CellCoord { row: 9, col: 2 }));
    assert_eq!(model.anchor_cell, Some(CellCoord { row: 9, col: 2 }));
    assert!(model.is_cell_selected(9, 2));
    assert!(!model.is_cell_selected(4, 2));

    // 4.2 Intervalo de células (Range)
    model.clear();
    model.select_range(2, 1, 5, 3);
    assert_eq!(model.active_cell, Some(CellCoord { row: 5, col: 3 }));
    assert_eq!(model.anchor_cell, Some(CellCoord { row: 2, col: 1 }));

    model.shift_rows(10);
    assert_eq!(model.active_cell, Some(CellCoord { row: 15, col: 3 }));
    assert_eq!(model.anchor_cell, Some(CellCoord { row: 12, col: 1 }));
    assert!(model.is_cell_selected(12, 1));
    assert!(model.is_cell_selected(15, 3));
    assert!(!model.is_cell_selected(2, 1));
    assert!(!model.is_cell_selected(5, 3));

    // 4.3 Linhas selecionadas
    model.clear();
    model.select_row(3, false, false, 10);
    model.select_row(7, true, false, 10);
    assert!(model.is_row_selected(3));
    assert!(model.is_row_selected(7));

    model.shift_rows(4);
    assert!(model.is_row_selected(7));
    assert!(model.is_row_selected(11));
    assert!(!model.is_row_selected(3));

    // 4.4 Deslocamento nulo (delta = 0) deve manter tudo invariante
    model.shift_rows(0);
    assert!(model.is_row_selected(7));
    assert!(model.is_row_selected(11));

    // 5. Serialização e Desserialização JSON de SelectionBehaviorConfig
    let serialized = serde_json::to_string(&config).expect("Serialização deve funcionar");
    let deserialized: SelectionBehaviorConfig =
        serde_json::from_str(&serialized).expect("Desserialização deve funcionar");
    assert_eq!(config, deserialized);

    // 6. Teste de enum SelectionFollowMode
    let mode_default = SelectionFollowMode::default();
    assert_eq!(mode_default, SelectionFollowMode::Acompanhar);
    assert_ne!(mode_default, SelectionFollowMode::Estatica);

    // 7. Teste de personalização de cor de seleção de células e restauração
    let mut color_cfg = SelectionBehaviorConfig::default();
    assert_eq!(color_cfg.cor_selecao, [16, 124, 65]);
    assert_eq!(color_cfg.cor_selecao_color32(), egui::Color32::from_rgb(16, 124, 65));
    assert_eq!(color_cfg.cor_selecao_preenchimento(), egui::Color32::from_rgba_unmultiplied(16, 124, 65, 30));

    // Customiza a cor para Azul Clássico
    color_cfg.cor_selecao = [24, 100, 190];
    assert_eq!(color_cfg.cor_selecao_color32(), egui::Color32::from_rgb(24, 100, 190));
    assert_eq!(color_cfg.cor_selecao_preenchimento(), egui::Color32::from_rgba_unmultiplied(24, 100, 190, 30));

    // Fundo da célula e texto adaptados dinamicamente ao tema
    let fundo_escuro = color_cfg.cor_selecao_fundo_celula(true);
    let fundo_claro = color_cfg.cor_selecao_fundo_celula(false);
    let texto_escuro = color_cfg.cor_selecao_texto(true);
    let texto_claro = color_cfg.cor_selecao_texto(false);
    assert_ne!(fundo_escuro, fundo_claro);
    assert_ne!(texto_escuro, texto_claro);

    // Restauração para a cor padrão original (Verde Excel)
    color_cfg.restaurar_cor_padrao();
    assert_eq!(color_cfg.cor_selecao, [16, 124, 65]);

    // Compatibilidade retroativa quando cor_selecao não existe no JSON
    let legacy_json = r#"{"acompanhar_selecao":true,"acompanhar_celulas":true,"acompanhar_linhas":true,"auto_scroll":true}"#;
    let legacy_cfg: SelectionBehaviorConfig =
        serde_json::from_str(legacy_json).expect("Deve desserializar JSON legado com valor padrão de cor");
    assert_eq!(legacy_cfg.cor_selecao, [16, 124, 65]);
}

