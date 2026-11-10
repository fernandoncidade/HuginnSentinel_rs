use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::locale::Localizer;
use huginn_rs::security::SecurityManager;
use huginn_rs::services::base_evento::BaseEvento;
use huginn_rs::services::gerenciador_colunas::LISTA_COLUNAS_TODAS;
use huginn_rs::services::metadados::{
    extensao_tipo_preferencial, extrair_metadados_ao_vivo, extrair_metadados_completos,
    extrair_metadados_scan, limpar_campos_conteudo, obter_tipo_e_mime, perfil_evento,
    preencher_metadados_item_excluido, tipo_eh_pasta,
};
use huginn_rs::ui::table_view::formatar_celula_evento;
use rusqlite::Connection;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

fn criar_pasta_isolada(prefixo: &str) -> PathBuf {
    let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("linceu_meta_cons_{}_{}_{}", prefixo, nanos, seq));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("Falha ao criar pasta temporária de teste");
    dir
}

/// As 56 colunas solicitadas pelo usuário (chaves de tradução canônicas).
const COLUNAS_SOLICITADAS_TRANSLATION_KEYS: &[&str] = &[
    "operation_type",
    "name",
    "prev_dir",
    "curr_dir",
    "creation_date",
    "modification_date",
    "access_date",
    "type",
    "size_b",
    "size_kb",
    "size_mb",
    "size_gb",
    "size_tb",
    "attributes",
    "author",
    "dimensions",
    "duration",
    "bit_rate",
    "protected",
    "pages",
    "lines",
    "words",
    "pages_estimated",
    "lines_code",
    "total_lines",
    "slides_estimated",
    "files",
    "unzipped_b",
    "unzipped_kb",
    "unzipped_mb",
    "unzipped_gb",
    "unzipped_tb",
    "slides",
    "binary_file_b",
    "binary_file_kb",
    "binary_file_mb",
    "binary_file_gb",
    "binary_file_tb",
    "spreadsheets",
    "columns",
    "records",
    "tables",
    "timestamp",
    "security_monitoring_profile",
    "security_sha256_current",
    "security_baseline_status",
    "security_alert_summary",
    "security_session_label",
    "security_diff_summary",
    "security_risk_score",
    "security_risk_level",
    "security_actor_user",
    "security_actor_process",
    "security_publisher_name",
    "security_quarantine_status",
    "security_evidence_id",
];

fn mapa_colunas_solicitadas() -> Vec<(&'static str, &'static str)> {
    COLUNAS_SOLICITADAS_TRANSLATION_KEYS
        .iter()
        .map(|&tkey| {
            let info = LISTA_COLUNAS_TODAS
                .iter()
                .find(|c| c.translation_key == tkey)
                .unwrap_or_else(|| panic!("Coluna solicitada ausente em LISTA_COLUNAS_TODAS: {tkey}"));
            (tkey, info.key)
        })
        .collect()
}

/// Retorna o conjunto de colunas (das 56 solicitadas, exceto `prev_dir` que depende da operação)
/// cujo valor formatado para UI (`formatar_celula_evento`) não é vazio, validando simultaneamente
/// que o caminho de exportação CSV (`EventModel::get_field`) possui paridade exata de presença.
fn conjunto_colunas_preenchidas(loc: &Localizer, ev: &EventModel) -> BTreeSet<&'static str> {
    let mut set = BTreeSet::new();
    for (tkey, col_key) in mapa_colunas_solicitadas() {
        if tkey == "prev_dir" {
            continue;
        }
        let val_ui = formatar_celula_evento(loc, ev, col_key);
        let val_csv = ev.get_field(col_key);
        assert_eq!(
            !val_ui.trim().is_empty(),
            !val_csv.trim().is_empty(),
            "Divergência entre UI (formatar_celula_evento='{}') e CSV (get_field='{}') na coluna '{}' ({}) para '{}'",
            val_ui,
            val_csv,
            tkey,
            col_key,
            ev.nome
        );
        if !val_csv.trim().is_empty() {
            set.insert(tkey);
        }
    }
    set
}

fn criar_png_minimo(caminho: &Path, w: u32, h: u32) {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([20, 120, 220, 255]));
    img.save_with_format(caminho, image::ImageFormat::Png)
        .expect("Falha ao salvar PNG de teste");
}

fn criar_wav_minimo(caminho: &Path) {
    let sample_rate: u32 = 8000;
    let num_samples: u32 = 8000; // 1 segundo mono 16-bit PCM
    let byte_rate: u32 = sample_rate * 2;
    let data_size: u32 = num_samples * 2;
    let riff_size: u32 = 36 + data_size;

    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 canal
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    bytes.resize((44 + data_size) as usize, 0);
    fs::write(caminho, bytes).unwrap();
}

fn criar_mp4_isobmff_com_tkhd(caminho: &Path, width: u16, height: u16, dur_secs: u32) {
    // ftyp (20 bytes)
    let mut ftyp = Vec::new();
    ftyp.extend_from_slice(&20u32.to_be_bytes());
    ftyp.extend_from_slice(b"ftypisom");
    ftyp.extend_from_slice(&0u32.to_be_bytes());
    ftyp.extend_from_slice(b"isom");

    // mvhd v0: header (8) + payload (100) = 108 bytes
    let mut mvhd_payload = vec![0u8; 100];
    let timescale: u32 = 1000;
    let duration: u32 = dur_secs * timescale;
    mvhd_payload[12..16].copy_from_slice(&timescale.to_be_bytes());
    mvhd_payload[16..20].copy_from_slice(&duration.to_be_bytes());
    let mut mvhd = Vec::new();
    mvhd.extend_from_slice(&((8 + mvhd_payload.len()) as u32).to_be_bytes());
    mvhd.extend_from_slice(b"mvhd");
    mvhd.extend_from_slice(&mvhd_payload);

    // tkhd v0: header (8) + payload (84) = 92 bytes (width em payload[76..80], height em payload[80..84] 16.16 fixed point)
    let mut tkhd_payload = vec![0u8; 84];
    tkhd_payload[76..78].copy_from_slice(&width.to_be_bytes());
    tkhd_payload[80..82].copy_from_slice(&height.to_be_bytes());
    let mut tkhd = Vec::new();
    tkhd.extend_from_slice(&((8 + tkhd_payload.len()) as u32).to_be_bytes());
    tkhd.extend_from_slice(b"tkhd");
    tkhd.extend_from_slice(&tkhd_payload);

    // trak contendo tkhd
    let mut trak = Vec::new();
    trak.extend_from_slice(&((8 + tkhd.len()) as u32).to_be_bytes());
    trak.extend_from_slice(b"trak");
    trak.extend_from_slice(&tkhd);

    // moov contendo mvhd + trak
    let mut moov = Vec::new();
    moov.extend_from_slice(&((8 + mvhd.len() + trak.len()) as u32).to_be_bytes());
    moov.extend_from_slice(b"moov");
    moov.extend_from_slice(&mvhd);
    moov.extend_from_slice(&trak);

    // mdat para representar payload de vídeo realista
    let mut mdat = Vec::new();
    let mdat_payload = vec![0xABu8; 16_384];
    mdat.extend_from_slice(&((8 + mdat_payload.len()) as u32).to_be_bytes());
    mdat.extend_from_slice(b"mdat");
    mdat.extend_from_slice(&mdat_payload);

    let mut all = Vec::new();
    all.extend_from_slice(&ftyp);
    all.extend_from_slice(&moov);
    all.extend_from_slice(&mdat);
    fs::write(caminho, all).unwrap();
}

fn criar_zip_ooxml(caminho: &Path, entradas: &[(&str, &str)]) {
    let f = File::create(caminho).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (nome, conteudo) in entradas {
        zip.start_file(*nome, opts).unwrap();
        zip.write_all(conteudo.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
}

fn criar_sqlite_amostra(caminho: &Path) {
    let conn = Connection::open(caminho).unwrap();
    conn.execute_batch(
        "CREATE TABLE clientes (id INTEGER PRIMARY KEY, nome TEXT, email TEXT);
         INSERT INTO clientes (nome, email) VALUES ('Ana', 'ana@exemplo.com'), ('Bruno', 'bruno@exemplo.com');",
    )
    .unwrap();
}

fn simular_evento_operacao(
    op: OperationType,
    caminho_origem: &Path,
    caminho_atual: &Path,
    sec: &SecurityManager,
    base: &BaseEvento,
) -> EventModel {
    let nome = caminho_atual
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir_atual = caminho_atual
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut ev = EventModel::new(op, nome.clone(), dir_atual.clone());

    match op {
        OperationType::Escaneado => {
            extrair_metadados_scan(caminho_atual, &mut ev);
            sec.enriquecer_evento_scan(&mut ev);
        }
        OperationType::Adicionado | OperationType::Modificado => {
            extrair_metadados_ao_vivo(caminho_atual, &mut ev);
            sec.enriquecer_evento(&mut ev);
        }
        OperationType::Renomeado => {
            ev.dir_anterior = caminho_origem.to_string_lossy().to_string();
            preencher_metadados_item_excluido(caminho_origem, &mut ev);
            let perfil_orig = perfil_evento(&ev);
            ev.nome = nome;
            ev.dir_atual = dir_atual;
            let is_dir_orig = tipo_eh_pasta(&ev.tipo, &ev.mime);
            if is_dir_orig || !extensao_tipo_preferencial(caminho_atual).is_empty() {
                let (t, m) = obter_tipo_e_mime(caminho_atual, is_dir_orig);
                ev.tipo = t;
                ev.mime = m;
            }
            if perfil_evento(&ev) != perfil_orig {
                limpar_campos_conteudo(&mut ev);
            }
            extrair_metadados_ao_vivo(caminho_atual, &mut ev);
            sec.enriquecer_evento(&mut ev);
        }
        OperationType::Movido => {
            ev.dir_anterior = caminho_origem
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            preencher_metadados_item_excluido(caminho_origem, &mut ev);
            ev.nome = nome;
            ev.dir_atual = dir_atual;
            extrair_metadados_ao_vivo(caminho_atual, &mut ev);
            sec.enriquecer_evento(&mut ev);
        }
        OperationType::Excluido => {
            ev.dir_anterior = dir_atual;
            if let Some((hash, size_bytes, data_reg)) = sec.baseline().obter_registro(caminho_atual)
            {
                if ev.sha256_atual.is_empty() {
                    ev.sha256_atual = hash.clone();
                }
                ev.sha256_baseline = hash;
                if ev.size_b == 0.0 && size_bytes > 0 {
                    ev.size_b = size_bytes as f64;
                    ev.size_kb = ev.size_b / 1024.0;
                    ev.size_mb = ev.size_kb / 1024.0;
                    ev.size_gb = ev.size_mb / 1024.0;
                    ev.size_tb = ev.size_gb / 1024.0;
                }
                if !data_reg.is_empty() {
                    ev.data_modificacao = data_reg;
                }
            }
            preencher_metadados_item_excluido(caminho_atual, &mut ev);
            sec.enriquecer_evento(&mut ev);
        }
    }

    base.registrar_evento(ev)
}

#[test]
fn test_paridade_metadados_todas_categorias_e_6_operacoes() {
    let raiz = criar_pasta_isolada("paridade_6_ops");
    let dir_a = raiz.join("origem");
    let dir_b = raiz.join("destino");
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();

    let sec = SecurityManager::new(Some(raiz.join("sec.db")));
    sec.configurar_diretorio_primario(&raiz);
    let base = BaseEvento::new(Some(raiz.join("events.db")));
    let loc = Localizer::new();

    // Prepara fixtures reais de todas as categorias de arquivos + diretório
    fs::write(
        dir_a.join("codigo.rs"),
        "// comentário\npub fn soma(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();
    fs::write(
        dir_a.join("componente.ts"),
        "export interface Item { id: number; nome: string; }\nexport const x: Item = { id: 1, nome: 'ok' };\n",
    )
    .unwrap();
    fs::write(dir_a.join("disco.vhd"), vec![0x56u8; 4096]).unwrap();
    fs::write(
        dir_a.join("notas.txt"),
        "Primeira linha do documento\nSegunda linha com mais palavras\n",
    )
    .unwrap();
    fs::write(
        dir_a.join("auditoria.log"),
        "2026-10-30 INFO Inicializando monitor\n2026-10-30 WARN Limite atingido\n",
    )
    .unwrap();
    fs::write(dir_a.join("config.json"), "{\"ativo\": true, \"porta\": 8080}\n").unwrap();
    fs::write(
        dir_a.join("manual.pdf"),
        b"%PDF-1.4\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Count 3 >> endobj\n3 0 obj << /Type /Page >> endobj\n%%EOF\n",
    )
    .unwrap();
    criar_zip_ooxml(
        &dir_a.join("proposta.docx"),
        &[
            (
                "docProps/core.xml",
                "<cp:coreProperties><dc:creator>Autor Teste</dc:creator></cp:coreProperties>",
            ),
            (
                "docProps/app.xml",
                "<Properties><Pages>2</Pages><Words>350</Words><Lines>28</Lines></Properties>",
            ),
            ("word/document.xml", "<w:document><w:body><w:p><w:t>Texto</w:t></w:p></w:body></w:document>"),
        ],
    );
    criar_zip_ooxml(
        &dir_a.join("orcamento.xlsx"),
        &[
            (
                "xl/workbook.xml",
                "<workbook><sheets><sheet name=\"Plan1\" sheetId=\"1\"/></sheets></workbook>",
            ),
            (
                "xl/worksheets/sheet1.xml",
                "<worksheet><dimension ref=\"A1:D12\"/><sheetData><row r=\"1\"><c r=\"A1\"/></row></sheetData></worksheet>",
            ),
        ],
    );
    fs::write(dir_a.join("tabela.csv"), "id,nome,valor\n1,alfa,10\n2,beta,20\n").unwrap();
    criar_zip_ooxml(
        &dir_a.join("slides.pptx"),
        &[
            ("docProps/app.xml", "<Properties><Slides>4</Slides><Words>180</Words></Properties>"),
            ("ppt/slides/slide1.xml", "<p:sld/>"),
        ],
    );
    criar_sqlite_amostra(&dir_a.join("dados.sqlite"));
    fs::write(dir_a.join("indice.ldb"), vec![0x11u8; 1024]).unwrap();
    fs::write(
        dir_a.join("mapa.geojson"),
        "{\"type\":\"FeatureCollection\",\"features\":[{\"type\":\"Feature\"}]}",
    )
    .unwrap();
    criar_png_minimo(&dir_a.join("imagem.png"), 64, 48);
    fs::write(
        dir_a.join("vetor.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"100\"></svg>",
    )
    .unwrap();
    fs::write(
        dir_a.join("icone.ico"),
        [0, 0, 1, 0, 1, 0, 32, 32, 0, 0, 1, 0, 32, 0, 0, 0, 0, 0, 22, 0, 0, 0],
    )
    .unwrap();
    fs::write(dir_a.join("metrica_fonte.pfm"), vec![0x00, 0x01, 0x02, 0x03, 0x44, 0x55]).unwrap();
    criar_wav_minimo(&dir_a.join("som.wav"));
    criar_mp4_isobmff_com_tkhd(&dir_a.join("clipe.mp4"), 1920, 1080, 12);
    criar_zip_ooxml(
        &dir_a.join("pacote.zip"),
        &[("a.txt", "conteudo a"), ("b.txt", "conteudo b")],
    );
    fs::write(
        dir_a.join("arquivo.7z"),
        [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04, 0x10, 0x20, 0x30, 0x40],
    )
    .unwrap();
    fs::write(dir_a.join("backup_vazio.old"), b"").unwrap();
    fs::write(dir_a.join("evento_win.evtx"), b"ElfFile\0\x01\x00\x00\x00").unwrap();
    fs::write(dir_a.join("aplicativo.exe"), b"MZ\x90\x00\x03\x00\x00\x00PE\0\0").unwrap();
    fs::write(dir_a.join("vazio.bin"), b"").unwrap();
    fs::write(dir_a.join("sessao_texto.tmp"), "linha temporária 1\nlinha 2\n").unwrap();
    fs::write(dir_a.join("sessao_bin.tmp"), [0x00, 0xFF, 0x10, 0x89, 0x00, 0x42]).unwrap();
    fs::write(dir_a.join("registro_bin.dat"), [0x72, 0x65, 0x67, 0x66, 0x00, 0x00, 0x01]).unwrap();
    fs::write(dir_a.join("registro_txt.dat"), "chave=valor\nativo=1\n").unwrap();
    fs::write(dir_a.join(".pid"), "4812").unwrap();
    fs::write(dir_a.join(".gitignore"), "/target\n*.log\n").unwrap();
    fs::write(
        dir_a.join("pacote.tar.gz"),
        [0x1F, 0x8B, 0x08, 0x00, 0, 0, 0, 0, 0, 3, 1, 2, 3, 4, 64, 0, 0, 0],
    )
    .unwrap();

    // Arquivos sem extensão (PNG mágico, SQLite mágico, README, texto puro, binário puro, 0 bytes)
    criar_png_minimo(&dir_a.join("f_000015"), 40, 30);
    criar_sqlite_amostra(&dir_a.join("MessageDB"));
    fs::write(dir_a.join("README"), "Guia rápido do projeto Linceu\nSegunda linha\n").unwrap();
    fs::write(dir_a.join("notas_sem_ext"), "Apenas texto puro sem extensão\n").unwrap();
    fs::write(dir_a.join("blob_sem_ext"), [0x00, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x13]).unwrap();
    fs::write(dir_a.join("vazio_sem_ext"), b"").unwrap();

    // Diretório monitorado
    let sub_pasta = dir_a.join("pasta_monitorada");
    fs::create_dir_all(&sub_pasta).unwrap();
    fs::write(sub_pasta.join("interno.txt"), "ok").unwrap();

    // Verifica que clipe.mp4 extrai exatamente 1920x1080 do tkhd corrigido
    {
        let mut ev_mp4 = EventModel::new(
            OperationType::Escaneado,
            "clipe.mp4",
            dir_a.to_string_lossy().to_string(),
        );
        extrair_metadados_completos(&dir_a.join("clipe.mp4"), &mut ev_mp4);
        assert_eq!(ev_mp4.dimensoes, "1920x1080");
        assert!(!ev_mp4.duracao.is_empty());
        assert!(!ev_mp4.taxa_bits.is_empty());
    }

    let itens: Vec<(String, &[&str], &[&str])> = vec![
        (
            "codigo.rs".into(),
            &["pages", "lines", "words", "pages_estimated", "lines_code", "total_lines"],
            &["dimensions", "duration", "files", "unzipped_b", "binary_file_b", "spreadsheets", "tables"],
        ),
        (
            "componente.ts".into(),
            &["pages", "lines", "words", "pages_estimated", "lines_code", "total_lines"],
            &["dimensions", "duration", "bit_rate", "binary_file_b"],
        ),
        (
            "disco.vhd".into(),
            &["files", "unzipped_b", "unzipped_kb", "unzipped_mb", "unzipped_gb", "unzipped_tb"],
            &["lines_code", "pages", "binary_file_b"],
        ),
        (
            "notas.txt".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["lines_code", "binary_file_b", "files", "unzipped_b", "dimensions"],
        ),
        (
            "auditoria.log".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files", "spreadsheets"],
        ),
        (
            "config.json".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files", "spreadsheets"],
        ),
        (
            "manual.pdf".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files", "unzipped_b"],
        ),
        (
            "proposta.docx".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files", "unzipped_b"],
        ),
        (
            "orcamento.xlsx".into(),
            &["spreadsheets", "columns", "records", "tables", "lines", "total_lines", "pages_estimated"],
            &["binary_file_b", "pages", "words", "files", "unzipped_b"],
        ),
        (
            "tabela.csv".into(),
            &["spreadsheets", "columns", "records", "tables", "lines", "total_lines", "pages_estimated"],
            &["binary_file_b", "pages", "words", "files"],
        ),
        (
            "slides.pptx".into(),
            &["slides", "slides_estimated", "pages", "pages_estimated", "words"],
            &["binary_file_b", "lines", "files", "unzipped_b", "spreadsheets"],
        ),
        (
            "dados.sqlite".into(),
            &["columns", "records", "tables", "lines", "total_lines"],
            &["spreadsheets", "pages", "binary_file_b", "files"],
        ),
        (
            "indice.ldb".into(),
            &["columns", "records", "tables", "lines", "total_lines"],
            &["spreadsheets", "pages", "binary_file_b"],
        ),
        (
            "mapa.geojson".into(),
            &["columns", "records", "tables", "lines", "total_lines"],
            &["spreadsheets", "pages", "binary_file_b"],
        ),
        (
            "imagem.png".into(),
            &["dimensions"],
            &["duration", "bit_rate", "pages", "lines", "binary_file_b", "files"],
        ),
        (
            "vetor.svg".into(),
            &["dimensions"],
            &["pages", "lines", "binary_file_b", "files"],
        ),
        (
            "icone.ico".into(),
            &["dimensions"],
            &["pages", "lines", "binary_file_b", "files"],
        ),
        (
            "metrica_fonte.pfm".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["dimensions", "pages", "lines"],
        ),
        (
            "som.wav".into(),
            &["duration", "bit_rate"],
            &["dimensions", "pages", "lines", "binary_file_b", "files"],
        ),
        (
            "clipe.mp4".into(),
            &["dimensions", "duration", "bit_rate"],
            &["pages", "lines", "binary_file_b", "files"],
        ),
        (
            "pacote.zip".into(),
            &["files", "unzipped_b", "unzipped_kb", "unzipped_mb", "unzipped_gb", "unzipped_tb"],
            &["pages", "lines", "binary_file_b", "dimensions"],
        ),
        (
            "arquivo.7z".into(),
            &["files", "unzipped_b", "unzipped_kb", "unzipped_mb", "unzipped_gb", "unzipped_tb"],
            &["pages", "lines", "binary_file_b", "dimensions"],
        ),
        (
            "backup_vazio.old".into(),
            &["files", "unzipped_b", "unzipped_kb", "unzipped_mb", "unzipped_gb", "unzipped_tb"],
            &["pages", "lines", "binary_file_b"],
        ),
        (
            "evento_win.evtx".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words"],
        ),
        (
            "aplicativo.exe".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "files", "unzipped_b"],
        ),
        (
            "vazio.bin".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "files", "unzipped_b"],
        ),
        (
            "sessao_texto.tmp".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words", "files"],
        ),
        (
            "sessao_bin.tmp".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words", "files"],
        ),
        (
            "registro_bin.dat".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words", "files"],
        ),
        (
            "registro_txt.dat".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words", "files"],
        ),
        (
            ".pid".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words", "files"],
        ),
        (
            ".gitignore".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files"],
        ),
        (
            "pacote.tar.gz".into(),
            &["files", "unzipped_b", "unzipped_kb", "unzipped_mb", "unzipped_gb", "unzipped_tb"],
            &["pages", "lines", "binary_file_b"],
        ),
        (
            "f_000015".into(),
            &["dimensions"],
            &["pages", "lines", "binary_file_b", "files"],
        ),
        (
            "MessageDB".into(),
            &["columns", "records", "tables", "lines", "total_lines"],
            &["pages", "binary_file_b", "files"],
        ),
        (
            "README".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files"],
        ),
        (
            "notas_sem_ext".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files"],
        ),
        (
            "blob_sem_ext".into(),
            &["binary_file_b", "binary_file_kb", "binary_file_mb", "binary_file_gb", "binary_file_tb"],
            &["pages", "lines", "words", "files"],
        ),
        (
            "vazio_sem_ext".into(),
            &["pages", "lines", "words", "pages_estimated", "total_lines"],
            &["binary_file_b", "files"],
        ),
        (
            "pasta_monitorada".into(),
            &["files"],
            &["security_sha256_current", "pages", "lines", "binary_file_b", "unzipped_b", "dimensions"],
        ),
    ];

    for (nome_inicial, obrigatorias, proibidas) in &itens {
        let p_orig = dir_a.join(nome_inicial);
        let eh_pasta = p_orig.is_dir();

        // 1. Escaneado
        let ev_scan = simular_evento_operacao(OperationType::Escaneado, &p_orig, &p_orig, &sec, &base);
        // 2. Adicionado
        let ev_add = simular_evento_operacao(OperationType::Adicionado, &p_orig, &p_orig, &sec, &base);
        // 3. Modificado
        let ev_mod = simular_evento_operacao(OperationType::Modificado, &p_orig, &p_orig, &sec, &base);

        // 4. Renomeado (mantendo mesma categoria/extensão)
        let nome_ren = if let Some(stem) = nome_inicial.strip_suffix(".tar.gz") {
            format!("{stem}_ren.tar.gz")
        } else if let Some(pos) = nome_inicial.rfind('.') {
            format!("{}_ren{}", &nome_inicial[..pos], &nome_inicial[pos..])
        } else if nome_inicial == "README" {
            "CHANGELOG".to_string()
        } else {
            format!("{}_ren", nome_inicial)
        };
        let p_ren = dir_a.join(&nome_ren);
        fs::rename(&p_orig, &p_ren).unwrap();
        let ev_ren = simular_evento_operacao(OperationType::Renomeado, &p_orig, &p_ren, &sec, &base);

        // 5. Movido para outro diretório
        let p_mov = dir_b.join(&nome_ren);
        fs::rename(&p_ren, &p_mov).unwrap();
        let ev_mov = simular_evento_operacao(OperationType::Movido, &p_ren, &p_mov, &sec, &base);

        // 6. Excluído
        if eh_pasta {
            fs::remove_dir_all(&p_mov).unwrap();
        } else {
            fs::remove_file(&p_mov).unwrap();
        }
        let ev_del = simular_evento_operacao(OperationType::Excluido, &p_mov, &p_mov, &sec, &base);

        let eventos_ops = [
            ("Escaneado", &ev_scan),
            ("Adicionado", &ev_add),
            ("Modificado", &ev_mod),
            ("Renomeado", &ev_ren),
            ("Movido", &ev_mov),
            ("Excluído", &ev_del),
        ];

        let cols_ref = conjunto_colunas_preenchidas(&loc, &ev_scan);

        for (op_label, ev) in eventos_ops {
            let cols_op = conjunto_colunas_preenchidas(&loc, ev);
            assert_eq!(
                cols_op, cols_ref,
                "Divergência de colunas para '{}' na operação '{}' vs 'Escaneado'.\nEm Escaneado mas não em {}: {:?}\nEm {} mas não em Escaneado: {:?}",
                nome_inicial,
                op_label,
                op_label,
                cols_ref.difference(&cols_op).collect::<Vec<_>>(),
                op_label,
                cols_op.difference(&cols_ref).collect::<Vec<_>>(),
            );

            for &col_obrig in *obrigatorias {
                assert!(
                    cols_op.contains(col_obrig),
                    "Item '{}' na operação '{}' deveria preencher a coluna '{}', mas veio vazia. Conjunto: {:?}",
                    nome_inicial,
                    op_label,
                    col_obrig,
                    cols_op
                );
            }
            for &col_proib in *proibidas {
                assert!(
                    !cols_op.contains(col_proib),
                    "Item '{}' na operação '{}' NÃO deveria preencher a coluna '{}' (contaminação entre categorias).",
                    nome_inicial,
                    op_label,
                    col_proib
                );
            }

            if matches!(op_label, "Renomeado" | "Movido" | "Excluído") {
                assert!(
                    !ev.dir_anterior.trim().is_empty(),
                    "Item '{}' na operação '{}' deve ter prev_dir preenchido",
                    nome_inicial,
                    op_label
                );
            }

            if eh_pasta {
                assert!(ev.sha256_atual.is_empty(), "Pasta não deve ter SHA-256");
                assert_eq!(ev.tipo, "folder");
            } else {
                assert_eq!(
                    ev.sha256_atual.len(),
                    64,
                    "Arquivo '{}' na operação '{}' deve possuir SHA-256 de 64 hex chars, obtido: '{}'",
                    nome_inicial,
                    op_label,
                    ev.sha256_atual
                );
                assert!(
                    ev.sha256_atual.chars().all(|c| c.is_ascii_hexdigit()),
                    "SHA-256 inválido para '{}': {}",
                    nome_inicial,
                    ev.sha256_atual
                );
                assert_ne!(
                    ev.tipo, "folder",
                    "Arquivo '{}' na operação '{}' jamais pode virar folder",
                    nome_inicial, op_label
                );
            }
        }
    }

    // Verifica que após persistência no SQLite e recarregamento (`carregar_todos_eventos`),
    // cada evento mantém exatamente o mesmo conjunto de colunas!
    base.flush();
    let recarregados = base.carregar_todos_eventos();
    assert_eq!(
        recarregados.len(),
        itens.len() * 6,
        "Todos os eventos das 6 operações devem ser recarregados do SQLite"
    );
    for chunk in recarregados.chunks_exact(6) {
        let cols_scan = conjunto_colunas_preenchidas(&loc, &chunk[0]);
        for ev in &chunk[1..] {
            let cols_ev = conjunto_colunas_preenchidas(&loc, ev);
            assert_eq!(
                cols_ev, cols_scan,
                "Após reload SQLite, evento '{}' ({}) divergiu do Escaneado ({})",
                ev.nome, ev.tipo_operacao, chunk[0].nome
            );
        }
    }

    // Verifica também que `.tmp` texto vs `.tmp` binário e `.dat` texto vs `.dat` binário têm conjuntos idênticos
    let tmp_txt = recarregados.iter().find(|e| e.nome == "sessao_texto.tmp").unwrap();
    let tmp_bin = recarregados.iter().find(|e| e.nome == "sessao_bin.tmp").unwrap();
    assert_eq!(
        conjunto_colunas_preenchidas(&loc, tmp_txt),
        conjunto_colunas_preenchidas(&loc, tmp_bin),
        "Arquivos .tmp não podem alternar entre colunas de texto e binário"
    );

    let dat_txt = recarregados.iter().find(|e| e.nome == "registro_txt.dat").unwrap();
    let dat_bin = recarregados.iter().find(|e| e.nome == "registro_bin.dat").unwrap();
    assert_eq!(
        conjunto_colunas_preenchidas(&loc, dat_txt),
        conjunto_colunas_preenchidas(&loc, dat_bin),
        "Arquivos .dat não podem alternar entre colunas de texto e binário"
    );

    let _ = fs::remove_dir_all(&raiz);
}

#[test]
fn test_exclusao_fria_sem_historico_mantem_mesmas_colunas_da_categoria() {
    let raiz = criar_pasta_isolada("exclusao_fria");
    let sec = SecurityManager::new(Some(raiz.join("sec.db")));
    let base = BaseEvento::new(Some(raiz.join("events.db")));
    let loc = Localizer::new();

    // Cria arquivos reais para obter conjunto de referência em `Escaneado` e testa `Excluído` em outro caminho nunca visto
    criar_png_minimo(&raiz.join("ref.png"), 120, 80);
    criar_zip_ooxml(
        &raiz.join("ref.docx"),
        &[("word/document.xml", "<w:document><w:body><w:p><w:t>Oi</w:t></w:p></w:body></w:document>")],
    );
    criar_zip_ooxml(
        &raiz.join("ref.xlsx"),
        &[
            ("xl/workbook.xml", "<workbook><sheets><sheet name=\"S1\"/></sheets></workbook>"),
            ("xl/worksheets/sheet1.xml", "<worksheet><dimension ref=\"A1:C5\"/></worksheet>"),
        ],
    );
    criar_zip_ooxml(&raiz.join("ref.zip"), &[("x.txt", "123")]);
    fs::write(raiz.join("ref.exe"), b"MZ\x90\x00\x03\x00\x00\x00").unwrap();
    criar_mp4_isobmff_com_tkhd(&raiz.join("ref.mp4"), 1280, 720, 5);

    for ext in ["png", "docx", "xlsx", "zip", "exe", "mp4"] {
        let p_ref = raiz.join(format!("ref.{ext}"));
        let ev_scan = simular_evento_operacao(OperationType::Escaneado, &p_ref, &p_ref, &sec, &base);

        // Caminho jamais visto nem em cache nem no SQLite
        let p_nunca_visto = raiz.join("sub_inexistente").join(format!("fantasma_novo.{ext}"));
        let ev_cold_del = simular_evento_operacao(
            OperationType::Excluido,
            &p_nunca_visto,
            &p_nunca_visto,
            &sec,
            &base,
        );

        assert_eq!(
            conjunto_colunas_preenchidas(&loc, &ev_cold_del),
            conjunto_colunas_preenchidas(&loc, &ev_scan),
            "Exclusão sem histórico prévio para .{ext} deve reportar exatamente as mesmas colunas que Escaneado"
        );
    }

    let _ = fs::remove_dir_all(&raiz);
}

#[test]
fn test_renomeacao_entre_extensoes_diferentes_nao_contamina_colunas() {
    let raiz = criar_pasta_isolada("rename_cross_ext");
    let sec = SecurityManager::new(Some(raiz.join("sec.db")));
    let base = BaseEvento::new(Some(raiz.join("events.db")));
    let loc = Localizer::new();

    // 1. Cria .tmp (binário) -> renomeia para .txt (documento) -> renomeia para .old (backup/compactado)
    let p_tmp = raiz.join("download.tmp");
    fs::write(&p_tmp, "Linha um de teste\nLinha dois de teste\n").unwrap();
    let ev_tmp = simular_evento_operacao(OperationType::Adicionado, &p_tmp, &p_tmp, &sec, &base);
    let cols_tmp = conjunto_colunas_preenchidas(&loc, &ev_tmp);
    assert!(cols_tmp.contains("binary_file_b"));
    assert!(!cols_tmp.contains("pages"));

    let p_txt = raiz.join("download.txt");
    fs::rename(&p_tmp, &p_txt).unwrap();
    let ev_txt = simular_evento_operacao(OperationType::Renomeado, &p_tmp, &p_txt, &sec, &base);
    let cols_txt = conjunto_colunas_preenchidas(&loc, &ev_txt);
    assert!(cols_txt.contains("pages"));
    assert!(cols_txt.contains("lines"));
    assert!(cols_txt.contains("words"));
    assert!(
        !cols_txt.contains("binary_file_b"),
        "Renomear .tmp -> .txt não pode manter binary_file_b"
    );

    let p_old = raiz.join("download.old");
    fs::rename(&p_txt, &p_old).unwrap();
    let ev_old = simular_evento_operacao(OperationType::Renomeado, &p_txt, &p_old, &sec, &base);
    let cols_old = conjunto_colunas_preenchidas(&loc, &ev_old);
    assert!(cols_old.contains("files"));
    assert!(cols_old.contains("unzipped_b"));
    assert!(
        !cols_old.contains("pages") && !cols_old.contains("lines") && !cols_old.contains("binary_file_b"),
        "Renomear .txt -> .old deve limpar páginas/linhas e expor apenas colunas de arquivo de backup"
    );

    let _ = fs::remove_dir_all(&raiz);
}

#[test]
fn test_reparo_registro_historico_corrompido_folder_com_hash_e_binario() {
    let raiz = criar_pasta_isolada("reparo_folder");
    let db_path = raiz.join("events.db");
    let base = BaseEvento::new(Some(db_path));
    let loc = Localizer::new();

    // Simula exatamente o bug histórico do CSV: arquivo sem extensão excluído que foi salvo como `folder`
    let mut ev_corrompido = EventModel::new(
        OperationType::Excluido,
        "f_0042dd",
        raiz.to_string_lossy().to_string(),
    );
    ev_corrompido.dir_anterior = raiz.to_string_lossy().to_string();
    ev_corrompido.tipo = "folder".to_string();
    ev_corrompido.mime = "inode/directory".to_string();
    ev_corrompido.atributos = "folder".to_string();
    ev_corrompido.size_b = 1024.0;
    ev_corrompido.binary_file_b = 1024.0;
    ev_corrompido.sha256_atual =
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_string();

    let corrigido = base.registrar_evento(ev_corrompido);
    assert_eq!(corrigido.tipo, "binary");
    assert_ne!(
        formatar_celula_evento(&loc, &corrigido, "tipo"),
        loc.t("folder")
    );
    assert_eq!(corrigido.sha256_atual.len(), 64);
    assert!(corrigido.arquivos.is_empty());

    let _ = fs::remove_dir_all(&raiz);
}

#[cfg(target_os = "windows")]
#[test]
fn test_locks_exclusivos_windows_transitorio_e_permanente_mantem_paridade() {
    use std::os::windows::fs::OpenOptionsExt;
    use std::thread;
    use std::time::Duration;

    let raiz = criar_pasta_isolada("locks_windows");
    let sec = SecurityManager::new(Some(raiz.join("sec.db")));
    let base = BaseEvento::new(Some(raiz.join("events.db")));
    let loc = Localizer::new();

    // 1. Lock exclusivo TRANSITÓRIO (simula cópia em andamento no Windows Explorer com share_mode(0) por 80ms)
    let p_docx = raiz.join("copia_em_andamento.docx");
    criar_zip_ooxml(
        &p_docx,
        &[
            ("docProps/app.xml", "<Properties><Pages>5</Pages><Words>900</Words><Lines>75</Lines></Properties>"),
            ("word/document.xml", "<w:document><w:body><w:p><w:t>Conteudo real</w:t></w:p></w:body></w:document>"),
        ],
    );

    let handle_lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&p_docx)
        .expect("Deveria adquirir lock exclusivo temporário");

    let liberador = thread::spawn(move || {
        thread::sleep(Duration::from_millis(80));
        drop(handle_lock);
    });

    let ev_live = simular_evento_operacao(OperationType::Adicionado, &p_docx, &p_docx, &sec, &base);
    liberador.join().unwrap();

    assert_eq!(ev_live.paginas, "5", "Deve aguardar liberação do lock transitório e ler páginas reais do DOCX");
    assert_eq!(ev_live.binary_file_b, 0.0, "Jamais pode contaminar DOCX com Binário");
    assert_eq!(ev_live.sha256_atual.len(), 64);

    // 2. Lock exclusivo PERMANENTE (simula NTUSER.DAT / LOCK / arquivo retido pelo SO durante toda a leitura)
    let p_png_ref = raiz.join("livre.png");
    let p_png_lock = raiz.join("travado.png");
    criar_png_minimo(&p_png_ref, 100, 60);
    criar_png_minimo(&p_png_lock, 100, 60);

    let ev_png_livre = simular_evento_operacao(OperationType::Escaneado, &p_png_ref, &p_png_ref, &sec, &base);

    let _guard_permanente = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&p_png_lock)
        .expect("Deveria manter lock exclusivo permanente");

    let mut ev_png_travado = EventModel::new(
        OperationType::Escaneado,
        "travado.png",
        raiz.to_string_lossy().to_string(),
    );
    extrair_metadados_completos(&p_png_lock, &mut ev_png_travado);
    sec.enriquecer_evento_scan(&mut ev_png_travado);
    let ev_png_travado = base.registrar_evento(ev_png_travado);

    assert_eq!(
        conjunto_colunas_preenchidas(&loc, &ev_png_travado),
        conjunto_colunas_preenchidas(&loc, &ev_png_livre),
        "Mesmo sob lock exclusivo permanente do Windows, .png deve reportar exatamente as mesmas colunas sem Binário"
    );
    assert_eq!(ev_png_travado.binary_file_b, 0.0);
    assert_eq!(ev_png_travado.sha256_atual.len(), 64);

    drop(_guard_permanente);
    let _ = fs::remove_dir_all(&raiz);
}

#[test]
fn test_carregamento_historico_volumoso_em_memoria_sem_bloqueio_de_disco() {
    let raiz = criar_pasta_isolada("boot_rapido_sqlite");
    let db_path = raiz.join("events_boot.db");
    let base = BaseEvento::new(Some(db_path));

    // Insere 3.000 eventos históricos variados (incluindo binários sem extensão, arquivos de 0 bytes,
    // arquivos bloqueados/inexistentes e registros históricos incompletos)
    let mut lote = Vec::with_capacity(3000);
    for i in 0..3000 {
        let (nome, tipo, size_b) = match i % 6 {
            0 => (format!("f_{i:06x}"), "binary", 65536.0),
            1 => (format!("LOCK_{i}"), "file", 0.0),
            2 => (format!("cache_{i}.pak"), ".pak", 4_194_304.0),
            3 => (format!("relatorio_{i}.docx"), ".docx", 24_576.0),
            4 => (format!("dados_{i}.sqlite"), ".sqlite", 131_072.0),
            _ => (format!("subpasta_{i}"), "folder", 0.0),
        };
        let mut ev = EventModel::new(
            OperationType::Escaneado,
            nome,
            r"C:\Caminho\Inexistente\AppData\Local\Google\Cache".to_string(),
        );
        ev.tipo = tipo.to_string();
        ev.size_b = size_b;
        ev.size_kb = size_b / 1024.0;
        if tipo != "folder" {
            ev.sha256_atual =
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
        }
        lote.push(ev);
    }

    base.registrar_eventos_lote(lote);
    base.flush();

    let inicio = std::time::Instant::now();
    let carregados = base.carregar_todos_eventos();
    let decorrido = inicio.elapsed();

    assert_eq!(carregados.len(), 3000);
    assert!(
        decorrido.as_millis() < 1500,
        "carregar_todos_eventos levou {:?} para 3000 itens — não deve fazer I/O de disco na inicialização!",
        decorrido
    );
    assert!(carregados[0].binary_file_b > 0.0);
    assert!(!carregados[3].paginas.is_empty());
    assert!(!carregados[4].tabelas.is_empty());
    assert_eq!(carregados[5].tipo, "folder");

    let _ = fs::remove_dir_all(&raiz);
}
