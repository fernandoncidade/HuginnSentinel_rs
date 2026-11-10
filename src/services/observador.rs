use crossbeam_channel::{unbounded, Sender};
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::metadados::{
    arquivo_acessivel, extensao_tipo_preferencial, extrair_metadados_ao_vivo,
    limpar_campos_conteudo, obter_tipo_e_mime, perfil_evento, preencher_metadados_item_excluido,
    tipo_eh_pasta,
};
use crate::core::event::EventModel;
use crate::core::types::OperationType;
use crate::security::SecurityManager;
use crate::utils::logger::get_logger;

pub struct Observador {
    diretorio: PathBuf,
    ativo: Arc<AtomicBool>,
    pausado: Arc<AtomicBool>,
    watcher: Option<RecommendedWatcher>,
    worker_handle: Option<JoinHandle<()>>,
}

/// Normaliza um caminho para comparação de diretório invariante a maiúsculas/minúsculas e separadores.
fn normalizar_dir_str(path: &Path) -> String {
    let s = path.to_string_lossy().replace('/', "\\");
    s.trim_end_matches('\\').to_lowercase()
}

/// Verifica se dois caminhos residem no mesmo diretório pai.
fn mesmo_diretorio(p1: &Path, p2: &Path) -> bool {
    let d1 = p1.parent().map(normalizar_dir_str).unwrap_or_default();
    let d2 = p2.parent().map(normalizar_dir_str).unwrap_or_default();
    !d1.is_empty() && d1 == d2
}

/// Localiza na fila de renomeações pendentes o registro de origem mais provável para o destino.
/// Segue a lógica de `localizar_registro_antigo` do C++:
/// 1. Mesmo diretório pai
/// 2. Mesmo nome base
/// 3. Registro mais antigo
fn localizar_registro_antigo(
    pendentes: &mut VecDeque<(PathBuf, Instant)>,
    to_path: &Path,
) -> Option<PathBuf> {
    if pendentes.is_empty() {
        return None;
    }

    // 1. Mesmo diretório pai
    if let Some(pos) = pendentes
        .iter()
        .position(|(from, _)| mesmo_diretorio(from, to_path))
    {
        return Some(pendentes.remove(pos).unwrap().0);
    }

    // 2. Mesmo nome base
    let to_name = to_path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase());
    if let Some(ref name) = to_name {
        if let Some(pos) = pendentes.iter().position(|(from, _)| {
            from.file_name()
                .map(|f| f.to_string_lossy().to_lowercase())
                .as_ref()
                == Some(name)
        }) {
            return Some(pendentes.remove(pos).unwrap().0);
        }
    }

    // 3. Fallback: registro mais antigo
    Some(pendentes.pop_front().unwrap().0)
}

/// Cria um EventModel devidamente preenchido para operações de Renomeado ou Movido.
fn criar_evento_rename_move(
    from_path: &Path,
    to_path: &Path,
    security_mgr: &SecurityManager,
) -> EventModel {
    let mudou_dir = !mesmo_diretorio(from_path, to_path);
    let op = if mudou_dir {
        OperationType::Movido
    } else {
        OperationType::Renomeado
    };

    let nome = to_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir_atual = to_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let dir_anterior = if mudou_dir {
        let from_name = from_path
            .file_name()
            .map(|f| f.to_string_lossy().to_lowercase());
        let to_name_lower = to_path
            .file_name()
            .map(|f| f.to_string_lossy().to_lowercase());
        if from_name == to_name_lower {
            from_path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            from_path.to_string_lossy().to_string()
        }
    } else {
        from_path.to_string_lossy().to_string()
    };

    let mut model = EventModel::new(op, nome.clone(), dir_atual.clone());
    model.dir_anterior = dir_anterior.clone();

    // Preserva metadados previamente conhecidos do caminho de origem caso o destino tenha sido movido/bloqueado
    preencher_metadados_item_excluido(from_path, &mut model);
    let perfil_origem = perfil_evento(&model);
    model.nome = nome;
    model.dir_anterior = dir_anterior;
    model.dir_atual = dir_atual;

    let is_dir_orig = tipo_eh_pasta(&model.tipo, &model.mime);
    if is_dir_orig || !extensao_tipo_preferencial(to_path).is_empty() {
        let (t, m) = obter_tipo_e_mime(to_path, is_dir_orig);
        model.tipo = t;
        model.mime = m;
    }
    if perfil_evento(&model) != perfil_origem {
        limpar_campos_conteudo(&mut model);
    }

    if let Some((hash, size_bytes, _)) = security_mgr.baseline().obter_registro(from_path) {
        if model.sha256_baseline.is_empty() {
            model.sha256_baseline = hash;
        }
        if model.size_b == 0.0 && size_bytes > 0 {
            model.size_b = size_bytes as f64;
            model.size_kb = model.size_b / 1024.0;
            model.size_mb = model.size_kb / 1024.0;
            model.size_gb = model.size_mb / 1024.0;
            model.size_tb = model.size_gb / 1024.0;
        }
    }

    extrair_metadados_ao_vivo(to_path, &mut model);
    security_mgr.enriquecer_evento(&mut model);

    model
}

/// Cria um EventModel padrão para Adicionado, Modificado, Excluido.
fn criar_evento_padrao(
    op: OperationType,
    path: &Path,
    security_mgr: &SecurityManager,
) -> EventModel {
    let nome = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut model = EventModel::new(op, nome, dir.clone());
    if op == OperationType::Excluido && model.dir_anterior.is_empty() {
        model.dir_anterior = dir;
    }

    if path.exists() && op != OperationType::Excluido {
        extrair_metadados_ao_vivo(path, &mut model);
    } else {
        // Complementa primeiro com histórico do baseline criptográfico se disponível
        if let Some((hash, size_bytes, data_reg)) = security_mgr.baseline().obter_registro(path) {
            if model.sha256_atual.is_empty() {
                model.sha256_atual = hash.clone();
            }
            model.sha256_baseline = hash;
            if model.size_b == 0.0 && size_bytes > 0 {
                model.size_b = size_bytes as f64;
                model.size_kb = model.size_b / 1024.0;
                model.size_mb = model.size_kb / 1024.0;
                model.size_gb = model.size_mb / 1024.0;
                model.size_tb = model.size_gb / 1024.0;
            }
            if !data_reg.is_empty() {
                model.data_modificacao = data_reg;
            }
        }
        preencher_metadados_item_excluido(path, &mut model);
    }
    security_mgr.enriquecer_evento(&mut model);

    model
}

impl Observador {
    pub fn novo(
        diretorio: PathBuf,
        security_mgr: Arc<SecurityManager>,
        event_sender: Sender<EventModel>,
    ) -> Result<Self, notify::Error> {
        let ativo = Arc::new(AtomicBool::new(true));
        let pausado = Arc::new(AtomicBool::new(false));

        let ativo_clone = ativo.clone();
        let pausado_clone = pausado.clone();

        let (notify_tx, notify_rx) = unbounded::<notify::Result<notify::Event>>();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = notify_tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(&diretorio, RecursiveMode::Recursive)?;
        security_mgr.configurar_diretorio_primario(&diretorio);

        get_logger().info(&format!(
            "Observador ativado para diretório: {:?}",
            diretorio
        ));

        let _dir_clone = diretorio.clone();
        let worker_handle = thread::spawn(move || {
            let mut recent_events: HashMap<PathBuf, Instant> = HashMap::new();
            let mut recent_renames: HashMap<PathBuf, Instant> = HashMap::new();
            let mut pending_renames: VecDeque<(PathBuf, Instant)> = VecDeque::new();
            let mut pending_removals: VecDeque<(PathBuf, Instant)> = VecDeque::new();

            while ativo_clone.load(Ordering::SeqCst) {
                match notify_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(Ok(event)) => {
                        if pausado_clone.load(Ordering::SeqCst) {
                            continue;
                        }

                        let now = Instant::now();
                        recent_events.retain(|_, time| {
                            now.duration_since(*time) < Duration::from_millis(500)
                        });
                        recent_renames.retain(|_, time| {
                            now.duration_since(*time) < Duration::from_millis(1000)
                        });

                        match event.kind {
                            // 1. Modificação de Nome (Renomeação / Movimentação nativa)
                            EventKind::Modify(ModifyKind::Name(rename_mode)) => {
                                match rename_mode {
                                    RenameMode::Both => {
                                        if event.paths.len() >= 2 {
                                            let from_path = &event.paths[0];
                                            let to_path = &event.paths[1];
                                            let model = criar_evento_rename_move(
                                                from_path,
                                                to_path,
                                                &security_mgr,
                                            );
                                            recent_renames.insert(to_path.clone(), now);
                                            recent_events.insert(to_path.clone(), now);
                                            let _ = event_sender.send(model);
                                        } else if let Some(path) = event.paths.first() {
                                            if path.exists() {
                                                if let Some(from_path) = localizar_registro_antigo(
                                                    &mut pending_renames,
                                                    path,
                                                ) {
                                                    let model = criar_evento_rename_move(
                                                        &from_path,
                                                        path,
                                                        &security_mgr,
                                                    );
                                                    recent_renames.insert(path.clone(), now);
                                                    recent_events.insert(path.clone(), now);
                                                    let _ = event_sender.send(model);
                                                } else {
                                                    let model = criar_evento_padrao(
                                                        OperationType::Adicionado,
                                                        path,
                                                        &security_mgr,
                                                    );
                                                    let _ = event_sender.send(model);
                                                }
                                            } else {
                                                pending_renames.push_back((path.clone(), now));
                                            }
                                        }
                                    }
                                    RenameMode::From => {
                                        for path in event.paths {
                                            pending_renames.push_back((path, now));
                                        }
                                    }
                                    RenameMode::To => {
                                        for path in event.paths {
                                            if let Some(from_path) = localizar_registro_antigo(
                                                &mut pending_renames,
                                                &path,
                                            ) {
                                                let model = criar_evento_rename_move(
                                                    &from_path,
                                                    &path,
                                                    &security_mgr,
                                                );
                                                recent_renames.insert(path.clone(), now);
                                                recent_events.insert(path.clone(), now);
                                                let _ = event_sender.send(model);
                                            } else {
                                                // Se não havia From correspondente, verifica se é movimentação via remoção inter-diretório
                                                let to_name = path
                                                    .file_name()
                                                    .map(|n| n.to_string_lossy().to_lowercase());
                                                let mut match_rem = None;
                                                if let Some(ref name) = to_name {
                                                    if let Some(pos) = pending_removals
                                                        .iter()
                                                        .position(|(from, _)| {
                                                            from.file_name()
                                                                .map(|f| {
                                                                    f.to_string_lossy()
                                                                        .to_lowercase()
                                                                })
                                                                .as_ref()
                                                                == Some(name)
                                                        })
                                                    {
                                                        match_rem = Some(
                                                            pending_removals.remove(pos).unwrap().0,
                                                        );
                                                    }
                                                }
                                                if let Some(from_path) = match_rem {
                                                    let model = criar_evento_rename_move(
                                                        &from_path,
                                                        &path,
                                                        &security_mgr,
                                                    );
                                                    recent_renames.insert(path.clone(), now);
                                                    recent_events.insert(path.clone(), now);
                                                    let _ = event_sender.send(model);
                                                } else {
                                                    let model = criar_evento_padrao(
                                                        OperationType::Adicionado,
                                                        &path,
                                                        &security_mgr,
                                                    );
                                                    let _ = event_sender.send(model);
                                                }
                                            }
                                        }
                                    }
                                    RenameMode::Any | RenameMode::Other => {
                                        if event.paths.len() >= 2 {
                                            let from_path = &event.paths[0];
                                            let to_path = &event.paths[1];
                                            let model = criar_evento_rename_move(
                                                from_path,
                                                to_path,
                                                &security_mgr,
                                            );
                                            recent_renames.insert(to_path.clone(), now);
                                            recent_events.insert(to_path.clone(), now);
                                            let _ = event_sender.send(model);
                                        } else if let Some(path) = event.paths.first() {
                                            if path.exists() {
                                                if let Some(from_path) = localizar_registro_antigo(
                                                    &mut pending_renames,
                                                    path,
                                                ) {
                                                    let model = criar_evento_rename_move(
                                                        &from_path,
                                                        path,
                                                        &security_mgr,
                                                    );
                                                    recent_renames.insert(path.clone(), now);
                                                    recent_events.insert(path.clone(), now);
                                                    let _ = event_sender.send(model);
                                                } else {
                                                    let model = criar_evento_padrao(
                                                        OperationType::Adicionado,
                                                        path,
                                                        &security_mgr,
                                                    );
                                                    let _ = event_sender.send(model);
                                                }
                                            } else {
                                                pending_renames.push_back((path.clone(), now));
                                            }
                                        }
                                    }
                                }
                            }

                            // 2. Eventos Modify com 2 caminhos (tratados como par de renomeação/movimentação)
                            EventKind::Modify(_) if event.paths.len() >= 2 => {
                                let from_path = &event.paths[0];
                                let to_path = &event.paths[1];
                                let model =
                                    criar_evento_rename_move(from_path, to_path, &security_mgr);
                                recent_renames.insert(to_path.clone(), now);
                                recent_events.insert(to_path.clone(), now);
                                let _ = event_sender.send(model);
                            }

                            // 3. Modificações comuns (conteúdo, metadados)
                            EventKind::Modify(_) => {
                                for path in event.paths {
                                    // Suprime eventos Modify imediatos decorrentes de renomeação recente (ruído do OS)
                                    if let Some(last_renamed) = recent_renames.get(&path) {
                                        if now.duration_since(*last_renamed)
                                            < Duration::from_millis(500)
                                        {
                                            continue;
                                        }
                                    }

                                    // Deduplicação por janela de tempo de 300ms
                                    if let Some(last_time) = recent_events.get(&path) {
                                        if now.duration_since(*last_time)
                                            < Duration::from_millis(300)
                                        {
                                            continue;
                                        }
                                    }
                                    recent_events.insert(path.clone(), now);

                                    let model = criar_evento_padrao(
                                        OperationType::Modificado,
                                        &path,
                                        &security_mgr,
                                    );
                                    let _ = event_sender.send(model);
                                }
                            }

                            // 4. Criação de arquivo (avalia reconciliação com remoção pendente para gerar Movido)
                            EventKind::Create(_) => {
                                for path in event.paths {
                                    let path_name = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_lowercase());
                                    let mut match_rem = None;
                                    if let Some(ref name) = path_name {
                                        if let Some(pos) =
                                            pending_removals.iter().position(|(from, _)| {
                                                !mesmo_diretorio(from, &path)
                                                    && from
                                                        .file_name()
                                                        .map(|f| f.to_string_lossy().to_lowercase())
                                                        .as_ref()
                                                        == Some(name)
                                            })
                                        {
                                            match_rem =
                                                Some(pending_removals.remove(pos).unwrap().0);
                                        }
                                    }

                                    if let Some(from_path) = match_rem {
                                        let model = criar_evento_rename_move(
                                            &from_path,
                                            &path,
                                            &security_mgr,
                                        );
                                        recent_renames.insert(path.clone(), now);
                                        recent_events.insert(path.clone(), now);
                                        let _ = event_sender.send(model);
                                    } else {
                                        if let Some(last_time) = recent_events.get(&path) {
                                            if now.duration_since(*last_time)
                                                < Duration::from_millis(300)
                                            {
                                                continue;
                                            }
                                        }

                                        let model = criar_evento_padrao(
                                            OperationType::Adicionado,
                                            &path,
                                            &security_mgr,
                                        );
                                        if (model.size_b > 0.0 && arquivo_acessivel(&path))
                                            || model.tipo == "folder"
                                        {
                                            recent_events.insert(path.clone(), now);
                                        }
                                        let _ = event_sender.send(model);
                                    }
                                }
                            }

                            // 5. Remoção de arquivo (enfileirado para reconciliação antes de emitir Excluído definitivo)
                            EventKind::Remove(_) => {
                                for path in event.paths {
                                    pending_removals.push_back((path, now));
                                }
                            }

                            _ => {}
                        }
                    }
                    Ok(Err(e)) => {
                        get_logger().warn(&format!("Erro de observação do sistema: {}", e));
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }

                // Esvaziamento de pendências expiradas (> 250ms sem correlação -> Excluído definitivo)
                let now = Instant::now();
                while let Some(front) = pending_removals.front() {
                    if now.duration_since(front.1) >= Duration::from_millis(250) {
                        let (path, _) = pending_removals.pop_front().unwrap();
                        let model =
                            criar_evento_padrao(OperationType::Excluido, &path, &security_mgr);
                        let _ = event_sender.send(model);
                    } else {
                        break;
                    }
                }

                // Renomeações pendentes sem destino (> 500ms -> Excluído por saída de escopo)
                while let Some(front) = pending_renames.front() {
                    if now.duration_since(front.1) >= Duration::from_millis(500) {
                        let (path, _) = pending_renames.pop_front().unwrap();
                        let model =
                            criar_evento_padrao(OperationType::Excluido, &path, &security_mgr);
                        let _ = event_sender.send(model);
                    } else {
                        break;
                    }
                }
            }

            // Descarrega quaisquer eventos pendentes ao finalizar
            for (path, _) in pending_removals {
                let model = criar_evento_padrao(OperationType::Excluido, &path, &security_mgr);
                let _ = event_sender.send(model);
            }
            for (path, _) in pending_renames {
                let model = criar_evento_padrao(OperationType::Excluido, &path, &security_mgr);
                let _ = event_sender.send(model);
            }

            get_logger().info("Thread observadora encerrada.");
        });

        Ok(Self {
            diretorio,
            ativo,
            pausado,
            watcher: Some(watcher),
            worker_handle: Some(worker_handle),
        })
    }

    pub fn diretorio(&self) -> &Path {
        &self.diretorio
    }

    pub fn pausar(&self) {
        self.pausado.store(true, Ordering::SeqCst);
        get_logger().info("Observador pausado.");
    }

    pub fn retomar(&self) {
        self.pausado.store(false, Ordering::SeqCst);
        get_logger().info("Observador retomado.");
    }

    pub fn is_pausado(&self) -> bool {
        self.pausado.load(Ordering::SeqCst)
    }

    pub fn parar(&mut self) {
        self.ativo.store(false, Ordering::SeqCst);
        if let Some(mut w) = self.watcher.take() {
            let _ = w.unwatch(&self.diretorio);
        }
        if let Some(h) = self.worker_handle.take() {
            let _ = h.join();
        }
        get_logger().info("Observador parado.");
    }
}

impl Drop for Observador {
    fn drop(&mut self) {
        self.parar();
    }
}
