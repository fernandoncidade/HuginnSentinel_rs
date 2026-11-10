use egui::{Color32, RichText, ScrollArea, Vec2};
use std::time::{Duration, Instant};

use crate::core::event::EventModel;
use crate::locale::Localizer;
use crate::security::sessions::CorrelacaoSessoes;

#[derive(Debug, Clone)]
pub struct SessionRowData {
    pub session_id: String,
    pub label: String,
    pub updated_at: String,
    pub started_at: String,
    pub total_eventos: usize,
    pub adicionados: usize,
    pub modificados: usize,
    pub excluidos: usize,
    pub movidos: usize,
    pub renomeados: usize,
    pub escaneados: usize,
    pub diretorios: String,
    pub nivel_risco: String,
    pub max_risco: i32,
}

pub struct SessionCenterDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub filtro_busca: String,
    pub sessao_selecionada: Option<usize>,
    pub cache_sessoes: Vec<SessionRowData>,
    pub ultimo_total_eventos: usize,
    pub ultimo_recalculo: Instant,
}

impl Default for SessionCenterDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            filtro_busca: String::new(),
            sessao_selecionada: None,
            cache_sessoes: Vec::new(),
            ultimo_total_eventos: 0,
            ultimo_recalculo: Instant::now() - Duration::from_secs(10),
        }
    }
}

impl SessionCenterDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        // Força recálculo ao abrir
        self.ultimo_total_eventos = usize::MAX;
    }

    pub fn recarregar_cache(&mut self, eventos: &[EventModel]) {
        let raw_sessions = CorrelacaoSessoes::agrupar_em_sessoes(eventos);
        let q = self.filtro_busca.trim().to_lowercase();
        let mut itens = Vec::new();

        for sess in raw_sessions {
            let dirs_summary = if sess.diretorios.is_empty() {
                "-".to_string()
            } else if sess.diretorios.len() == 1 {
                sess.diretorios[0].clone()
            } else {
                format!("{} (+{})", sess.diretorios[0], sess.diretorios.len() - 1)
            };

            let match_busca = if q.is_empty() {
                true
            } else {
                sess.session_id.to_lowercase().contains(&q)
                    || dirs_summary.to_lowercase().contains(&q)
                    || sess.nivel_risco.to_lowercase().contains(&q)
            };

            if match_busca {
                itens.push(SessionRowData {
                    session_id: sess.session_id.clone(),
                    label: format!("Sessão {}", sess.session_id),
                    updated_at: sess.fim.clone(),
                    started_at: sess.inicio.clone(),
                    total_eventos: sess.total_eventos,
                    adicionados: sess.adicionados,
                    modificados: sess.modificados,
                    excluidos: sess.excluidos,
                    movidos: sess.movidos,
                    renomeados: sess.renomeados,
                    escaneados: sess.escaneados,
                    diretorios: dirs_summary,
                    nivel_risco: sess.nivel_risco,
                    max_risco: sess.max_risco,
                });
            }
        }

        itens.reverse();
        self.cache_sessoes = itens;
        self.ultimo_total_eventos = eventos.len();
        self.ultimo_recalculo = Instant::now();
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, eventos: &[EventModel]) {
        if !self.aberto {
            return;
        }

        let precisa_recalcular = self.cache_sessoes.is_empty()
            || (self.ultimo_total_eventos != eventos.len()
                && self.ultimo_recalculo.elapsed() >= Duration::from_millis(1500));

        if precisa_recalcular {
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
            "dialog_session_center",
            format!("⏱️ {}", loc.t("security_session_center")),
            [680.0, 280.0],
            [560.0, 190.0],
            &mut state,
            |ui, fechar| {
                // 1. Cabeçalho de Resumo e Busca
                ui.horizontal(|ui| {
                    let total_str = self.cache_sessoes.len().to_string();
                    let summary_text = format!(
                        "{}: {} | Backend: SQLite",
                        loc.t("security_session_center"),
                        total_str
                    );
                    ui.label(RichText::new(summary_text).strong().size(13.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let busca_antiga = self.filtro_busca.clone();
                        ui.add(
                            egui::TextEdit::singleline(&mut self.filtro_busca)
                                .hint_text(loc.t("search"))
                                .desired_width(160.0),
                        );
                        if self.filtro_busca != busca_antiga {
                            self.recarregar_cache(eventos);
                        }
                    });
                });

                ui.separator();

                if self.cache_sessoes.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.label(
                            RichText::new(loc.t("no_sessions_identified"))
                                .color(ui.visuals().weak_text_color())
                                .size(14.0),
                        );
                    });
                } else {
                    // 2. Tabela de 5 Colunas com ScrollArea bidirecional integrado (sem vazamento)
                    let larguras = [120.0, 95.0, 85.0, 180.0, 90.0];
                    let total_table_w: f32 = larguras.iter().sum();
                    let row_height = 24.0;
                    let num_rows = self.cache_sessoes.len();
                    let selected_idx = self.sessao_selecionada;

                    let header_color = if ui.visuals().dark_mode {
                        Color32::from_rgb(180, 200, 220)
                    } else {
                        Color32::from_rgb(40, 60, 90)
                    };
                    let titulos = [
                        loc.t("security_updated_at"),
                        loc.t("security_session_label"),
                        loc.t("security_total_events"),
                        loc.t("security_monitored_directories"),
                        loc.t("security_evidence_id"),
                    ];

                    let table_h = (ui.available_height() - if self.sessao_selecionada.is_some() { 100.0 } else { 45.0 }).max(70.0);

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

                            // Linhas da tabela
                            for row_idx in 0..num_rows {
                                let item = &self.cache_sessoes[row_idx];
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
                                        ui.label(RichText::new(&item.updated_at).size(11.0));
                                    });
                                    // Col 1: Sessão
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[1], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.label).strong().size(11.0));
                                    });
                                    // Col 2: Total de Eventos
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[2], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(item.total_eventos.to_string()).size(11.0));
                                    });
                                    // Col 3: Diretórios Monitorados
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[3], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.diretorios).size(10.5));
                                    });
                                    // Col 4: Identificador
                                    ui.allocate_ui_with_layout(Vec2::new(larguras[4], row_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&item.session_id).color(ui.visuals().weak_text_color()).size(10.5));
                                    });
                                });

                                if row_resp.response.interact(egui::Sense::click()).clicked() {
                                    self.sessao_selecionada = Some(row_idx);
                                }
                            }
                        });

                    // 4. Detalhes da Sessão Selecionada
                    if let Some(sel) = self.sessao_selecionada {
                        if let Some(det) = self.cache_sessoes.get(sel) {
                            ui.separator();
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&det.session_id).strong());
                                    ui.label(loc.t_args("period_label", &[&det.started_at, &det.updated_at]));

                                    let cor = match det.nivel_risco.as_str() {
                                        "Crítico" | "Alto" | "High" | "Critical" => Color32::from_rgb(244, 67, 54),
                                        "Médio" | "Medium" => Color32::from_rgb(255, 152, 0),
                                        _ => Color32::from_rgb(76, 175, 80),
                                    };
                                    let risco_trad = loc.traduzir_risco(&det.nivel_risco);
                                    let sc_str = det.max_risco.to_string();
                                    ui.colored_label(cor, loc.t_args("max_risk_label", &[&risco_trad, &sc_str]));
                                });

                                ui.horizontal(|ui| {
                                    ui.label(loc.t_args("total_events_short", &[&det.total_eventos.to_string()]));
                                    ui.label(loc.t_args("added_count", &[&det.adicionados.to_string()]));
                                    ui.label(loc.t_args("modified_count", &[&det.modificados.to_string()]));
                                    ui.label(loc.t_args("deleted_count", &[&det.excluidos.to_string()]));
                                    ui.label(loc.t_args("moved_renamed_count", &[&(det.movidos + det.renomeados).to_string()]));
                                });
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
