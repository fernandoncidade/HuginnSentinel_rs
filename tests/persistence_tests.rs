use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::services::base_evento::BaseEvento;
use std::fs;

#[test]
fn test_sqlite_persistence_and_query() {
    let temp_db = std::env::temp_dir().join(format!(
        "linceu_test_{}.db",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let base = BaseEvento::new(Some(temp_db.clone()));

    let mut ev1 = EventModel::new(OperationType::Adicionado, "main.rs", "C:\\Projetos");
    ev1.sha256_atual = "abc123hash".to_string();
    ev1.size_b = 4096.0;

    let mut ev2 = EventModel::new(OperationType::Modificado, "main.rs", "C:\\Projetos");
    ev2.sha256_atual = "def456hash".to_string();
    ev2.size_b = 5120.0;

    base.registrar_evento(ev1);
    base.registrar_evento(ev2);
    base.flush();

    let recentes = base.carregar_eventos_recentes(10);
    assert_eq!(recentes.len(), 2);
    assert_eq!(recentes[0].nome, "main.rs");

    base.limpar_dados();
    base.flush();
    let apos_limpeza = base.carregar_eventos_recentes(10);
    assert_eq!(apos_limpeza.len(), 0);

    let _ = fs::remove_file(&temp_db);
    let _ = fs::remove_file(temp_db.with_extension("db-wal"));
    let _ = fs::remove_file(temp_db.with_extension("db-shm"));
}

#[test]
fn test_recreate_and_vacuum_eventos_db() {
    let temp_db = std::env::temp_dir().join(format!(
        "huginn_test_clear_{}.db",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let base = BaseEvento::new(Some(temp_db.clone()));

    for i in 0..50 {
        let mut ev = EventModel::new(OperationType::Adicionado, format!("file_{}.txt", i), "C:\\Teste");
        ev.sha256_atual = format!("hash_{}", i);
        base.registrar_evento(ev);
    }
    base.flush();

    let antes = base.carregar_todos_eventos();
    assert_eq!(antes.len(), 50);

    let res = base.limpar_banco_eventos();
    assert!(res.is_ok(), "Limpeza deve retornar Ok: {:?}", res);

    let depois = base.carregar_todos_eventos();
    assert_eq!(depois.len(), 0, "Banco deve estar vazio após limpeza");

    // Verifica que o gravador continua funcionando perfeitamente após a recriação
    let mut novo_ev = EventModel::new(OperationType::Adicionado, "novo_arquivo.rs", "C:\\Teste");
    novo_ev.sha256_atual = "novohash".to_string();
    base.registrar_evento(novo_ev);
    base.flush();

    let pos_recriacao = base.carregar_todos_eventos();
    assert_eq!(pos_recriacao.len(), 1, "Deve registrar novo evento após recriação");
    assert_eq!(pos_recriacao[0].nome, "novo_arquivo.rs");

    let _ = fs::remove_file(&temp_db);
    let _ = fs::remove_file(temp_db.with_extension("db-wal"));
    let _ = fs::remove_file(temp_db.with_extension("db-shm"));
}
