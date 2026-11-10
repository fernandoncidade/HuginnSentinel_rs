use crate::services::metadados::calcular_sha256;
use crate::services::metadados::documents::ler_texto_flexivel;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BaselineItem {
    pub id: i64,
    pub caminho: String,
    pub hash_sha256: String,
    pub tamanho_bytes: u64,
    pub data_registro: String,
    pub publisher: String,
    pub signature_state: String,
    pub snapshot_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineStatus {
    Integral,
    Modificado,
    Novo,
    Excluido,
    Diretorio,
    Desconhecido,
}

impl BaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BaselineStatus::Integral => "unchanged",
            BaselineStatus::Modificado => "changed",
            BaselineStatus::Novo => "new",
            BaselineStatus::Excluido => "missing",
            BaselineStatus::Diretorio => "directory",
            BaselineStatus::Desconhecido => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
struct BaselineCacheEntry {
    hash_sha256: String,
    tamanho_bytes: u64,
    data_registro: String,
    texto_snapshot: Option<String>,
}

pub struct BaselineCriptografico {
    db_path: PathBuf,
    conn: Mutex<Option<Connection>>,
    mem_cache: Mutex<HashMap<String, BaselineCacheEntry>>,
}

fn normalizar_chave_caminho(caminho: &Path) -> String {
    caminho
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase()
}

impl BaselineCriptografico {
    pub fn new(db_path: PathBuf) -> Self {
        let mut conn_opt = Self::abrir_e_configurar(&db_path).ok();
        let mut initial_cache = HashMap::new();

        if let Some(ref mut conn) = conn_opt {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT caminho, hash_sha256, tamanho_bytes, data_registro FROM baseline",
            ) {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
                        row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    ))
                }) {
                    for r in rows.flatten() {
                        let key = r.0.replace('/', "\\").to_lowercase();
                        initial_cache.insert(
                            key,
                            BaselineCacheEntry {
                                hash_sha256: r.1,
                                tamanho_bytes: r.2,
                                data_registro: r.3,
                                texto_snapshot: None,
                            },
                        );
                    }
                }
            }
        }

        Self {
            db_path,
            conn: Mutex::new(conn_opt),
            mem_cache: Mutex::new(initial_cache),
        }
    }

    fn abrir_e_configurar(db_path: &Path) -> Result<Connection, rusqlite::Error> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;
             CREATE TABLE IF NOT EXISTS baseline (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 caminho TEXT UNIQUE,
                 hash_sha256 TEXT,
                 tamanho_bytes INTEGER,
                 data_registro TEXT,
                 publisher TEXT DEFAULT '',
                 signature_state TEXT DEFAULT '',
                 snapshot_path TEXT DEFAULT ''
             );
             CREATE INDEX IF NOT EXISTS idx_baseline_caminho ON baseline (caminho);",
        )?;
        let _ = conn.execute("ALTER TABLE baseline ADD COLUMN publisher TEXT DEFAULT ''", []);
        let _ = conn.execute(
            "ALTER TABLE baseline ADD COLUMN signature_state TEXT DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE baseline ADD COLUMN snapshot_path TEXT DEFAULT ''",
            [],
        );
        Ok(conn)
    }

    pub fn limpar_conexao_e_cache(&self) {
        let mut lock = self.conn.lock();
        *lock = None;
        let mut cache = self.mem_cache.lock();
        cache.clear();
    }

    pub fn verificar_e_atualizar(
        &self,
        caminho: &Path,
        atualizar: bool,
    ) -> (BaselineStatus, Option<String>) {
        let (st, prev, _, _) = self.verificar_com_snapshot(caminho, None, atualizar, "", "");
        (st, prev)
    }

    pub fn verificar_e_atualizar_com_hash(
        &self,
        caminho: &Path,
        hash_precomputado: Option<&str>,
        atualizar: bool,
    ) -> (BaselineStatus, Option<String>) {
        let (st, prev, _, _) =
            self.verificar_com_snapshot(caminho, hash_precomputado, atualizar, "", "");
        (st, prev)
    }

    /// Avalia o baseline criptográfico e retorna `(status, hash_anterior, texto_anterior, texto_atual)`.
    pub fn verificar_com_snapshot(
        &self,
        caminho: &Path,
        hash_precomputado: Option<&str>,
        atualizar: bool,
        publisher: &str,
        signature_state: &str,
    ) -> (
        BaselineStatus,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let key = normalizar_chave_caminho(caminho);

        if !caminho.exists() {
            let prev = self
                .mem_cache
                .lock()
                .get(&key)
                .map(|e| e.hash_sha256.clone());
            return (BaselineStatus::Excluido, prev, None, None);
        }

        if caminho.is_dir() {
            return (BaselineStatus::Diretorio, None, None, None);
        }

        let tamanho = caminho.metadata().map(|m| m.len()).unwrap_or(0);
        let mut hash_atual = match hash_precomputado {
            Some(h) if !h.is_empty() && !h.starts_with("size_") => h.to_string(),
            _ => calcular_sha256(caminho).unwrap_or_default(),
        };
        if hash_atual.is_empty() {
            if tamanho == 0 {
                hash_atual =
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
            } else {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(b"linceu-meta-sha256:");
                if let Some(name) = caminho.file_name().and_then(|n| n.to_str()) {
                    hasher.update(name.to_lowercase().as_bytes());
                }
                hasher.update(b":");
                hasher.update(tamanho.to_le_bytes());
                hash_atual = hex::encode(hasher.finalize());
            }
        }

        let texto_atual = if tamanho > 0 && tamanho <= 131_072 {
            ler_texto_flexivel(caminho, 131_072)
        } else {
            None
        };

        let existing = self.mem_cache.lock().get(&key).cloned();

        match existing {
            Some(prev_entry) => {
                if prev_entry.hash_sha256 == hash_atual {
                    return (
                        BaselineStatus::Integral,
                        Some(prev_entry.hash_sha256),
                        prev_entry.texto_snapshot,
                        texto_atual,
                    );
                }

                let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                if atualizar {
                    self.mem_cache.lock().insert(
                        key,
                        BaselineCacheEntry {
                            hash_sha256: hash_atual.clone(),
                            tamanho_bytes: tamanho,
                            data_registro: now_str.clone(),
                            texto_snapshot: texto_atual.clone(),
                        },
                    );
                    self.persistir_no_banco(
                        caminho,
                        &hash_atual,
                        tamanho,
                        &now_str,
                        publisher,
                        signature_state,
                    );
                }

                (
                    BaselineStatus::Modificado,
                    Some(prev_entry.hash_sha256),
                    prev_entry.texto_snapshot,
                    texto_atual,
                )
            }
            None => {
                let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                if atualizar {
                    self.mem_cache.lock().insert(
                        key,
                        BaselineCacheEntry {
                            hash_sha256: hash_atual.clone(),
                            tamanho_bytes: tamanho,
                            data_registro: now_str.clone(),
                            texto_snapshot: texto_atual.clone(),
                        },
                    );
                    self.persistir_no_banco(
                        caminho,
                        &hash_atual,
                        tamanho,
                        &now_str,
                        publisher,
                        signature_state,
                    );
                }

                (BaselineStatus::Novo, None, None, texto_atual)
            }
        }
    }

    fn persistir_no_banco(
        &self,
        caminho: &Path,
        hash_atual: &str,
        tamanho: u64,
        data_reg: &str,
        publisher: &str,
        signature_state: &str,
    ) {
        let mut lock = self.conn.lock();
        if lock.is_none() {
            *lock = Self::abrir_e_configurar(&self.db_path).ok();
        }
        let Some(ref mut conn) = *lock else {
            return;
        };

        let caminho_str = caminho.to_string_lossy().to_string();
        let sql = "INSERT INTO baseline (caminho, hash_sha256, tamanho_bytes, data_registro, publisher, signature_state)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                   ON CONFLICT(caminho) DO UPDATE SET
                       hash_sha256 = excluded.hash_sha256,
                       tamanho_bytes = excluded.tamanho_bytes,
                       data_registro = excluded.data_registro,
                       publisher = CASE WHEN excluded.publisher != '' THEN excluded.publisher ELSE baseline.publisher END,
                       signature_state = CASE WHEN excluded.signature_state != '' THEN excluded.signature_state ELSE baseline.signature_state END";
        if let Ok(mut stmt) = conn.prepare_cached(sql) {
            let _ = stmt.execute(params![
                caminho_str,
                hash_atual,
                tamanho as i64,
                data_reg,
                publisher,
                signature_state
            ]);
        };
    }

    pub fn obter_registro(&self, caminho: &Path) -> Option<(String, u64, String)> {
        let key = normalizar_chave_caminho(caminho);
        if let Some(entry) = self.mem_cache.lock().get(&key) {
            return Some((
                entry.hash_sha256.clone(),
                entry.tamanho_bytes,
                entry.data_registro.clone(),
            ));
        }

        let mut lock = self.conn.lock();
        if lock.is_none() {
            *lock = Self::abrir_e_configurar(&self.db_path).ok();
        }

        let conn = lock.as_mut()?;
        let caminho_str = caminho.to_string_lossy().to_string();
        let query =
            "SELECT hash_sha256, tamanho_bytes, data_registro FROM baseline WHERE caminho = ?1";
        let mut stmt = conn.prepare_cached(query).ok()?;
        stmt.query_row(params![caminho_str], |r| {
            Ok((
                r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                r.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ))
        })
        .ok()
    }

    pub fn listar(&self) -> Vec<BaselineItem> {
        let mut lock = self.conn.lock();
        if lock.is_none() {
            *lock = Self::abrir_e_configurar(&self.db_path).ok();
        }
        let Some(ref mut conn) = *lock else {
            return Vec::new();
        };

        let query = "SELECT id, caminho, hash_sha256, tamanho_bytes, data_registro, COALESCE(publisher, ''), COALESCE(signature_state, ''), COALESCE(snapshot_path, '') FROM baseline ORDER BY id DESC";
        let Ok(mut stmt) = conn.prepare(query) else {
            return Vec::new();
        };

        let rows = stmt.query_map([], |row| {
            Ok(BaselineItem {
                id: row.get(0)?,
                caminho: row.get(1)?,
                hash_sha256: row.get(2)?,
                tamanho_bytes: row.get::<_, i64>(3)? as u64,
                data_registro: row.get(4)?,
                publisher: row.get(5)?,
                signature_state: row.get(6)?,
                snapshot_path: row.get(7)?,
            })
        });

        match rows {
            Ok(it) => it.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn restaurar_snapshot(
        &self,
        item_id: i64,
        destino_personalizado: Option<&Path>,
    ) -> Result<(), String> {
        let mut lock = self.conn.lock();
        if lock.is_none() {
            *lock = Self::abrir_e_configurar(&self.db_path).ok();
        }
        let Some(ref mut conn) = *lock else {
            return Err("Falha ao abrir conexão com baseline".to_string());
        };

        let item: Option<(String, String, String)> = conn
            .query_row(
                "SELECT caminho, snapshot_path, hash_sha256 FROM baseline WHERE id = ?1",
                params![item_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();

        let Some((orig_str, snap_str, hash_esperado)) = item else {
            return Err("Registro de baseline não encontrado".to_string());
        };

        let snap_path = Path::new(&snap_str);
        if snap_str.is_empty() || !snap_path.exists() {
            return Err(format!("Snapshot ausente para o caminho: {}", orig_str));
        }

        let hash_atual = calcular_sha256(snap_path).unwrap_or_default();
        if !hash_esperado.is_empty() && hash_atual != hash_esperado {
            return Err(format!(
                "Falha de integridade SHA-256 no snapshot! Esperado: {}, Atual: {}",
                hash_esperado, hash_atual
            ));
        }

        let destino = if let Some(dest) = destino_personalizado {
            dest.to_path_buf()
        } else {
            PathBuf::from(orig_str)
        };

        if let Some(parent) = destino.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::fs::copy(snap_path, &destino)
            .map_err(|e| format!("Falha ao copiar snapshot: {}", e))?;
        Ok(())
    }
}
