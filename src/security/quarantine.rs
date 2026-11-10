use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

use crate::services::metadados::calcular_sha256;
use crate::utils::paths::obter_caminho_persistente;

#[derive(Debug, Clone)]
pub struct QuarantinedItem {
    pub id: i64,
    pub caminho_original: String,
    pub caminho_quarentena: String,
    pub hash_sha256: String,
    pub tamanho_bytes: u64,
    pub data_quarentena: String,
    pub motivo: String,
    pub status: String,
    pub evidence_id: String,
}

pub struct Quarentena {
    db_path: PathBuf,
    quarantine_dir: PathBuf,
}

impl Quarentena {
    pub fn new(db_path: PathBuf) -> Self {
        let quarantine_dir = obter_caminho_persistente().join("quarantine");
        let _ = fs::create_dir_all(&quarantine_dir);
        let s = Self {
            db_path,
            quarantine_dir,
        };
        let _ = s.inicializar_tabela();
        s
    }

    fn inicializar_tabela(&self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS quarentena (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                caminho_original TEXT,
                caminho_quarentena TEXT,
                hash_sha256 TEXT,
                tamanho_bytes INTEGER,
                data_quarentena TEXT,
                motivo TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_quar_original ON quarentena (caminho_original);",
        )?;
        Ok(())
    }

    pub fn isolar_arquivo(&self, caminho: &Path, motivo: &str) -> Option<QuarantinedItem> {
        if !caminho.exists() || !caminho.is_file() {
            return None;
        }

        let hash = calcular_sha256(caminho).unwrap_or_default();
        let tamanho = caminho.metadata().map(|m| m.len()).unwrap_or(0);
        let file_name = caminho.file_name()?.to_string_lossy();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let dest_filename = format!("{}_{}.quar", file_name, timestamp);
        let dest_path = self.quarantine_dir.join(&dest_filename);

        // Move o arquivo com segurança
        if fs::rename(caminho, &dest_path).is_err() {
            // Se falhar o rename atômico (ex: drives diferentes), faz cópia e exclusão
            if fs::copy(caminho, &dest_path).is_ok() {
                let _ = fs::remove_file(caminho);
            } else {
                return None;
            }
        }

        let original_str = caminho.to_string_lossy().to_string();
        let quar_str = dest_path.to_string_lossy().to_string();
        let data_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let Ok(conn) = Connection::open(&self.db_path) else {
            return None;
        };

        let _ = conn.execute(
            "INSERT INTO quarentena (caminho_original, caminho_quarentena, hash_sha256, tamanho_bytes, data_quarentena, motivo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![original_str, quar_str, hash, tamanho as i64, data_str, motivo],
        );

        let id = conn.last_insert_rowid();

        Some(QuarantinedItem {
            id,
            caminho_original: original_str,
            caminho_quarentena: quar_str,
            hash_sha256: hash,
            tamanho_bytes: tamanho,
            data_quarentena: data_str,
            motivo: motivo.to_string(),
            status: "Capturado".to_string(),
            evidence_id: format!("EV-QUAR-{:04}", id),
        })
    }

    pub fn listar(&self) -> Vec<QuarantinedItem> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Vec::new();
        };

        let mut stmt = match conn.prepare("SELECT id, caminho_original, caminho_quarentena, hash_sha256, tamanho_bytes, data_quarentena, motivo FROM quarentena ORDER BY id DESC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            Ok(QuarantinedItem {
                id,
                caminho_original: row.get(1)?,
                caminho_quarentena: row.get(2)?,
                hash_sha256: row.get(3)?,
                tamanho_bytes: row.get::<_, i64>(4)? as u64,
                data_quarentena: row.get(5)?,
                motivo: row.get(6)?,
                status: "Capturado".to_string(),
                evidence_id: format!("EV-QUAR-{:04}", id),
            })
        });

        match rows {
            Ok(it) => it.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn restaurar(&self, id: i64, destino_personalizado: Option<&Path>) -> Result<(), String> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Err("Falha ao abrir banco de quarentena".to_string());
        };

        let item: Option<(String, String, String)> = conn
            .query_row(
                "SELECT caminho_original, caminho_quarentena, hash_sha256 FROM quarentena WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();

        let Some((orig_str, quar_str, hash_esperado)) = item else {
            return Err("Registro de quarentena não encontrado".to_string());
        };

        let quar_path = Path::new(&quar_str);
        if !quar_path.exists() {
            return Err(format!("Arquivo de quarentena ausente: {}", quar_str));
        }

        // Validação estrita de integridade via SHA-256 antes da restauração
        let hash_atual = calcular_sha256(quar_path).unwrap_or_default();
        if !hash_esperado.is_empty() && hash_atual != hash_esperado {
            return Err(format!(
                "Falha de integridade criptográfica SHA-256! Esperado: {}, Atual: {}",
                hash_esperado, hash_atual
            ));
        }

        let destino = if let Some(dest) = destino_personalizado {
            dest.to_path_buf()
        } else {
            PathBuf::from(orig_str)
        };

        if let Some(parent) = destino.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if fs::copy(quar_path, &destino).is_err() {
            return Err(format!("Falha ao copiar para destino: {:?}", destino));
        }

        let _ = fs::remove_file(quar_path);
        let _ = conn.execute("DELETE FROM quarentena WHERE id = ?1", params![id]);

        Ok(())
    }
}
