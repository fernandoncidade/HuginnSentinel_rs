use std::fs::File;
use std::io::Write;

use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::data::charts::TipoGrafico;
use huginn_rs::data::generator::GeradorEstatisticas;
use huginn_rs::data::render::exportar_grafico_png;
use huginn_rs::locale::Localizer;
use huginn_rs::security::SecurityManager;
use huginn_rs::services::metadados::{extrair_metadados_completos, extrair_midia};
use huginn_rs::ui::dialogs::about::{AboutDialog, AboutTab};

fn criar_tempdir_teste() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "linceu_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&p);
    p
}

#[test]
fn test_all_6_operations_metadata_completeness() {
    let tmp = criar_tempdir_teste();
    let sec_mgr = SecurityManager::new(Some(tmp.join("test_sec.db")));

    let file_path = tmp.join("documento_teste.rs");
    {
        let mut f = File::create(&file_path).expect("Criar arquivo");
        writeln!(f, "// Arquivo de código fonte Rust").unwrap();
        writeln!(f, "fn main() {{").unwrap();
        writeln!(f, "    println!(\"Huginn\");").unwrap();
        writeln!(f, "}}").unwrap();
    }

    let ops = [
        OperationType::Adicionado,
        OperationType::Escaneado,
        OperationType::Movido,
        OperationType::Modificado,
        OperationType::Excluido,
        OperationType::Renomeado,
    ];

    for op in ops {
        let mut model = EventModel::new(
            op,
            "documento_teste.rs".to_string(),
            tmp.to_string_lossy().to_string(),
        );

        if op == OperationType::Movido || op == OperationType::Renomeado {
            model.dir_anterior = "C:\\antigo\\dir".to_string();
        }

        extrair_metadados_completos(&file_path, &mut model);
        sec_mgr.enriquecer_evento(&mut model);

        assert!(
            !model.tipo_operacao.is_empty(),
            "Operação não pode ser vazia para {:?}",
            op
        );
        assert!(!model.nome.is_empty(), "Nome não pode ser vazio");
        assert!(!model.dir_atual.is_empty(), "Dir atual não pode ser vazio");
        assert_eq!(model.tipo, ".rs");
        assert_eq!(model.mime, "text/x-rust");
        assert!(model.size_b > 0.0, "Tamanho deve ser > 0");
        assert!(model.size_kb > 0.0);
        assert!(
            !model.data_criacao.is_empty(),
            "Data de criação deve existir"
        );
        assert!(
            !model.data_modificacao.is_empty(),
            "Data de modificação deve existir"
        );
        assert!(!model.data_acesso.is_empty(), "Data de acesso deve existir");
        assert!(!model.autor.is_empty(), "Autor deve ser atribuído");
        assert!(!model.protegido.is_empty(), "Status protegido deve existir");
        assert_eq!(model.linhas, "4");
        assert_eq!(model.linhas_codigo, "3");
        assert_eq!(model.total_linhas, "4");
        assert!(!model.sha256_atual.is_empty(), "SHA-256 deve ser calculado");
        assert!(
            !model.actor_user.is_empty(),
            "Actor user deve ser atribuído"
        );
        assert!(
            !model.risk_level.is_empty(),
            "Nível de risco deve ser avaliado"
        );
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_deleted_file_metadata_fallback() {
    let tmp = criar_tempdir_teste();
    let sec_mgr = SecurityManager::new(Some(tmp.join("test_sec_del.db")));

    let file_path = tmp.join("arquivo_excluido.py");
    // O arquivo propositalmente NÃO existe no disco!
    assert!(!file_path.exists());

    let mut model = EventModel::new(
        OperationType::Excluido,
        "arquivo_excluido.py".to_string(),
        tmp.to_string_lossy().to_string(),
    );

    extrair_metadados_completos(&file_path, &mut model);
    sec_mgr.enriquecer_evento(&mut model);

    assert_eq!(model.tipo_operacao, "Excluído");
    assert_eq!(model.tipo, ".py");
    assert_eq!(model.mime, "text/x-python");
    assert!(
        !model.autor.is_empty(),
        "Autor deve ter fallback para usuário do sistema"
    );
    assert_eq!(model.protegido, "Não");
    assert!(
        !model.data_modificacao.is_empty(),
        "Data de modificação deve ser preenchida"
    );
    assert!(
        !model.actor_user.is_empty(),
        "Actor user deve ser atribuído"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_media_metadata_extraction_wav() {
    let tmp = criar_tempdir_teste();
    let wav_path = tmp.join("som_teste.wav");

    // Gera cabeçalho WAV canônico de 44 bytes
    let mut f = File::create(&wav_path).expect("Criar wav");
    let mut header = [0u8; 44];
    header[0..4].copy_from_slice(b"RIFF");
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16u32.to_le_bytes()); // subchunk size
    header[20..22].copy_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
    header[22..24].copy_from_slice(&2u16.to_le_bytes()); // num channels (stereo)
    header[24..28].copy_from_slice(&44100u32.to_le_bytes()); // sample rate
    let byte_rate = 44100u32 * 2 * 2; // 176400 bytes/sec
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&4u16.to_le_bytes()); // block align
    header[34..36].copy_from_slice(&16u16.to_le_bytes()); // bits per sample
    header[36..40].copy_from_slice(b"data");
    let data_len = byte_rate * 5; // 5 segundos de áudio
    header[40..44].copy_from_slice(&data_len.to_le_bytes());
    f.write_all(&header).unwrap();
    // Escreve os 5 segundos de dados
    f.write_all(&vec![0u8; data_len as usize]).unwrap();

    let meta = extrair_midia(&wav_path, 44 + data_len as u64);
    assert!(meta.is_some(), "Deveria extrair metadados WAV");
    let m = meta.unwrap();
    assert_eq!(m.duracao, "00:00:05");
    assert_eq!(m.taxa_bits, "1411 kbps");

    let mut event = EventModel::new(
        OperationType::Adicionado,
        "som_teste.wav".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&wav_path, &mut event);
    assert_eq!(event.duracao, "00:00:05");
    assert_eq!(event.taxa_bits, "1411 kbps");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_all_17_charts_data_generation_and_png_export() {
    let loc = Localizer::new();
    let tmp = criar_tempdir_teste();

    let mut eventos = Vec::new();
    let tipos_ops = [
        "Adicionado",
        "Modificado",
        "Excluído",
        "Renomeado",
        "Movido",
        "Escaneado",
    ];
    let tipos_arqs = [
        "Código Fonte (Rust)",
        "Imagem PNG",
        "Documento PDF",
        "Executável Windows",
        "Script Python",
    ];

    for i in 0..60 {
        let op = tipos_ops[i % tipos_ops.len()];
        let tp = tipos_arqs[i % tipos_arqs.len()];
        let h = i % 24;
        let ts = format!("2026-10-27 {:02}:15:30", h);
        let mut ev = EventModel::new(
            OperationType::from_raw(op),
            format!("arquivo_{}.ext", i),
            format!("C:\\dados\\pasta_{}", i % 4),
        );
        ev.tipo = tp.to_string();
        ev.timestamp = ts;
        ev.size_kb = ((i + 1) * 150) as f64;
        ev.size_b = ev.size_kb * 1024.0;
        ev.size_mb = ev.size_kb / 1024.0;
        ev.risk_score = if i % 5 == 0 { 60 } else { 10 };
        ev.actor_user = "Analista".to_string();
        eventos.push(ev);
    }

    for tipo_grafico in TipoGrafico::all() {
        let chart_data = match tipo_grafico {
            TipoGrafico::PizzaOperacoes => {
                GeradorEstatisticas::gerar_pizza_operacoes_loc(&eventos, &loc)
            }
            TipoGrafico::BarrasTipos => GeradorEstatisticas::gerar_barras_tipos_loc(&eventos, &loc),
            TipoGrafico::TimelineOperacoes => {
                GeradorEstatisticas::gerar_timeline_operacoes_loc(&eventos, &loc)
            }
            TipoGrafico::TreemapTipos => {
                GeradorEstatisticas::gerar_treemap_tipos_loc(&eventos, &loc)
            }
            TipoGrafico::HistogramaHorarios => {
                GeradorEstatisticas::gerar_histograma_horarios(&eventos)
            }
            TipoGrafico::ParetoExtensoes => GeradorEstatisticas::gerar_pareto_extensoes(&eventos),
            TipoGrafico::LinhaEventosHora => {
                GeradorEstatisticas::gerar_linha_eventos_hora_loc(&eventos, &loc)
            }
            TipoGrafico::BoxplotTamanhos => GeradorEstatisticas::gerar_boxplot_tamanhos(&eventos),
            TipoGrafico::RadarOperacoes => {
                GeradorEstatisticas::gerar_radar_operacoes_loc(&eventos, &loc)
            }
            TipoGrafico::HeatmapDiaHora => GeradorEstatisticas::gerar_heatmap_dia_hora(&eventos),
            TipoGrafico::ScatterTamanhoHora => {
                GeradorEstatisticas::gerar_scatter_tamanho_hora(&eventos)
            }
            TipoGrafico::SankeyOperacaoTipo => {
                GeradorEstatisticas::gerar_sankey_operacao_tipo_loc(&eventos, &loc)
            }
            TipoGrafico::RadarMultidimensional => {
                GeradorEstatisticas::gerar_radar_multidimensional_loc(&eventos, &loc)
            }
            TipoGrafico::DotplotDistribuicao => {
                GeradorEstatisticas::gerar_dotplot_distribuicao_loc(&eventos, &loc)
            }
            TipoGrafico::SankeyEventoDiretorio => {
                GeradorEstatisticas::gerar_sankey_evento_diretorio_loc(&eventos, &loc)
            }
            TipoGrafico::SankeyTipoDiretorio => {
                GeradorEstatisticas::gerar_sankey_tipo_diretorio_loc(&eventos, &loc)
            }
            TipoGrafico::ArvoreDiretorios => {
                GeradorEstatisticas::gerar_arvore_diretorios_loc(&eventos, &loc)
            }
        };

        let out_png = tmp.join(format!("grafico_{:?}.png", tipo_grafico));
        let res = exportar_grafico_png(*tipo_grafico, &chart_data, &out_png, 800, 500);
        assert!(res.is_ok(), "Falha ao exportar PNG para {:?}", tipo_grafico);
        assert!(
            out_png.exists(),
            "Arquivo PNG não foi gerado no disco para {:?}",
            tipo_grafico
        );
        let metadata = std::fs::metadata(&out_png).unwrap();
        assert!(
            metadata.len() > 1000,
            "Arquivo PNG gerado deve conter imagem renderizada (> 1KB)"
        );
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_about_dialog_all_8_asset_directories() {
    let idiomas = ["pt_BR", "en_US", "es_ES", "de_DE", "fr_FR", "it_IT"];

    for lang in idiomas {
        // 1. RELEASE
        let release = AboutDialog::obter_texto_aba(AboutTab::Release, lang);
        assert!(
            !release.is_empty(),
            "RELEASE não deve ser vazio para {}",
            lang
        );

        // 2. ABOUT
        let about = AboutDialog::obter_texto_aba(AboutTab::Sobre, lang);
        assert!(!about.is_empty(), "ABOUT não deve ser vazio para {}", lang);
        let history = AboutDialog::obter_texto_aba(AboutTab::Historico, lang);
        assert!(
            !history.is_empty(),
            "History_APP não deve ser vazio para {}",
            lang
        );

        // 3. CLC
        let clc = AboutDialog::obter_texto_aba(AboutTab::Clc, lang);
        assert!(!clc.is_empty(), "CLC não deve ser vazio para {}", lang);

        // 4. COPYRIGHT
        let copyright = AboutDialog::obter_texto_aba(AboutTab::Copyright, lang);
        assert!(
            !copyright.is_empty(),
            "COPYRIGHT não deve ser vazio para {}",
            lang
        );

        // 5. EULA
        let eula = AboutDialog::obter_texto_aba(AboutTab::Eula, lang);
        assert!(!eula.is_empty(), "EULA não deve ser vazio para {}", lang);

        // 6. NOTICES
        let notices = AboutDialog::obter_texto_aba(AboutTab::Avisos, lang);
        assert!(
            !notices.is_empty(),
            "NOTICES não deve ser vazio para {}",
            lang
        );

        // 7. PRIVACY_POLICY
        let privacy = AboutDialog::obter_texto_aba(AboutTab::Privacidade, lang);
        assert!(
            !privacy.is_empty(),
            "PRIVACY_POLICY não deve ser vazio para {}",
            lang
        );

        // 8. LICENSES
        let licenses = AboutDialog::obter_texto_aba(AboutTab::Licencas, lang);
        assert!(
            !licenses.is_empty(),
            "LICENSES não deve ser vazio para {}",
            lang
        );
        assert!(
            licenses.to_lowercase().contains("mit")
                || licenses.contains("Licença")
                || licenses.contains("License"),
            "LICENSES deve conter termos de licenciamento"
        );
    }
}

#[test]
fn test_language_detection_and_persistence() {
    let os_lang = Localizer::detectar_idioma_sistema();
    let valid_langs = ["pt_BR", "en_US", "es_ES", "de_DE", "fr_FR", "it_IT"];
    assert!(
        valid_langs.contains(&os_lang.as_str()),
        "Idioma do sistema detectado '{}' deve ser suportado",
        os_lang
    );

    let loc = Localizer::new();
    loc.set_idioma("fr_FR");
    assert_eq!(loc.idioma_atual(), "fr_FR");

    loc.set_idioma("pt_BR");
    assert_eq!(loc.idioma_atual(), "pt_BR");
}

#[test]
fn test_all_40_metadata_and_security_columns_across_all_types() {
    let tmp = criar_tempdir_teste();
    let sec_mgr = SecurityManager::new(Some(tmp.join("test_all_40_sec.db")));
    let loc = Localizer::new();
    loc.set_idioma("pt_BR");

    // 1. Código-fonte e texto (.rs): lines, words, pages_estimated, lines_code, total_lines, slides_estimated
    let code_path = tmp.join("modulo.rs");
    {
        let mut f = File::create(&code_path).unwrap();
        writeln!(f, "// Comentário de cabeçalho").unwrap();
        writeln!(f, "pub fn calcular(x: i32) -> i32 {{").unwrap();
        writeln!(f, "    x * 2").unwrap();
        writeln!(f, "}}").unwrap();
    }
    let mut ev_code = EventModel::new(
        OperationType::Escaneado,
        "modulo.rs".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&code_path, &mut ev_code);
    sec_mgr.enriquecer_evento_scan(&mut ev_code);

    assert_eq!(ev_code.tipo, ".rs");
    assert_eq!(loc.traduzir_tipo_arquivo(&ev_code.tipo), ".rs");
    assert!(!ev_code.autor.is_empty());
    assert!(!ev_code.protegido.is_empty());
    assert_eq!(ev_code.linhas, "4");
    assert!(!ev_code.palavras.is_empty() && ev_code.palavras != "0");
    assert_eq!(ev_code.paginas_estimadas, "1");
    assert_eq!(ev_code.linhas_codigo, "3");
    assert_eq!(ev_code.total_linhas, "4");
    assert!(!ev_code.monitor_profile_name.is_empty());
    assert!(!ev_code.baseline_status.is_empty());
    assert!(!ev_code.alerta_resumo.is_empty());
    assert!(!ev_code.session_label.is_empty());
    assert!(!ev_code.diff_resumo.is_empty());
    assert!(!ev_code.risk_level.is_empty());
    assert!(!ev_code.actor_user.is_empty());
    assert!(!ev_code.actor_process.is_empty());
    assert!(!ev_code.publisher.is_empty());
    assert!(!ev_code.quarantine_status.is_empty());
    assert!(!ev_code.evidence_id.is_empty());

    // 2. Documento PDF (.pdf): pages, author, protected
    let pdf_path = tmp.join("relatorio.pdf");
    {
        let mut f = File::create(&pdf_path).unwrap();
        f.write_all(
            b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
              2 0 obj << /Type /Pages /Count 7 >> endobj\n\
              3 0 obj << /Author (Engenheiro Linceu) >> endobj\n\
              4 0 obj << /Encrypt 5 0 R >> endobj\n%%EOF",
        )
        .unwrap();
    }
    let mut ev_pdf = EventModel::new(
        OperationType::Adicionado,
        "relatorio.pdf".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&pdf_path, &mut ev_pdf);
    assert_eq!(ev_pdf.tipo, ".pdf");
    assert_eq!(ev_pdf.paginas, "7");
    assert_eq!(ev_pdf.autor, "Engenheiro Linceu");
    assert_eq!(ev_pdf.protegido, "Sim");

    // 3. Imagem (.svg / .png): dimensions, author
    let svg_path = tmp.join("diagrama.svg");
    {
        let mut f = File::create(&svg_path).unwrap();
        f.write_all(
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1920\" height=\"1080\">\
              <metadata><dc:creator>Designer Linceu</dc:creator></metadata></svg>",
        )
        .unwrap();
    }
    let mut ev_svg = EventModel::new(
        OperationType::Adicionado,
        "diagrama.svg".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&svg_path, &mut ev_svg);
    assert_eq!(ev_svg.tipo, ".svg");
    assert_eq!(ev_svg.dimensoes, "1920x1080");
    assert_eq!(ev_svg.autor, "Designer Linceu");

    // 4. Planilha (.csv): spreadsheets, columns, records
    let csv_path = tmp.join("dados.csv");
    {
        let mut f = File::create(&csv_path).unwrap();
        writeln!(f, "id,nome,valor,ativo").unwrap();
        writeln!(f, "1,Item A,10.5,true").unwrap();
        writeln!(f, "2,Item B,20.0,false").unwrap();
        writeln!(f, "3,Item C,30.2,true").unwrap();
    }
    let mut ev_csv = EventModel::new(
        OperationType::Adicionado,
        "dados.csv".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&csv_path, &mut ev_csv);
    assert_eq!(ev_csv.tipo, ".csv");
    assert_eq!(ev_csv.planilhas, "1");
    assert_eq!(ev_csv.colunas, "4");
    assert_eq!(ev_csv.registros, "3");

    // 5. Apresentação (.pptx): slides
    let pptx_path = tmp.join("apresentacao.pptx");
    {
        let f = File::create(&pptx_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(f);
        let opts = zip::write::SimpleFileOptions::default();
        zip_writer
            .start_file("ppt/slides/slide1.xml", opts)
            .unwrap();
        zip_writer.write_all(b"<p:sld/>").unwrap();
        zip_writer
            .start_file("ppt/slides/slide2.xml", opts)
            .unwrap();
        zip_writer.write_all(b"<p:sld/>").unwrap();
        zip_writer.finish().unwrap();
    }
    let mut ev_pptx = EventModel::new(
        OperationType::Adicionado,
        "apresentacao.pptx".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&pptx_path, &mut ev_pptx);
    assert_eq!(ev_pptx.tipo, ".pptx");
    assert_eq!(ev_pptx.slides, "2");
    assert_eq!(ev_pptx.slides_estimados, "2");

    // 6. Banco de dados (.sqlite): tables, columns, records
    let db_path = tmp.join("sistema.sqlite");
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE clientes (id INTEGER PRIMARY KEY, nome TEXT, email TEXT)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO clientes (nome, email) VALUES ('Ana', 'ana@test.com'), ('Bruno', 'bruno@test.com')",
            [],
        )
        .unwrap();
    }
    let mut ev_db = EventModel::new(
        OperationType::Adicionado,
        "sistema.sqlite".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&db_path, &mut ev_db);
    assert_eq!(ev_db.tipo, ".sqlite");
    assert_eq!(ev_db.tabelas, "1");
    assert_eq!(ev_db.colunas, "3");
    assert_eq!(ev_db.registros, "2");

    // 7. Arquivo compactado (.zip): files, unzipped_b..tb
    let zip_path = tmp.join("pacote.zip");
    {
        let f = File::create(&zip_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(f);
        let opts = zip::write::SimpleFileOptions::default();
        zip_writer.start_file("interno1.txt", opts).unwrap();
        zip_writer.write_all(&vec![b'A'; 2048]).unwrap();
        zip_writer.start_file("interno2.txt", opts).unwrap();
        zip_writer.write_all(&vec![b'B'; 1024]).unwrap();
        zip_writer.finish().unwrap();
    }
    let mut ev_zip = EventModel::new(
        OperationType::Adicionado,
        "pacote.zip".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&zip_path, &mut ev_zip);
    assert_eq!(ev_zip.tipo, ".zip");
    assert_eq!(ev_zip.arquivos, "2");
    assert_eq!(ev_zip.unzipped_b, 3072.0);
    assert!(ev_zip.unzipped_kb > 0.0);
    assert!(!ev_zip.get_field("unzipped_b").is_empty());
    assert!(!ev_zip.get_field("unzipped_kb").is_empty());

    // 8. Executável PE (.exe): binary_file_b..tb, publisher, security columns
    let exe_path = tmp.join("aplicativo.exe");
    {
        let mut f = File::create(&exe_path).unwrap();
        let mut pe_buf = vec![0u8; 4096];
        pe_buf[0..2].copy_from_slice(b"MZ");
        pe_buf[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        pe_buf[0x80..0x84].copy_from_slice(b"PE\0\0");
        pe_buf[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes()); // x86_64
        pe_buf[0x86..0x88].copy_from_slice(&1u16.to_le_bytes()); // 1 section
        pe_buf[0x94..0x96].copy_from_slice(&0xF0u16.to_le_bytes()); // SizeOfOptionalHeader
        pe_buf[0x96..0x98].copy_from_slice(&0x0022u16.to_le_bytes()); // Executable
        pe_buf[0x98..0x9A].copy_from_slice(&0x020Bu16.to_le_bytes()); // PE32+
        f.write_all(&pe_buf).unwrap();
    }
    let mut ev_exe = EventModel::new(
        OperationType::Adicionado,
        "aplicativo.exe".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&exe_path, &mut ev_exe);
    sec_mgr.enriquecer_evento(&mut ev_exe);
    assert_eq!(ev_exe.tipo, ".exe");
    assert_eq!(ev_exe.binary_file_b, 4096.0);
    assert!(ev_exe.binary_file_kb > 0.0);
    assert!(!ev_exe.get_field("binary_file_b").is_empty());
    assert!(!ev_exe.publisher.is_empty());
    assert!(!ev_exe.monitor_profile_name.is_empty());
    assert!(!ev_exe.baseline_status.is_empty());
    assert!(!ev_exe.alerta_resumo.is_empty());
    assert!(!ev_exe.session_label.is_empty());
    assert!(!ev_exe.diff_resumo.is_empty());
    assert!(ev_exe.risk_score > 0);
    assert!(!ev_exe.risk_level.is_empty());
    assert!(!ev_exe.actor_user.is_empty());
    assert!(!ev_exe.actor_process.is_empty());
    assert!(!ev_exe.quarantine_status.is_empty());
    assert!(!ev_exe.evidence_id.is_empty());

    // 9. Diretório ("folder"): tipo = "folder" (traduzido como "Pasta"), files count & size
    let mut ev_dir = EventModel::new(
        OperationType::Escaneado,
        tmp.file_name().unwrap().to_string_lossy().to_string(),
        tmp.parent().unwrap().to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&tmp, &mut ev_dir);
    assert_eq!(ev_dir.tipo, "folder");
    assert_eq!(loc.traduzir_tipo_arquivo(&ev_dir.tipo), "Pasta");
    assert!(!ev_dir.arquivos.is_empty() && ev_dir.arquivos != "0");
    assert!(ev_dir.size_b > 0.0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_gui_table_56_columns_and_all_6_operations_lifecycle() {
    use huginn_rs::services::base_evento::BaseEvento;
    use huginn_rs::services::gerenciador_colunas::GerenciadorColunas;
    use huginn_rs::ui::table_view::formatar_celula_evento;

    let tmp = criar_tempdir_teste();
    let sec_mgr = SecurityManager::new(Some(tmp.join("test_56_sec.db")));
    let base = BaseEvento::new(Some(tmp.join("test_56_events.db")));
    let loc = Localizer::new();
    loc.set_idioma("pt_BR");

    let gc = GerenciadorColunas::new();
    assert_eq!(
        gc.colunas().len(),
        57,
        "A tabela GUI deve gerenciar todas as 57 colunas (56 especificadas + mime)"
    );

    let file_path = tmp.join("relatorio_auditoria.txt");
    {
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "Linha 1 de auditoria técnica Huginn").unwrap();
        writeln!(f, "Linha 2 de auditoria técnica Huginn").unwrap();
        writeln!(f, "Linha 3 de auditoria técnica Huginn").unwrap();
    }

    let ops = [
        OperationType::Adicionado,
        OperationType::Escaneado,
        OperationType::Modificado,
        OperationType::Renomeado,
        OperationType::Movido,
    ];

    for op in ops {
        let mut ev = EventModel::new(
            op,
            "relatorio_auditoria.txt".to_string(),
            tmp.to_string_lossy().to_string(),
        );
        if op == OperationType::Renomeado || op == OperationType::Movido {
            ev.dir_anterior = tmp.to_string_lossy().to_string();
        }
        extrair_metadados_completos(&file_path, &mut ev);
        if op == OperationType::Escaneado {
            sec_mgr.enriquecer_evento_scan(&mut ev);
        } else {
            sec_mgr.enriquecer_evento(&mut ev);
        }
        let saved = base.registrar_evento(ev);
        assert_eq!(
            formatar_celula_evento(&loc, &saved, "tipo"),
            ".txt",
            "Coluna 'type' deve exibir '.txt' para {:?}",
            op
        );
        assert_eq!(saved.linhas, "3");
        assert!(!saved.palavras.is_empty());
        assert!(!saved.paginas.is_empty());
        assert!(!saved.paginas_estimadas.is_empty());
    }

    // Agora remove fisicamente o arquivo do disco e testa o evento Excluído
    std::fs::remove_file(&file_path).unwrap();
    let mut ev_del = EventModel::new(
        OperationType::Excluido,
        "relatorio_auditoria.txt".to_string(),
        tmp.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&file_path, &mut ev_del);
    sec_mgr.enriquecer_evento(&mut ev_del);
    let saved_del = base.registrar_evento(ev_del);

    assert_eq!(formatar_celula_evento(&loc, &saved_del, "tipo"), ".txt");
    assert_eq!(saved_del.linhas, "3");
    assert!(!saved_del.palavras.is_empty());
    assert!(!saved_del.paginas.is_empty());
    assert!(!saved_del.paginas_estimadas.is_empty());
    assert!(saved_del.size_b > 0.0);
    assert!(!saved_del.dir_anterior.is_empty());
    assert!(!saved_del.dir_atual.is_empty());

    base.flush();
    let carregados = base.carregar_todos_eventos();
    assert_eq!(carregados.len(), 6);
    for ev in &carregados {
        assert_eq!(formatar_celula_evento(&loc, ev, "tipo"), ".txt");
        assert!(ev.size_b > 0.0);
        assert!(!ev.autor.is_empty());
        assert!(!ev.protegido.is_empty());
        assert!(!ev.atributos.is_empty());
        assert!(!ev.monitor_profile_name.is_empty());
        assert!(!ev.baseline_status.is_empty());
        assert!(!ev.alerta_resumo.is_empty());
        assert!(!ev.session_label.is_empty());
        assert!(!ev.diff_resumo.is_empty());
        assert!(!ev.risk_level.is_empty());
        assert!(!ev.actor_user.is_empty());
        assert!(!ev.actor_process.is_empty());
        assert!(!ev.publisher.is_empty());
        assert!(!ev.quarantine_status.is_empty());
        assert!(!ev.evidence_id.is_empty());
    }

    let _ = std::fs::remove_dir_all(&tmp);
}
