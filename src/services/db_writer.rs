use crossbeam_channel::{unbounded, Sender};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::core::event::EventModel;
use crate::utils::logger::get_logger;

pub enum DbCommand {
    InsertEvent(String, EventModel),
    InsertBatch(String, Vec<EventModel>),
    ClearTable(String),
    Flush(crossbeam_channel::Sender<()>),
    RecreateAndVacuum(crossbeam_channel::Sender<Result<(), String>>),
    Shutdown,
}

pub struct DbWriter {
    sender: Sender<DbCommand>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl DbWriter {
    pub fn new(db_path: PathBuf) -> Self {
        let (sender, receiver) = unbounded::<DbCommand>();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let Ok(mut conn) = inicializar_conexao(&db_path) else {
                get_logger().error(&format!("Falha ao abrir banco SQLite: {:?}", db_path));
                return;
            };

            let mut batch: Vec<(String, EventModel)> = Vec::with_capacity(2048);

            let processar_comando =
                |cmd: DbCommand, conn: &mut Connection, batch: &mut Vec<(String, EventModel)>| {
                    match cmd {
                        DbCommand::InsertEvent(tabela, evento) => {
                            batch.push((tabela, evento));
                            if batch.len() >= 2048 {
                                persistir_lote(conn, batch);
                                batch.clear();
                            }
                        }
                        DbCommand::InsertBatch(tabela, eventos) => {
                            for ev in eventos {
                                batch.push((tabela.clone(), ev));
                            }
                            if batch.len() >= 2048 {
                                persistir_lote(conn, batch);
                                batch.clear();
                            }
                        }
                        DbCommand::ClearTable(tabela) => {
                            if !batch.is_empty() {
                                persistir_lote(conn, batch);
                                batch.clear();
                            }
                            let sql = format!("DELETE FROM {}", tabela);
                            let _ = conn.execute(&sql, []);
                        }
                        DbCommand::Flush(ack_sender) => {
                            if !batch.is_empty() {
                                persistir_lote(conn, batch);
                                batch.clear();
                            }
                            let _ = ack_sender.send(());
                        }
                        DbCommand::RecreateAndVacuum(ack_sender) => {
                            batch.clear();
                            // 1. Substitui temporariamente por conexão em memória para fechar os handles do arquivo em disco
                            if let Ok(mem_conn) = Connection::open_in_memory() {
                                let old_conn = std::mem::replace(conn, mem_conn);
                                drop(old_conn);
                            }
                            // 2. Remove arquivos do disco
                            let wal = db_path.with_extension("db-wal");
                            let shm = db_path.with_extension("db-shm");
                            let _ = std::fs::remove_file(&wal);
                            let _ = std::fs::remove_file(&shm);
                            let removed = std::fs::remove_file(&db_path).is_ok();

                            // 3. Reabre e inicializa a conexão principal
                            match inicializar_conexao(&db_path) {
                                Ok(new_conn) => {
                                    *conn = new_conn;
                                    if !removed {
                                        let sql = "DELETE FROM monitoramento;
                                                   DELETE FROM adicionado;
                                                   DELETE FROM excluido;
                                                   DELETE FROM modificado;
                                                   DELETE FROM renomeado;
                                                   DELETE FROM movido;
                                                   DELETE FROM escaneado;
                                                   DELETE FROM snapshot;
                                                   DELETE FROM sqlite_sequence WHERE name IN ('monitoramento', 'adicionado', 'excluido', 'modificado', 'renomeado', 'movido', 'escaneado', 'snapshot');
                                                   PRAGMA wal_checkpoint(TRUNCATE);
                                                   VACUUM;";
                                        let _ = conn.execute_batch(sql);
                                    }
                                    let _ = ack_sender.send(Ok(()));
                                }
                                Err(e) => {
                                    let _ = ack_sender.send(Err(format!("Falha ao recriar banco de eventos: {}", e)));
                                }
                            }
                        }
                        DbCommand::Shutdown => {}
                    }
                };

            while running_clone.load(Ordering::SeqCst) || !receiver.is_empty() {
                // Tenta receber comandos por até 50ms
                match receiver.recv_timeout(Duration::from_millis(50)) {
                    Ok(DbCommand::Shutdown) => break,
                    Ok(cmd) => {
                        processar_comando(cmd, &mut conn, &mut batch);

                        // Drena em lote rápido se houver mais comandos acumulados
                        while batch.len() < 2048 {
                            match receiver.try_recv() {
                                Ok(DbCommand::Shutdown) => break,
                                Ok(c) => processar_comando(c, &mut conn, &mut batch),
                                Err(_) => break,
                            }
                        }

                        if batch.len() >= 2048 {
                            persistir_lote(&mut conn, &batch);
                            batch.clear();
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        if !batch.is_empty() {
                            persistir_lote(&mut conn, &batch);
                            batch.clear();
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }

            if !batch.is_empty() {
                persistir_lote(&mut conn, &batch);
            }
        });

        Self {
            sender,
            running,
            handle: Some(handle),
        }
    }

    pub fn insert_event(&self, tabela: &str, evento: EventModel) {
        let _ = self
            .sender
            .send(DbCommand::InsertEvent(tabela.to_string(), evento));
    }

    pub fn insert_batch(&self, tabela: &str, eventos: Vec<EventModel>) {
        let _ = self
            .sender
            .send(DbCommand::InsertBatch(tabela.to_string(), eventos));
    }

    pub fn clear_table(&self, tabela: &str) {
        let _ = self.sender.send(DbCommand::ClearTable(tabela.to_string()));
    }

    pub fn flush(&self) {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = self.sender.send(DbCommand::Flush(tx));
        let _ = rx.recv_timeout(Duration::from_secs(2));
    }

    pub fn recreate_and_vacuum(&self) -> Result<(), String> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.sender
            .send(DbCommand::RecreateAndVacuum(tx))
            .map_err(|e| format!("Falha ao comunicar com gravador do banco de eventos: {}", e))?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|e| format!("Tempo limite esgotado ao limpar banco de eventos: {}", e))?
    }
}

impl Drop for DbWriter {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.sender.send(DbCommand::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

pub fn inicializar_conexao(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA cache_size = -65536;
         PRAGMA page_size = 4096;
         PRAGMA busy_timeout = 30000;
         PRAGMA foreign_keys = OFF;",
    )?;

    criar_tabelas_e_indices(&conn)?;

    Ok(conn)
}

pub fn criar_tabelas_e_indices(conn: &Connection) -> Result<(), rusqlite::Error> {
    let tabelas = [
        "monitoramento",
        "adicionado",
        "excluido",
        "modificado",
        "renomeado",
        "movido",
        "escaneado",
    ];

    for tab in tabelas {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {tab} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tipo_operacao TEXT,
                nome TEXT,
                dir_anterior TEXT,
                dir_atual TEXT,
                data_criacao TEXT,
                data_modificacao TEXT,
                data_acesso TEXT,
                tipo TEXT,
                mime TEXT,
                size_b REAL,
                size_kb REAL,
                size_mb REAL,
                size_gb REAL,
                size_tb REAL,
                atributos TEXT,
                autor TEXT,
                dimensoes TEXT,
                duracao TEXT,
                taxa_bits TEXT,
                protegido TEXT,
                paginas TEXT,
                linhas TEXT,
                palavras TEXT,
                paginas_estimadas TEXT,
                linhas_codigo TEXT,
                total_linhas TEXT,
                slides_estimados TEXT,
                arquivos TEXT,
                unzipped_b REAL,
                unzipped_kb REAL,
                unzipped_mb REAL,
                unzipped_gb REAL,
                unzipped_tb REAL,
                slides TEXT,
                binary_file_b REAL,
                binary_file_kb REAL,
                binary_file_mb REAL,
                binary_file_gb REAL,
                binary_file_tb REAL,
                planilhas TEXT,
                colunas TEXT,
                registros TEXT,
                tabelas TEXT,
                timestamp TEXT,
                monitor_profile_name TEXT,
                sha256_atual TEXT,
                baseline_status TEXT,
                alerta_resumo TEXT,
                session_label TEXT,
                diff_resumo TEXT,
                risk_score INTEGER,
                risk_level TEXT,
                actor_user TEXT,
                actor_process TEXT,
                publisher TEXT,
                quarantine_status TEXT,
                evidence_id TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_{tab}_nome ON {tab} (nome);
            CREATE INDEX IF NOT EXISTS idx_{tab}_dirs ON {tab} (dir_anterior, dir_atual);
            CREATE INDEX IF NOT EXISTS idx_{tab}_timestamp ON {tab} (timestamp);"
        );
        conn.execute_batch(&sql)?;
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS snapshot (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT,
            diretorio TEXT,
            tipo TEXT,
            mime TEXT,
            data_criacao TEXT,
            data_modificacao TEXT,
            data_acesso TEXT,
            size_b REAL,
            size_kb REAL,
            size_mb REAL,
            size_gb REAL,
            size_tb REAL,
            atributos TEXT,
            autor TEXT,
            dimensoes TEXT,
            duracao TEXT,
            taxa_bits TEXT,
            protegido TEXT,
            paginas TEXT,
            linhas TEXT,
            palavras TEXT,
            paginas_estimadas TEXT,
            linhas_codigo TEXT,
            total_linhas TEXT,
            slides_estimados TEXT,
            arquivos TEXT,
            unzipped_b REAL,
            unzipped_kb REAL,
            unzipped_mb REAL,
            unzipped_gb REAL,
            unzipped_tb REAL,
            slides TEXT,
            binary_file_b REAL,
            binary_file_kb REAL,
            binary_file_mb REAL,
            binary_file_gb REAL,
            binary_file_tb REAL,
            planilhas TEXT,
            colunas TEXT,
            registros TEXT,
            tabelas TEXT,
            timestamp TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_snapshot_nome ON snapshot (nome);
        CREATE INDEX IF NOT EXISTS idx_snapshot_diretorio ON snapshot (diretorio);
        CREATE INDEX IF NOT EXISTS idx_snapshot_nome_timestamp ON snapshot (nome, timestamp);",
    )?;

    Ok(())
}

fn sql_insert_tabela(tabela: &str) -> String {
    format!(
        "INSERT INTO {tabela} (
            tipo_operacao, nome, dir_anterior, dir_atual, data_criacao, data_modificacao, data_acesso,
            tipo, mime, size_b, size_kb, size_mb, size_gb, size_tb, atributos, autor, dimensoes,
            duracao, taxa_bits, protegido, paginas, linhas, palavras, paginas_estimadas, linhas_codigo,
            total_linhas, slides_estimados, arquivos, unzipped_b, unzipped_kb, unzipped_mb, unzipped_gb,
            unzipped_tb, slides, binary_file_b, binary_file_kb, binary_file_mb, binary_file_gb,
            binary_file_tb, planilhas, colunas, registros, tabelas, timestamp, monitor_profile_name,
            sha256_atual, baseline_status, alerta_resumo, session_label, diff_resumo, risk_score,
            risk_level, actor_user, actor_process, publisher, quarantine_status, evidence_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
            ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36,
            ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49, ?50, ?51, ?52, ?53,
            ?54, ?55, ?56, ?57
        )"
    )
}

fn persistir_lote(conn: &mut Connection, lote: &[(String, EventModel)]) {
    if lote.is_empty() {
        return;
    }

    let Ok(tx) = conn.transaction() else {
        return;
    };

    let mut sql_cache: std::collections::HashMap<&str, String> =
        std::collections::HashMap::with_capacity(8);

    for (tabela, ev) in lote {
        let sql = sql_cache
            .entry(tabela.as_str())
            .or_insert_with(|| sql_insert_tabela(tabela));

        if let Ok(mut stmt) = tx.prepare_cached(sql) {
            let _ = stmt.execute(params![
                ev.tipo_operacao,
                ev.nome,
                ev.dir_anterior,
                ev.dir_atual,
                ev.data_criacao,
                ev.data_modificacao,
                ev.data_acesso,
                ev.tipo,
                ev.mime,
                ev.size_b,
                ev.size_kb,
                ev.size_mb,
                ev.size_gb,
                ev.size_tb,
                ev.atributos,
                ev.autor,
                ev.dimensoes,
                ev.duracao,
                ev.taxa_bits,
                ev.protegido,
                ev.paginas,
                ev.linhas,
                ev.palavras,
                ev.paginas_estimadas,
                ev.linhas_codigo,
                ev.total_linhas,
                ev.slides_estimados,
                ev.arquivos,
                ev.unzipped_b,
                ev.unzipped_kb,
                ev.unzipped_mb,
                ev.unzipped_gb,
                ev.unzipped_tb,
                ev.slides,
                ev.binary_file_b,
                ev.binary_file_kb,
                ev.binary_file_mb,
                ev.binary_file_gb,
                ev.binary_file_tb,
                ev.planilhas,
                ev.colunas,
                ev.registros,
                ev.tabelas,
                ev.timestamp,
                ev.monitor_profile_name,
                ev.sha256_atual,
                ev.baseline_status,
                ev.alerta_resumo,
                ev.session_label,
                ev.diff_resumo,
                ev.risk_score,
                ev.risk_level,
                ev.actor_user,
                ev.actor_process,
                ev.publisher,
                ev.quarantine_status,
                ev.evidence_id
            ]);
        }
    }

    let _ = tx.commit();
}
