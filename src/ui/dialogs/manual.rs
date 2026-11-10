use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};

use crate::locale::{obter_secoes_manual, Localizer, ManualSection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchHit {
    pub secao_idx: usize,
    pub item_idx: usize, // 0 para título, 1+ para parágrafos/bullets
}

pub struct ManualDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    busca: String,
    secao_selecionada: usize,
    hits: Vec<SearchHit>,
    hit_atual_idx: usize,
    ultima_busca: String,
}

impl Default for ManualDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            busca: String::new(),
            secao_selecionada: 0,
            hits: Vec::new(),
            hit_atual_idx: 0,
            ultima_busca: String::new(),
        }
    }
}

impl ManualDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer) {
        if !self.aberto {
            return;
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        let secoes = obter_secoes_manual(&loc.idioma_atual());

        // Atualiza índice de busca caso o termo tenha sido alterado
        if self.busca != self.ultima_busca {
            self.recalcular_busca(&secoes);
            self.ultima_busca = self.busca.clone();
        }

        super::show_modeless_dialog(
            ctx,
            "dialog_manual",
            format!("📖 {}", loc.t("manual")),
            [920.0, 560.0],
            [650.0, 400.0],
            &mut state,
            |ui, fechar| {
                // Barra Superior: Busca com Localizar, Botões Anterior/Próximo e Contador de Ocorrências
                ui.horizontal(|ui| {
                    ui.label(RichText::new(loc.t("find")).strong());
                    let search_w = (ui.available_width() - 240.0).clamp(200.0, 480.0);
                    let edit = TextEdit::singleline(&mut self.busca)
                        .hint_text(loc.t("search_manual_hint"))
                        .desired_width(search_w);
                    let edit_resp = ui.add(edit);

                    if !self.busca.is_empty() {
                        if crate::ui::widgets::render_centered_emoji_button(ui, "✖", "").clicked() {
                            self.busca.clear();
                            self.hits.clear();
                            self.hit_atual_idx = 0;
                            self.ultima_busca.clear();
                        }
                    }

                    let btn_prev = ui
                        .add_enabled_ui(!self.hits.is_empty(), |ui| {
                            crate::ui::widgets::render_centered_emoji_button(ui, "◀", "")
                        })
                        .inner;
                    let btn_next = ui
                        .add_enabled_ui(!self.hits.is_empty(), |ui| {
                            crate::ui::widgets::render_centered_emoji_button(ui, "▶", "")
                        })
                        .inner;

                    let mut navegar = 0;
                    if btn_prev.clicked() {
                        navegar = -1;
                    } else if btn_next.clicked() {
                        navegar = 1;
                    } else if edit_resp.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        navegar = if ui.input(|i| i.modifiers.shift) {
                            -1
                        } else {
                            1
                        };
                    }

                    if navegar != 0 && !self.hits.is_empty() {
                        if navegar > 0 {
                            self.hit_atual_idx = (self.hit_atual_idx + 1) % self.hits.len();
                        } else {
                            self.hit_atual_idx = if self.hit_atual_idx == 0 {
                                self.hits.len() - 1
                            } else {
                                self.hit_atual_idx - 1
                            };
                        }
                        self.secao_selecionada = self.hits[self.hit_atual_idx].secao_idx;
                    }

                    if !self.busca.is_empty() {
                        if self.hits.is_empty() {
                            ui.colored_label(Color32::from_rgb(244, 67, 54), loc.t("no_matches"));
                        } else {
                            let count_text =
                                format!("{} / {}", self.hit_atual_idx + 1, self.hits.len());
                            ui.colored_label(Color32::from_rgb(33, 150, 243), count_text);
                        }
                    }
                });

                ui.separator();

                // Divisão responsiva: Esquerda proporcional para Tópicos/TOC e Direita para Conteúdo Detalhado
                let total_w = ui.available_width();
                let sidebar_w = (total_w * 0.28).clamp(220.0, 320.0);
                let content_w = (total_w - sidebar_w - ui.spacing().item_spacing.x - 4.0).max(200.0);
                let footer_h = 36.0;
                let available_h = (ui.available_height() - footer_h).max(160.0);

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), available_h),
                    egui::Layout::left_to_right(egui::Align::Min),
                    |ui| {
                        // Coluna 1: Tópicos
                        ui.allocate_ui_with_layout(
                            egui::vec2(sidebar_w, available_h),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.heading(
                                    RichText::new(loc.t("topics")).color(Color32::from_rgb(33, 150, 243)),
                                );
                                ui.add_space(4.0);

                                ScrollArea::vertical()
                                    .id_salt("manual_topicos_scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for (idx, sec) in secoes.iter().enumerate() {
                                            let match_count = self.contar_hits_na_secao(idx);
                                            let rotulo = if match_count > 0 && !self.busca.is_empty() {
                                                format!("{} [{}]", sec.title, match_count)
                                            } else {
                                                sec.title.clone()
                                            };

                                            let selecionado = self.secao_selecionada == idx;
                                            if ui.selectable_label(selecionado, rotulo).clicked() {
                                                self.secao_selecionada = idx;
                                            }
                                        }
                                    });
                            },
                        );

                        ui.separator();

                        // Coluna 2: Conteúdo formatado com realce de pesquisa
                        ui.allocate_ui_with_layout(
                            egui::vec2(content_w, available_h),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ScrollArea::vertical()
                                    .id_salt("manual_conteudo_scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        if let Some(sec) = secoes.get(self.secao_selecionada) {
                                            ui.heading(
                                                RichText::new(&sec.title).color(Color32::from_rgb(76, 175, 80)),
                                            );
                                            ui.add_space(8.0);

                                            for p in &sec.paragraphs {
                                                Self::render_texto_com_realce(ui, p, &self.busca, 13.5, false);
                                                ui.add_space(6.0);
                                            }

                                            if !sec.bullets.is_empty() {
                                                ui.add_space(6.0);
                                                for b in &sec.bullets {
                                                    ui.horizontal_wrapped(|ui| {
                                                        ui.label(RichText::new("•").strong());
                                                        Self::render_texto_com_realce(
                                                            ui,
                                                            b,
                                                            &self.busca,
                                                            13.0,
                                                            false,
                                                        );
                                                    });
                                                    ui.add_space(2.0);
                                                }
                                            }

                                            for det in &sec.details {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    RichText::new(&det.summary)
                                                        .strong()
                                                        .size(14.0)
                                                        .color(Color32::from_rgb(33, 150, 243)),
                                                );
                                                ui.add_space(4.0);
                                                for p in &det.paragraphs {
                                                    Self::render_texto_com_realce(
                                                        ui,
                                                        p,
                                                        &self.busca,
                                                        13.0,
                                                        false,
                                                    );
                                                    ui.add_space(4.0);
                                                }
                                                for b in &det.bullets {
                                                    ui.horizontal_wrapped(|ui| {
                                                        ui.label("  -");
                                                        Self::render_texto_com_realce(
                                                            ui,
                                                            b,
                                                            &self.busca,
                                                            12.5,
                                                            false,
                                                        );
                                                    });
                                                    ui.add_space(2.0);
                                                }
                                            }
                                        }
                                    });
                            },
                        );
                    },
                );

                ui.separator();

                // Rodapé com botão Fechar fixo no canto inferior direito
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                [85.0, 24.0],
                                egui::Button::new(RichText::new(loc.t("close")).size(12.5)),
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
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
        self.elevar_z_order = state.elevar_z_order;
    }

    fn recalcular_busca(&mut self, secoes: &[ManualSection]) {
        self.hits.clear();
        self.hit_atual_idx = 0;
        let query = self.busca.trim().to_lowercase();
        if query.is_empty() {
            return;
        }

        for (s_idx, sec) in secoes.iter().enumerate() {
            let mut item_id = 0;
            if sec.title.to_lowercase().contains(&query) {
                self.hits.push(SearchHit {
                    secao_idx: s_idx,
                    item_idx: item_id,
                });
            }
            item_id += 1;

            for p in &sec.paragraphs {
                if p.to_lowercase().contains(&query) {
                    self.hits.push(SearchHit {
                        secao_idx: s_idx,
                        item_idx: item_id,
                    });
                }
                item_id += 1;
            }

            for b in &sec.bullets {
                if b.to_lowercase().contains(&query) {
                    self.hits.push(SearchHit {
                        secao_idx: s_idx,
                        item_idx: item_id,
                    });
                }
                item_id += 1;
            }

            for det in &sec.details {
                if det.summary.to_lowercase().contains(&query) {
                    self.hits.push(SearchHit {
                        secao_idx: s_idx,
                        item_idx: item_id,
                    });
                }
                item_id += 1;
                for p in &det.paragraphs {
                    if p.to_lowercase().contains(&query) {
                        self.hits.push(SearchHit {
                            secao_idx: s_idx,
                            item_idx: item_id,
                        });
                    }
                    item_id += 1;
                }
                for b in &det.bullets {
                    if b.to_lowercase().contains(&query) {
                        self.hits.push(SearchHit {
                            secao_idx: s_idx,
                            item_idx: item_id,
                        });
                    }
                    item_id += 1;
                }
            }
        }

        if !self.hits.is_empty() {
            self.secao_selecionada = self.hits[0].secao_idx;
        }
    }

    fn contar_hits_na_secao(&self, s_idx: usize) -> usize {
        self.hits.iter().filter(|h| h.secao_idx == s_idx).count()
    }

    fn render_texto_com_realce(
        ui: &mut Ui,
        texto: &str,
        busca: &str,
        tamanho: f32,
        _destaque_ativo: bool,
    ) {
        let q = busca.trim().to_lowercase();
        if q.is_empty() || !texto.to_lowercase().contains(&q) {
            ui.label(RichText::new(texto).size(tamanho));
            return;
        }

        ui.horizontal_wrapped(|ui| {
            let lower = texto.to_lowercase();
            let mut start = 0;
            while let Some(pos) = lower[start..].find(&q) {
                let match_start = start + pos;
                let match_end = match_start + q.len();

                if match_start > start {
                    ui.label(RichText::new(&texto[start..match_start]).size(tamanho));
                }

                ui.label(
                    RichText::new(&texto[match_start..match_end])
                        .size(tamanho)
                        .background_color(Color32::from_rgb(255, 214, 0))
                        .color(Color32::BLACK)
                        .strong(),
                );

                start = match_end;
            }

            if start < texto.len() {
                ui.label(RichText::new(&texto[start..]).size(tamanho));
            }
        });
    }
}
