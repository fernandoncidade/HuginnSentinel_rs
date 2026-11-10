use egui::{Color32, RichText, ScrollArea, Vec2};

use crate::core::event::EventModel;
use crate::locale::Localizer;

#[derive(Debug, Clone)]
pub struct AlertRowData {
    pub id: i64,
    pub created_at: String,
    pub risk_level: String,
    pub risk_score: i32,
    pub summary: String,
    pub rules: String,
    pub caminho: String,
    pub nome: String,
    pub session_id: String,
    pub evidence_id: String,
    pub sha256: String,
    pub actor_process: String,
}

pub struct AlertCenterDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub filtro_busca: String,
    pub alerta_selecionado: Option<usize>,
    pub cache_alertas: Vec<AlertRowData>,
    pub ultimo_total_eventos: usize,
}

impl Default for AlertCenterDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            filtro_busca: String::new(),
            alerta_selecionado: None,
            cache_alertas: Vec::new(),
            ultimo_total_eventos: 0,
        }
    }
}

impl AlertCenterDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        // Força recarregamento ao reabrir
        self.ultimo_total_eventos = usize::MAX;
    }

    pub fn recarregar_cache(&mut self, eventos: &[EventModel]) {
        let q = self.filtro_busca.trim().to_lowercase();
        let mut itens = Vec::new();

        for (idx, ev) in eventos.iter().enumerate() {
            if ev.risk_score > 0 {
                let match_busca = if q.is_empty() {
                    true
                } else {
                    ev.nome.to_lowercase().contains(&q)
                        || ev.full_path().to_lowercase().contains(&q)
                        || ev.alerta_resumo.to_lowercase().contains(&q)
                        || ev.risk_reasons.to_lowercase().contains(&q)
                        || ev.session_id.to_lowercase().contains(&q)
                };

                if match_busca {
                    let ev_id = if !ev.evidence_id.is_empty() {
                        ev.evidence_id.clone()
                    } else {
                        format!("EV-{:05}", idx + 1)
                    };

                    let sess_id = if !ev.session_id.is_empty() {
                        ev.session_id.clone()
                    } else {
                        "SESS-01".to_string()
                    };

                    itens.push(AlertRowData {
                        id: (idx + 1) as i64,
                        created_at: ev.timestamp.clone(),
                        risk_level: ev.risk_level.clone(),
                        risk_score: ev.risk_score,
                        summary: if ev.alerta_resumo.is_empty() {
                            ev.risk_reasons.clone()
                        } else {
                            ev.alerta_resumo.clone()
                        },
                        rules: ev.risk_reasons.clone(),
                        caminho: ev.full_path(),
                        nome: ev.nome.clone(),
                        session_id: sess_id,
                        evidence_id: ev_id,
                        sha256: ev.sha256_atual.clone(),
                        actor_process: ev.actor_process.clone(),
                    });
                }
            }
        }

        // Mais recentes no topo
        itens.reverse();
        self.cache_alertas = itens;
        self.ultimo_total_eventos = eventos.len();
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, eventos: &[EventModel]) {
        if !self.aberto {
            return;
        }

        if self.cache_alertas.is_empty() || self.ultimo_total_eventos != eventos.len() {
            self.recarregar_cache(eventos);
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        super::show_modeless_dialog(
            ctx,
            "dialog_alerts",
            format!("🛡️ {}", loc.t("security_alert_center")),
            [840.0, 320.0],
            [660.0, 210.0],
            &mut state,
            |ui, fechar| {
                // 1. Cabeçalho Resumo e Barra de Busca
                ui.horizontal(|ui| {
                    let total_str = self.cache_alertas.len().to_string();
                    let summary_text = format!(
                        "{}: {} | Backend: SQLite",
                        loc.t("security_alert_center"),
                        total_str
                    );
                    ui.label(RichText::new(summary_text).strong().size(13.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let busca_antiga = self.filtro_busca.clone();
                        ui.add(
                            egui::TextEdit::singleline(&mut self.filtro_busca)
                                .hint_text(loc.t("search"))
                                .desired_width(170.0),
                        );
                        if self.filtro_busca != busca_antiga {
                            self.recarregar_cache(eventos);
                        }
                    });
                });

                ui.separator();

                if self.cache_alertas.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.label(
                            RichText::new(loc.t("no_violations_detected"))
                                .color(Color32::from_rgb(76, 175, 80))
                                .size(14.0),
                        );
                    });
                } else {
                    // 2. Tabela de 8 Colunas com ScrollArea bidirecional integrado (sem vazamento)
                    let larguras = [120.0, 75.0, 135.0, 115.0, 155.0, 95.0, 75.0, 75.0];
                    let total_table_w: f32 = larguras.iter().sum();
                    let row_height = 24.0;
                    let num_rows = self.cache_alertas.len();
                    let selected_idx = self.alerta_selecionado;

                    let header_color = if ui.visuals().dark_mode {
                        Color32::from_rgb(180, 200, 220)
                    } else {
                        Color32::from_rgb(40, 60, 90)
                    };
                    let titulos = [
                        loc.t("security_created_at"),
                        loc.t("security_risk_level"),
                        loc.t("security_alert_summary"),
                        loc.t("security_rules"),
                        loc.t("security_path"),
                        loc.t("security_name"),
                        loc.t("security_session_label"),
                        loc.t("security_evidence_id"),
                    ];

                    let table_h = (ui.available_height() - if self.alerta_selecionado.is_some() { 110.0 } else { 45.0 }).max(70.0);

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

                            // Linhas virtualizadas
                            for row_idx in 0..num_rows {
                                let item = &self.cache_alertas[row_idx];
                                let is_selected = selected_idx == Some(row_idx);

                                let cor_badge = match item.risk_level.as_str() {
                                    "Crítico" | "Critical" => Color32::from_rgb(183, 28, 28),
                                    "Alto" | "High" => Color32::from_rgb(244, 67, 54),
                                    "Médio" | "Medium" => Color32::from_rgb(255, 152, 0),
                                    _ => Color32::from_rgb(76, 175, 80),
                                };
                                let risco_traduzido = loc.traduzir_risco(&item.risk_level);
                                let resumo_traduzido = loc.traduzir_alerta(&item.summary);

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
                                        ui.label(RichText::new(&item.created_at).size(11.0));
                                    });
                                    // Col 1: Nível de Risco
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[1], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.colored_label(cor_badge, RichText::new(format!("[{}]", risco_traduzido)).strong().size(11.0));
                                    });
                                    // Col 2: Resumo do Alerta
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[2], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&resumo_traduzido).size(11.0));
                                    });
                                    // Col 3: Regras
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[3], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.rules).size(10.5).color(ui.visuals().weak_text_color()));
                                    });
                                    // Col 4: Caminho
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[4], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.caminho).size(10.5));
                                    });
                                    // Col 5: Nome
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[5], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.nome).strong().size(11.0));
                                    });
                                    // Col 6: Sessão
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[6], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.session_id).size(11.0));
                                    });
                                    // Col 7: Evidência
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[7], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.evidence_id).size(10.5).color(ui.visuals().weak_text_color()));
                                    });
                                });

                                if row_resp.response.interact(egui::Sense::click()).clicked() {
                                    self.alerta_selecionado = Some(row_idx);
                                }
                            }
                        });

                    // 4. Detalhes do Alerta Selecionado
                    if let Some(sel) = self.alerta_selecionado {
                        if let Some(detalhe) = self.cache_alertas.get(sel) {
                            ui.separator();
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{}: {}", loc.t("security_name"), detalhe.nome)).strong());
                                    ui.label(format!("| {}: {}", loc.t("security_risk_score"), detalhe.risk_score));
                                    if !detalhe.actor_process.is_empty() {
                                        ui.label(format!("| {}: {}", loc.t("security_actor_process"), detalhe.actor_process));
                                    }
                                });
                                ui.label(format!("{}: {}", loc.t("security_path"), detalhe.caminho));
                                if !detalhe.sha256.is_empty() {
                                    ui.label(RichText::new(format!("{}: {}", loc.t("security_sha256_current"), detalhe.sha256)).size(10.5).color(ui.visuals().weak_text_color()));
                                }
                            });
                        }
                    }
                }

                ui.separator();
                ui.add_space(4.0);

                // 5. Botões de Rodapé: Atualizar e Fechar
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [90.0, 26.0],
                            egui::Button::new(RichText::new(loc.t("security_refresh")).size(12.5)),
                        )
                        .clicked()
                    {
                        self.recarregar_cache(eventos);
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
