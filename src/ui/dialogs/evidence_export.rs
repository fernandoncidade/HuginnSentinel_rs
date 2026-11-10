use egui::{Color32, RichText, ScrollArea};
use std::path::{Path, PathBuf};

use crate::core::event::EventModel;
use crate::locale::Localizer;
use crate::security::SecurityManager;
use crate::utils::paths::obter_caminho_persistente;

pub struct EvidenceDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub destino_personalizado: String,
    pub ultimo_pacote_gerado: Option<PathBuf>,
    pub status_resultado: Option<(String, bool)>,
    pub texto_relatorio: String,
    pub ultimo_total_eventos: usize,
}

impl Default for EvidenceDialog {
    fn default() -> Self {
        let default_dest = obter_caminho_persistente()
            .join(format!(
                "evidencia_huginn_{}.zip",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            ))
            .to_string_lossy()
            .to_string();

        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            destino_personalizado: default_dest,
            ultimo_pacote_gerado: None,
            status_resultado: None,
            texto_relatorio: String::new(),
            ultimo_total_eventos: usize::MAX,
        }
    }
}

impl EvidenceDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        self.ultimo_total_eventos = usize::MAX; // Força recálculo
    }

    pub fn atualizar_relatorio(&mut self, loc: &Localizer, sec_mgr: &SecurityManager, eventos: &[EventModel]) {
        let perfis = sec_mgr.multi_dir().listar_perfis();
        let perfil_ativo = perfis.iter().find(|p| p.ativo);
        let nome_perfil = perfil_ativo
            .map(|p| p.nome.as_str())
            .unwrap_or_else(|| loc.t("security_no_selection").leak());
        let total_diretorios = perfil_ativo
            .map(|p| p.diretorios.len())
            .unwrap_or(0);

        let mut lines = Vec::new();
        lines.push(format!("{}: SQLite", loc.t("security_active_backend").replace("%1", "SQLite")));
        if !self.destino_personalizado.is_empty() {
            lines.push(format!("{}: {}", loc.t("security_destination"), self.destino_personalizado));
        }
        lines.push(format!("{}: {}", loc.t("security_evidence_center"), eventos.len()));

        if let Some(ref ult) = self.ultimo_pacote_gerado {
            lines.push(format!("{}: {}", loc.t("security_last_package"), ult.display()));
        }

        if eventos.is_empty() {
            lines.push(String::new());
            lines.push(loc.t("security_no_evidence_recorded"));
        } else {
            lines.push(String::new());
            lines.push(loc.t("security_recent_records"));
            let total = eventos.len();
            let inicio = total.saturating_sub(25);
            for (idx, ev) in eventos[inicio..total].iter().rev().enumerate() {
                let ev_id = if !ev.evidence_id.is_empty() {
                    &ev.evidence_id
                } else {
                    "EV-AUTO"
                };
                let risco = loc.traduzir_risco(&ev.risk_level);
                lines.push(format!(
                    "[{}] {} | {} | {} | Risco: {}",
                    ev.timestamp, ev_id, ev.nome, ev.tipo_operacao, risco
                ));
                if idx >= 20 {
                    break;
                }
            }
        }

        self.texto_relatorio = lines.join("\n");
        self.ultimo_total_eventos = eventos.len();
        let _ = (nome_perfil, total_diretorios);
    }

    pub fn mostrar(
        &mut self,
        ctx: &egui::Context,
        loc: &Localizer,
        sec_mgr: &SecurityManager,
        eventos: &[EventModel],
    ) {
        if !self.aberto {
            return;
        }

        if self.ultimo_total_eventos != eventos.len() || self.texto_relatorio.is_empty() {
            self.atualizar_relatorio(loc, sec_mgr, eventos);
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        super::show_modeless_dialog(
            ctx,
            "dialog_evidence_export",
            format!("📦 {}", loc.t("security_evidence_center")),
            [580.0, 390.0],
            [480.0, 280.0],
            &mut state,
            |ui, fechar| {
                // 1. Resumo no Topo (Paridade C++: Nome do perfil: ... | Total de diretórios: ... | Backend: SQLite)
                let perfis = sec_mgr.multi_dir().listar_perfis();
                let perfil_ativo = perfis.iter().find(|p| p.ativo);
                let nome_perfil = perfil_ativo
                    .map(|p| p.nome.as_str())
                    .unwrap_or("Padrão");
                let count_dirs = perfil_ativo
                    .map(|p| p.diretorios.len())
                    .unwrap_or(1);

                ui.horizontal(|ui| {
                    let summary_text = format!(
                        "{}: {} | {}: {} | Backend: SQLite",
                        loc.t("security_profile_name"),
                        nome_perfil,
                        loc.t("security_total_directories"),
                        count_dirs
                    );
                    ui.label(RichText::new(summary_text).strong().size(13.0));
                });

                ui.separator();

                // 2. Destino do Pacote (Paridade C++: Label + TextEdit + Selecionar Destino)
                ui.label(RichText::new(loc.t("security_destination")).strong());
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.destino_personalizado)
                            .desired_width(ui.available_width() - 170.0),
                    );

                    if ui.button(loc.t("security_select_destination")).clicked() {
                        let default_name = format!(
                            "evidencia_huginn_{}.zip",
                            chrono::Local::now().format("%Y%m%d_%H%M%S")
                        );
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(&default_name)
                            .add_filter("ZIP", &["zip"])
                            .save_file()
                        {
                            self.destino_personalizado = path.to_string_lossy().to_string();
                        }
                    }
                });

                ui.add_space(4.0);

                // Mensagem de Status de Exportação
                if let Some((ref msg, is_err)) = self.status_resultado {
                    let color = if is_err {
                        Color32::from_rgb(244, 67, 54)
                    } else {
                        Color32::from_rgb(76, 175, 80)
                    };
                    ui.colored_label(color, RichText::new(msg).strong());
                    ui.add_space(2.0);
                }

                // 3. Área de Log / Detalhes das Evidências (Result Browser do C++)
                let log_h = (ui.available_height() - 48.0).max(110.0);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(log_h)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.texto_relatorio.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(ui.available_width())
                                .desired_rows(8),
                        );
                    });

                ui.separator();
                ui.add_space(4.0);

                // 4. Barra de Botões (Paridade C++: Exportar agora, Abrir pasta selecionada, Atualizar, Fechar)
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_sized(
                            [110.0, 25.0],
                            egui::Button::new(RichText::new(loc.t("security_export_now")).strong().size(12.0)),
                        )
                        .clicked()
                    {
                        let path_alvo = if self.destino_personalizado.trim().is_empty() {
                            None
                        } else {
                            Some(Path::new(&self.destino_personalizado))
                        };

                        match sec_mgr.exportar_evidencia(eventos, path_alvo) {
                            Ok(p) => {
                                self.ultimo_pacote_gerado = Some(p.clone());
                                self.status_resultado = Some((
                                    format!("{}\n{:?}", loc.t("security_export_success"), p),
                                    false,
                                ));
                                self.atualizar_relatorio(loc, sec_mgr, eventos);
                            }
                            Err(e) => {
                                self.status_resultado = Some((
                                    format!("{}: {}", loc.t("security_operation_failed"), e),
                                    true,
                                ));
                            }
                        }
                    }

                    let tem_pacote = self.ultimo_pacote_gerado.is_some()
                        || Path::new(&self.destino_personalizado).exists();

                    if ui
                        .add_enabled(
                            tem_pacote,
                            egui::Button::new(RichText::new(loc.t("security_open_selected_folder")).size(12.0)),
                        )
                        .clicked()
                    {
                        let target_path = if let Some(ref ult) = self.ultimo_pacote_gerado {
                            ult.to_string_lossy().to_string()
                        } else {
                            self.destino_personalizado.clone()
                        };
                        crate::ui::dialogs::recovery::revelar_no_explorer(&target_path);
                    }

                    if ui
                        .add_sized([80.0, 25.0], egui::Button::new(RichText::new(loc.t("security_refresh")).size(12.0)))
                        .clicked()
                    {
                        self.atualizar_relatorio(loc, sec_mgr, eventos);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [80.0, 25.0],
                                egui::Button::new(RichText::new(loc.t("security_close")).size(12.0)),
                            )
                            .clicked()
                        {
                            *fechar = true;
                        }
                    });
                });
            },
        );

        self.aberto = state.aberto;
        self.elevar_z_order = state.elevar_z_order;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
    }
}
