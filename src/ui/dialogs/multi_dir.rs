use egui::{RichText, ScrollArea};

use crate::locale::Localizer;
use crate::security::multi_directory::MonitoringProfile;
use crate::security::MultiDiretorio;

pub struct MultiDirectoryDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub novo_nome_perfil: String,
    pub novos_diretorios: Vec<String>,
    pub perfis_cache: Vec<MonitoringProfile>,
    pub carregado: bool,
}

impl Default for MultiDirectoryDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            novo_nome_perfil: String::new(),
            novos_diretorios: Vec::new(),
            perfis_cache: Vec::new(),
            carregado: false,
        }
    }
}

impl MultiDirectoryDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        self.carregado = false; // Força recarregar
    }

    pub fn recarregar(&mut self, multi_dir: &MultiDiretorio) {
        self.perfis_cache = multi_dir.listar_perfis();
        self.carregado = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, multi_dir: &MultiDiretorio) {
        if !self.aberto {
            return;
        }

        if !self.carregado {
            self.recarregar(multi_dir);
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        super::show_modeless_dialog(
            ctx,
            "dialog_multi_dir",
            format!("📁 {}", loc.t("security_manage_directories")),
            [780.0, 320.0],
            [680.0, 220.0],
            &mut state,
            |ui, fechar| {
                ui.label(RichText::new(loc.t("multi_dir_description")).size(12.0));
                ui.separator();

                ui.heading(RichText::new(loc.t("create_new_profile")).size(13.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new(loc.t("profile_name")).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.novo_nome_perfil)
                            .desired_width(180.0),
                    );

                    if ui.button(loc.t("add_folder_to_profile")).clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            let p = folder.to_string_lossy().to_string();
                            if !self.novos_diretorios.contains(&p) {
                                self.novos_diretorios.push(p);
                            }
                        }
                    }

                    if ui.button(loc.t("save_profile")).clicked()
                        && !self.novo_nome_perfil.is_empty()
                    {
                        let _ = multi_dir.salvar_perfil(
                            &self.novo_nome_perfil,
                            &self.novos_diretorios,
                            true,
                        );
                        self.novo_nome_perfil.clear();
                        self.novos_diretorios.clear();
                        self.recarregar(multi_dir);
                    }
                    ui.add_space(8.0);
                });

                if !self.novos_diretorios.is_empty() {
                    ui.label(RichText::new(loc.t("selected_folders")).size(11.5));
                    for d in &self.novos_diretorios {
                        ui.label(RichText::new(format!("• {}", d)).size(11.0));
                    }
                }

                ui.separator();
                ui.heading(RichText::new(loc.t("existing_profiles")).size(13.0));

                let scroll_h = (ui.available_height() - 40.0).max(70.0);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(scroll_h)
                    .show(ui, |ui| {
                        for perfil in &self.perfis_cache {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&perfil.nome).strong().size(12.0));
                                    if perfil.ativo {
                                        ui.label(
                                            RichText::new(loc.t("active_badge"))
                                                .color(egui::Color32::from_rgb(76, 175, 80))
                                                .size(11.5),
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button(loc.t("delete")).clicked() {
                                                let _ = multi_dir.remover_perfil(perfil.id);
                                                // Recarrega no próximo frame
                                                self.carregado = false;
                                            }
                                        },
                                    );
                                });
                                for d in &perfil.diretorios {
                                    ui.label(RichText::new(format!("• {}", d)).size(11.0));
                                }
                            });
                            ui.add_space(2.0);
                        }
                    });

                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button(loc.t("security_refresh")).clicked() {
                        self.recarregar(multi_dir);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
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
