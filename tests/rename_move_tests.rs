use crossbeam_channel::unbounded;
use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::security::SecurityManager;
use huginn_rs::services::base_evento::BaseEvento;
use huginn_rs::services::Observador;
use rusqlite::Connection;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_operation_types_renomeado_and_movido() {
    assert_eq!(
        OperationType::from_raw("Renomeado"),
        OperationType::Renomeado
    );
    assert_eq!(OperationType::from_raw("renamed"), OperationType::Renomeado);
    assert_eq!(
        OperationType::from_raw("op_renamed"),
        OperationType::Renomeado
    );
    assert_eq!(
        OperationType::from_raw("renombrado"),
        OperationType::Renomeado
    );

    assert_eq!(OperationType::from_raw("Movido"), OperationType::Movido);
    assert_eq!(OperationType::from_raw("moved"), OperationType::Movido);
    assert_eq!(OperationType::from_raw("op_moved"), OperationType::Movido);

    assert_eq!(OperationType::Renomeado.display_pt(), "Renomeado");
    assert_eq!(OperationType::Movido.display_pt(), "Movido");

    assert_eq!(OperationType::Renomeado.as_str(), "renomeado");
    assert_eq!(OperationType::Movido.as_str(), "movido");

    let (bg_ren, fg_ren) = OperationType::Renomeado.default_color_hex();
    assert_eq!(bg_ren, "#6a1b9a");
    assert_eq!(fg_ren, "#ffffff");

    let (bg_mov, fg_mov) = OperationType::Movido.default_color_hex();
    assert_eq!(bg_mov, "#ef6c00");
    assert_eq!(fg_mov, "#ffffff");
}

#[test]
fn test_sqlite_persistence_for_renomeado_and_movido() {
    let temp_db = std::env::temp_dir().join(format!(
        "linceu_test_ren_mov_{}.db",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let base = BaseEvento::new(Some(temp_db.clone()));

    let mut ev_ren = EventModel::new(
        OperationType::Renomeado,
        "documento_novo.pdf",
        "C:\\Projetos\\Documentos",
    );
    ev_ren.dir_anterior = "C:\\Projetos\\Documentos\\documento_antigo.pdf".to_string();
    ev_ren.tipo = "pdf".to_string();
    ev_ren.size_kb = 124.5;

    let mut ev_mov = EventModel::new(
        OperationType::Movido,
        "relatorio.xlsx",
        "C:\\Projetos\\Arquivo",
    );
    ev_mov.dir_anterior = "C:\\Projetos\\Rascunhos".to_string();
    ev_mov.tipo = "xlsx".to_string();
    ev_mov.size_kb = 58.2;

    base.registrar_evento(ev_ren);
    base.registrar_evento(ev_mov);
    base.flush();

    // Verifica persistência e leitura das tabelas específicas
    let conn = Connection::open(&temp_db).expect("Erro ao abrir banco de teste");

    let count_monitoramento: i64 = conn
        .query_row("SELECT count(*) FROM monitoramento", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        count_monitoramento, 2,
        "Monitoramento deve conter os 2 eventos"
    );

    let count_renomeado: i64 = conn
        .query_row("SELECT count(*) FROM renomeado", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count_renomeado, 1, "Tabela renomeado deve conter 1 evento");

    let (nome_ren, dir_ant_ren, dir_atual_ren): (String, String, String) = conn
        .query_row(
            "SELECT nome, dir_anterior, dir_atual FROM renomeado LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(nome_ren, "documento_novo.pdf");
    assert_eq!(
        dir_ant_ren,
        "C:\\Projetos\\Documentos\\documento_antigo.pdf"
    );
    assert_eq!(dir_atual_ren, "C:\\Projetos\\Documentos");

    let count_movido: i64 = conn
        .query_row("SELECT count(*) FROM movido", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count_movido, 1, "Tabela movido deve conter 1 evento");

    let (nome_mov, dir_ant_mov, dir_atual_mov): (String, String, String) = conn
        .query_row(
            "SELECT nome, dir_anterior, dir_atual FROM movido LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(nome_mov, "relatorio.xlsx");
    assert_eq!(dir_ant_mov, "C:\\Projetos\\Rascunhos");
    assert_eq!(dir_atual_mov, "C:\\Projetos\\Arquivo");

    drop(conn);
    let _ = fs::remove_file(temp_db);
}

#[test]
fn test_observador_detects_real_rename_and_move() {
    let base_temp = std::env::temp_dir().join(format!(
        "linceu_obs_test_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let _ = fs::create_dir_all(&base_temp);

    let sub_dir = base_temp.join("pasta_destino");
    let _ = fs::create_dir_all(&sub_dir);

    let sec = Arc::new(SecurityManager::new(None));
    let (tx, rx) = unbounded();

    let mut obs = match Observador::novo(base_temp.clone(), sec, tx) {
        Ok(o) => o,
        Err(e) => panic!("Falha ao inicializar Observador no teste: {}", e),
    };

    // Aguarda o watcher se registrar no sistema operacional
    thread::sleep(Duration::from_millis(200));

    // 1. Criação do arquivo inicial
    let file1 = base_temp.join("arquivo_teste.txt");
    {
        let mut f = File::create(&file1).expect("Erro ao criar arquivo teste");
        writeln!(f, "Conteúdo de teste para auditoria").unwrap();
    }
    thread::sleep(Duration::from_millis(250));

    // 2. Operação de Renomeação no mesmo diretório
    let file1_renomeado = base_temp.join("arquivo_teste_renomeado.txt");
    fs::rename(&file1, &file1_renomeado).expect("Erro ao renomear arquivo");
    thread::sleep(Duration::from_millis(300));

    // 3. Operação de Movimentação para outro diretório (subpasta)
    let file1_movido = sub_dir.join("arquivo_teste_renomeado.txt");
    fs::rename(&file1_renomeado, &file1_movido).expect("Erro ao mover arquivo");
    thread::sleep(Duration::from_millis(350));

    // Coleta todos os eventos emitidos pelo Observador
    let mut eventos = Vec::new();
    while let Ok(ev) = rx.try_recv() {
        eventos.push(ev);
    }

    // Para o observador
    obs.parar();

    // Validações
    let renomeado_opt = eventos
        .iter()
        .find(|e| e.op_type() == OperationType::Renomeado);
    assert!(
        renomeado_opt.is_some(),
        "Observador deve reconhecer e reportar evento Renomeado. Eventos recebidos: {:?}",
        eventos
            .iter()
            .map(|e| (&e.tipo_operacao, &e.nome, &e.dir_anterior, &e.dir_atual))
            .collect::<Vec<_>>()
    );
    let ren = renomeado_opt.unwrap();
    assert_eq!(ren.nome, "arquivo_teste_renomeado.txt");
    assert!(
        ren.dir_anterior.contains("arquivo_teste.txt"),
        "dir_anterior deve registrar a origem da renomeação: {}",
        ren.dir_anterior
    );

    let movido_opt = eventos
        .iter()
        .find(|e| e.op_type() == OperationType::Movido);
    assert!(
        movido_opt.is_some(),
        "Observador deve reconhecer e reportar evento Movido. Eventos recebidos: {:?}",
        eventos
            .iter()
            .map(|e| (&e.tipo_operacao, &e.nome, &e.dir_anterior, &e.dir_atual))
            .collect::<Vec<_>>()
    );
    let mov = movido_opt.unwrap();
    assert_eq!(mov.nome, "arquivo_teste_renomeado.txt");
    assert!(
        mov.dir_atual.contains("pasta_destino"),
        "dir_atual deve apontar para o novo diretório: {}",
        mov.dir_atual
    );

    // Limpeza
    let _ = fs::remove_dir_all(base_temp);
}
