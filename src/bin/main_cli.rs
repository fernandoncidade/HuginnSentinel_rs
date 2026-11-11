use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use huginn_rs::core::event::EventModel;
use huginn_rs::core::types::OperationType;
use huginn_rs::security::{BaselineStatus, SecurityManager};
use huginn_rs::services::metadados::extrair_metadados_completos;
use huginn_rs::services::{BaseEvento, Observador};

static _STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

fn print_banner() {
    println!("===============================================================");
    println!("  Huginn Sentinel CLI — File Integrity & Security Auditor   ");
    println!("  Version 2026.11.11.0 | Rust Security & Monitoring Engine    ");
    println!("===============================================================\n");
}

fn print_help() {
    print_banner();
    println!("Usage:");
    println!("  huginnsentinel-cli.exe <command> [options]\n");
    println!("Commands:");
    println!("  scan <path>              Scan directory for files, hashes, and security risks");
    println!(
        "  monitor <path>           Real-time filesystem observation and live security alerting"
    );
    println!("  metadata <file>          Extract deep technical metadata (PE, hashes, timestamps)");
    println!("  baseline <path>          Verify files against stored cryptographic baseline or detect drift");
    println!("  alerts                   List security alerts and rule violations from database");
    println!(
        "  quarantine <action>      Quarantine actions: 'list' or 'restore <id> <destination>'"
    );
    println!("  sessions                 List observation and audit sessions from database");
    println!("  clear, cls               Clear console screen");
    println!("  --version, -v            Show official version");
    println!("  --help, -h               Show this help message\n");
    println!("Options:");
    println!("  --format <text|json|csv> Output format (default: text)");
    println!("  --depth <n>              Scan recursion depth limit (default: unlimited, 0 = target only)");
    println!("  --risks-only             Only output files/events with warnings or security risks");
    println!("  --duration <seconds>     Duration limit for monitoring in seconds (default: 0 = until Ctrl+C)");
    println!("  --update                 Update/create cryptographic baseline entries during baseline check");
    println!("  --no-banner              Suppress banner header in text mode\n");
    println!("Examples:");
    println!("  huginnsentinel-cli.exe scan \"C:\\inetpub\\wwwroot\" --format json");
    println!("  huginnsentinel-cli.exe scan \"D:\\Finance\" --depth 3 --risks-only");
    println!("  huginnsentinel-cli.exe monitor \"C:\\Windows\\Temp\" --duration 60");
    println!("  huginnsentinel-cli.exe metadata \"C:\\Windows\\System32\\cmd.exe\"");
    println!("  huginnsentinel-cli.exe baseline \"D:\\ImportantDocs\" --format csv");
    println!("  huginnsentinel-cli.exe alerts --risks-only");
    println!("  huginnsentinel-cli.exe quarantine list");
    println!("  huginnsentinel-cli.exe quarantine restore 12 \"C:\\Restored\\file.bin\"");
}

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    if raw_args.len() >= 2 {
        execute_command(&raw_args[1..]);
        return;
    }

    // Modo interativo (REPL Shell) quando executado sem parâmetros (ex: atalho no Menu Iniciar)
    run_interactive_shell();
}

fn run_interactive_shell() {
    print_banner();
    println!("  [MODO INTERATIVO / HUGINN SENTINEL CLI SHELL]");
    println!("  Digite um comando para executar ou 'help' para listar as opções.");
    println!("  Para encerrar a sessão, digite 'exit', 'quit' ou 'sair'.\n");

    let stdin = std::io::stdin();
    loop {
        print!("huginn-sentinel> ");
        let _ = std::io::stdout().flush();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("exit")
            || trimmed.eq_ignore_ascii_case("quit")
            || trimmed.eq_ignore_ascii_case("sair")
            || trimmed.eq_ignore_ascii_case(":q")
        {
            println!("Encerrando sessão do Huginn Sentinel CLI. Até logo!");
            break;
        }

        let tokens = parse_shell_line(trimmed);
        if !tokens.is_empty() {
            execute_command(&tokens);
            println!();
        }
    }
}

fn parse_shell_line(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for ch in line.chars() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn execute_command(args: &[String]) {
    if args.is_empty() {
        return;
    }

    // Suporte para invocação com prefixo do executável ou aplicação (ex: "huginnsentinel scan .")
    let mut effective_args = args;
    if !effective_args.is_empty() {
        let first = effective_args[0].to_lowercase();
        if first == "huginnsentinel"
            || first == "huginnsentinel.exe"
            || first == "huginnsentinel-cli"
            || first == "huginnsentinel-cli.exe"
            || first == "huginn-sentinel"
            || first == "huginn-sentinel.exe"
            || first == "huginn"
            || first == "huginn.exe"
            || first == "huginn-cli"
            || first == "huginn-cli.exe"
        {
            effective_args = &effective_args[1..];
        }
    }

    if effective_args.is_empty() {
        print_help();
        return;
    }

    let cmd = effective_args[0].to_lowercase();
    match cmd.as_str() {
        "--version" | "-v" | "version" => {
            println!("Huginn Sentinel CLI v2026.11.11.0 (Rust Edition)");
        }
        "--help" | "-h" | "help" | "?" => {
            print_help();
        }
        "clear" | "cls" => {
            print!("\x1B[2J\x1B[1;1H");
            let _ = std::io::stdout().flush();
        }
        "scan" => {
            if effective_args.len() < 2 {
                eprintln!("[ERROR] Argumento <path> ausente para comando scan. Ex: scan .");
                return;
            }
            let target = &effective_args[1];
            let format = get_arg_value(effective_args, "--format").unwrap_or_else(|| "text".to_string());
            let depth: i32 = get_arg_value(effective_args, "--depth")
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1);
            let risks_only = effective_args.iter().any(|a| a == "--risks-only");
            let no_banner = effective_args.iter().any(|a| a == "--no-banner");
            handle_scan(Path::new(target), depth, &format, risks_only, no_banner);
        }
        "metadata" => {
            if effective_args.len() < 2 {
                eprintln!("[ERROR] Argumento <file> ausente para comando metadata. Ex: metadata huginnsentinel.exe");
                return;
            }
            handle_metadata(Path::new(&effective_args[1]));
        }
        "baseline" => {
            if effective_args.len() < 2 {
                eprintln!("[ERROR] Argumento <path> ausente para comando baseline. Ex: baseline .");
                return;
            }
            let update = effective_args.iter().any(|a| a == "--update");
            handle_baseline(Path::new(&effective_args[1]), update);
        }
        "alerts" => {
            let risks_only = effective_args.iter().any(|a| a == "--risks-only");
            handle_alerts(risks_only);
        }
        "sessions" => {
            handle_sessions();
        }
        "quarantine" => {
            if effective_args.len() < 2 {
                println!("Uso: quarantine list | quarantine restore <id> <destino>");
                return;
            }
            let action = &effective_args[1];
            handle_quarantine(action, &effective_args[2..]);
        }
        "monitor" => {
            if effective_args.len() < 2 {
                eprintln!("[ERROR] Argumento <path> ausente para comando monitor. Ex: monitor C:\\Windows\\Temp");
                return;
            }
            let duration_secs: u64 = get_arg_value(effective_args, "--duration")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            handle_monitor(Path::new(&effective_args[1]), duration_secs);
        }
        _ => {
            eprintln!("[ERROR] Comando desconhecido: '{}'. Digite 'help' para ver os comandos disponíveis.", cmd);
        }
    }
}

fn handle_alerts(risks_only: bool) {
    print_banner();
    let base = BaseEvento::new(None);
    let eventos = base.carregar_eventos_recentes(500);
    let mut count = 0;
    println!("=== Alertas de Segurança e Violações Recentes ===\n");
    for ev in eventos {
        if (!risks_only || ev.risk_score > 0) && (!ev.alerta_resumo.is_empty() || ev.risk_score > 0) {
            count += 1;
            println!(
                "[{}] [{}] {:<30} | Score: {:<3} | {}\n    Arquivo: {}",
                ev.timestamp,
                ev.risk_level,
                truncate(&ev.nome, 30),
                ev.risk_score,
                ev.alerta_resumo,
                ev.full_path()
            );
        }
    }
    if count == 0 {
        println!("Nenhum alerta ou risco registrado no banco de dados.");
    } else {
        println!("\nTotal de alertas exibidos: {}", count);
    }
}

fn handle_sessions() {
    print_banner();
    let base = BaseEvento::new(None);
    let eventos = base.carregar_todos_eventos();
    let mut map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for ev in eventos {
        let label = if ev.session_label.is_empty() {
            "Sessão Padrão".to_string()
        } else {
            ev.session_label
        };
        *map.entry(label).or_insert(0) += 1;
    }
    println!("=== Sessões de Monitoramento e Auditoria ===\n");
    if map.is_empty() {
        println!("Nenhuma sessão gravada no banco de dados.");
    } else {
        for (sess, cnt) in map {
            println!("- {:<40} ({} eventos registrados)", sess, cnt);
        }
    }
}

fn get_arg_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == flag {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn handle_scan(target: &Path, depth: i32, format: &str, risks_only: bool, no_banner: bool) {
    if !target.exists() {
        eprintln!("[ERROR] Caminho de destino não existe: {:?}", target);
        return;
    }

    if !no_banner && format == "text" {
        print_banner();
        println!("[*] Iniciando varredura em: {:?}", target);
    }

    let security_mgr = Arc::new(SecurityManager::new(None));
    let mut files = Vec::new();
    coletar_arquivos_com_profundidade(target, 0, depth, &mut files);

    let mut scanned_events = Vec::new();

    for path in files {
        let nome = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let dir = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut ev = EventModel::new(OperationType::Escaneado, nome, dir);
        extrair_metadados_completos(&path, &mut ev);
        security_mgr.enriquecer_evento(&mut ev);

        if risks_only && ev.risk_score == 0 {
            continue;
        }

        scanned_events.push(ev);
    }

    match format {
        "json" => {
            if let Ok(json) = serde_json::to_string_pretty(&scanned_events) {
                println!("{}", json);
            }
        }
        "csv" => {
            println!("Nome,Diretorio,TamanhoKB,Tipo,NivelRisco,ScoreRisco,SHA256");
            for ev in scanned_events {
                println!(
                    "\"{}\",\"{}\",{:.2},\"{}\",\"{}\",{},\"{}\"",
                    ev.nome.replace('"', "\"\""),
                    ev.dir_atual.replace('"', "\"\""),
                    ev.size_kb,
                    ev.tipo,
                    ev.risk_level,
                    ev.risk_score,
                    ev.sha256_atual
                );
            }
        }
        _ => {
            println!("Total de arquivos analisados: {}\n", scanned_events.len());
            for ev in scanned_events {
                let badge = format!("[{}]", ev.risk_level);
                println!(
                    "{:<10} {:<35} ({:.2} KB) | Score: {:<3} | {}",
                    badge,
                    truncate(&ev.nome, 35),
                    ev.size_kb,
                    ev.risk_score,
                    if ev.alerta_resumo.is_empty() {
                        "OK"
                    } else {
                        &ev.alerta_resumo
                    }
                );
            }
        }
    }
}

fn handle_metadata(target: &Path) {
    if !target.exists() {
        eprintln!("[ERROR] Arquivo não encontrado: {:?}", target);
        return;
    }

    print_banner();
    println!("Metadados Detalhados para: {:?}", target);
    println!("---------------------------------------------------------------");

    let nome = target
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = target
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut ev = EventModel::new(OperationType::Escaneado, nome, dir);
    extrair_metadados_completos(target, &mut ev);

    let sec = SecurityManager::new(None);
    sec.enriquecer_evento(&mut ev);

    println!("Arquivo:            {}", ev.nome);
    println!("Diretório:          {}", ev.dir_atual);
    println!("Categoria / MIME:   {} ({})", ev.tipo, ev.mime);
    println!(
        "Tamanho:            {:.2} KB ({} bytes)",
        ev.size_kb, ev.size_b
    );
    println!("Data Modificação:   {}", ev.data_modificacao);
    println!("Data Criação:       {}", ev.data_criacao);
    println!("Atributos:          {}", ev.atributos);
    println!("Protegido:          {}", ev.protegido);
    if !ev.dimensoes.is_empty() {
        println!("Dimensões:          {}", ev.dimensoes);
    }
    if !ev.total_linhas.is_empty() {
        println!(
            "Linhas de Texto:    {} (Código: {})",
            ev.total_linhas, ev.linhas_codigo
        );
    }
    if !ev.arquivos.is_empty() {
        println!(
            "Arquivos no Zip:    {} (Descompactado: {:.2} KB)",
            ev.arquivos, ev.unzipped_kb
        );
    }
    println!("SHA-256:            {}", ev.sha256_atual);
    println!("Assinatura Digital: {}", ev.signature_state);
    println!("Publicador:         {}", ev.publisher);
    println!(
        "Nível de Risco:     {} (Score: {}/100)",
        ev.risk_level, ev.risk_score
    );
    if !ev.alerta_resumo.is_empty() {
        println!("Alertas Ativos:     {}", ev.alerta_resumo);
    }
    println!("---------------------------------------------------------------");
}

fn handle_baseline(target: &Path, update: bool) {
    print_banner();
    let sec = SecurityManager::new(None);
    let baseline = sec.baseline();

    if target.is_file() {
        let (status, prev_hash) = baseline.verificar_e_atualizar(target, update);
        println!(
            "[{}] {:?} (Hash Anterior: {})",
            status.as_str(),
            target,
            prev_hash.unwrap_or_else(|| "Nenhum".to_string())
        );
    } else if target.is_dir() {
        println!("[*] Verificando baseline recursivo em: {:?}", target);
        let mut files = Vec::new();
        coletar_arquivos_com_profundidade(target, 0, -1, &mut files);

        let mut drift_count = 0;
        for f in files {
            let (status, _) = baseline.verificar_e_atualizar(&f, update);
            if status != BaselineStatus::Integral {
                drift_count += 1;
                println!("{:<12} {:?}", format!("[{}]", status.as_str()), f);
            }
        }
        println!(
            "\nVerificação de baseline concluída. Desvios detectados: {}",
            drift_count
        );
    }
}

fn handle_quarantine(action: &str, args: &[String]) {
    let sec = SecurityManager::new(None);
    let quar = sec.quarentena();

    match action {
        "list" => {
            print_banner();
            let itens = quar.listar();
            println!("Arquivos em Quarentena: {}\n", itens.len());
            for item in itens {
                println!(
                    "ID: {:<4} | Data: {} | Tamanho: {} B | SHA-256: {}\n      Original: {}\n      Motivo:   {}\n",
                    item.id, item.data_quarentena, item.tamanho_bytes, truncate(&item.hash_sha256, 16), item.caminho_original, item.motivo
                );
            }
        }
        "restore" => {
            if args.is_empty() {
                eprintln!("Uso: quarantine restore <id> [caminho_destino]");
                return;
            }
            let id: i64 = args[0].parse().unwrap_or(0);
            let dest = args.get(1).map(Path::new);
            match quar.restaurar(id, dest) {
                Ok(()) => println!(
                    "[SUCESSO] Arquivo ID {} restaurado com integridade SHA-256 confirmada.",
                    id
                ),
                Err(e) => eprintln!("[ERRO] Falha ao restaurar: {}", e),
            }
        }
        _ => eprintln!("[ERRO] Ação de quarentena desconhecida: {}", action),
    }
}

fn handle_monitor(target: &Path, duration_secs: u64) {
    print_banner();
    println!("[*] Iniciando monitoramento em tempo real em: {:?}", target);
    println!("[*] Pressione Ctrl+C para encerrar.\n");

    let sec = Arc::new(SecurityManager::new(None));
    let (tx, rx) = crossbeam_channel::unbounded();

    let _obs = match Observador::novo(target.to_path_buf(), sec, tx) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[ERRO] Falha ao criar observador: {}", e);
            return;
        }
    };

    let start = std::time::Instant::now();

    loop {
        if duration_secs > 0 && start.elapsed().as_secs() >= duration_secs {
            println!(
                "\n[*] Limite de tempo de monitoramento atingido ({}s). Encerrando...",
                duration_secs
            );
            break;
        }

        while let Ok(ev) = rx.try_recv() {
            println!(
                "[{}] [{}] {:<30} | Risco: {:<7} | {}",
                ev.timestamp,
                ev.tipo_operacao,
                truncate(&ev.nome, 30),
                ev.risk_level,
                ev.full_path()
            );
        }

        std::thread::sleep(Duration::from_millis(200));
    }
}

fn coletar_arquivos_com_profundidade(
    dir: &Path,
    current_depth: i32,
    max_depth: i32,
    out: &mut Vec<PathBuf>,
) {
    if max_depth >= 0 && current_depth > max_depth {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let p = entry.path();
        if p.is_dir() {
            coletar_arquivos_com_profundidade(&p, current_depth + 1, max_depth, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max && max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
