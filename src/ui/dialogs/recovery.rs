use egui::{Color32, RichText, ScrollArea, Vec2};
use std::path::Path;

use crate::locale::Localizer;
use crate::security::baseline::BaselineItem;
use crate::security::quarantine::QuarantinedItem;
use crate::security::SecurityManager;

pub fn revelar_no_explorer(caminho: &str) {
    if caminho.is_empty() {
        return;
    }
    let p = Path::new(caminho);
    if p.exists() {
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", p.display()))
            .spawn();
    } else if let Some(parent) = p.parent() {
        if parent.exists() {
            let _ = std::process::Command::new("explorer")
                .arg(parent)
                .spawn();
        }
    }
}

pub struct RecoveryDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub tab_ativa: usize, // 0: Quarentena, 1: Versões monitoradas (Baseline)
    pub sel_quarentena: Option<usize>,
    pub sel_baseline: Option<usize>,
    pub cache_quarentena: Vec<QuarantinedItem>,
    pub cache_baseline: Vec<BaselineItem>,
    pub carregado: bool,
    status_msg: Option<(String, bool)>, // (mensagem, is_error)
}

impl Default for RecoveryDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            tab_ativa: 0,
            sel_quarentena: None,
            sel_baseline: None,
            cache_quarentena: Vec::new(),
            cache_baseline: Vec::new(),
            carregado: false,
            status_msg: None,
        }
    }
}

impl RecoveryDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        self.carregado = false; // Força recarregar do banco
    }

    pub fn recarregar(&mut self, sec_mgr: &SecurityManager) {
        self.cache_quarentena = sec_mgr.quarentena().listar();
        self.cache_baseline = sec_mgr.baseline().listar();
        self.carregado = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, sec_mgr: &SecurityManager) {
        if !self.aberto {
            return;
        }

        if !self.carregado {
            self.recarregar(sec_mgr);
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        super::show_modeless_dialog(
            ctx,
            "dialog_recovery",
            format!("🔒 {}", loc.t("security_recovery_center")),
            [740.0, 320.0],
            [580.0, 210.0],
            &mut state,
            |ui, fechar| {
                // 1. Resumo no Topo (Paridade C++: Quarentena: X itens | Versões monitoradas: Y itens | Backend: SQLite)
                ui.horizontal(|ui| {
                    let total_q = self.cache_quarentena.len();
                    let total_b = self.cache_baseline.len();
                    let unit = loc.t("security_popup_unit_quarantine");
                    let summary_text = format!(
                        "{}: {} {} | {}: {} {} | Backend: SQLite",
                        loc.t("security_quarantine_status"),
                        total_q,
                        unit,
                        loc.t("security_baseline_status"),
                        total_b,
                        unit
                    );
                    ui.label(RichText::new(summary_text).strong().size(13.0));
                });

                ui.separator();

                // Mensagem de Status / Feedback
                if let Some((ref msg, is_err)) = self.status_msg {
                    let color = if is_err {
                        Color32::from_rgb(244, 67, 54)
                    } else {
                        Color32::from_rgb(76, 175, 80)
                    };
                    ui.colored_label(color, RichText::new(msg).strong());
                    ui.add_space(2.0);
                }

                // 2. Abas: Quarentena e Versões Monitoradas
                ui.horizontal(|ui| {
                    let tab0_txt = format!("🛡️ {}", loc.t("security_quarantine_status"));
                    let tab1_txt = format!("📜 {}", loc.t("security_baseline_status"));
                    if ui.selectable_label(self.tab_ativa == 0, RichText::new(tab0_txt).strong()).clicked() {
                        self.tab_ativa = 0;
                    }
                    if ui.selectable_label(self.tab_ativa == 1, RichText::new(tab1_txt).strong()).clicked() {
                        self.tab_ativa = 1;
                    }
                });

                ui.separator();

                if self.tab_ativa == 0 {
                    // === ABA 1: QUARENTENA ===
                    if self.cache_quarentena.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(30.0);
                            ui.label(
                                RichText::new(loc.t("no_files_in_quarantine"))
                                    .color(ui.visuals().weak_text_color())
                                    .size(14.0),
                            );
                        });
                    } else {
                        let larguras = [115.0, 75.0, 160.0, 140.0, 100.0, 80.0];
                        let total_table_w: f32 = larguras.iter().sum();
                        let row_height = 24.0;
                        let num_rows = self.cache_quarentena.len();
                        let selected_idx = self.sel_quarentena;

                        let header_color = if ui.visuals().dark_mode {
                            Color32::from_rgb(180, 200, 220)
                        } else {
                            Color32::from_rgb(40, 60, 90)
                        };
                        let titulos = [
                            loc.t("security_created_at"),
                            loc.t("security_quarantine_status"),
                            loc.t("security_path"),
                            loc.t("security_snapshot_path"),
                            loc.t("security_sha256_current"),
                            loc.t("security_evidence_id"),
                        ];

                        let table_h = (ui.available_height() - 75.0).max(70.0);

                        ScrollArea::both()
                            .auto_shrink([false, false])
                            .max_height(table_h)
                            .show(ui, |ui| {
                                ui.set_min_width(total_table_w);

                                // Cabeçalho da tabela
                                ui.horizontal(|ui| {
                                    for (i, &w) in larguras.iter().enumerate() {
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(w, 20.0),
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.label(RichText::new(&titulos[i]).strong().color(header_color).size(11.5));
                                            },
                                        );
                                    }
                                });
                                ui.separator();

                                for row_idx in 0..num_rows {
                                    let item = &self.cache_quarentena[row_idx];
                                    let is_selected = selected_idx == Some(row_idx);

                                    let row_resp = ui.horizontal(|ui| {
                                        let bg_color = if is_selected {
                                            if ui.visuals().dark_mode {
                                                Color32::from_rgba_premultiplied(40, 70, 110, 120)
                                            } else {
                                                Color32::from_rgba_premultiplied(200, 225, 255, 120)
                                            }
                                        } else if row_idx % 2 == 1 {
                                            if ui.visuals().dark_mode {
                                                Color32::from_rgba_premultiplied(35, 35, 35, 60)
                                            } else {
                                                Color32::from_rgba_premultiplied(245, 245, 245, 80)
                                            }
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        let rect = egui::Rect::from_min_size(
                                            ui.cursor().min,
                                            Vec2::new(total_table_w, row_height),
                                        );
                                        ui.painter().rect_filled(rect, 0.0, bg_color);

                                        // Col 0: Criado em
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[0], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.data_quarentena).size(11.0));
                                        });
                                        // Col 1: Status
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[1], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.colored_label(Color32::from_rgb(255, 152, 0), RichText::new(&item.status).strong().size(11.0));
                                        });
                                        // Col 2: Caminho
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[2], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.caminho_original).size(10.5));
                                        });
                                        // Col 3: Snapshot em Quarentena
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[3], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.caminho_quarentena).size(10.5).color(ui.visuals().weak_text_color()));
                                        });
                                        // Col 4: SHA-256
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[4], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.hash_sha256).size(10.0));
                                        });
                                        // Col 5: Evidência
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[5], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.evidence_id).size(11.0));
                                        });
                                    });

                                    if row_resp.response.interact(egui::Sense::click()).clicked() {
                                        self.sel_quarentena = Some(row_idx);
                                    }
                                }
                            });

                        // Botões de Ação da Quarentena (Paridade C++)
                        ui.separator();
                        ui.horizontal(|ui| {
                            let tem_sel = self.sel_quarentena.is_some();

                            if ui.add_enabled(tem_sel, egui::Button::new(loc.t("security_open_selected_folder"))).clicked() {
                                if let Some(idx) = self.sel_quarentena {
                                    if let Some(item) = self.cache_quarentena.get(idx) {
                                        revelar_no_explorer(&item.caminho_quarentena);
                                    }
                                }
                            }

                            if ui.add_enabled(tem_sel, egui::Button::new(loc.t("security_restore_selected_copy"))).clicked() {
                                if let Some(idx) = self.sel_quarentena {
                                    if let Some(item) = self.cache_quarentena.get(idx) {
                                        match sec_mgr.quarentena().restaurar(item.id, None) {
                                            Ok(()) => {
                                                self.status_msg = Some((loc.t("security_restore_success"), false));
                                                self.recarregar(sec_mgr);
                                                self.sel_quarentena = None;
                                            }
                                            Err(e) => {
                                                self.status_msg = Some((e, true));
                                            }
                                        }
                                    }
                                }
                            }

                            if ui.add_enabled(tem_sel, egui::Button::new(loc.t("restore_to_folder"))).clicked() {
                                if let Some(idx) = self.sel_quarentena {
                                    if let Some(item) = self.cache_quarentena.get(idx) {
                                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                            let file_name = Path::new(&item.caminho_original)
                                                .file_name()
                                                .map(|f| f.to_string_lossy().to_string())
                                                .unwrap_or_else(|| "arquivo_restaurado.bin".to_string());
                                            let target = folder.join(file_name);
                                            match sec_mgr.quarentena().restaurar(item.id, Some(&target)) {
                                                Ok(()) => {
                                                    let target_str = target.to_string_lossy();
                                                    self.status_msg = Some((loc.t_args("restore_success", &[&target_str]), false));
                                                    self.recarregar(sec_mgr);
                                                    self.sel_quarentena = None;
                                                }
                                                Err(e) => {
                                                    self.status_msg = Some((e, true));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                } else {
                    // === ABA 2: VERSÕES MONITORADAS (BASELINE) ===
                    if self.cache_baseline.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(30.0);
                            ui.label(
                                RichText::new(loc.t("no_files_found"))
                                    .color(ui.visuals().weak_text_color())
                                    .size(14.0),
                            );
                        });
                    } else {
                        let larguras = [115.0, 170.0, 100.0, 110.0, 65.0, 110.0];
                        let total_table_w: f32 = larguras.iter().sum();
                        let row_height = 24.0;
                        let num_rows = self.cache_baseline.len();
                        let selected_idx = self.sel_baseline;

                        let header_color = if ui.visuals().dark_mode {
                            Color32::from_rgb(180, 200, 220)
                        } else {
                            Color32::from_rgb(40, 60, 90)
                        };
                        let titulos = [
                            loc.t("security_updated_at"),
                            loc.t("security_path"),
                            loc.t("security_sha256_current"),
                            loc.t("security_publisher_name"),
                            loc.t("signed"),
                            loc.t("security_snapshot_path"),
                        ];

                        let table_h = (ui.available_height() - 75.0).max(70.0);

                        ScrollArea::both()
                            .auto_shrink([false, false])
                            .max_height(table_h)
                            .show(ui, |ui| {
                                ui.set_min_width(total_table_w);

                                // Cabeçalho da tabela
                                ui.horizontal(|ui| {
                                    for (i, &w) in larguras.iter().enumerate() {
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(w, 20.0),
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.label(RichText::new(&titulos[i]).strong().color(header_color).size(11.5));
                                            },
                                        );
                                    }
                                });
                                ui.separator();

                                for row_idx in 0..num_rows {
                                    let item = &self.cache_baseline[row_idx];
                                    let is_selected = selected_idx == Some(row_idx);

                                    let row_resp = ui.horizontal(|ui| {
                                        let bg_color = if is_selected {
                                            if ui.visuals().dark_mode {
                                                Color32::from_rgba_premultiplied(40, 70, 110, 120)
                                            } else {
                                                Color32::from_rgba_premultiplied(200, 225, 255, 120)
                                            }
                                        } else if row_idx % 2 == 1 {
                                            if ui.visuals().dark_mode {
                                                Color32::from_rgba_premultiplied(35, 35, 35, 60)
                                            } else {
                                                Color32::from_rgba_premultiplied(245, 245, 245, 80)
                                            }
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        let rect = egui::Rect::from_min_size(
                                            ui.cursor().min,
                                            Vec2::new(total_table_w, row_height),
                                        );
                                        ui.painter().rect_filled(rect, 0.0, bg_color);

                                        // Col 0: Atualizado em
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[0], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.data_registro).size(11.0));
                                        });
                                        // Col 1: Caminho
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[1], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.caminho).size(10.5));
                                        });
                                        // Col 2: SHA-256
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[2], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.hash_sha256).size(10.0));
                                        });
                                        // Col 3: Publicador
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[3], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.publisher).size(10.5));
                                        });
                                        // Col 4: Assinado
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[4], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            let signed_text = if !item.signature_state.is_empty() {
                                                item.signature_state.clone()
                                            } else {
                                                "-".to_string()
                                            };
                                            ui.label(RichText::new(signed_text).size(10.5));
                                        });
                                        // Col 5: Snapshot
                                        ui.allocate_ui_with_layout(Vec2::new(larguras[5], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(&item.snapshot_path).size(10.5).color(ui.visuals().weak_text_color()));
                                        });
                                    });

                                    if row_resp.response.interact(egui::Sense::click()).clicked() {
                                        self.sel_baseline = Some(row_idx);
                                    }
                                }
                            });

                        // Botões de Ação do Baseline (Paridade C++)
                        ui.separator();
                        ui.horizontal(|ui| {
                            let tem_sel = self.sel_baseline.is_some();

                            if ui.add_enabled(tem_sel, egui::Button::new(loc.t("security_open_selected_folder"))).clicked() {
                                if let Some(idx) = self.sel_baseline {
                                    if let Some(item) = self.cache_baseline.get(idx) {
                                        revelar_no_explorer(&item.caminho);
                                    }
                                }
                            }

                            if ui.add_enabled(tem_sel, egui::Button::new(loc.t("security_restore_selected_version"))).clicked() {
                                if let Some(idx) = self.sel_baseline {
                                    if let Some(item) = self.cache_baseline.get(idx) {
                                        match sec_mgr.baseline().restaurar_snapshot(item.id, None) {
                                            Ok(()) => {
                                                self.status_msg = Some((loc.t("security_restore_success"), false));
                                                self.recarregar(sec_mgr);
                                                self.sel_baseline = None;
                                            }
                                            Err(e) => {
                                                self.status_msg = Some((e, true));
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                }

                ui.separator();
                ui.add_space(4.0);

                // 3. Botões de Rodapé: Atualizar e Fechar
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [90.0, 26.0],
                            egui::Button::new(RichText::new(loc.t("security_refresh")).size(12.5)),
                        )
                        .clicked()
                    {
                        self.recarregar(sec_mgr);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [85.0, 26.0],
                                egui::Button::new(RichText::new(loc.t("security_close")).size(12.5)),
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
