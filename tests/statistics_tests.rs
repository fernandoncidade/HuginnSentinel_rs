use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::data::charts::TipoGrafico;
use huginn_rs::data::generator::GeradorEstatisticas;
use huginn_rs::locale::Localizer;

#[test]
fn test_all_17_charts_data_generation() {
    let mut eventos = Vec::new();

    let mut e1 = EventModel::new(OperationType::Adicionado, "main.rs", "C:\\Projetos");
    e1.size_kb = 12.5;
    e1.tipo = "rs".to_string();
    eventos.push(e1);

    let mut e2 = EventModel::new(OperationType::Modificado, "Cargo.toml", "C:\\Projetos");
    e2.size_kb = 1.2;
    e2.tipo = "toml".to_string();
    eventos.push(e2);

    let mut e3 = EventModel::new(OperationType::Excluido, "temp.log", "C:\\Temp");
    e3.size_kb = 0.5;
    e3.tipo = "log".to_string();
    eventos.push(e3);

    assert_eq!(
        TipoGrafico::all().len(),
        17,
        "Devem existir exatamente 17 tipos de gráficos catalogados"
    );

    let pizza = GeradorEstatisticas::gerar_pizza_operacoes(&eventos);
    assert!(!pizza.labels.is_empty());
    assert!(!pizza.values.is_empty());

    let barras = GeradorEstatisticas::gerar_barras_tipos(&eventos);
    assert!(!barras.labels.is_empty());

    let pareto = GeradorEstatisticas::gerar_pareto_extensoes(&eventos);
    assert!(!pareto.labels.is_empty());

    let histograma = GeradorEstatisticas::gerar_histograma_horarios(&eventos);
    assert_eq!(histograma.labels.len(), 24);

    let heatmap = GeradorEstatisticas::gerar_heatmap_dia_hora(&eventos);
    assert_eq!(heatmap.labels.len(), 7 * 24);

    let boxplot = GeradorEstatisticas::gerar_boxplot_tamanhos(&eventos);
    assert_eq!(boxplot.labels.len(), 5);
}

#[test]
fn test_enhanced_charts_specifications() {
    let loc = huginn_rs::locale::Localizer::new();
    loc.set_idioma("pt_BR");
    let mut eventos = Vec::new();

    let mut e1 = EventModel::new(OperationType::Adicionado, "main.rs", "C:\\Projetos\\Linceu");
    e1.size_kb = 120.0;
    e1.timestamp = "1000".to_string();
    eventos.push(e1);

    let mut e2 = EventModel::new(OperationType::Adicionado, "lib.rs", "C:\\Projetos\\Linceu\\src");
    e2.size_kb = 80.0;
    e2.timestamp = "1010".to_string();
    eventos.push(e2);

    let mut e3 = EventModel::new(OperationType::Modificado, "readme.txt", "C:\\Projetos\\Linceu");
    e3.size_kb = 15.0;
    e3.timestamp = "1025".to_string();
    eventos.push(e3);

    // 1.1 Treemap reporta por extensões
    let treemap = GeradorEstatisticas::gerar_treemap_tipos_loc(&eventos, &loc);
    assert!(treemap.treemap_items.iter().any(|(lbl, _, _)| lbl.contains(".rs")));
    assert!(treemap.treemap_items.iter().any(|(lbl, _, _)| lbl.contains(".txt")));

    // 1.2 Linha de eventos começa em segundos e ajusta escala dinamicamente
    let linha = GeradorEstatisticas::gerar_linha_eventos_hora_loc(&eventos, &loc);
    assert!(!linha.time_unit_label.is_empty());
    assert!(
        linha.time_unit_label == loc.t("scale_seconds")
            || linha.time_unit_label.contains("Segundos")
            || linha.time_unit_label.contains("Seconds")
    );

    // 1.3 Boxplot multi-extensão
    let boxplot = GeradorEstatisticas::gerar_boxplot_tamanhos(&eventos);
    assert!(!boxplot.boxplot_groups.is_empty());
    assert!(boxplot.boxplot_groups.iter().any(|(ext, _, _, _, _, _)| ext.contains(".rs")));

    // 1.4 Dispersão com escala em segundos
    let scatter = GeradorEstatisticas::gerar_scatter_tamanho_hora(&eventos);
    assert!(scatter.time_unit_label.contains("Segundos"));
    assert_eq!(scatter.points.len(), 3);

    // 1.5 Sankey Operação -> Tipo de Extensão
    let sankey_tipo = GeradorEstatisticas::gerar_sankey_operacao_tipo_loc(&eventos, &loc);
    assert!(!sankey_tipo.flows.is_empty());
    assert!(sankey_tipo.flows.iter().any(|(_, dest, _)| dest.contains(".rs") || dest.contains(".txt")));

    // 1.6 Radar Multidimensional (10 dimensões: 6 operações separadas + 4 forenses)
    let radar = GeradorEstatisticas::gerar_radar_multidimensional_loc(&eventos, &loc);
    assert_eq!(radar.radar_dimensions.len(), 10);

    // 1.8 Sankey Operação -> Diretório
    let sankey_dir = GeradorEstatisticas::gerar_sankey_evento_diretorio_loc(&eventos, &loc);
    assert!(!sankey_dir.flows.is_empty());

    // 1.9 Sankey Diretório -> Tipo de Extensão
    let sankey_tipo_dir = GeradorEstatisticas::gerar_sankey_tipo_diretorio_loc(&eventos, &loc);
    assert!(!sankey_tipo_dir.flows.is_empty());

    // 2.0 Árvore completa de diretórios sem truncamento
    let arvore = GeradorEstatisticas::gerar_arvore_diretorios_loc(&eventos, &loc);
    assert!(!arvore.tree_nodes.is_empty());
    assert!(arvore.tree_nodes.iter().any(|(p, _, _, _)| p.contains("Linceu")));
}

#[test]
fn test_treemap_proportional_and_legends_all_languages() {
    let loc = huginn_rs::locale::Localizer::new();
    let idiomas = ["pt_BR", "en_US", "es_ES", "de_DE", "fr_FR", "it_IT"];

    // Verifica que todas as novas chaves de legendas e textos de gráficos existem em todos os 6 idiomas
    let required_keys = [
        "chart_legend_title",
        "chart_legend_treemap",
        "chart_legend_bars_type",
        "chart_legend_timeline",
        "chart_legend_hist_hour",
        "chart_legend_pareto_individual",
        "chart_legend_pareto_cumulative",
        "chart_legend_line_events",
        "chart_legend_boxplot",
        "chart_legend_radar_ops",
        "chart_legend_heat_intensity",
        "chart_legend_heat_low",
        "chart_legend_heat_high",
        "chart_legend_scatter",
        "chart_legend_sankey_flow",
        "chart_legend_radar_multi",
        "chart_legend_dotplot",
        "chart_legend_tree",
        "chart_no_data",
        "chart_no_data_treemap",
        "chart_no_data_timeline",
        "stat_no_extension",
        "stat_others",
        "stat_iqr",
        "stat_median",
        "stat_whiskers",
        "radar_dim_global_volume",
        "radar_dim_modifications",
        "radar_dim_deletions",
        "radar_dim_moves_renames",
        "radar_dim_executables",
        "radar_dim_baseline_alerts",
        "radar_dim_high_risk",
        "radar_dim_traceability",
        "chart_axis_operations_qty",
        "chart_axis_timeline_events",
        "chart_axis_extensions_qty",
        "chart_legend_dotplot_size",
        "chart_legend_dotplot_filled",
        "chart_legend_dotplot_hollow",
        "chart_legend_dotplot_proportional",
        "chart_limit_items_label",
        "chart_limit_all",
        "chart_showing_of_total",
        "chart_sankey_limit_operations",
        "chart_sankey_limit_file_types",
        "chart_sankey_limit_directories",
        "chart_sankey_showing_summary",
        "chart_entity_operations",
        "chart_entity_file_types",
        "chart_entity_directories",
    ];

    for lang in idiomas {
        loc.set_idioma(lang);
        for key in required_keys {
            let translated = loc.t(key);
            assert!(
                !translated.is_empty() && translated != key,
                "Chave '{}' deve estar traduzida para o idioma '{}' (obtido: '{}')",
                key,
                lang,
                translated
            );
        }
    }

    // Testa proporcionalidade da geração e distribuição de dados do Treemap
    let mut eventos = Vec::new();
    // 60 eventos de arquivos .rs
    for i in 0..60 {
        let mut e = EventModel::new(OperationType::Adicionado, &format!("code_{}.rs", i), "C:\\Src");
        e.size_kb = 10.0;
        eventos.push(e);
    }
    // 30 eventos de arquivos .toml
    for i in 0..30 {
        let mut e = EventModel::new(OperationType::Modificado, &format!("conf_{}.toml", i), "C:\\Config");
        e.size_kb = 5.0;
        eventos.push(e);
    }
    // 10 eventos de arquivos sem extensão
    for i in 0..10 {
        let mut e = EventModel::new(OperationType::Excluido, &format!("LICENSE_{}", i), "C:\\Root");
        e.size_kb = 2.0;
        eventos.push(e);
    }

    loc.set_idioma("pt_BR");
    let treemap_data = GeradorEstatisticas::gerar_treemap_tipos_loc(&eventos, &loc);
    assert_eq!(treemap_data.treemap_items.len(), 3);

    let rs_item = treemap_data.treemap_items.iter().find(|(lbl, _, _)| lbl == ".rs").expect(".rs deve existir");
    let toml_item = treemap_data.treemap_items.iter().find(|(lbl, _, _)| lbl == ".toml").expect(".toml deve existir");
    let no_ext_item = treemap_data.treemap_items.iter().find(|(lbl, _, _)| lbl == "(sem extensão)").expect("(sem extensão) deve existir");

    assert_eq!(rs_item.1, 60.0);
    assert_eq!(toml_item.1, 30.0);
    assert_eq!(no_ext_item.1, 10.0);

    let total: f64 = treemap_data.treemap_items.iter().map(|item| item.1).sum();
    assert_eq!(total, 100.0);

    // Testa com idioma inglês para verificar tradução dinâmica em tempo de execução
    loc.set_idioma("en_US");
    let treemap_en = GeradorEstatisticas::gerar_treemap_tipos_loc(&eventos, &loc);
    assert!(treemap_en.treemap_items.iter().any(|(lbl, _, _)| lbl == "(no extension)"));
}

#[test]
fn test_timeline_radar_and_dotplot_enhancements() {
    let loc = huginn_rs::locale::Localizer::new();
    loc.set_idioma("pt_BR");

    let mut eventos = Vec::new();
    for i in 0..40 {
        let mut e = EventModel::new(OperationType::Adicionado, &format!("documento_{}.pdf", i), "C:\\Docs");
        e.timestamp = format!("2026-10-29 10:{:02}:00", i);
        e.size_kb = (i as f64) * 25.0 + 10.0;
        eventos.push(e);
    }
    for i in 0..20 {
        let mut e = EventModel::new(OperationType::Modificado, &format!("foto_{}.png", i), "C:\\Imagens");
        e.timestamp = format!("2026-10-29 11:{:02}:00", i);
        e.size_kb = (i as f64) * 500.0 + 100.0;
        eventos.push(e);
    }
    for i in 0..10 {
        let mut e = EventModel::new(OperationType::Excluido, &format!("script_{}.rs", i), "C:\\Src");
        e.timestamp = format!("2026-10-29 12:{:02}:00", i);
        e.size_kb = (i as f64) * 5.0 + 2.0;
        eventos.push(e);
    }

    // 1. Linha do tempo: deve ter itens de linha do tempo e agregação por operação em pairs
    let timeline = GeradorEstatisticas::gerar_timeline_operacoes_loc(&eventos, &loc);
    assert_eq!(timeline.timeline_items.len(), 70);
    assert!(timeline.pairs.iter().any(|(op, count)| op.contains("Adicionado") && *count == 40.0));
    assert!(timeline.pairs.iter().any(|(op, count)| op.contains("Modificado") && *count == 20.0));
    assert!(timeline.pairs.iter().any(|(op, count)| op.contains("Excluído") && *count == 10.0));

    // 2. Radar de operações: deve conter 6 operações com valores proporcionais
    let radar = GeradorEstatisticas::gerar_radar_operacoes_loc(&eventos, &loc);
    assert_eq!(radar.labels.len(), 6);
    assert_eq!(radar.values.len(), 6);
    let total_v: f64 = radar.values.iter().sum();
    assert_eq!(total_v, 70.0);
    let pct_adicionado = (40.0 / total_v) * 100.0;
    assert!((pct_adicionado - 57.14).abs() < 0.1);

    // 3. Dotplot de distribuição de eventos por extensão e tamanho
    let dotplot = GeradorEstatisticas::gerar_dotplot_distribuicao_loc(&eventos, &loc);
    assert_eq!(dotplot.dotplot_groups.len(), 3);
    let pdf_group = dotplot.dotplot_groups.iter().find(|(ext, _)| ext == ".pdf").expect(".pdf deve existir");
    assert_eq!(pdf_group.1.len(), 40);
    let png_group = dotplot.dotplot_groups.iter().find(|(ext, _)| ext == ".png").expect(".png deve existir");
    assert_eq!(png_group.1.len(), 20);
    let rs_group = dotplot.dotplot_groups.iter().find(|(ext, _)| ext == ".rs").expect(".rs deve existir");
    assert_eq!(rs_group.1.len(), 10);

    // Testa no idioma inglês
    loc.set_idioma("en_US");
    let dotplot_en = GeradorEstatisticas::gerar_dotplot_distribuicao_loc(&eventos, &loc);
    assert_eq!(dotplot_en.dotplot_groups.len(), 3);
}

#[test]
fn test_barras_pareto_boxplot_all_extensions_without_truncation() {
    let loc = huginn_rs::locale::Localizer::new();
    loc.set_idioma("pt_BR");

    let extensoes = [
        "rs", "toml", "json", "md", "txt", "log", "png", "jpg", "pdf", "docx", "xlsx", "csv",
        "xml", "html", "css", "js", "py", "sql",
    ];
    let mut eventos = Vec::new();
    for (idx, ext) in extensoes.iter().enumerate() {
        let qtd = extensoes.len() - idx;
        for j in 0..qtd {
            let mut ev = EventModel::new(
                OperationType::Adicionado,
                &format!("arquivo_{}_{}.{}", idx, j, ext),
                "C:\\Monitorados",
            );
            ev.size_kb = ((idx + 1) * 10 + j) as f64;
            eventos.push(ev);
        }
    }

    // 1. Barras por Tipo de Arquivo: deve reportar TODAS as 18 extensões sem truncamento
    let barras = GeradorEstatisticas::gerar_barras_tipos_loc(&eventos, &loc);
    assert_eq!(barras.labels.len(), extensoes.len());
    assert_eq!(barras.values.len(), extensoes.len());
    assert_eq!(barras.labels[0], ".rs");
    assert_eq!(barras.values[0], 18.0);

    // 2. Gráfico de Pareto de Extensões: todas as 18 extensões ordenadas e curva acumulando até 100%
    let pareto = GeradorEstatisticas::gerar_pareto_extensoes_loc(&eventos, &loc);
    assert_eq!(pareto.labels.len(), extensoes.len());
    assert_eq!(pareto.secondary_values.len(), extensoes.len());
    assert!((*pareto.secondary_values.last().unwrap() - 100.0).abs() < 1e-6);

    // 3. Boxplot de Tamanhos de Arquivo: todas as 18 extensões com quantidade total e soma total
    let boxplot = GeradorEstatisticas::gerar_boxplot_tamanhos_loc(&eventos, &loc);
    assert_eq!(boxplot.boxplot_groups.len(), extensoes.len());
    assert_eq!(boxplot.boxplot_totais.len(), extensoes.len());
    assert_eq!(boxplot.boxplot_groups[0].0, ".rs");
    for (idx, (ext, qtd, soma_kb)) in boxplot.boxplot_totais.iter().enumerate() {
        assert_eq!(&boxplot.boxplot_groups[idx].0, ext);
        assert!(*qtd >= 1);
        assert!(*soma_kb > 0.0);
    }

    // 4. Filtro de limite de itens plotados (10, 20, ..., 100 e Todos)
    let barras_top10 = barras.com_limite_extensoes(Some(10));
    assert_eq!(barras_top10.labels.len(), 10);
    assert_eq!(barras_top10.values.len(), 10);
    assert_eq!(barras_top10.labels[0], ".rs");

    let pareto_top10 = pareto.com_limite_extensoes(Some(10));
    assert_eq!(pareto_top10.labels.len(), 10);
    assert_eq!(pareto_top10.secondary_values.len(), 10);
    assert!((*pareto_top10.secondary_values.last().unwrap() - 100.0).abs() < 1e-6);

    let boxplot_top10 = boxplot.com_limite_extensoes(Some(10));
    assert_eq!(boxplot_top10.boxplot_groups.len(), 10);
    assert_eq!(boxplot_top10.boxplot_totais.len(), 10);
    assert_eq!(boxplot_top10.boxplot_groups[0].0, ".rs");

    let boxplot_todos = boxplot.com_limite_extensoes(None);
    assert_eq!(boxplot_todos.boxplot_groups.len(), extensoes.len());
}

#[test]
fn test_tukey_boxplot_and_logarithmic_scales() {
    use huginn_rs::data::generator::{
        calcular_estatisticas_boxplot_tukey, calcular_log10_kb,
    };

    // Verifica cercas de Tukey (1.5 * IQR) contra outlier extremo (10000 KB)
    let amostra = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 10000.0];
    let (min_w, q1, med, q3, max_w) = calcular_estatisticas_boxplot_tukey(&amostra);
    assert!((q1 - 30.0).abs() < 1e-6);
    assert!((med - 50.0).abs() < 1e-6);
    assert!((q3 - 70.0).abs() < 1e-6);
    assert!((min_w - 10.0).abs() < 1e-6);
    // Cerca superior = 70 + 1.5 * (70 - 30) = 130 -> maior valor <= 130 é 80.0 (não 10000.0)
    assert!((max_w - 80.0).abs() < 1e-6);

    // Verifica cálculo logarítmico de base 10 para Dotplot (eixo X) e Dispersão (eixo Y)
    assert!((calcular_log10_kb(1.0) - 0.0).abs() < 1e-9);
    assert!((calcular_log10_kb(10.0) - 1.0).abs() < 1e-9);
    assert!((calcular_log10_kb(100.0) - 2.0).abs() < 1e-9);
    assert!((calcular_log10_kb(1000.0) - 3.0).abs() < 1e-9);
}

#[test]
fn test_radar_operations_separated_and_movido_reporting() {
    let loc = huginn_rs::locale::Localizer::new();
    loc.set_idioma("pt_BR");

    let mut eventos = Vec::new();
    eventos.push(EventModel::new(OperationType::Adicionado, "arq1.txt", "C:\\DirA"));
    eventos.push(EventModel::new(OperationType::Escaneado, "arq2.txt", "C:\\DirB"));
    eventos.push(EventModel::new(OperationType::Excluido, "arq3.txt", "C:\\DirC"));
    eventos.push(EventModel::new(OperationType::Modificado, "arq4.txt", "C:\\DirD"));
    let mut ev_mov = EventModel::new(OperationType::Movido, "arq5.txt", "C:\\DirE");
    ev_mov.dir_anterior = "C:\\DirAnt".to_string();
    eventos.push(ev_mov);
    let mut ev_renom = EventModel::new(OperationType::Renomeado, "arq6.txt", "C:\\DirF");
    ev_renom.dir_anterior = "C:\\DirF\\arq6_antigo.txt".to_string();
    assert_eq!(ev_renom.op_type(), OperationType::Renomeado);
    eventos.push(ev_renom);

    // 1. Radar Multidimensional de Risco reporta as 6 operações separadamente
    let radar = GeradorEstatisticas::gerar_radar_multidimensional_loc(&eventos, &loc);
    assert_eq!(radar.radar_dimensions.len(), 10);

    let dim_names: Vec<&str> = radar.radar_dimensions.iter().map(|(n, _)| n.as_str()).collect();
    assert!(dim_names.contains(&loc.t("op_added").as_str()));
    assert!(dim_names.contains(&loc.t("op_scanned").as_str()));
    assert!(dim_names.contains(&loc.t("op_deleted").as_str()));
    assert!(dim_names.contains(&loc.t("op_modified").as_str()));
    assert!(dim_names.contains(&loc.t("op_moved").as_str()));
    assert!(dim_names.contains(&loc.t("op_renamed").as_str()));

    let ren_dim = radar.radar_dimensions.iter().find(|(n, _)| n == &loc.t("op_renamed")).expect("Dimensão Renomeado deve existir");
    assert!(ren_dim.1 > 15.0 && ren_dim.1 < 18.0, "Renomeado deve reportar score proporcional positivo: {}", ren_dim.1);
    let mov_dim = radar.radar_dimensions.iter().find(|(n, _)| n == &loc.t("op_moved")).expect("Dimensão Movido deve existir");
    assert!(mov_dim.1 > 15.0 && mov_dim.1 < 18.0, "Movido deve reportar score proporcional positivo: {}", mov_dim.1);

    // 2. Timeline reporta a operação Movido
    let timeline = GeradorEstatisticas::gerar_timeline_operacoes_loc(&eventos, &loc);
    let movido_trad = loc.traduzir_tipo_operacao("movido");
    assert!(
        timeline.timeline_items.iter().any(|(_, op, _, _)| *op == movido_trad),
        "Timeline deve reportar itens da operação Movido"
    );
    assert!(
        timeline.pairs.iter().any(|(op, count)| *op == movido_trad && *count >= 1.0),
        "Timeline deve contabilizar a operação Movido em pairs"
    );

    // 3. Sankey: Operação -> Diretório reporta a operação Movido
    let sankey_dir = GeradorEstatisticas::gerar_sankey_evento_diretorio_loc(&eventos, &loc);
    assert!(
        sankey_dir.flows.iter().any(|(op, _, val)| *op == movido_trad && *val >= 1.0),
        "Sankey Operação -> Diretório deve reportar fluxos com a operação Movido"
    );

    // 4. Sankey: Operação -> Tipo de Arquivo reporta a operação Movido
    let sankey_tipo = GeradorEstatisticas::gerar_sankey_operacao_tipo_loc(&eventos, &loc);
    assert!(
        sankey_tipo.flows.iter().any(|(op, _, val)| *op == movido_trad && *val >= 1.0),
        "Sankey Operação -> Tipo de Arquivo deve reportar fluxos com a operação Movido"
    );

    // 5. Pizza de Operações reporta a operação Movido
    let pizza = GeradorEstatisticas::gerar_pizza_operacoes_loc(&eventos, &loc);
    assert!(
        pizza.labels.contains(&movido_trad),
        "Pizza de Operações deve reportar a operação Movido"
    );

    // 6. Teste de tradução completa de todos os alertas de segurança monitorados
    let alertas_teste = [
        ("rule_ransomware", "Extensão Suspeita de Ransomware"),
        ("rule_double_ext", "Extensão Dupla Dissimulada"),
        ("rule_binary_temp", "Binário em Diretório Temporário"),
        ("rule_script_temp", "Script em Diretório de Sistema/Temp"),
        ("rule_hidden_executable", "Executável com Atributo Oculto"),
        ("rule_baseline_divergence", "Divergência de Baseline Criptográfico"),
        ("security_baseline_unchanged", "Inalterado"),
        ("security_baseline_changed", "Alterado"),
        ("security_baseline_new", "Novo"),
        ("security_baseline_missing", "Ausente"),
        ("security_baseline_directory", "Diretório"),
    ];

    for (chave, esperado_pt) in alertas_teste {
        let trad = loc.traduzir_alerta(chave);
        assert_eq!(
            trad, esperado_pt,
            "Alerta '{}' deve ser traduzido como '{}'",
            chave, esperado_pt
        );
    }
}

#[test]
fn test_sankey_limits_and_renamed_operation_preserved() {
    let loc = Localizer::new();
    let mut eventos = Vec::new();

    // Cria 25 extensões distintas distribuídas em várias operações
    for i in 0..25 {
        let ext = format!("ext{:02}", i);
        let nome = format!("arquivo_{}.{}", i, ext);
        let op = match i % 6 {
            0 => OperationType::Adicionado,
            1 => OperationType::Modificado,
            2 => OperationType::Excluido,
            3 => OperationType::Renomeado,
            4 => OperationType::Movido,
            _ => OperationType::Escaneado,
        };
        let mut ev = EventModel::new(op, &nome, "C:\\Dados");
        if op == OperationType::Renomeado {
            ev.dir_anterior = format!("C:\\Dados\\antigo_{}.{}", i, ext);
            assert_eq!(ev.op_type(), OperationType::Renomeado, "Renomeado não deve ser reportado como Movido");
        } else if op == OperationType::Movido {
            ev.dir_anterior = "C:\\OutroDir".to_string();
            assert_eq!(ev.op_type(), OperationType::Movido);
        }
        eventos.push(ev);
    }

    // 1. Sankey: Operação -> Tipo de Arquivo
    let sankey_tipo_bruto = GeradorEstatisticas::gerar_sankey_operacao_tipo_loc(&eventos, &loc);
    let total_dest = sankey_tipo_bruto.total_destinos_sankey();
    assert_eq!(total_dest, 25, "Deve conter 25 destinos de extensões");

    // Limite 10
    let s_10 = sankey_tipo_bruto.com_limite_sankey(Some(10));
    assert_eq!(s_10.total_destinos_sankey(), 10, "Limite 10 deve manter exatamente 10 destinos");

    // Limite 20
    let s_20 = sankey_tipo_bruto.com_limite_sankey(Some(20));
    assert_eq!(s_20.total_destinos_sankey(), 20, "Limite 20 deve manter exatamente 20 destinos");

    // Limite Todos (None)
    let s_todos = sankey_tipo_bruto.com_limite_sankey(None);
    assert_eq!(s_todos.total_destinos_sankey(), 25, "Limite Todos deve preservar todos os destinos");

    // Teste de Limites Duplos (Origem e Destino independentes)
    let total_orig = sankey_tipo_bruto.total_origens_sankey();
    assert_eq!(total_orig, 6, "Deve conter as 6 operações distintas como origens");
    assert_eq!(sankey_tipo_bruto.max_nos_sankey(), 25, "Maior contagem entre nós de origem e destino deve ser 25");

    // Limita origens a 2 e destinos a 5
    let s_duplo = sankey_tipo_bruto.com_limite_sankey_duplo(Some(2), Some(5));
    assert!(s_duplo.total_origens_sankey() <= 2, "Origens devem ser no máximo 2");
    assert!(s_duplo.total_destinos_sankey() <= 5, "Destinos devem ser no máximo 5");
    assert!(!s_duplo.flows.is_empty(), "Fluxos filtrados não devem ser vazios");

    // 2. Sankey: Operação -> Diretório
    let sankey_dir_bruto = GeradorEstatisticas::gerar_sankey_evento_diretorio_loc(&eventos, &loc);
    assert!(sankey_dir_bruto.total_destinos_sankey() >= 1);
    assert_eq!(sankey_dir_bruto.total_origens_sankey(), 6);

    // 3. Sankey: Tipo -> Diretório
    let sankey_tipo_dir = GeradorEstatisticas::gerar_sankey_tipo_diretorio_loc(&eventos, &loc);
    assert_eq!(sankey_tipo_dir.total_origens_sankey(), 25, "Sankey Tipo -> Diretório tem extensões como origem");
    assert!(sankey_tipo_dir.total_destinos_sankey() >= 1, "Sankey Tipo -> Diretório tem diretórios como destino");
    let s_td_duplo = sankey_tipo_dir.com_limite_sankey_duplo(Some(10), Some(1));
    assert!(s_td_duplo.total_origens_sankey() <= 10);
    assert!(s_td_duplo.total_destinos_sankey() <= 1);
}

#[test]
fn test_sankey_operation_checkbox_filtering_and_determinism() {
    let loc = Localizer::new();
    let mut eventos = Vec::new();

    for i in 0..60 {
        let op = match i % 6 {
            0 => OperationType::Adicionado,
            1 => OperationType::Modificado,
            2 => OperationType::Excluido,
            3 => OperationType::Renomeado,
            4 => OperationType::Movido,
            _ => OperationType::Escaneado,
        };
        let ext = format!("ext{:02}", i % 15);
        let nome = format!("arquivo_{}.{}", i, ext);
        let dir = format!("C:\\Pasta_{:02}", i % 5);
        let mut ev = EventModel::new(op, &nome, &dir);
        if op == OperationType::Renomeado {
            ev.dir_anterior = format!("C:\\Pasta_{:02}\\antigo_{}.{}", i % 5, i, ext);
        } else if op == OperationType::Movido {
            ev.dir_anterior = "C:\\OutraPasta".to_string();
        }
        eventos.push(ev);
    }

    // 1. Sankey: Operação -> Tipo de Arquivo
    let sankey_op_tipo = GeradorEstatisticas::gerar_sankey_operacao_tipo_loc(&eventos, &loc);
    assert_eq!(sankey_op_tipo.total_origens_sankey(), 6);

    // Filtro por única operação selecionada: Renomeado
    let sel_renom = [OperationType::Renomeado];
    let f_renom = sankey_op_tipo.com_filtro_operacoes_e_limite_destino(&sel_renom, None, &loc);
    assert_eq!(f_renom.total_origens_sankey(), 1);
    let op_renom_str = loc.traduzir_tipo_operacao(OperationType::Renomeado.as_str());
    for (o, _, _) in &f_renom.flows {
        assert_eq!(o, &op_renom_str, "Todos os fluxos devem ter origem '{}'", op_renom_str);
    }

    // Filtro por múltiplas operações selecionadas: Renomeado e Movido
    let sel_dupla = [OperationType::Renomeado, OperationType::Movido];
    let f_dupla = sankey_op_tipo.com_filtro_operacoes_e_limite_destino(&sel_dupla, None, &loc);
    assert_eq!(f_dupla.total_origens_sankey(), 2);
    let op_mov_str = loc.traduzir_tipo_operacao(OperationType::Movido.as_str());
    for (o, _, _) in &f_dupla.flows {
        assert!(o == &op_renom_str || o == &op_mov_str);
    }

    // Filtro com limite de destinos
    let f_com_dest = sankey_op_tipo.com_filtro_operacoes_e_limite_destino(&sel_dupla, Some(3), &loc);
    assert!(f_com_dest.total_destinos_sankey() <= 3);

    // Filtro vazio (nenhuma checkbox marcada)
    let f_vazio = sankey_op_tipo.com_filtro_operacoes_e_limite_destino(&[], None, &loc);
    assert_eq!(f_vazio.total_origens_sankey(), 0);
    assert!(f_vazio.flows.is_empty());

    // Determinismo absoluto da ordenação dos fluxos (previne tremor/oscilação entre frames)
    let f_iter1 = sankey_op_tipo.com_filtro_operacoes_e_limite_destino(OperationType::all(), Some(10), &loc);
    let f_iter2 = sankey_op_tipo.com_filtro_operacoes_e_limite_destino(OperationType::all(), Some(10), &loc);
    assert_eq!(f_iter1.flows, f_iter2.flows, "Ordenação dos fluxos deve ser estritamente determinística");

    // 2. Sankey: Operação -> Diretório
    let sankey_op_dir = GeradorEstatisticas::gerar_sankey_evento_diretorio_loc(&eventos, &loc);
    let f_dir_mov = sankey_op_dir.com_filtro_operacoes_e_limite_destino(&[OperationType::Movido], Some(2), &loc);
    assert_eq!(f_dir_mov.total_origens_sankey(), 1);
    assert!(f_dir_mov.total_destinos_sankey() <= 2);
}

#[test]
fn test_sankey_operation_localization_keys_all_languages() {
    let loc = Localizer::new();
    let idiomas = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];

    for idm in idiomas {
        loc.set_idioma(idm);

        let op_filter = loc.t("chart_sankey_operation_filter");
        assert!(!op_filter.is_empty() && !op_filter.contains("missing"), "Falta chart_sankey_operation_filter em {}", idm);

        let all_ops = loc.t("chart_limit_all_operations");
        assert!(!all_ops.is_empty() && !all_ops.contains("missing"), "Falta chart_limit_all_operations em {}", idm);

        let sel_count = loc.t_args("chart_operations_selected_count", &["3"]);
        assert!(sel_count.contains('3'), "chart_operations_selected_count em {} deve interpolar '%1', obteve: {}", idm, sel_count);

        let none_sel = loc.t("chart_none_selected");
        assert!(!none_sel.is_empty() && !none_sel.contains("missing"), "Falta chart_none_selected em {}", idm);
    }
}

#[test]
fn test_dynamic_localization_and_theme_adaptation() {
    use huginn_rs::data::render::raster::ChartTheme;

    let loc = Localizer::new();
    let idiomas = ["pt_BR", "en_US", "es_ES", "fr_FR", "it_IT", "de_DE"];
    let chaves = [
        "chart_axis_x",
        "chart_axis_y",
        "chart_sankey_legend_origin",
        "chart_sankey_legend_flow",
        "chart_sankey_legend_destination",
        "stat_total",
        "stat_median_short",
    ];

    for idm in idiomas {
        loc.set_idioma(idm);
        for chave in chaves {
            let trad = loc.t(chave);
            assert!(
                !trad.is_empty() && !trad.contains("missing"),
                "Chave '{}' deve estar traduzida em {}",
                chave,
                idm
            );
        }
    }

    // Valida adaptação dinâmica de idiomas em tempo de execução
    let mut eventos = Vec::new();
    let mut e1 = EventModel::new(OperationType::Adicionado, "app.rs", "C:\\Projetos");
    e1.size_kb = 45.0;
    eventos.push(e1);

    loc.set_idioma("pt_BR");
    let heatmap_pt = GeradorEstatisticas::gerar_heatmap_dia_hora_loc(&eventos, &loc);
    assert!(heatmap_pt.labels[0].starts_with("Seg"));

    loc.set_idioma("en_US");
    let heatmap_en = GeradorEstatisticas::gerar_heatmap_dia_hora_loc(&eventos, &loc);
    assert!(heatmap_en.labels[0].starts_with("Mon"));

    // Valida adaptação ao tema Windows 11 (Dark Mode vs Light Mode)
    let theme_dark = ChartTheme::new(true);
    let theme_light = ChartTheme::new(false);

    assert!(theme_dark.dark_mode);
    assert!(!theme_light.dark_mode);
    assert_ne!(theme_dark.bg_card, theme_light.bg_card);
    assert_ne!(theme_dark.border_card, theme_light.border_card);
    assert_ne!(theme_dark.text_primary, theme_light.text_primary);
    assert_ne!(theme_dark.text_secondary, theme_light.text_secondary);
    assert_ne!(theme_dark.grid_line, theme_light.grid_line);
    assert_ne!(theme_dark.panel_bg, theme_light.panel_bg);
}



