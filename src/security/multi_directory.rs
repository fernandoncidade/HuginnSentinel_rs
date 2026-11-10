use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MonitoringProfile {
    pub id: i64,
    pub nome: String,
    pub diretorios: Vec<String>,
    pub ativo: bool,
}

#[derive(Debug, Clone)]
struct ActiveProfileState {
    profile_id: String,
    profile_name: String,
    directories: Vec<String>,
}

pub struct MultiDiretorio {
    db_path: PathBuf,
    active_state: Mutex<ActiveProfileState>,
}

impl MultiDiretorio {
    pub fn new(db_path: PathBuf) -> Self {
        let s = Self {
            db_path,
            active_state: Mutex::new(ActiveProfileState {
                profile_id: "perfil_padrao".to_string(),
                profile_name: String::new(),
                directories: Vec::new(),
            }),
        };
        let _ = s.inicializar_tabela();
        s.sincronizar_perfil_ativo();
        s
    }

    fn inicializar_tabela(&self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS perfis_monitoramento (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nome TEXT UNIQUE,
                diretorios_json TEXT,
                ativo INTEGER DEFAULT 0
            );",
        )?;
        Ok(())
    }

    fn sincronizar_perfil_ativo(&self) {
        let perfis = self.listar_perfis();
        if let Some(ativo) = perfis.iter().find(|p| p.ativo).or_else(|| perfis.first()) {
            let mut st = self.active_state.lock();
            st.profile_id = format!("perfil_{}", ativo.id);
            st.profile_name = ativo.nome.clone();
            st.directories = ativo.diretorios.clone();
        }
    }

    pub fn configurar_diretorio_primario(&self, diretorio: &Path) {
        let dir_str = diretorio.to_string_lossy().to_string();
        let folder_name = diretorio
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Monitoramento Principal")
            .to_string();

        let mut st = self.active_state.lock();
        st.profile_id = "perfil_principal".to_string();
        st.profile_name = folder_name;
        if !st.directories.contains(&dir_str) {
            st.directories = vec![dir_str];
        }
    }

    pub fn resolver_perfil_evento(&self, dir_evento: &str) -> (String, String, String) {
        let st = self.active_state.lock();
        if !st.profile_name.is_empty() {
            let dirs = if st.directories.is_empty() {
                dir_evento.to_string()
            } else {
                st.directories.join("; ")
            };
            return (st.profile_id.clone(), st.profile_name.clone(), dirs);
        }
        drop(st);

        let fallback_name = Path::new(dir_evento)
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Monitoramento Local")
            .to_string();

        (
            "perfil_padrao".to_string(),
            fallback_name,
            dir_evento.to_string(),
        )
    }

    pub fn listar_perfis(&self) -> Vec<MonitoringProfile> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Vec::new();
        };

        let mut stmt = match conn.prepare(
            "SELECT id, nome, diretorios_json, ativo FROM perfis_monitoramento ORDER BY id ASC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
            let json_str: String = row.get(2)?;
            let dirs: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(MonitoringProfile {
                id: row.get(0)?,
                nome: row.get(1)?,
                diretorios: dirs,
                ativo: row.get::<_, i32>(3)? != 0,
            })
        });

        match rows {
            Ok(it) => it.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn salvar_perfil(
        &self,
        nome: &str,
        diretorios: &[String],
        ativo: bool,
    ) -> Result<(), String> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Err("Falha ao abrir banco de dados".to_string());
        };

        let json_str = serde_json::to_string(diretorios).unwrap_or_default();
        let ativo_int = if ativo { 1 } else { 0 };

        conn.execute(
            "INSERT INTO perfis_monitoramento (nome, diretorios_json, ativo)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(nome) DO UPDATE SET diretorios_json = excluded.diretorios_json, ativo = excluded.ativo",
            params![nome, json_str, ativo_int],
        )
        .map_err(|e| format!("Erro ao salvar perfil: {}", e))?;

        if ativo {
            let mut st = self.active_state.lock();
            st.profile_name = nome.to_string();
            st.directories = diretorios.to_vec();
        }

        Ok(())
    }

    pub fn remover_perfil(&self, id: i64) -> Result<(), String> {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return Err("Falha ao abrir banco de dados".to_string());
        };

        conn.execute(
            "DELETE FROM perfis_monitoramento WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Erro ao excluir perfil: {}", e))?;

        self.sincronizar_perfil_ativo();
        Ok(())
    }
}
