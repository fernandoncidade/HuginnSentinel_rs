use std::path::{Path, PathBuf};

fn migrar_dados_legados(old_dir: &Path, new_dir: &Path) {
    if !old_dir.exists() || !old_dir.is_dir() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(old_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(n) => n,
                None => continue,
            };
            let target = new_dir.join(file_name);
            if path.is_dir() {
                let _ = std::fs::create_dir_all(&target);
                migrar_dados_legados(&path, &target);
            } else if path.is_file() && !target.exists() {
                let _ = std::fs::copy(&path, &target);
            }
        }
    }
}

pub fn obter_caminho_persistente() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(&local_app_data).join("HuginnSentinel");
        let _ = std::fs::create_dir_all(&p);

        // Migração transparente de dados legados de AppData\Local\Huginn se existirem
        let old_p = PathBuf::from(&local_app_data).join("Huginn");
        if old_p.exists() && old_p.is_dir() {
            migrar_dados_legados(&old_p, &p);
        }

        return p;
    }
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(&user_profile).join(".huginn_sentinel");
        let _ = std::fs::create_dir_all(&p);

        let old_p = PathBuf::from(&user_profile).join(".huginn");
        if old_p.exists() && old_p.is_dir() {
            migrar_dados_legados(&old_p, &p);
        }

        return p;
    }
    let p = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".huginn_sentinel");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Retorna o caminho do banco de segurança em diretório próprio:
/// `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`
pub fn obter_caminho_db_seguranca() -> PathBuf {
    let base = obter_caminho_persistente();
    let sec_dir = base.join("security");
    let _ = std::fs::create_dir_all(&sec_dir);
    let new_db = sec_dir.join("huginn_security.db");

    // Migração transparente de arquivos legados caso ainda existam na raiz
    let old_candidates = [
        base.join("linceu_security.db"),
        base.join("huginn_security.db"),
        sec_dir.join("linceu_security.db"),
    ];

    for old_db in old_candidates {
        if old_db.exists() && !new_db.exists() {
            let _ = std::fs::rename(&old_db, &new_db);
            let old_wal = old_db.with_extension("db-wal");
            let old_shm = old_db.with_extension("db-shm");
            let new_wal = new_db.with_extension("db-wal");
            let new_shm = new_db.with_extension("db-shm");
            if old_wal.exists() {
                let _ = std::fs::rename(&old_wal, &new_wal);
            }
            if old_shm.exists() {
                let _ = std::fs::rename(&old_shm, &new_shm);
            }
        } else if old_db.exists() && new_db.exists() {
            // Se o novo banco já existe, remove os arquivos legados da raiz
            let _ = std::fs::remove_file(&old_db);
            let _ = std::fs::remove_file(old_db.with_extension("db-wal"));
            let _ = std::fs::remove_file(old_db.with_extension("db-shm"));
        }
    }

    new_db
}

/// Retorna o caminho do banco de eventos e operações em diretório próprio:
/// `%LOCALAPPDATA%\HuginnSentinel\events\huginn_events.db`
pub fn obter_caminho_db_eventos() -> PathBuf {
    let base = obter_caminho_persistente();
    let ev_dir = base.join("events");
    let _ = std::fs::create_dir_all(&ev_dir);
    let new_db = ev_dir.join("huginn_events.db");

    // Migração transparente de arquivos legados caso ainda existam na raiz
    let old_db = base.join("huginn_events.db");
    if old_db.exists() && !new_db.exists() {
        let _ = std::fs::rename(&old_db, &new_db);
        let old_wal = old_db.with_extension("db-wal");
        let old_shm = old_db.with_extension("db-shm");
        let new_wal = new_db.with_extension("db-wal");
        let new_shm = new_db.with_extension("db-shm");
        if old_wal.exists() {
            let _ = std::fs::rename(&old_wal, &new_wal);
        }
        if old_shm.exists() {
            let _ = std::fs::rename(&old_shm, &new_shm);
        }
    } else if old_db.exists() && new_db.exists() {
        // Se o novo já existe, limpa arquivo legado na raiz
        let _ = std::fs::remove_file(&old_db);
        let _ = std::fs::remove_file(old_db.with_extension("db-wal"));
        let _ = std::fs::remove_file(old_db.with_extension("db-shm"));
    }

    new_db
}

pub fn obter_diretorio_base() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn resolver_caminho_asset(rel_path: &str) -> Option<PathBuf> {
    let clean = rel_path.trim_start_matches(['/', '\\']);
    let stripped = clean
        .strip_prefix("src/assets/")
        .or_else(|| clean.strip_prefix("src\\assets\\"))
        .or_else(|| clean.strip_prefix("assets/"))
        .or_else(|| clean.strip_prefix("assets\\"))
        .unwrap_or(clean);

    let base_dir = obter_diretorio_base();

    // Raízes candidatas onde os arquivos podem estar instalados ou no ambiente de desenvolvimento
    let mut roots: Vec<PathBuf> = Vec::new();

    // 1. Diretório do executável (onde o binário compilado está rodando no PC do cliente final)
    roots.push(base_dir.join("src").join("assets"));
    roots.push(base_dir.join("src").join("language"));
    roots.push(base_dir.join("src"));
    roots.push(base_dir.clone());
    roots.push(base_dir.join("assets"));
    roots.push(base_dir.join("language"));

    // 2. Diretório de trabalho atual (CWD)
    if let Ok(cwd) = std::env::current_dir() {
        if cwd != base_dir {
            roots.push(cwd.join("src").join("assets"));
            roots.push(cwd.join("src").join("language"));
            roots.push(cwd.join("src"));
            roots.push(cwd.clone());
            roots.push(cwd.join("assets"));
            roots.push(cwd.join("language"));
        }
    }

    // 3. Subindo diretórios a partir do executável (caso executado em target/release ou target/debug durante desenvolvimento)
    if let Some(target_dir) = base_dir.parent() {
        if let Some(proj_dir) = target_dir.parent() {
            roots.push(proj_dir.join("src").join("assets"));
            roots.push(proj_dir.join("src").join("language"));
            roots.push(proj_dir.join("src"));
            roots.push(proj_dir.to_path_buf());
            roots.push(proj_dir.join("assets"));
            roots.push(proj_dir.join("language"));
        }
    }

    // 4. Diretório do manifesto Cargo em tempo de compilação
    if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest);
        roots.push(p.join("src").join("assets"));
        roots.push(p.join("src").join("language"));
        roots.push(p.join("src"));
        roots.push(p.clone());
        roots.push(p.join("assets"));
        roots.push(p.join("language"));
    }

    // Subprefixos adicionais para resolver arquivos como ícones, docs, etc.
    let subprefixes = [
        "",
        "src/assets",
        "src/language",
        "assets",
        "language",
        "images",
        "images/icons",
        "images/imgs",
        "docs",
    ];

    let items_to_test = if stripped != clean {
        vec![clean, stripped]
    } else {
        vec![clean]
    };

    for item in items_to_test {
        for root in &roots {
            for sub in subprefixes {
                let cand = if sub.is_empty() {
                    root.join(item)
                } else {
                    root.join(sub).join(item)
                };
                if cand.exists() {
                    return Some(cand);
                }
            }
        }
    }

    None
}
