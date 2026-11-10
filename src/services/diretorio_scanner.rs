use crossbeam_channel::{bounded, Sender};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::metadados::extrair_metadados_scan;
use crate::core::event::EventModel;
use crate::core::types::OperationType;
use crate::security::SecurityManager;
use crate::utils::logger::get_logger;
use parking_lot::Mutex;

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub itens_escaneados: usize,
    pub total_estimado: usize,
    pub progresso_pct: f32,
    pub item_atual: String,
    pub concluido: bool,
}

pub struct DiretorioScanner {
    cancelar: Arc<AtomicBool>,
    pausado: Arc<AtomicBool>,
    _handle: Option<JoinHandle<()>>,
}

impl DiretorioScanner {
    pub fn iniciar_scan(
        diretorio: PathBuf,
        security_mgr: Arc<SecurityManager>,
        event_sender: Sender<EventModel>,
        progress_sender: Sender<ScanProgress>,
    ) -> Self {
        let cancelar = Arc::new(AtomicBool::new(false));
        let pausado = Arc::new(AtomicBool::new(false));

        let cancelar_clone = cancelar.clone();
        let pausado_clone = pausado.clone();

        let handle = thread::spawn(move || {
            security_mgr.configurar_diretorio_primario(&diretorio);
            get_logger().info(&format!(
                "Iniciando varredura recursiva em: {:?}",
                diretorio
            ));

            // 1. Coleta rápida de arquivos
            let mut caminhos = Vec::new();
            coletar_arquivos(&diretorio, &mut caminhos, &cancelar_clone);

            let total = caminhos.len();
            get_logger().info(&format!(
                "Varredura identificou {} arquivos para processamento",
                total
            ));

            if total == 0 || cancelar_clone.load(Ordering::SeqCst) {
                let _ = progress_sender.send(ScanProgress {
                    itens_escaneados: 0,
                    total_estimado: 0,
                    progresso_pct: 100.0,
                    item_atual: if cancelar_clone.load(Ordering::SeqCst) {
                        "Varredura cancelada".to_string()
                    } else {
                        "Concluído".to_string()
                    },
                    concluido: true,
                });
                return;
            }

            // Paralelismo de varredura (2 a 8 threads conforme núcleos disponíveis)
            let num_workers = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
                .clamp(2, 8);

            let (path_tx, path_rx) = bounded::<PathBuf>(1024);
            let processed_counter = Arc::new(AtomicUsize::new(0));
            let last_progress = Arc::new(Mutex::new(Instant::now()));
            let mut worker_handles = Vec::with_capacity(num_workers);

            for _ in 0..num_workers {
                let rx = path_rx.clone();
                let event_tx = event_sender.clone();
                let progress_tx = progress_sender.clone();
                let sec_mgr = security_mgr.clone();
                let cancel = cancelar_clone.clone();
                let pause = pausado_clone.clone();
                let counter = processed_counter.clone();
                let last_prog = last_progress.clone();

                let handle = thread::spawn(move || loop {
                    if cancel.load(Ordering::SeqCst) {
                        break;
                    }

                    while pause.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_millis(50));
                        if cancel.load(Ordering::SeqCst) {
                            break;
                        }
                    }

                    let path = match rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(p) => p,
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    };

                    let nome = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let dir = path
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let mut evento = EventModel::new(OperationType::Escaneado, nome, dir);
                    extrair_metadados_scan(&path, &mut evento);
                    sec_mgr.enriquecer_evento_scan(&mut evento);

                    let _ = event_tx.send(evento);
                    let current = counter.fetch_add(1, Ordering::Relaxed) + 1;

                    if current % 5 == 0 || current == total {
                        let mut lock = last_prog.lock();
                        if lock.elapsed() >= Duration::from_millis(20) {
                            *lock = Instant::now();
                            let pct = if total > 0 {
                                (current as f32 / total as f32) * 100.0
                            } else {
                                100.0
                            };
                            let _ = progress_tx.send(ScanProgress {
                                itens_escaneados: current,
                                total_estimado: total,
                                progresso_pct: pct.min(99.9),
                                item_atual: format!("{} / {}", current, total),
                                concluido: false,
                            });
                        }
                    }
                });

                worker_handles.push(handle);
            }
            drop(path_rx); // Permite aos workers drenar e sair quando path_tx fechar

            for path in caminhos {
                if cancelar_clone.load(Ordering::SeqCst) {
                    break;
                }

                while pausado_clone.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(50));
                    if cancelar_clone.load(Ordering::SeqCst) {
                        break;
                    }
                }

                loop {
                    if cancelar_clone.load(Ordering::SeqCst) {
                        break;
                    }
                    match path_tx.send_timeout(path.clone(), Duration::from_millis(50)) {
                        Ok(()) => break,
                        Err(crossbeam_channel::SendTimeoutError::Timeout(_)) => continue,
                        Err(crossbeam_channel::SendTimeoutError::Disconnected(_)) => break,
                    }
                }
            }
            drop(path_tx);

            for h in worker_handles {
                let _ = h.join();
            }

            let final_count = processed_counter.load(Ordering::SeqCst);
            let is_cancelled = cancelar_clone.load(Ordering::SeqCst);

            let _ = progress_sender.send(ScanProgress {
                itens_escaneados: final_count,
                total_estimado: total,
                progresso_pct: 100.0,
                item_atual: if is_cancelled {
                    "Varredura cancelada".to_string()
                } else {
                    "Concluído".to_string()
                },
                concluido: true,
            });

            if is_cancelled {
                get_logger().info(&format!(
                    "Varredura cancelada. {} itens processados.",
                    final_count
                ));
            } else {
                get_logger().info(&format!(
                    "Varredura concluída. {} itens processados.",
                    final_count
                ));
            }
        });

        Self {
            cancelar,
            pausado,
            _handle: Some(handle),
        }
    }

    pub fn cancelar(&self) {
        self.cancelar.store(true, Ordering::SeqCst);
    }

    pub fn pausar(&self) {
        self.pausado.store(true, Ordering::SeqCst);
    }

    pub fn retomar(&self) {
        self.pausado.store(false, Ordering::SeqCst);
    }

    pub fn is_pausado(&self) -> bool {
        self.pausado.load(Ordering::SeqCst)
    }
}

impl Drop for DiretorioScanner {
    fn drop(&mut self) {
        self.cancelar.store(true, Ordering::SeqCst);
        self.pausado.store(false, Ordering::SeqCst);
    }
}

fn coletar_arquivos(dir: &Path, lista: &mut Vec<PathBuf>, cancelar: &Arc<AtomicBool>) {
    if cancelar.load(Ordering::SeqCst) {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        if cancelar.load(Ordering::SeqCst) {
            return;
        }

        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_symlink() {
            continue;
        }

        let path = entry.path();
        if ft.is_dir() {
            lista.push(path.clone());
            coletar_arquivos(&path, lista, cancelar);
        } else if ft.is_file() {
            lista.push(path);
        }
    }
}
