use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};

use crate::locale::Localizer;
use crate::utils::paths::resolver_caminho_asset;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AboutTab {
    Sobre,
    Historico,
    Clc,
    Copyright,
    Eula,
    Licencas,
    Avisos,
    Privacidade,
    Release,
}

pub struct AboutDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    tab_ativa: AboutTab,
    busca: String,
    hit_atual: usize,
    total_hits: usize,
}

impl Default for AboutDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            tab_ativa: AboutTab::Sobre,
            busca: String::new(),
            hit_atual: 0,
            total_hits: 0,
        }
    }
}

impl AboutDialog {
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

        let mut tab_ativa = self.tab_ativa;

        super::show_modeless_dialog(
            ctx,
            "dialog_about",
            format!("ℹ️ {} - HUGINN SENTINEL", loc.t("about")),
            [880.0, 680.0],
            [600.0, 480.0],
            &mut state,
            |ui, fechar| {
                // 1. Cabeçalho Fixo do Aplicativo (Paridade com build_about_header_html em C++)
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let title_color = if ui.visuals().dark_mode {
                            Color32::from_rgb(100, 181, 246)
                        } else {
                            Color32::from_rgb(25, 118, 210)
                        };
                        let privacy_color = if ui.visuals().dark_mode {
                            Color32::from_rgb(129, 199, 132)
                        } else {
                            Color32::from_rgb(46, 125, 50)
                        };
                        ui.heading(RichText::new("HUGINN SENTINEL").strong().size(18.0).color(title_color));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(loc.t("sovereign_privacy_badge")).size(12.0).color(privacy_color));
                        });
                    });
                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}:", loc.t("version"))).strong());
                        ui.label("2026.11.11.0");
                        ui.add_space(16.0);
                        ui.label(RichText::new(format!("{}:", loc.t("authors"))).strong());
                        ui.label("Fernando Nillsson Cidade");
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}:", loc.t("description"))).strong());
                        ui.label(loc.t("app_description"));
                    });
                });

                ui.add_space(6.0);

                // 2. Barra de Busca Interativa
                let texto_atual = Self::obter_texto_aba(tab_ativa, &loc.idioma_atual());
                self.atualizar_contagem_hits(&texto_atual);

                ui.horizontal(|ui| {
                    ui.label(RichText::new(loc.t("find")).strong());
                    let search_w = (ui.available_width() - 240.0).clamp(200.0, 480.0);
                    let edit = TextEdit::singleline(&mut self.busca)
                        .hint_text(loc.t("search_about_hint"))
                        .desired_width(search_w);
                    let edit_resp = ui.add(edit);

                    if !self.busca.is_empty() {
                        if crate::ui::widgets::render_centered_emoji_button(ui, "✖", "").clicked() {
                            self.busca.clear();
                            self.hit_atual = 0;
                            self.total_hits = 0;
                        }
                    }

                    let btn_prev = ui
                        .add_enabled_ui(self.total_hits > 0, |ui| {
                            crate::ui::widgets::render_centered_emoji_button(ui, "◀", "")
                        })
                        .inner;
                    let btn_next = ui
                        .add_enabled_ui(self.total_hits > 0, |ui| {
                            crate::ui::widgets::render_centered_emoji_button(ui, "▶", "")
                        })
                        .inner;

                    if btn_prev.clicked() {
                        if self.total_hits > 0 {
                            self.hit_atual = if self.hit_atual == 0 {
                                self.total_hits - 1
                            } else {
                                self.hit_atual - 1
                            };
                        }
                    } else if btn_next.clicked()
                        || (edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if self.total_hits > 0 {
                            self.hit_atual = (self.hit_atual + 1) % self.total_hits;
                        }
                    }

                    if !self.busca.is_empty() {
                        if self.total_hits == 0 {
                            ui.colored_label(Color32::from_rgb(244, 67, 54), loc.t("no_matches"));
                        } else {
                            let count_text =
                                format!("{} / {}", self.hit_atual + 1, self.total_hits);
                            ui.colored_label(Color32::from_rgb(33, 150, 243), count_text);
                        }
                    }
                });

                ui.separator();

                // 3. Barra de Abas (Tabs)
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut tab_ativa, AboutTab::Sobre, loc.t("tab_about"));
                    ui.selectable_value(&mut tab_ativa, AboutTab::Historico, loc.t("tab_history"));
                    ui.selectable_value(&mut tab_ativa, AboutTab::Clc, loc.t("tab_clc"));
                    ui.selectable_value(
                        &mut tab_ativa,
                        AboutTab::Copyright,
                        loc.t("tab_copyright"),
                    );
                    ui.selectable_value(&mut tab_ativa, AboutTab::Eula, loc.t("tab_eula"));
                    ui.selectable_value(&mut tab_ativa, AboutTab::Licencas, loc.t("tab_licenses"));
                    ui.selectable_value(&mut tab_ativa, AboutTab::Avisos, loc.t("tab_notices"));
                    ui.selectable_value(
                        &mut tab_ativa,
                        AboutTab::Privacidade,
                        loc.t("tab_privacy"),
                    );
                    ui.selectable_value(&mut tab_ativa, AboutTab::Release, loc.t("tab_release"));
                });

                ui.separator();

                // 4. Área de Conteúdo da Aba com Realce de Pesquisa
                let footer_h = 36.0;
                let available_h = (ui.available_height() - footer_h).max(180.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), available_h),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ScrollArea::vertical()
                            .id_salt("about_conteudo_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                Self::render_documento_com_realce(ui, &texto_atual, &self.busca);

                                // Se estiver na aba Licenças, exibe a lista de sites oficiais externos (Paridade C++)
                                if tab_ativa == AboutTab::Licencas {
                                    ui.add_space(16.0);
                                    ui.separator();
                                    ui.heading(
                                        RichText::new(loc.t("official_websites"))
                                            .color(Color32::from_rgb(33, 150, 243)),
                                    );
                                    ui.add_space(6.0);

                                    let sites = [
                                        ("Linguagem Rust", "https://www.rust-lang.org"),
                                        ("egui GUI Framework", "https://github.com/emilk/egui"),
                                        ("SQLite Embedded Database", "https://www.sqlite.org"),
                                        (
                                            "Notify Cross-Platform File Watching",
                                            "https://github.com/notify-rs/notify",
                                        ),
                                        (
                                            "Windows SDK / Win32 API",
                                            "https://developer.microsoft.com/windows",
                                        ),
                                        (
                                            "Chrono Date and Time Library",
                                            "https://github.com/chronotope/chrono",
                                        ),
                                        ("Serde JSON / Serialization Framework", "https://serde.rs"),
                                    ];

                                    for (nome, url) in sites {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("🌐").strong());
                                            ui.hyperlink_to(nome, url);
                                            ui.label(
                                                RichText::new(format!("({})", url))
                                                    .size(11.0)
                                                    .color(ui.visuals().weak_text_color()),
                                            );
                                        });
                                    }
                                }
                            });
                    },
                );

                ui.separator();

                // 5. Rodapé: centralização idêntica ao botão Fechar do Manual
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                [85.0, 24.0],
                                egui::Button::new(RichText::new(loc.t("ok")).size(12.5)),
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
        self.tab_ativa = tab_ativa;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
        self.elevar_z_order = state.elevar_z_order;
    }

    fn atualizar_contagem_hits(&mut self, texto: &str) {
        let q = self.busca.trim().to_lowercase();
        if q.is_empty() {
            self.total_hits = 0;
            self.hit_atual = 0;
            return;
        }

        let mut hits = 0;
        let lower = texto.to_lowercase();
        let mut start = 0;
        while let Some(pos) = lower[start..].find(&q) {
            hits += 1;
            start += pos + q.len();
        }
        self.total_hits = hits;
        if self.hit_atual >= hits && hits > 0 {
            self.hit_atual = 0;
        }
    }

    fn render_documento_com_realce(ui: &mut Ui, texto: &str, busca: &str) {
        let q = busca.trim().to_lowercase();
        let body_color = ui.visuals().text_color();
        if q.is_empty() || !texto.to_lowercase().contains(&q) {
            ui.label(RichText::new(texto).size(13.0).color(body_color));
            return;
        }

        for linha in texto.lines() {
            if linha.trim().is_empty() {
                ui.add_space(6.0);
                continue;
            }

            let lower = linha.to_lowercase();
            if !lower.contains(&q) {
                ui.label(RichText::new(linha).size(13.0).color(body_color));
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut start = 0;
                    while let Some(pos) = lower[start..].find(&q) {
                        let match_start = start + pos;
                        let match_end = match_start + q.len();

                        if match_start > start {
                            ui.label(
                                RichText::new(&linha[start..match_start])
                                    .size(13.0)
                                    .color(body_color),
                            );
                        }

                        ui.label(
                            RichText::new(&linha[match_start..match_end])
                                .size(13.0)
                                .background_color(Color32::from_rgb(255, 214, 0))
                                .color(Color32::BLACK)
                                .strong(),
                        );

                        start = match_end;
                    }

                    if start < linha.len() {
                        ui.label(
                            RichText::new(&linha[start..])
                                .size(13.0)
                                .color(body_color),
                        );
                    }
                });
            }
        }
    }

    pub fn obter_texto_aba(tab: AboutTab, lang: &str) -> String {
        if tab == AboutTab::Licencas {
            return Self::carregar_licencas_completas();
        }

        let candidatos = match tab {
            AboutTab::Sobre => vec![
                format!("ABOUT/ABOUT_{}.txt", lang),
                "ABOUT/ABOUT_pt_BR.txt".to_string(),
                "ABOUT/ABOUT_en_US.txt".to_string(),
            ],
            AboutTab::Historico => vec![
                format!("ABOUT/History_APP_{}.txt", lang),
                "ABOUT/History_APP_pt_BR.txt".to_string(),
                "ABOUT/History_APP_en_US.txt".to_string(),
            ],
            AboutTab::Clc => vec![
                format!("CLC/CLC_{} - Huginn.txt", lang),
                format!("CLC/CLC_{}.txt", lang),
                "CLC/CLC_pt_BR - Huginn.txt".to_string(),
                "CLC/CLC_en_US - Huginn.txt".to_string(),
            ],
            AboutTab::Copyright => vec![
                match lang {
                    "pt_BR" => {
                        "COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt".to_string()
                    }
                    "en_US" => "COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt".to_string(),
                    "es_ES" => "COPYRIGHT/AVISO DE DERECHOS DE AUTOR Y MARCA REGISTRADA_es_ES.txt"
                        .to_string(),
                    "fr_FR" => {
                        "COPYRIGHT/AVIS DE COPYRIGHT ET DE MARQUE DÉPOSÉE_fr_FR.txt".to_string()
                    }
                    "it_IT" => {
                        "COPYRIGHT/AVVISO SU COPYRIGHT E MARCHIO REGISTRATO_it_IT.txt".to_string()
                    }
                    "de_DE" => {
                        "COPYRIGHT/HINWEIS ZU URHEBERRECHT UND MARKENZEICHEN_de_DE.txt".to_string()
                    }
                    _ => "COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt".to_string(),
                },
                "COPYRIGHT/AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt".to_string(),
                "COPYRIGHT/COPYRIGHT AND TRADEMARK NOTICE_en_US.txt".to_string(),
            ],
            AboutTab::Eula => vec![
                format!("EULA/EULA_{} - Huginn.txt", lang),
                format!("EULA/EULA_{}.txt", lang),
                "EULA/EULA_pt_BR - Huginn.txt".to_string(),
                "EULA/EULA_en_US - Huginn.txt".to_string(),
            ],
            AboutTab::Licencas => vec![],
            AboutTab::Avisos => vec![
                format!("NOTICES/NOTICE_{}.txt", lang),
                format!("NOTICES/NOTICES_{}.txt", lang),
                "NOTICES/NOTICE_pt_BR.txt".to_string(),
                "NOTICES/NOTICE_en_US.txt".to_string(),
            ],
            AboutTab::Privacidade => vec![
                format!("PRIVACY_POLICY/Privacy_Policy_{}.txt", lang),
                format!("PRIVACY_POLICY/PRIVACY_POLICY_{}.txt", lang),
                "PRIVACY_POLICY/Privacy_Policy_pt_BR.txt".to_string(),
                "PRIVACY_POLICY/Privacy_Policy_en_US.txt".to_string(),
            ],
            AboutTab::Release => vec![
                format!("RELEASE/RELEASE NOTES_{}.txt", lang),
                format!("RELEASE/RELEASE_{}.txt", lang),
                "RELEASE/RELEASE NOTES_pt_BR.txt".to_string(),
                "RELEASE/RELEASE NOTES_en_US.txt".to_string(),
            ],
        };

        for cand in candidatos {
            if let Some(caminho) = resolver_caminho_asset(&cand) {
                if let Ok(conteudo) = std::fs::read_to_string(&caminho) {
                    if !conteudo.trim().is_empty() {
                        return conteudo;
                    }
                }
            }
        }

        // Fallback embutido caso os assets de texto não sejam encontrados
        match tab {
            AboutTab::Sobre => "Huginn Sentinel 2026.11.4.0\nPlataforma Desktop Local de Monitoramento, Auditoria Forense e Segurança de Arquivos.\nArquitetura Local-First / Sovereign Privacy.\nDesenvolvido em Rust com egui.".to_string(),
            AboutTab::Historico => "Histórico e Justificativa do Nome:\n\n'Huginn Sentinel' evoca o corvo mitológico do pensamento e vigilância atenta combinado à postura de sentinela protetora.\nO projeto sintetiza a união de dois pilares fundamentais da segurança digital: a percepção do imperceptível e a clareza da orientação estratégica.".to_string(),
            AboutTab::Clc => "Termos de Licenciamento Comercial e Distribuição Microsoft Store (CLC):\n\nO software é distribuído comercialmente via Microsoft Store com isenção de copyleft para uso interno, e em código aberto comunitário sob a licença GNU AGPLv3. Suporte técnico prestado pelo autor exclusivamente via e-mail: linceu_lighthouse@outlook.com.".to_string(),
            AboutTab::Copyright => "Copyright (c) 2025-2026 Fernando Nillsson Cidade. Todos os direitos reservados.\nO nome 'Huginn Sentinel' é uma marca comercial de Fernando Nillsson Cidade. Bibliotecas de terceiros pertencem aos seus respectivos autores.".to_string(),
            AboutTab::Eula => "Contrato de Licença de Usuário Final (EULA) — Duplo Licenciamento:\n\nO Huginn Sentinel adota Duplo Licenciamento: código aberto sob GNU AGPLv3 e binários oficiais comerciais na Microsoft Store. O software é fornecido 'no estado em que se encontra' (as-is), sem garantias implícitas.".to_string(),
            AboutTab::Licencas => "Licença GNU AGPLv3 / Duplo Licenciamento:\n\nO código-fonte é distribuído sob os termos da GNU Affero General Public License v3 (AGPLv3). Para binários comerciais oficiais na Microsoft Store, aplica-se o EULA comercial correspondente.".to_string(),
            AboutTab::Avisos => "Avisos e Declarações de Terceiros:\n\nEste produto utiliza bibliotecas de código aberto sob licenças MIT, Apache 2.0, CC0-1.0 e Domínio Público (SQLite).".to_string(),
            AboutTab::Privacidade => "Política de Privacidade Sovereign Privacy:\n\nO Huginn Sentinel opera 100% offline e localmente. Nenhum arquivo, metadado, hash ou telemetria é coletado ou transmitido para servidores remotos.".to_string(),
            AboutTab::Release => "Versão 2026.11.11.0:\n- Refatoração integral para Rust moderno e egui\n- Monitoramento em tempo real resiliente com ReadDirectoryChangesW\n- 17 gráficos estatísticos nativos interativos\n- Utilitário autônomo de console CLI (huginnsentinel-cli)".to_string(),
        }
    }

    fn carregar_licencas_completas() -> String {
        let mut out = String::new();
        out.push_str(
            "================================================================================\n",
        );
        out.push_str(
            "                   HUGINN SENTINEL — LICENÇAS E TERMOS               \n",
        );
        out.push_str(
            "================================================================================\n\n",
        );
        out.push_str("PORTAL DE LICENÇAS OFICIAIS:\n");
        out.push_str("- GNU LGPL v3.0: https://www.gnu.org/licenses/lgpl-3.0.html.en\n");
        out.push_str(
            "- GNU LGPL v2.1: https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html.en\n",
        );
        out.push_str("- BSD 2-Clause:  https://opensource.org/licenses/BSD-2-Clause\n");
        out.push_str("- BSD 3-Clause:  https://opensource.org/licenses/BSD-3-Clause\n");
        out.push_str("- Apache 2.0:    https://www.apache.org/licenses/LICENSE-2.0\n");
        out.push_str("- MIT License:   https://opensource.org/licenses/MIT\n");
        out.push_str("- SQLite Public: https://www.sqlite.org/copyright.html\n");
        out.push_str("- ISC License:   https://opensource.org/license/isc-license-txt\n\n");

        let arquivos_licenca = [
            "GNU AFFERO GENERAL PUBLIC LICENSE Version3.txt",
            "MIT License.txt",
            "Apache License Version 2.0.txt",
            "GNU LESSER GENERAL PUBLIC LICENSE Version 3.txt",
            "BSD License Clause-2.txt",
            "BSD License Clause-3.txt",
            "GNU GENERAL PUBLIC LICENSE Version 3.txt",
            "ISC LICENSE.txt",
            "PYTHON SOFTWARE FOUNDATION LICENSE.txt",
            "SQLite Is Public Domain.txt",
        ];

        for arq in &arquivos_licenca {
            let rel = format!("LICENSES/{}", arq);
            if let Some(p) = resolver_caminho_asset(&rel) {
                if let Ok(conteudo) = std::fs::read_to_string(&p) {
                    if !conteudo.trim().is_empty() {
                        out.push_str("--------------------------------------------------------------------------------\n");
                        out.push_str(&format!(">>> {}\n", arq));
                        out.push_str("--------------------------------------------------------------------------------\n");
                        out.push_str(&conteudo);
                        out.push_str("\n\n");
                    }
                }
            }
        }

        if out.len() > 600 {
            out
        } else {
            "Licenças de Código Aberto: MIT, Apache 2.0, LGPL v3, BSD-2/3-Clause, SQLite Public Domain, ISC.".to_string()
        }
    }
}
