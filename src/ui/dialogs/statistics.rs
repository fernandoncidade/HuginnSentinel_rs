use egui::{Color32, RichText, ScrollArea, Vec2};

use crate::core::event::EventModel;
use crate::data::charts::TipoGrafico;
use crate::data::generator::{ChartData, GeradorEstatisticas};
use crate::data::render::{exportar_grafico_png, renderizar_grafico};
use crate::locale::Localizer;
use crate::utils::logger::get_logger;

pub struct StatisticsDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    grafico_selecionado: TipoGrafico,
    chart_data_cache: Option<crate::data::ChartData>,
    ultimo_grafico: Option<TipoGrafico>,
    ultimo_total_eventos: usize,
    ultimo_recalculo: std::time::Instant,
    abas_scroll_offset: f32,
    abas_offset_manual: Option<f32>,
    scroll_para_aba_selecionada: bool,
    limite_itens_extensao: Option<usize>,
    ops_sankey_op_tipo: std::collections::HashSet<crate::core::types::OperationType>,
    ops_sankey_op_dir: std::collections::HashSet<crate::core::types::OperationType>,
    limite_sankey_tipos: Option<usize>,
    limite_sankey_diretorios: Option<usize>,
    ultimo_idioma: String,
}

impl Default for StatisticsDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            grafico_selecionado: TipoGrafico::ArvoreDiretorios,
            chart_data_cache: None,
            ultimo_grafico: None,
            ultimo_total_eventos: 0,
            ultimo_recalculo: std::time::Instant::now() - std::time::Duration::from_secs(10),
            ultimo_idioma: String::new(),
            abas_scroll_offset: 0.0,
            abas_offset_manual: None,
            scroll_para_aba_selecionada: false,
            limite_itens_extensao: None,
            ops_sankey_op_tipo: crate::core::types::OperationType::all().iter().copied().collect(),
            ops_sankey_op_dir: crate::core::types::OperationType::all().iter().copied().collect(),
            limite_sankey_tipos: Some(20),
            limite_sankey_diretorios: Some(20),
        }
    }
}

impl StatisticsDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn abrir_com_grafico(&mut self, tipo: TipoGrafico) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
        self.grafico_selecionado = tipo;
        self.scroll_para_aba_selecionada = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer, eventos: &[EventModel]) {
        if !self.aberto {
            return;
        }

        let mut grafico_selecionado = self.grafico_selecionado;

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };

        let stats_title = loc.t_args("statistics_title", &[&loc.t("statistics")]);

        super::show_modeless_dialog(
            ctx,
            "dialog_statistics",
            stats_title,
            [980.0, 580.0],
            [680.0, 400.0],
            &mut state,
            |ui, fechar| {
                // 1. Layout "Diagramas Estatísticos": ocupa o MENOR espaço possível em relação ao eixo "y" / vertical (~36px)
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("📊 {}:", loc.t("statistical_diagrams")))
                                .strong()
                                .size(12.5),
                        );

                        // Dropdown compacto para seleção direta e precisa
                        let combo_diag = egui::ComboBox::from_id_salt("combo_stats_diagramas")
                            .selected_text(
                                RichText::new(grafico_selecionado.titulo(loc)).strong().size(12.0),
                            )
                            .width(230.0)
                            .show_ui(ui, |ui| {
                                for tipo in TipoGrafico::all() {
                                    let sel = grafico_selecionado == *tipo;
                                    if ui
                                        .selectable_label(
                                            sel,
                                            RichText::new(tipo.titulo(loc)).size(11.5),
                                        )
                                        .clicked()
                                    {
                                        grafico_selecionado = *tipo;
                                        self.scroll_para_aba_selecionada = true;
                                        egui::Popup::close_all(ui.ctx());
                                    }
                                }
                            });

                        let diag_popup_id = combo_diag.response.id.with("popup");
                        if egui::ComboBox::is_open(ui.ctx(), combo_diag.response.id) {
                            let mut full_rect = combo_diag.response.rect;
                            if let Some(pr) = ui.ctx().memory(|m| m.area_rect(diag_popup_id)) {
                                full_rect = full_rect.union(pr);
                            }
                            if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos())) {
                                if !full_rect.expand(8.0).contains(pos) {
                                    egui::Popup::close_id(ui.ctx(), diag_popup_id);
                                    ui.ctx().request_repaint();
                                }
                            }
                        }

                        ui.separator();

                        // Layout das abas com "PreviousNavigationButton" (extremo esquerdo) e
                        // "NextNavigationButton" (extremo direito), sem scrollbar e com rolagem via scroll do mouse
                        let nav_btn_w = 26.0;
                        let nav_btn_h = 22.0;
                        let passo_scroll = 180.0;

                        let prev_resp = Self::desenhar_botao_navegacao_aba(
                            ui,
                            "PreviousNavigationButton",
                            "◀",
                            Vec2::new(nav_btn_w, nav_btn_h),
                            format!("PreviousNavigationButton — {}", loc.t("nav_previous_tab")),
                        );
                        let prev_acionado = prev_resp.clicked();

                        let mut offset_override = self.abas_offset_manual.take();
                        if prev_acionado {
                            offset_override =
                                Some((self.abas_scroll_offset - passo_scroll).max(0.0));
                        }

                        let spacing = ui.spacing().item_spacing.x;
                        let scroll_w = (ui.available_width() - nav_btn_w - spacing).max(60.0);

                        let scroll_out = ui
                            .allocate_ui(Vec2::new(scroll_w, nav_btn_h), |ui| {
                                // Permite rolar as abas horizontalmente com o botão scroll do mouse sem exigir Shift
                                ui.style_mut().always_scroll_the_only_direction = true;

                                let mut scroll_area = ScrollArea::horizontal()
                                    .id_salt("stats_chips_diagramas")
                                    .max_width(scroll_w)
                                    .max_height(nav_btn_h)
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                    )
                                    .scroll_source(egui::scroll_area::ScrollSource {
                                        scroll_bar: false,
                                        drag: egui::scroll_area::DragScroll::Never,
                                        mouse_wheel: true,
                                    });

                                if let Some(target_x) = offset_override {
                                    scroll_area = scroll_area.horizontal_scroll_offset(target_x);
                                }

                                scroll_area.show(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.set_min_height(nav_btn_h);
                                            for tipo in TipoGrafico::all() {
                                                let sel = grafico_selecionado == *tipo;
                                                let resp = ui.selectable_label(
                                                    sel,
                                                    RichText::new(tipo.titulo(loc)).size(11.0),
                                                );
                                                if resp.clicked() {
                                                    grafico_selecionado = *tipo;
                                                }
                                                if sel && self.scroll_para_aba_selecionada {
                                                    resp.scroll_to_me(Some(egui::Align::Center));
                                                }
                                            }
                                        },
                                    );
                                })
                            })
                            .inner;

                        let next_resp = Self::desenhar_botao_navegacao_aba(
                            ui,
                            "NextNavigationButton",
                            "▶",
                            Vec2::new(nav_btn_w, nav_btn_h),
                            format!("NextNavigationButton — {}", loc.t("nav_next_tab")),
                        );
                        let next_acionado = next_resp.clicked();

                        let max_offset =
                            (scroll_out.content_size.x - scroll_out.inner_rect.width()).max(0.0);
                        self.abas_scroll_offset =
                            scroll_out.state.clamped_offset().x.clamp(0.0, max_offset);
                        self.scroll_para_aba_selecionada = false;

                        if next_acionado {
                            let novo = (self.abas_scroll_offset + passo_scroll).min(max_offset);
                            self.abas_scroll_offset = novo;
                            self.abas_offset_manual = Some(novo);
                            ui.ctx().request_repaint();
                        }
                    });
                });

                ui.add_space(3.0);

                // 2. Layout "Hierarquia de Diretórios" / Gráfico Ativo: ocupa o MAIOR espaço possível em relação ao eixo "y" / vertical
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let title_color = if ui.visuals().dark_mode {
                            Color32::from_rgb(100, 181, 246)
                        } else {
                            Color32::from_rgb(25, 118, 210)
                        };
                        ui.horizontal(|ui| {
                            ui.heading(
                                RichText::new(grafico_selecionado.titulo(loc))
                                    .size(14.0)
                                    .color(title_color),
                            );
                            ui.label(
                                RichText::new(format!("— {}", grafico_selecionado.descricao(loc)))
                                    .size(11.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });
                        ui.add_space(2.0);

                        let precisa_regerar = self.chart_data_cache.is_none()
                            || self.ultimo_grafico != Some(grafico_selecionado)
                            || self.ultimo_idioma != loc.idioma_atual()
                            || (self.ultimo_total_eventos != eventos.len()
                                && self.ultimo_recalculo.elapsed() >= std::time::Duration::from_millis(1500));

                        if precisa_regerar {
                            self.chart_data_cache = Some(Self::gerar_dados(grafico_selecionado, eventos, loc));
                            self.ultimo_grafico = Some(grafico_selecionado);
                            self.ultimo_total_eventos = eventos.len();
                            self.ultimo_recalculo = std::time::Instant::now();
                            self.ultimo_idioma = loc.idioma_atual();
                        }

                        let chart_data_bruto = self.chart_data_cache.as_ref().unwrap();
                        let eh_grafico_extensoes = matches!(
                            grafico_selecionado,
                            TipoGrafico::BarrasTipos
                                | TipoGrafico::ParetoExtensoes
                                | TipoGrafico::BoxplotTamanhos
                        );
                        let eh_grafico_sankey = matches!(
                            grafico_selecionado,
                            TipoGrafico::SankeyOperacaoTipo
                                | TipoGrafico::SankeyEventoDiretorio
                                | TipoGrafico::SankeyTipoDiretorio
                        );
                        let dados_plotados = if eh_grafico_extensoes {
                            chart_data_bruto.com_limite_extensoes(self.limite_itens_extensao)
                        } else if eh_grafico_sankey {
                            match grafico_selecionado {
                                TipoGrafico::SankeyOperacaoTipo => {
                                    let ops: Vec<crate::core::types::OperationType> =
                                        self.ops_sankey_op_tipo.iter().copied().collect();
                                    chart_data_bruto.com_filtro_operacoes_e_limite_destino(
                                        &ops,
                                        self.limite_sankey_tipos,
                                        loc,
                                    )
                                }
                                TipoGrafico::SankeyEventoDiretorio => {
                                    let ops: Vec<crate::core::types::OperationType> =
                                        self.ops_sankey_op_dir.iter().copied().collect();
                                    chart_data_bruto.com_filtro_operacoes_e_limite_destino(
                                        &ops,
                                        self.limite_sankey_diretorios,
                                        loc,
                                    )
                                }
                                TipoGrafico::SankeyTipoDiretorio => {
                                    chart_data_bruto.com_limite_sankey_duplo(
                                        self.limite_sankey_tipos,
                                        self.limite_sankey_diretorios,
                                    )
                                }
                                _ => chart_data_bruto.clone(),
                            }
                        } else {
                            chart_data_bruto.clone()
                        };
                        let chart_data = &dados_plotados;

                        // Altura reservada para a barra inferior (botões "Exportar Gráfico (PNG)", "Exportar Todos" e "Fechar")
                        let footer_height = 38.0;
                        let chart_w = ui.available_width().max(300.0);
                        let chart_h = (ui.available_height() - footer_height).max(180.0);

                        // Envolve a área de plotagem num container de dimensões exatas (chart_w x chart_h)
                        // para garantir que a scrollbar nunca empurre os botões inferiores para fora da janela.
                        ui.allocate_ui(Vec2::new(chart_w, chart_h), |ui| {
                            ui.set_max_size(Vec2::new(chart_w, chart_h));

                            if eh_grafico_extensoes {
                                let bar_ctrl_h = 26.0;
                                let total_ext = chart_data_bruto.total_extensoes();
                                let exibidas = chart_data.total_extensoes();

                                let is_dark = ui.visuals().dark_mode;
                                let bar_bg = if is_dark {
                                    Color32::from_rgb(24, 26, 31)
                                } else {
                                    Color32::from_rgb(246, 248, 251)
                                };
                                let bar_stroke = if is_dark {
                                    Color32::from_rgb(45, 48, 58)
                                } else {
                                    Color32::from_rgb(218, 224, 233)
                                };
                                let summary_color = if is_dark {
                                    Color32::from_rgb(144, 202, 249)
                                } else {
                                    Color32::from_rgb(25, 118, 210)
                                };
                                let combo_lbl_color = if is_dark {
                                    Color32::from_rgb(180, 210, 255)
                                } else {
                                    Color32::from_rgb(30, 80, 160)
                                };

                                // Faixa superior da área de plotagem contendo a lista suspensa (10 em 10 até 100 e Todos)
                                egui::Frame::NONE
                                    .fill(bar_bg)
                                    .stroke(egui::Stroke::new(1.0, bar_stroke))
                                    .corner_radius(6_u8)
                                    .inner_margin(egui::Margin::symmetric(10, 3))
                                    .show(ui, |ui| {
                                        ui.set_width((chart_w - 20.0).max(100.0));
                                        ui.set_height(bar_ctrl_h - 6.0);
                                        ui.horizontal(|ui| {
                                            let info_txt = loc.t_args(
                                                "chart_showing_of_total",
                                                &[
                                                    &exibidas.to_string(),
                                                    &total_ext.to_string(),
                                                ],
                                            );
                                            ui.label(
                                                RichText::new(format!("📊 {}", info_txt))
                                                    .size(11.0)
                                                    .color(summary_color),
                                            );

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    let texto_selecionado =
                                                        match self.limite_itens_extensao {
                                                            Some(n) => n.to_string(),
                                                            None => loc.t("chart_limit_all"),
                                                        };

                                                    let combo_lim = egui::ComboBox::from_id_salt(
                                                        "combo_limite_extensoes_plotadas",
                                                    )
                                                    .selected_text(
                                                        RichText::new(texto_selecionado)
                                                            .strong()
                                                            .size(11.5),
                                                    )
                                                    .width(88.0)
                                                    .show_ui(ui, |ui| {
                                                        let opcoes: [Option<usize>; 11] = [
                                                            Some(10),
                                                            Some(20),
                                                            Some(30),
                                                            Some(40),
                                                            Some(50),
                                                            Some(60),
                                                            Some(70),
                                                            Some(80),
                                                            Some(90),
                                                            Some(100),
                                                            None,
                                                        ];
                                                        for op in opcoes {
                                                            let rotulo = match op {
                                                                 Some(n) => n.to_string(),
                                                                 None => loc.t("chart_limit_all"),
                                                            };
                                                            let sel =
                                                                 self.limite_itens_extensao == op;
                                                            if ui
                                                                 .selectable_label(
                                                                     sel,
                                                                     RichText::new(rotulo)
                                                                         .size(11.5),
                                                                 )
                                                                 .clicked()
                                                            {
                                                                 self.limite_itens_extensao = op;
                                                                 egui::Popup::close_all(ui.ctx());
                                                            }
                                                        }
                                                    });

                                                    let lim_popup_id = combo_lim.response.id.with("popup");
                                                    if egui::ComboBox::is_open(ui.ctx(), combo_lim.response.id) {
                                                        let mut full_rect = combo_lim.response.rect;
                                                        if let Some(pr) = ui.ctx().memory(|m| m.area_rect(lim_popup_id)) {
                                                            full_rect = full_rect.union(pr);
                                                        }
                                                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos())) {
                                                            if !full_rect.expand(8.0).contains(pos) {
                                                                egui::Popup::close_id(ui.ctx(), lim_popup_id);
                                                                ui.ctx().request_repaint();
                                                            }
                                                        }
                                                    }

                                                    ui.label(
                                                        RichText::new(format!(
                                                            "🔢 {}",
                                                            loc.t("chart_limit_items_label")
                                                        ))
                                                        .strong()
                                                        .size(11.0)
                                                        .color(combo_lbl_color),
                                                    );
                                                },
                                            );
                                        });
                                    });

                                ui.add_space(2.0);

                                let scroll_region_h =
                                    (chart_h - bar_ctrl_h - 4.0).max(140.0);
                                let scrollbar_h = 14.0;
                                let canvas_h = (scroll_region_h - scrollbar_h).max(126.0);
                                let n_itens = exibidas.max(1);
                                let needed_w = match grafico_selecionado {
                                    TipoGrafico::BarrasTipos => {
                                        (n_itens as f32 * 54.0 + 48.0).max(chart_w - 4.0)
                                    }
                                    TipoGrafico::ParetoExtensoes => {
                                        (n_itens as f32 * 58.0 + 84.0).max(chart_w - 4.0)
                                    }
                                    TipoGrafico::BoxplotTamanhos => {
                                        (n_itens as f32 * 80.0 + 150.0).max(chart_w - 4.0)
                                    }
                                    _ => chart_w,
                                };

                                // Scrollbar horizontal posicionada na parte inferior da área de plotagem
                                ui.scope(|ui| {
                                    ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                                    ScrollArea::horizontal()
                                        .id_salt((
                                            "stats_chart_horizontal_scroll",
                                            grafico_selecionado as usize,
                                        ))
                                        .max_width(chart_w)
                                        .max_height(scroll_region_h)
                                        .auto_shrink([false, false])
                                        .scroll_bar_visibility(
                                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                        )
                                        .show(ui, |ui| {
                                            renderizar_grafico(
                                                ui,
                                                grafico_selecionado,
                                                chart_data,
                                                Vec2::new(needed_w, canvas_h),
                                                loc,
                                            );
                                        });
                                });
                            } else if eh_grafico_sankey {
                                let bar_ctrl_h = 28.0;

                                let (id_orig, id_dest, lbl_orig_combo, lbl_dest_combo, entity_orig, entity_dest) = match grafico_selecionado {
                                    TipoGrafico::SankeyOperacaoTipo => (
                                        "combo_sankey_op_tipo_operacoes",
                                        "combo_sankey_op_tipo_extensoes",
                                        format!("⚡ {}", loc.t("chart_sankey_operation_filter")),
                                        format!("📁 {}", loc.t("chart_sankey_limit_file_types")),
                                        loc.t("chart_entity_operations"),
                                        loc.t("chart_entity_file_types"),
                                    ),
                                    TipoGrafico::SankeyEventoDiretorio => (
                                        "combo_sankey_op_dir_operacoes",
                                        "combo_sankey_op_dir_diretorios",
                                        format!("⚡ {}", loc.t("chart_sankey_operation_filter")),
                                        format!("📂 {}", loc.t("chart_sankey_limit_directories")),
                                        loc.t("chart_entity_operations"),
                                        loc.t("chart_entity_directories"),
                                    ),
                                    TipoGrafico::SankeyTipoDiretorio => (
                                        "combo_sankey_tipo_dir_tipos",
                                        "combo_sankey_tipo_dir_diretorios",
                                        format!("📁 {}", loc.t("chart_sankey_limit_file_types")),
                                        format!("📂 {}", loc.t("chart_sankey_limit_directories")),
                                        loc.t("chart_entity_file_types"),
                                        loc.t("chart_entity_directories"),
                                    ),
                                    _ => (
                                        "combo_sankey_orig",
                                        "combo_sankey_dest",
                                        loc.t("chart_sankey_limit_operations"),
                                        loc.t("chart_sankey_limit_file_types"),
                                        loc.t("chart_entity_operations"),
                                        loc.t("chart_entity_file_types"),
                                    ),
                                };

                                let orig_plotted = chart_data.total_origens_sankey();
                                let orig_total = chart_data_bruto.total_origens_sankey();
                                let dest_plotted = chart_data.total_destinos_sankey();
                                let dest_total = chart_data_bruto.total_destinos_sankey();

                                let is_dark = ui.visuals().dark_mode;
                                let bar_bg = if is_dark {
                                    Color32::from_rgb(24, 26, 31)
                                } else {
                                    Color32::from_rgb(246, 248, 251)
                                };
                                let bar_stroke = if is_dark {
                                    Color32::from_rgb(45, 48, 58)
                                } else {
                                    Color32::from_rgb(218, 224, 233)
                                };
                                let summary_color = if is_dark {
                                    Color32::from_rgb(144, 202, 249)
                                } else {
                                    Color32::from_rgb(25, 118, 210)
                                };
                                let combo_lbl_color = if is_dark {
                                    Color32::from_rgb(180, 210, 255)
                                } else {
                                    Color32::from_rgb(30, 80, 160)
                                };

                                // Faixa superior da área de plotagem contendo a lista suspensa (10 em 10 até 100 e Todos)
                                egui::Frame::NONE
                                    .fill(bar_bg)
                                    .stroke(egui::Stroke::new(1.0, bar_stroke))
                                    .corner_radius(6_u8)
                                    .inner_margin(egui::Margin::symmetric(10, 3))
                                    .show(ui, |ui| {
                                        ui.set_width((chart_w - 20.0).max(100.0));
                                        ui.set_height(bar_ctrl_h - 6.0);
                                        ui.horizontal(|ui| {
                                            let info_txt = loc.t_args(
                                                "chart_sankey_showing_summary",
                                                &[
                                                    &orig_plotted.to_string(),
                                                    &orig_total.to_string(),
                                                    &entity_orig,
                                                    &dest_plotted.to_string(),
                                                    &dest_total.to_string(),
                                                    &entity_dest,
                                                ],
                                            );
                                            ui.label(
                                                RichText::new(format!("📊 {}", info_txt))
                                                    .size(11.0)
                                                    .color(summary_color),
                                            );

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    // Dropdown 2: Destino (à direita no layout RTL)
                                                    let mut limite_dest_val = match grafico_selecionado {
                                                        TipoGrafico::SankeyOperacaoTipo => self.limite_sankey_tipos,
                                                        TipoGrafico::SankeyEventoDiretorio => self.limite_sankey_diretorios,
                                                        TipoGrafico::SankeyTipoDiretorio => self.limite_sankey_diretorios,
                                                        _ => None,
                                                    };

                                                    let txt_dest = match limite_dest_val {
                                                        Some(n) => n.to_string(),
                                                        None => loc.t("chart_limit_all"),
                                                    };

                                                    let combo_dest = egui::ComboBox::from_id_salt(id_dest)
                                                        .selected_text(
                                                            RichText::new(txt_dest)
                                                                .strong()
                                                                .size(11.0),
                                                        )
                                                        .width(76.0)
                                                        .show_ui(ui, |ui| {
                                                            let opcoes: [Option<usize>; 11] = [
                                                                Some(10),
                                                                Some(20),
                                                                Some(30),
                                                                Some(40),
                                                                Some(50),
                                                                Some(60),
                                                                Some(70),
                                                                Some(80),
                                                                Some(90),
                                                                Some(100),
                                                                None,
                                                            ];
                                                            for op in opcoes {
                                                                let rotulo = match op {
                                                                    Some(n) => n.to_string(),
                                                                    None => loc.t("chart_limit_all"),
                                                                };
                                                                let sel = limite_dest_val == op;
                                                                if ui
                                                                    .selectable_label(
                                                                        sel,
                                                                        RichText::new(rotulo)
                                                                            .size(11.0),
                                                                    )
                                                                    .clicked()
                                                                {
                                                                    limite_dest_val = op;
                                                                    match grafico_selecionado {
                                                                        TipoGrafico::SankeyOperacaoTipo => {
                                                                            self.limite_sankey_tipos = op;
                                                                        }
                                                                        TipoGrafico::SankeyEventoDiretorio => {
                                                                            self.limite_sankey_diretorios = op;
                                                                        }
                                                                        TipoGrafico::SankeyTipoDiretorio => {
                                                                            self.limite_sankey_diretorios = op;
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                    egui::Popup::close_all(ui.ctx());
                                                                }
                                                            }
                                                        });

                                                    let dest_popup_id = combo_dest.response.id.with("popup");
                                                    if egui::ComboBox::is_open(ui.ctx(), combo_dest.response.id) {
                                                        let mut full_rect = combo_dest.response.rect;
                                                        if let Some(pr) = ui.ctx().memory(|m| m.area_rect(dest_popup_id)) {
                                                            full_rect = full_rect.union(pr);
                                                        }
                                                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos())) {
                                                            if !full_rect.expand(8.0).contains(pos) {
                                                                egui::Popup::close_id(ui.ctx(), dest_popup_id);
                                                                ui.ctx().request_repaint();
                                                            }
                                                        }
                                                    }

                                                    ui.label(
                                                        RichText::new(lbl_dest_combo)
                                                            .strong()
                                                            .size(11.0)
                                                            .color(combo_lbl_color),
                                                    );

                                                    ui.add_space(8.0);

                                                    // Dropdown 1: Origem
                                                    if matches!(
                                                        grafico_selecionado,
                                                        TipoGrafico::SankeyOperacaoTipo
                                                            | TipoGrafico::SankeyEventoDiretorio
                                                    ) {
                                                        let total_ops =
                                                            crate::core::types::OperationType::all().len();
                                                        let ops_set = match grafico_selecionado {
                                                            TipoGrafico::SankeyOperacaoTipo => {
                                                                &mut self.ops_sankey_op_tipo
                                                            }
                                                            _ => &mut self.ops_sankey_op_dir,
                                                        };

                                                        let sel_count = ops_set.len();
                                                        let txt_orig = if sel_count == total_ops {
                                                            loc.t("chart_limit_all_operations")
                                                        } else if sel_count == 0 {
                                                            loc.t("chart_none_selected")
                                                        } else if sel_count == 1 {
                                                            let op = ops_set.iter().next().unwrap();
                                                            loc.traduzir_tipo_operacao(op.as_str())
                                                        } else {
                                                            loc.t_args(
                                                                "chart_operations_selected_count",
                                                                &[&sel_count.to_string()],
                                                            )
                                                        };

                                                        let combo_orig = egui::ComboBox::from_id_salt(id_orig)
                                                            .selected_text(
                                                                RichText::new(txt_orig)
                                                                    .strong()
                                                                    .size(11.0),
                                                            )
                                                            .width(135.0)
                                                            .show_ui(ui, |ui| {
                                                                let mut todas_selecionadas =
                                                                    ops_set.len() == total_ops;
                                                                if ui
                                                                    .checkbox(
                                                                        &mut todas_selecionadas,
                                                                        RichText::new(loc.t("chart_limit_all_operations"))
                                                                            .strong()
                                                                            .size(11.0),
                                                                    )
                                                                    .changed()
                                                                {
                                                                    if todas_selecionadas {
                                                                        *ops_set = crate::core::types::OperationType::all()
                                                                            .iter()
                                                                            .copied()
                                                                            .collect();
                                                                    } else {
                                                                        ops_set.clear();
                                                                    }
                                                                }

                                                                ui.separator();

                                                                for op in crate::core::types::OperationType::all() {
                                                                    let mut checked = ops_set.contains(op);
                                                                    let nome_op = loc.traduzir_tipo_operacao(op.as_str());
                                                                    let cor_chip = match op {
                                                                        crate::core::types::OperationType::Adicionado => {
                                                                            Color32::from_rgb(46, 125, 50)
                                                                        }
                                                                        crate::core::types::OperationType::Modificado => {
                                                                            Color32::from_rgb(21, 101, 192)
                                                                        }
                                                                        crate::core::types::OperationType::Excluido => {
                                                                            Color32::from_rgb(198, 40, 40)
                                                                        }
                                                                        crate::core::types::OperationType::Renomeado => {
                                                                            Color32::from_rgb(106, 27, 154)
                                                                        }
                                                                        crate::core::types::OperationType::Movido => {
                                                                            Color32::from_rgb(239, 108, 0)
                                                                        }
                                                                        crate::core::types::OperationType::Escaneado => {
                                                                            Color32::from_rgb(55, 71, 79)
                                                                        }
                                                                    };

                                                                    ui.horizontal(|ui| {
                                                                        if ui.checkbox(&mut checked, "").changed() {
                                                                            if checked {
                                                                                ops_set.insert(*op);
                                                                            } else {
                                                                                ops_set.remove(op);
                                                                            }
                                                                        }
                                                                        let (rect_dot, _) = ui.allocate_exact_size(
                                                                            Vec2::new(10.0, 14.0),
                                                                            egui::Sense::hover(),
                                                                        );
                                                                        ui.painter().circle_filled(
                                                                            rect_dot.center(),
                                                                            3.5,
                                                                            cor_chip,
                                                                        );
                                                                        if ui
                                                                            .selectable_label(
                                                                                checked,
                                                                                RichText::new(nome_op).size(11.0),
                                                                            )
                                                                            .clicked()
                                                                        {
                                                                            if checked {
                                                                                ops_set.remove(op);
                                                                            } else {
                                                                                ops_set.insert(*op);
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                            });

                                                        let orig_popup_id = combo_orig.response.id.with("popup");
                                                        if egui::ComboBox::is_open(ui.ctx(), combo_orig.response.id) {
                                                            let mut full_rect = combo_orig.response.rect;
                                                            if let Some(pr) = ui.ctx().memory(|m| m.area_rect(orig_popup_id)) {
                                                                full_rect = full_rect.union(pr);
                                                            }
                                                            if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos())) {
                                                                if !full_rect.expand(8.0).contains(pos) {
                                                                    egui::Popup::close_id(ui.ctx(), orig_popup_id);
                                                                    ui.ctx().request_repaint();
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        // SankeyTipoDiretorio: Limite de Origem (Tipos de Arquivo)
                                                        let limite_orig_val = self.limite_sankey_tipos;
                                                        let txt_orig = match limite_orig_val {
                                                            Some(n) => n.to_string(),
                                                            None => loc.t("chart_limit_all"),
                                                        };

                                                        let combo_orig = egui::ComboBox::from_id_salt(id_orig)
                                                            .selected_text(
                                                                RichText::new(txt_orig)
                                                                    .strong()
                                                                    .size(11.0),
                                                            )
                                                            .width(76.0)
                                                            .show_ui(ui, |ui| {
                                                                let opcoes: [Option<usize>; 11] = [
                                                                    Some(10),
                                                                    Some(20),
                                                                    Some(30),
                                                                    Some(40),
                                                                    Some(50),
                                                                    Some(60),
                                                                    Some(70),
                                                                    Some(80),
                                                                    Some(90),
                                                                    Some(100),
                                                                    None,
                                                                ];
                                                                for op in opcoes {
                                                                    let rotulo = match op {
                                                                        Some(n) => n.to_string(),
                                                                        None => loc.t("chart_limit_all"),
                                                                    };
                                                                    let sel = limite_orig_val == op;
                                                                    if ui
                                                                        .selectable_label(
                                                                            sel,
                                                                            RichText::new(rotulo)
                                                                                .size(11.0),
                                                                        )
                                                                        .clicked()
                                                                    {
                                                                        self.limite_sankey_tipos = op;
                                                                        egui::Popup::close_all(ui.ctx());
                                                                    }
                                                                }
                                                            });

                                                        let orig_popup_id = combo_orig.response.id.with("popup");
                                                        if egui::ComboBox::is_open(ui.ctx(), combo_orig.response.id) {
                                                            let mut full_rect = combo_orig.response.rect;
                                                            if let Some(pr) = ui.ctx().memory(|m| m.area_rect(orig_popup_id)) {
                                                                full_rect = full_rect.union(pr);
                                                            }
                                                            if let Some(pos) = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos())) {
                                                                if !full_rect.expand(8.0).contains(pos) {
                                                                    egui::Popup::close_id(ui.ctx(), orig_popup_id);
                                                                    ui.ctx().request_repaint();
                                                                }
                                                            }
                                                        }
                                                    }

                                                    ui.label(
                                                        RichText::new(lbl_orig_combo)
                                                            .strong()
                                                            .size(11.0)
                                                            .color(combo_lbl_color),
                                                    );
                                                },
                                            );
                                        });
                                    });

                                ui.add_space(2.0);

                                let scroll_region_h =
                                    (chart_h - bar_ctrl_h - 4.0).max(140.0);
                                let needed_h = (chart_data.max_nos_sankey() as f32 * 34.0 + 80.0)
                                    .max(scroll_region_h);

                                if needed_h > scroll_region_h {
                                    ui.scope(|ui| {
                                        ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                                        ScrollArea::vertical()
                                            .id_salt((
                                                "stats_sankey_vertical_scroll",
                                                grafico_selecionado as usize,
                                            ))
                                            .max_width(chart_w)
                                            .max_height(scroll_region_h)
                                            .auto_shrink([false, false])
                                            .scroll_bar_visibility(
                                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                            )
                                            .show(ui, |ui| {
                                                renderizar_grafico(
                                                    ui,
                                                    grafico_selecionado,
                                                    chart_data,
                                                    Vec2::new(chart_w - 18.0, needed_h),
                                                    loc,
                                                );
                                            });
                                    });
                                } else {
                                    renderizar_grafico(
                                        ui,
                                        grafico_selecionado,
                                        chart_data,
                                        Vec2::new(chart_w, scroll_region_h),
                                        loc,
                                    );
                                }
                            } else {
                                let needed_h = match grafico_selecionado {
                                    TipoGrafico::ArvoreDiretorios => {
                                        (chart_data.tree_nodes.len() as f32 * 22.0 + 40.0)
                                            .max(chart_h)
                                    }
                                    _ => chart_h,
                                };

                                if needed_h > chart_h {
                                    ScrollArea::vertical()
                                        .id_salt("stats_chart_vertical_scroll")
                                        .max_width(chart_w)
                                        .max_height(chart_h)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            renderizar_grafico(
                                                ui,
                                                grafico_selecionado,
                                                chart_data,
                                                Vec2::new(chart_w - 14.0, needed_h),
                                                loc,
                                            );
                                        });
                                } else {
                                    renderizar_grafico(
                                        ui,
                                        grafico_selecionado,
                                        chart_data,
                                        Vec2::new(chart_w, chart_h),
                                        loc,
                                    );
                                }
                            }
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            // 1. Exportar gráfico ativo individual
                            if ui
                                .add_sized(
                                    [140.0, 24.0],
                                    egui::Button::new(RichText::new(loc.t("export_chart_png")).size(12.0)),
                                )
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name(&format!(
                                        "{}.png",
                                        grafico_selecionado
                                            .titulo(loc)
                                            .replace([' ', ':', '→'], "_")
                                    ))
                                    .add_filter(&loc.t("png_image"), &["png"])
                                    .save_file()
                                {
                                    if let Err(e) = exportar_grafico_png(
                                        grafico_selecionado,
                                        &chart_data,
                                        &path,
                                        chart_w.max(800.0) as u32,
                                        chart_h.max(500.0) as u32,
                                    ) {
                                        get_logger()
                                            .error(&format!("Falha ao salvar gráfico: {}", e));
                                    }
                                }
                            }

                            ui.add_space(8.0);

                            // 2. Exportar todos os gráficos juntos de uma só vez
                            if ui
                                .add_sized(
                                    [140.0, 24.0],
                                    egui::Button::new(
                                        RichText::new(format!("📁 {}", loc.t("export_all_charts")))
                                            .size(12.0),
                                    ),
                                )
                                .on_hover_text("Exportar todos os 17 gráficos e diagramas juntos para uma pasta")
                                .clicked()
                            {
                                Self::exportar_todos_graficos(eventos, loc, chart_w, chart_h);
                            }

                            // 3. Botão Fechar no canto inferior direito
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(8.0);
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
            },
        );

        self.aberto = state.aberto;
        self.grafico_selecionado = grafico_selecionado;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
        self.elevar_z_order = state.elevar_z_order;
    }

    fn desenhar_botao_navegacao_aba(
        ui: &mut egui::Ui,
        id_str: &str,
        emoji: &str,
        tamanho: Vec2,
        tooltip: String,
    ) -> egui::Response {
        let resp = ui
            .push_id(id_str, |ui| {
                let (rect, response) = ui.allocate_exact_size(tamanho, egui::Sense::click());
                if ui.is_rect_visible(rect) {
                    let visuals = ui.style().interact(&response);
                    let rect_exp = rect.expand(visuals.expansion);
                    ui.painter().rect(
                        rect_exp,
                        visuals.corner_radius,
                        visuals.bg_fill,
                        visuals.bg_stroke,
                        egui::StrokeKind::Inside,
                    );

                    // Centraliza o emoji tanto na horizontal quanto na vertical das dimensões do botão
                    let font_id = egui::FontId::proportional(11.5);
                    let text_color = visuals.text_color();
                    let galley =
                        ui.painter()
                            .layout_no_wrap(emoji.to_string(), font_id, text_color);
                    let glyph_center = if galley.mesh_bounds.is_positive() {
                        galley.mesh_bounds.center()
                    } else {
                        galley.rect.center()
                    };
                    let glyph_pos = rect.center() - glyph_center.to_vec2();
                    ui.painter().galley(glyph_pos, galley, text_color);
                }
                response
            })
            .inner;
        resp.on_hover_text(tooltip)
    }

    fn gerar_dados(tipo: TipoGrafico, eventos: &[EventModel], loc: &Localizer) -> ChartData {
        match tipo {
            TipoGrafico::PizzaOperacoes => {
                GeradorEstatisticas::gerar_pizza_operacoes_loc(eventos, loc)
            }
            TipoGrafico::BarrasTipos => GeradorEstatisticas::gerar_barras_tipos_loc(eventos, loc),
            TipoGrafico::TimelineOperacoes => {
                GeradorEstatisticas::gerar_timeline_operacoes_loc(eventos, loc)
            }
            TipoGrafico::TreemapTipos => GeradorEstatisticas::gerar_treemap_tipos_loc(eventos, loc),
            TipoGrafico::HistogramaHorarios => {
                GeradorEstatisticas::gerar_histograma_horarios(eventos)
            }
            TipoGrafico::ParetoExtensoes => {
                GeradorEstatisticas::gerar_pareto_extensoes_loc(eventos, loc)
            }
            TipoGrafico::LinhaEventosHora => {
                GeradorEstatisticas::gerar_linha_eventos_hora_loc(eventos, loc)
            }
            TipoGrafico::BoxplotTamanhos => {
                GeradorEstatisticas::gerar_boxplot_tamanhos_loc(eventos, loc)
            }
            TipoGrafico::RadarOperacoes => {
                GeradorEstatisticas::gerar_radar_operacoes_loc(eventos, loc)
            }
            TipoGrafico::HeatmapDiaHora => {
                GeradorEstatisticas::gerar_heatmap_dia_hora_loc(eventos, loc)
            }
            TipoGrafico::ScatterTamanhoHora => {
                GeradorEstatisticas::gerar_scatter_tamanho_hora_loc(eventos, loc)
            }
            TipoGrafico::SankeyOperacaoTipo => {
                GeradorEstatisticas::gerar_sankey_operacao_tipo_loc(eventos, loc)
            }
            TipoGrafico::RadarMultidimensional => {
                GeradorEstatisticas::gerar_radar_multidimensional_loc(eventos, loc)
            }
            TipoGrafico::DotplotDistribuicao => {
                GeradorEstatisticas::gerar_dotplot_distribuicao_loc(eventos, loc)
            }
            TipoGrafico::SankeyEventoDiretorio => {
                GeradorEstatisticas::gerar_sankey_evento_diretorio_loc(eventos, loc)
            }
            TipoGrafico::SankeyTipoDiretorio => {
                GeradorEstatisticas::gerar_sankey_tipo_diretorio_loc(eventos, loc)
            }
            TipoGrafico::ArvoreDiretorios => {
                GeradorEstatisticas::gerar_arvore_diretorios_loc(eventos, loc)
            }
        }
    }

    pub fn exportar_todos_graficos(
        eventos: &[EventModel],
        loc: &Localizer,
        chart_w: f32,
        chart_h: f32,
    ) {
        if let Some(pasta) = rfd::FileDialog::new()
            .set_title("Selecionar Pasta para Exportar Todos os Gráficos")
            .pick_folder()
        {
            let mut total_exportados = 0;
            let mut erros = Vec::new();
            let lista_graficos = TipoGrafico::all();

            for (idx, &tipo) in lista_graficos.iter().enumerate() {
                let dados = Self::gerar_dados(tipo, eventos, loc);

                let titulo = tipo.titulo(loc);
                let nome_limpo: String = titulo
                    .chars()
                    .map(|c| match c {
                        ' ' | '-' => '_',
                        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '→' => '_',
                        _ => c,
                    })
                    .collect();

                let nome_arquivo = format!("{:02}_{}.png", idx + 1, nome_limpo);
                let caminho_arquivo = pasta.join(nome_arquivo);

                let export_h = match tipo {
                    TipoGrafico::ArvoreDiretorios => {
                        ((dados.tree_nodes.len() as u32 * 24) + 120).max(720)
                    }
                    TipoGrafico::SankeyOperacaoTipo
                    | TipoGrafico::SankeyEventoDiretorio
                    | TipoGrafico::SankeyTipoDiretorio => {
                        ((dados.flows.len() as u32 * 28) + 120).max(720)
                    }
                    _ => chart_h.max(600.0) as u32,
                };
                let export_w = chart_w.max(1000.0) as u32;

                match exportar_grafico_png(tipo, &dados, &caminho_arquivo, export_w, export_h) {
                    Ok(_) => total_exportados += 1,
                    Err(e) => {
                        get_logger().error(&format!("Falha ao exportar {:?}: {}", tipo, e));
                        erros.push(format!("{:?}: {}", tipo, e));
                    }
                }
            }

            if erros.is_empty() {
                get_logger().info(&format!(
                    "Todos os {} gráficos foram exportados com sucesso para {:?}",
                    total_exportados, pasta
                ));
                let msg = loc.t_args(
                    "export_all_charts_success",
                    &[&total_exportados.to_string(), &pasta.display().to_string()],
                );
                let _ = rfd::MessageDialog::new()
                    .set_title(&loc.t("export_all_charts_title"))
                    .set_description(&msg)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Info)
                    .show();
            } else {
                let msg = format!(
                    "Exportação concluída com avisos:\n{} gráficos exportados com sucesso.\n{} falhas:\n{}",
                    total_exportados,
                    erros.len(),
                    erros.join("\n")
                );
                let _ = rfd::MessageDialog::new()
                    .set_title("Exportação com Avisos")
                    .set_description(&msg)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Warning)
                    .show();
            }
        }
    }
}
