use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::security::{
    BaselineCriptografico, BaselineStatus, DiffConteudo, MotorRegras, Quarentena, ScoreRisco,
};
use std::fs;
use std::path::Path;

#[test]
fn test_rules_threat_detection() {
    let ev_ransom = EventModel::new(
        OperationType::Adicionado,
        "relatorio.pdf.locked",
        "C:\\Users\\Public",
    );
    let matches_ransom = MotorRegras::avaliar(&ev_ransom);
    assert!(
        !matches_ransom.is_empty(),
        "Deveria detectar extensão de ransomware .locked"
    );
    assert!(matches_ransom.iter().any(|m| m.rule_id == "SEC-R01"));

    let mut ev_double_ext = EventModel::new(
        OperationType::Adicionado,
        "fatura.pdf.exe",
        "C:\\Users\\Downloads",
    );
    ev_double_ext.tipo = "exe".to_string();
    let matches_double = MotorRegras::avaliar(&ev_double_ext);
    assert!(matches_double.iter().any(|m| m.rule_id == "SEC-R02"));

    ScoreRisco::avaliar_evento(&mut ev_double_ext);
    assert!(
        ev_double_ext.risk_score >= 60,
        "Risco de extensão dupla executável deve ser alto ou crítico"
    );
    assert!(!ev_double_ext.alerta_resumo.is_empty());
}

#[test]
fn test_diff_conteudo_formatting() {
    let diff_plus = DiffConteudo::resumir_diferenca(1024.0, 2048.0);
    assert_eq!(diff_plus, "+1024 bytes");

    let diff_minus = DiffConteudo::resumir_diferenca(2048.0, 1024.0);
    assert_eq!(diff_minus, "-1024 bytes");

    let diff_zero = DiffConteudo::resumir_diferenca(100.0, 100.0);
    assert_eq!(diff_zero, "Sem variação de tamanho");
}

#[test]
fn test_baseline_criptografico() {
    let temp_dir = std::env::temp_dir().join(format!(
        "linceu_test_base_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("arquivo_alvo.txt");
    fs::write(&test_file, "CONTEUDO_ORIGINAL_INTEGRO").unwrap();

    let db_path = temp_dir.join("baseline.db");
    let baseline = BaselineCriptografico::new(db_path);

    // Primeiro registro no baseline
    let (status1, _) = baseline.verificar_e_atualizar(&test_file, true);
    assert_eq!(status1, BaselineStatus::Novo);

    // Verificação sem modificação
    let (status2, _) = baseline.verificar_e_atualizar(&test_file, false);
    assert_eq!(status2, BaselineStatus::Integral);

    // Modificação no conteúdo do arquivo
    fs::write(&test_file, "CONTEUDO_ADULTERADO").unwrap();
    let (status3, _) = baseline.verificar_e_atualizar(&test_file, false);
    assert_eq!(status3, BaselineStatus::Modificado);

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_quarentena_isolamento_e_restauracao() {
    let temp_dir = std::env::temp_dir().join(format!(
        "linceu_test_quar_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let arquivo_teste = temp_dir.join("arquivo_suspeito.txt");
    fs::write(&arquivo_teste, "CONTEUDO_SUSPEITO_PARA_ISOLAMENTO").unwrap();

    let db_path = temp_dir.join("quarentena.db");
    let quarentena = Quarentena::new(db_path);

    let item = quarentena
        .isolar_arquivo(&arquivo_teste, "Teste de ameaça heurística")
        .expect("Falha ao isolar arquivo");
    assert!(
        !arquivo_teste.exists(),
        "Arquivo original deve ter sido removido ao ser isolado"
    );
    assert!(
        Path::new(&item.caminho_quarentena).exists(),
        "Arquivo deve estar presente no cofre da quarentena"
    );

    // Restauração com validação de hash SHA-256
    quarentena
        .restaurar(item.id, None)
        .expect("Falha ao restaurar arquivo da quarentena");
    assert!(
        arquivo_teste.exists(),
        "Arquivo deve existir novamente no local de origem"
    );
    assert_eq!(
        fs::read_to_string(&arquivo_teste).unwrap(),
        "CONTEUDO_SUSPEITO_PARA_ISOLAMENTO"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
