use crossbeam_channel::unbounded;
use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::security::SecurityManager;
use huginn_rs::services::diretorio_scanner::DiretorioScanner;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_scanner_starts_and_cancels_cleanly() {
    let temp_dir = std::env::temp_dir().join("linceu_test_scanner_cancel");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Cria alguns arquivos temporários
    for i in 0..100 {
        let f = temp_dir.join(format!("file_{}.txt", i));
        let _ = std::fs::write(&f, format!("conteudo teste {}", i));
    }

    let (event_tx, _event_rx) = unbounded::<EventModel>();
    let (progress_tx, progress_rx) = unbounded();

    let sec_mgr = Arc::new(SecurityManager::new(None));

    let scanner = DiretorioScanner::iniciar_scan(temp_dir.clone(), sec_mgr, event_tx, progress_tx);

    // Cancela imediatamente
    scanner.cancelar();

    // Aguarda conclusão (deve responder em menos de 2 segundos)
    let mut concluido = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if let Ok(prog) = progress_rx.recv_timeout(Duration::from_millis(50)) {
            if prog.concluido {
                concluido = true;
                break;
            }
        }
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    assert!(concluido, "O scanner deveria concluir/cancelar prontamente");
}

#[test]
fn test_eventos_clear_and_safe_bounds() {
    let mut eventos: Vec<EventModel> = Vec::new();
    for i in 0..100 {
        eventos.push(EventModel::new(
            OperationType::Adicionado,
            format!("arq_{}.txt", i),
            "C:\\test".to_string(),
        ));
    }

    let mut indices: Vec<usize> = (0..100).collect();

    // Simula a limpeza: ambos devem ser limpos
    eventos.clear();
    indices.clear();

    assert_eq!(eventos.len(), 0);
    assert_eq!(indices.len(), 0);

    // Teste de acesso seguro com .get()
    let row_idx = 0;
    let ev_opt = indices.get(row_idx).and_then(|&idx| eventos.get(idx));
    assert!(ev_opt.is_none());
}

#[test]
fn test_eventos_beyond_10000_unlimited() {
    let mut eventos: Vec<EventModel> = Vec::new();
    for i in 0..15_000 {
        eventos.push(EventModel::new(
            OperationType::Adicionado,
            format!("arq_{}.txt", i),
            "C:\\test".to_string(),
        ));
    }
    assert_eq!(
        eventos.len(),
        15_000,
        "Contagem de eventos não deve ser limitada a 10.000"
    );

    let filtros = huginn_rs::tools::filters::FiltroEstado::default();
    let contadores = filtros.calcular_contadores(&eventos);
    assert_eq!(contadores.total, 15_000);
    assert_eq!(contadores.adicionados, 15_000);
}

#[test]
fn test_scanner_produces_only_escaneado_events() {
    let temp_dir = std::env::temp_dir().join(format!(
        "linceu_test_scanner_only_scan_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let sub_dir = temp_dir.join("subpasta");
    let _ = std::fs::create_dir_all(&sub_dir);

    for i in 0..20 {
        let f = if i % 2 == 0 {
            temp_dir.join(format!("doc_{}.txt", i))
        } else {
            sub_dir.join(format!("sub_{}.bin", i))
        };
        let _ = std::fs::write(&f, format!("conteudo teste scan {}", i));
    }

    let (event_tx, event_rx) = unbounded::<EventModel>();
    let (progress_tx, progress_rx) = unbounded();
    let sec_mgr = Arc::new(SecurityManager::new(None));

    let _scanner = DiretorioScanner::iniciar_scan(temp_dir.clone(), sec_mgr, event_tx, progress_tx);

    let mut eventos = Vec::new();
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        while let Ok(ev) = event_rx.try_recv() {
            eventos.push(ev);
        }
        if let Ok(prog) = progress_rx.try_recv() {
            if prog.concluido {
                // Drena quaisquer eventos restantes
                while let Ok(ev) = event_rx.try_recv() {
                    eventos.push(ev);
                }
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(
        !eventos.is_empty(),
        "Scanner deve ter encontrado arquivos na pasta temporária"
    );

    // Todos os eventos reportados durante o escaneamento DEVEM ser estritamente 'Escaneado'
    for ev in &eventos {
        assert_eq!(
            ev.op_type(),
            OperationType::Escaneado,
            "Operação inválida reportada durante escaneamento: {:?}",
            ev.op_type()
        );
        assert_eq!(
            ev.tipo_operacao, "Escaneado",
            "Texto da operação deve ser 'Escaneado', obtido: '{}'",
            ev.tipo_operacao
        );
    }

    let filtros = huginn_rs::tools::filters::FiltroEstado::default();
    let contadores = filtros.calcular_contadores(&eventos);
    assert_eq!(contadores.escaneados, eventos.len());
    assert_eq!(contadores.adicionados, 0);
    assert_eq!(contadores.modificados, 0);
    assert_eq!(contadores.excluidos, 0);
    assert_eq!(contadores.renomeados, 0);
    assert_eq!(contadores.movidos, 0);
}

#[test]
fn test_scan_monitor_config_and_chaining_workflow() {
    use huginn_rs::core::scan_monitor_config::{
        MonitorActionOption, ScanActionOption, ScanMonitorConfig,
    };

    // 1. Configuração padrão
    let mut config = ScanMonitorConfig::default();
    assert!(config.perguntar_ao_escanear);
    assert_eq!(config.acao_escaneamento, ScanActionOption::ApenasEscanear);
    assert!(config.perguntar_ao_monitorar);
    assert_eq!(
        config.acao_monitoramento,
        MonitorActionOption::ApenasMonitorar
    );

    // 2. Modificação de comportamento e desativação de prompts
    config.perguntar_ao_escanear = false;
    config.acao_escaneamento = ScanActionOption::EscanearEMonitorar;
    config.perguntar_ao_monitorar = false;
    config.acao_monitoramento = MonitorActionOption::EscanearEDepoisMonitorar;

    // 3. Serialização e Desserialização JSON
    let serialized = serde_json::to_string_pretty(&config).expect("Deve serializar para JSON");
    let deserialized: ScanMonitorConfig =
        serde_json::from_str(&serialized).expect("Deve desserializar do JSON");
    assert_eq!(config, deserialized);
    assert!(!deserialized.perguntar_ao_escanear);
    assert_eq!(
        deserialized.acao_escaneamento,
        ScanActionOption::EscanearEMonitorar
    );
    assert!(!deserialized.perguntar_ao_monitorar);
    assert_eq!(
        deserialized.acao_monitoramento,
        MonitorActionOption::EscanearEDepoisMonitorar
    );
}
