use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;

#[test]
fn test_event_model_creation_and_58_fields() {
    let mut ev = EventModel::new(OperationType::Adicionado, "documento.docx", "C:\\TestDir");
    ev.tipo = "docx".to_string();
    ev.mime = "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string();
    ev.size_b = 2048.0;
    ev.size_kb = 2.0;
    ev.size_mb = 0.002;
    ev.paginas = "4".to_string();
    ev.palavras = "1250".to_string();
    ev.linhas = "85".to_string();
    ev.sha256_atual =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    ev.risk_score = 15;
    ev.risk_level = "Baixo".to_string();
    ev.baseline_status = "Verificado".to_string();

    assert_eq!(ev.op_type(), OperationType::Adicionado);
    assert_eq!(ev.full_path(), "C:\\TestDir\\documento.docx");
    assert_eq!(ev.get_field("tipo_operacao"), "Adicionado");
    assert_eq!(ev.get_field("nome"), "documento.docx");
    assert_eq!(ev.get_field("dir_atual"), "C:\\TestDir");
    assert_eq!(ev.get_field("tipo"), "docx");
    assert_eq!(ev.get_field("size_b"), "2048");
    assert_eq!(ev.get_field("paginas"), "4");
    assert_eq!(ev.get_field("palavras"), "1250");
    assert_eq!(ev.get_field("linhas"), "85");
    assert_eq!(
        ev.get_field("sha256_atual"),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(ev.get_field("risk_score"), "15");
    assert_eq!(ev.get_field("risk_level"), "Baixo");
    assert_eq!(ev.get_field("baseline_status"), "Verificado");
}

#[test]
fn test_event_model_serialization_roundtrip() {
    let ev = EventModel::new(OperationType::Modificado, "codigo.rs", "C:\\Projetos");
    let json = serde_json::to_string(&ev).expect("Falha ao serializar EventModel");
    let deserialized: EventModel =
        serde_json::from_str(&json).expect("Falha ao desserializar EventModel");

    assert_eq!(deserialized.tipo_operacao, "Modificado");
    assert_eq!(deserialized.nome, "codigo.rs");
    assert_eq!(deserialized.dir_atual, "C:\\Projetos");
}
