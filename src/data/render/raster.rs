use egui::{Color32, FontId, Pos2, Rect, Response, Stroke, Ui, Vec2};
use std::f32::consts::PI;

use super::super::charts::TipoGrafico;
use super::super::generator::{calcular_limites_log10_kb, calcular_log10_kb, ChartData};
use crate::locale::Localizer;

const CORES_PALETA: &[Color32] = &[
    Color32::from_rgb(46, 125, 50),  // Verde Floresta
    Color32::from_rgb(21, 101, 192), // Azul Safira
    Color32::from_rgb(198, 40, 40),  // Vermelho Rubi
    Color32::from_rgb(106, 27, 154), // Roxo Ametista
    Color32::from_rgb(239, 108, 0),  // Laranja Âmbar
    Color32::from_rgb(0, 137, 123),  // Ciano Esmeralda
    Color32::from_rgb(92, 107, 115), // Ardósia
    Color32::from_rgb(124, 77, 255), // Violeta Neon
    Color32::from_rgb(255, 179, 0),  // Amarelo Dourado
    Color32::from_rgb(233, 30, 99),  // Rosa Escarlate
];

#[derive(Debug, Clone, Copy)]
pub struct ChartTheme {
    pub dark_mode: bool,
    pub bg_card: Color32,
    pub border_card: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_accent: Color32,
    pub grid_line: Color32,
    pub grid_line_subtle: Color32,
    pub axis_line: Color32,
    pub panel_bg: Color32,
    pub panel_border: Color32,
}

impl ChartTheme {
    pub fn new(dark_mode: bool) -> Self {
        if dark_mode {
            Self {
                dark_mode: true,
                bg_card: Color32::from_rgb(24, 26, 31),
                border_card: Color32::from_rgb(45, 48, 58),
                text_primary: Color32::from_rgb(240, 243, 248),
                text_secondary: Color32::from_rgb(180, 190, 205),
                text_muted: Color32::from_rgb(130, 138, 150),
                text_accent: Color32::from_rgb(100, 181, 246),
                grid_line: Color32::from_rgb(45, 48, 58),
                grid_line_subtle: Color32::from_rgb(34, 37, 46),
                axis_line: Color32::from_rgb(180, 190, 205),
                panel_bg: Color32::from_rgb(30, 33, 41),
                panel_border: Color32::from_rgb(55, 60, 74),
            }
        } else {
            Self {
                dark_mode: false,
                bg_card: Color32::from_rgb(255, 255, 255),
                border_card: Color32::from_rgb(218, 224, 233),
                text_primary: Color32::from_rgb(20, 24, 35),
                text_secondary: Color32::from_rgb(70, 80, 95),
                text_muted: Color32::from_rgb(115, 125, 140),
                text_accent: Color32::from_rgb(21, 101, 192),
                grid_line: Color32::from_rgb(228, 233, 242),
                grid_line_subtle: Color32::from_rgb(240, 244, 250),
                axis_line: Color32::from_rgb(90, 105, 125),
                panel_bg: Color32::from_rgb(245, 247, 250),
                panel_border: Color32::from_rgb(210, 218, 228),
            }
        }
    }
}

pub fn renderizar_grafico(
    ui: &mut Ui,
    tipo: TipoGrafico,
    data: &ChartData,
    tamanho: Vec2,
    loc: &Localizer,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(tamanho, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let theme = ChartTheme::new(ui.visuals().dark_mode);

    // Fundo do gráfico com cantos arredondados e borda sensível dinamicamente ao tema escuro/claro
    painter.rect_filled(rect, 8_u8, theme.bg_card);
    painter.rect_stroke(
        rect,
        8_u8,
        Stroke::new(1.0, theme.border_card),
        egui::StrokeKind::Inside,
    );

    let inner_rect = rect.shrink(16.0);

    match tipo {
        TipoGrafico::PizzaOperacoes => desenhar_pizza(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::BarrasTipos => desenhar_barras(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::TimelineOperacoes => desenhar_timeline(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::TreemapTipos => desenhar_treemap(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::HistogramaHorarios => desenhar_histograma(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::ParetoExtensoes => desenhar_pareto(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::LinhaEventosHora => desenhar_linha(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::BoxplotTamanhos => desenhar_boxplot(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::RadarOperacoes => desenhar_radar(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::HeatmapDiaHora => desenhar_heatmap(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::ScatterTamanhoHora => desenhar_scatter(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::SankeyOperacaoTipo
        | TipoGrafico::SankeyEventoDiretorio
        | TipoGrafico::SankeyTipoDiretorio => desenhar_sankey(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::RadarMultidimensional => {
            desenhar_radar_multidimensional(&painter, inner_rect, data, loc, &theme)
        }
        TipoGrafico::DotplotDistribuicao => desenhar_dotplot(&painter, inner_rect, data, loc, &theme),
        TipoGrafico::ArvoreDiretorios => desenhar_arvore_diretorios(&painter, inner_rect, data, loc, &theme),
    }

    response
}

fn desenhar_pizza(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.values.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let total: f64 = data.values.iter().sum();
    if total <= 0.0 {
        return;
    }

    // Reserva faixa dedicada à direita para a legenda sem sobrepor a circunferência da pizza
    let legend_panel_w = (rect.width() * 0.30).clamp(200.0, 260.0);
    let pie_area_w = (rect.width() - legend_panel_w - 32.0).max(80.0);
    let radius = ((rect.height() - 20.0).min(pie_area_w) / 2.0).max(30.0);
    let center = Pos2::new(rect.left() + pie_area_w * 0.5, rect.center().y);

    let active_slices = data.values.iter().filter(|&&v| v > 0.0).count();

    if active_slices == 1 {
        // Operação única: renderiza circunferência preenchida integral sem emendas ou linhas divisórias
        if let Some((idx, _)) = data.values.iter().enumerate().find(|(_, &v)| v > 0.0) {
            let color = CORES_PALETA[idx % CORES_PALETA.len()];
            painter.circle_filled(center, radius, color);
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(1.0, theme.border_card),
            );
        }
    } else {
        let mut start_angle = -PI / 2.0;

        for (idx, &val) in data.values.iter().enumerate() {
            if val <= 0.0 {
                continue;
            }
            let pct = (val / total) as f32;
            let sweep = pct * 2.0 * PI;
            let color = CORES_PALETA[idx % CORES_PALETA.len()];

            // Malha triangular vetorial precisa sem artefatos ou linhas divisórias indesejadas
            let segments = (sweep * 32.0).ceil().max(4.0) as usize;
            let mut mesh = egui::Mesh::default();
            let c_idx = 0u32;
            mesh.vertices.push(egui::epaint::Vertex {
                pos: center,
                uv: Pos2::ZERO,
                color,
            });
            for i in 0..=segments {
                let a = start_angle + (i as f32 / segments as f32) * sweep;
                let pt = Pos2::new(center.x + a.cos() * radius, center.y + a.sin() * radius);
                let v_idx = mesh.vertices.len() as u32;
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: pt,
                    uv: Pos2::ZERO,
                    color,
                });
                if i > 0 {
                    mesh.add_triangle(c_idx, v_idx - 1, v_idx);
                }
            }
            painter.add(egui::Shape::mesh(mesh));

            // Linha sutil de raio demarcando apenas a transição entre fatias reais
            let spoke_pt = Pos2::new(
                center.x + start_angle.cos() * radius,
                center.y + start_angle.sin() * radius,
            );
            painter.line_segment(
                [center, spoke_pt],
                Stroke::new(1.5, theme.bg_card),
            );

            start_angle += sweep;
        }

        // Borda perimetral externa da pizza
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0, theme.border_card),
        );
    }

    // Legenda lateral posicionada à direita da extremidade externa da pizza (center.x + radius)
    let mut legend_y = rect.top() + 14.0;
    let legend_x = center.x + radius + 36.0;

    let leg_title = format!("🏷️ {}:", loc.t("chart_legend_title"));
    painter.text(
        Pos2::new(legend_x, legend_y),
        egui::Align2::LEFT_TOP,
        leg_title,
        FontId::proportional(12.0),
        theme.text_accent,
    );
    legend_y += 20.0;

    for (idx, label) in data.labels.iter().enumerate().take(10) {
        let color = CORES_PALETA[idx % CORES_PALETA.len()];
        let val = data.values.get(idx).copied().unwrap_or(0.0);
        let pct = (val / total * 100.0) as f32;
        let trad_label = loc.traduzir_tipo_operacao(label);

        painter.rect_filled(
            Rect::from_min_size(Pos2::new(legend_x, legend_y + 2.0), Vec2::new(10.0, 10.0)),
            2_u8,
            color,
        );
        let text = format!("{}: {:.0} ({:.1}%)", trad_label, val, pct);
        painter.text(
            Pos2::new(legend_x + 16.0, legend_y),
            egui::Align2::LEFT_TOP,
            text,
            FontId::proportional(11.5),
            theme.text_primary,
        );
        legend_y += 19.0;
    }
}

/// Abrevia rótulos longos com segurança para UTF-8 (sem fatiar bytes no meio de um caractere).
fn abreviar_rotulo(label: &str, max_chars: usize) -> String {
    let max_chars = max_chars.max(3);
    if label.chars().count() > max_chars {
        let prefixo: String = label.chars().take(max_chars - 2).collect();
        format!("{}..", prefixo)
    } else {
        label.to_string()
    }
}

/// Formata um tamanho expresso em KB para a unidade mais legível (KB, MB ou GB).
fn formatar_tamanho_kb(kb: f64) -> String {
    if kb >= 1024.0 * 1024.0 {
        format!("{:.2} GB", kb / (1024.0 * 1024.0))
    } else if kb >= 1024.0 {
        format!("{:.1} MB", kb / 1024.0)
    } else {
        format!("{:.1} KB", kb)
    }
}

/// Rótulo legível correspondente à década logarítmica de base 10 (`10^k` KB).
fn rotulo_decada_log10_kb(k: i32) -> &'static str {
    match k {
        ..=-1 => "0.1 KB",
        0 => "1 KB",
        1 => "10 KB",
        2 => "100 KB",
        3 => "1 MB",
        4 => "10 MB",
        5 => "100 MB",
        _ => "1 GB",
    }
}

/// Barras por Tipo de Arquivo: quantidade total por extensão, de todas as extensões monitoradas.
fn desenhar_barras(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.values.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let total_v: f64 = data.values.iter().sum();
    let n = data.values.len();
    let leg_text = format!(
        "🏷️ {}: {} ({}: {} | {}: {:.0})",
        loc.t("chart_legend_title"),
        loc.t("chart_legend_bars_extension"),
        loc.t("chart_extensions_label"),
        n,
        loc.t("stat_total"),
        total_v
    );
    painter.text(
        Pos2::new(rect.left() + 5.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let max_val = data.values.iter().copied().fold(0.0, f64::max).max(1.0) as f32;
    let bar_spacing = if n > 30 { 5.0 } else { 8.0 };
    let total_spacing = bar_spacing * (n as f32 + 1.0);
    let bar_width = ((rect.width() - total_spacing) / n as f32).max(4.0);
    // Quantidade de caracteres que cabem sob cada barra (~6 px por caractere na fonte 10)
    let max_chars = (((bar_width + bar_spacing) / 6.0).floor() as usize).clamp(3, 14);

    let base_y = rect.bottom() - 30.0;
    let plot_height = base_y - rect.top() - 35.0;

    // Linha de base X
    painter.line_segment(
        [
            Pos2::new(rect.left(), base_y),
            Pos2::new(rect.right(), base_y),
        ],
        Stroke::new(1.0, theme.axis_line),
    );

    for (i, (&val, label)) in data.values.iter().zip(&data.labels).enumerate() {
        let x = rect.left() + bar_spacing + i as f32 * (bar_width + bar_spacing);
        let h = ((val as f32) / max_val) * plot_height;
        let y = base_y - h;

        let bar_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(bar_width, h));
        let color = CORES_PALETA[i % CORES_PALETA.len()];

        painter.rect_filled(bar_rect, 4_u8, color);

        // Valor acima da barra
        painter.text(
            Pos2::new(x + bar_width / 2.0, y - 4.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{:.0}", val),
            FontId::proportional(11.0),
            theme.text_secondary,
        );

        // Rótulo da extensão abaixo da barra (abreviado se longo)
        painter.text(
            Pos2::new(x + bar_width / 2.0, base_y + 6.0),
            egui::Align2::CENTER_TOP,
            abreviar_rotulo(label, max_chars),
            FontId::proportional(10.0),
            theme.text_muted,
        );
    }
}

fn desenhar_histograma(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.values.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let total_v: f64 = data.values.iter().sum();
    let leg_text = format!(
        "🏷️ {}: {} ({}: {:.0})",
        loc.t("chart_legend_title"),
        loc.t("chart_legend_hist_hour"),
        loc.t("stat_total"),
        total_v
    );
    painter.text(
        Pos2::new(rect.left() + 5.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let max_val = data.values.iter().copied().fold(0.0, f64::max).max(1.0) as f32;
    let n = data.values.len();
    let bar_spacing = 4.0;
    let total_spacing = bar_spacing * (n as f32 + 1.0);
    let bar_width = ((rect.width() - total_spacing) / n as f32).max(6.0);

    let base_y = rect.bottom() - 30.0;
    let plot_height = base_y - rect.top() - 35.0;

    painter.line_segment(
        [
            Pos2::new(rect.left(), base_y),
            Pos2::new(rect.right(), base_y),
        ],
        Stroke::new(1.0, theme.axis_line),
    );

    for (i, (&val, label)) in data.values.iter().zip(&data.labels).enumerate() {
        let x = rect.left() + bar_spacing + i as f32 * (bar_width + bar_spacing);
        let h = ((val as f32) / max_val) * plot_height;
        let y = base_y - h;

        let bar_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(bar_width, h));
        let color = CORES_PALETA[i % CORES_PALETA.len()];

        painter.rect_filled(bar_rect, 3_u8, color);

        if val > 0.0 {
            painter.text(
                Pos2::new(x + bar_width / 2.0, y - 4.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{:.0}", val),
                FontId::proportional(9.5),
                theme.text_secondary,
            );
        }

        let display_label = if label.len() > 6 {
            &label[..5]
        } else {
            label.as_str()
        };
        painter.text(
            Pos2::new(x + bar_width / 2.0, base_y + 6.0),
            egui::Align2::CENTER_TOP,
            display_label,
            FontId::proportional(9.0),
            theme.text_muted,
        );
    }
}

/// Gráfico de Pareto: quantidade total por extensão (todas as extensões monitoradas), ordenada de
/// forma decrescente, com curva de percentual acumulado, eixo secundário (0–100%), linha de corte
/// em 80% e destaque dos "poucos vitais" (extensões que concentram até 80% das ocorrências).
fn desenhar_pareto(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.values.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let cor_vital = Color32::from_rgb(21, 101, 192);
    let cor_trivial = Color32::from_rgb(92, 107, 115);
    let cor_curva = Color32::from_rgb(255, 179, 0);
    let cor_limite = Color32::from_rgb(239, 83, 80);

    let n = data.values.len();
    let total_v: f64 = data.values.iter().sum();

    // Quantidade de extensões "vitais": até (e incluindo) a primeira que atinge 80% acumulado
    let qtd_vitais = data
        .secondary_values
        .iter()
        .position(|&pct| pct >= 80.0 - 1e-9)
        .map(|idx| idx + 1)
        .unwrap_or(n)
        .min(n);

    // Linha 1 da legenda
    let leg_text = format!(
        "🏷️ {}: [◼] {}   |   [●] {}   |   [┄] {}",
        loc.t("chart_legend_title"),
        loc.t("chart_legend_pareto_individual"),
        loc.t("chart_legend_pareto_cumulative"),
        loc.t("chart_legend_pareto_threshold"),
    );
    painter.text(
        Pos2::new(rect.left() + 5.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    // Linha 2 da legenda: poucos vitais x muitos triviais
    let leg2_y = rect.top() + 26.0;
    let mut cur_x = rect.left() + 8.0;
    painter.rect_filled(
        Rect::from_center_size(Pos2::new(cur_x + 5.0, leg2_y), Vec2::splat(10.0)),
        2_u8,
        cor_vital,
    );
    let txt_vital = format!(
        "{}: {} / {} {}",
        loc.t("chart_legend_pareto_vital"),
        qtd_vitais,
        n,
        loc.t("chart_extensions_label").to_lowercase()
    );
    let r_vital = painter.text(
        Pos2::new(cur_x + 14.0, leg2_y),
        egui::Align2::LEFT_CENTER,
        txt_vital,
        FontId::proportional(10.5),
        theme.text_secondary,
    );
    cur_x = r_vital.right() + 18.0;
    painter.rect_filled(
        Rect::from_center_size(Pos2::new(cur_x + 5.0, leg2_y), Vec2::splat(10.0)),
        2_u8,
        cor_trivial,
    );
    let r_trivial = painter.text(
        Pos2::new(cur_x + 14.0, leg2_y),
        egui::Align2::LEFT_CENTER,
        loc.t("chart_legend_pareto_trivial"),
        FontId::proportional(10.5),
        theme.text_secondary,
    );
    painter.text(
        Pos2::new(r_trivial.right() + 18.0, leg2_y),
        egui::Align2::LEFT_CENTER,
        format!("({}: {:.0})", loc.t("stat_total"), total_v),
        FontId::proportional(10.5),
        theme.text_muted,
    );

    // Área de plotagem (reserva margem direita para o eixo secundário em %)
    let plot_left = rect.left();
    let plot_right = rect.right() - 38.0;
    let plot_w = (plot_right - plot_left).max(10.0);
    let base_y = rect.bottom() - 30.0;
    let top_y = rect.top() + 50.0;
    let plot_height = (base_y - top_y).max(10.0);

    let max_val = data.values.iter().copied().fold(0.0, f64::max).max(1.0) as f32;
    let bar_spacing = if n > 30 { 5.0 } else { 8.0 };
    let total_spacing = bar_spacing * (n as f32 + 1.0);
    let bar_width = ((plot_w - total_spacing) / n as f32).max(4.0);
    let slot = bar_width + bar_spacing;
    let max_chars = ((slot / 6.0).floor() as usize).clamp(3, 14);
    let map_pct_y = |pct: f32| -> f32 { base_y - (pct / 100.0) * plot_height };

    // Grade e eixo secundário (0% .. 100%)
    painter.line_segment(
        [Pos2::new(plot_right, top_y), Pos2::new(plot_right, base_y)],
        Stroke::new(1.0, theme.axis_line),
    );
    for step in 0..=5 {
        let pct = step as f32 * 20.0;
        let y = map_pct_y(pct);
        painter.line_segment(
            [Pos2::new(plot_left, y), Pos2::new(plot_right, y)],
            Stroke::new(0.5, theme.grid_line),
        );
        painter.line_segment(
            [Pos2::new(plot_right, y), Pos2::new(plot_right + 4.0, y)],
            Stroke::new(1.0, theme.axis_line),
        );
        painter.text(
            Pos2::new(plot_right + 7.0, y),
            egui::Align2::LEFT_CENTER,
            format!("{:.0}%", pct),
            FontId::proportional(9.5),
            Color32::from_rgb(255, 213, 79),
        );
    }

    // Linha de base X
    painter.line_segment(
        [
            Pos2::new(plot_left, base_y),
            Pos2::new(plot_right, base_y),
        ],
        Stroke::new(1.0, theme.axis_line),
    );

    for (i, (&val, label)) in data.values.iter().zip(&data.labels).enumerate() {
        let x = plot_left + bar_spacing + i as f32 * slot;
        let h = ((val as f32) / max_val) * plot_height;
        let y = base_y - h;

        let bar_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(bar_width, h));
        let color = if i < qtd_vitais { cor_vital } else { cor_trivial };

        painter.rect_filled(bar_rect, 4_u8, color);

        painter.text(
            Pos2::new(x + bar_width / 2.0, y - 4.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{:.0}", val),
            FontId::proportional(11.0),
            theme.text_primary,
        );

        painter.text(
            Pos2::new(x + bar_width / 2.0, base_y + 6.0),
            egui::Align2::CENTER_TOP,
            abreviar_rotulo(label, max_chars),
            FontId::proportional(10.0),
            if i < qtd_vitais { theme.text_accent } else { theme.text_muted },
        );
    }

    // Linha de corte de 80% (princípio de Pareto)
    let y80 = map_pct_y(80.0);
    painter.extend(egui::Shape::dashed_line(
        &[Pos2::new(plot_left, y80), Pos2::new(plot_right, y80)],
        Stroke::new(1.2, cor_limite),
        6.0,
        4.0,
    ));

    // Separador vertical entre "poucos vitais" e "muitos triviais"
    if qtd_vitais < n {
        let sep_x = plot_left + bar_spacing / 2.0 + qtd_vitais as f32 * slot;
        painter.extend(egui::Shape::dashed_line(
            &[Pos2::new(sep_x, top_y), Pos2::new(sep_x, base_y)],
            Stroke::new(1.0, theme.axis_line),
            4.0,
            4.0,
        ));
    }

    // Linha de curva cumulativa percentual
    if !data.secondary_values.is_empty() {
        let mut pts = Vec::with_capacity(data.secondary_values.len());
        for (i, &pct) in data.secondary_values.iter().enumerate() {
            let x = plot_left + bar_spacing + i as f32 * slot + bar_width / 2.0;
            let y = map_pct_y(pct as f32);
            pts.push(Pos2::new(x, y));
        }

        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(2.5, cor_curva));
        }
        let mostrar_pct = slot >= 34.0;
        for (i, pt) in pts.iter().enumerate() {
            painter.circle_filled(*pt, 3.5, Color32::from_rgb(255, 215, 0));
            if mostrar_pct {
                let pct = data.secondary_values[i];
                painter.text(
                    Pos2::new(pt.x, pt.y - 6.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.0}%", pct),
                    FontId::proportional(9.0),
                    Color32::from_rgb(255, 224, 130),
                );
            }
        }
    }
}

fn desenhar_heatmap(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.matrix_7x24.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let dias = [
        loc.t("weekday_mon"),
        loc.t("weekday_tue"),
        loc.t("weekday_wed"),
        loc.t("weekday_thu"),
        loc.t("weekday_fri"),
        loc.t("weekday_sat"),
        loc.t("weekday_sun"),
    ];

    let top_y = rect.top() + 6.0;
    let bottom_y = rect.bottom() - 32.0;
    let cell_w = (rect.width() - 50.0) / 24.0;
    let cell_h = (bottom_y - top_y) / 7.0;

    let mut max_val: f64 = 1.0;
    for row in &data.matrix_7x24 {
        for &v in row {
            if v > max_val {
                max_val = v;
            }
        }
    }

    for d in 0..7 {
        let y = top_y + d as f32 * cell_h;
        painter.text(
            Pos2::new(rect.left() + 5.0, y + cell_h / 2.0),
            egui::Align2::LEFT_CENTER,
            &dias[d],
            FontId::proportional(11.0),
            theme.text_secondary,
        );

        for h in 0..24 {
            let x = rect.left() + 40.0 + h as f32 * cell_w;
            let v = data
                .matrix_7x24
                .get(d)
                .and_then(|row| row.get(h))
                .copied()
                .unwrap_or(0.0);
            let intensity = (v / max_val) as f32;

            let color = if intensity <= 0.0 {
                theme.panel_bg
            } else {
                Color32::from_rgb(
                    (30.0 + intensity * 200.0) as u8,
                    (60.0 + intensity * 80.0) as u8,
                    (120.0 - intensity * 60.0).max(0.0) as u8,
                )
            };

            let cell_rect =
                Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_w - 2.0, cell_h - 2.0));
            painter.rect_filled(cell_rect, 2_u8, color);
        }
    }

    // Legenda de intensidade no rodapé
    let leg_y = rect.bottom() - 18.0;
    let leg_text = format!(
        "🏷️ {}: {} ──► {} ({}: {:.0})",
        loc.t("chart_legend_heat_intensity"),
        loc.t("chart_legend_heat_low"),
        loc.t("chart_legend_heat_high"),
        loc.t("stat_max"),
        max_val
    );
    painter.text(
        Pos2::new(rect.left() + 10.0, leg_y),
        egui::Align2::LEFT_CENTER,
        leg_text,
        FontId::proportional(10.5),
        theme.text_accent,
    );

    // Barra de gradiente ilustrativa
    let mut bar_x = rect.left() + 320.0;
    for step in 0..=5 {
        let frac = step as f32 / 5.0;
        let c = if frac <= 0.0 {
            theme.panel_bg
        } else {
            Color32::from_rgb(
                (30.0 + frac * 200.0) as u8,
                (60.0 + frac * 80.0) as u8,
                (120.0 - frac * 60.0).max(0.0) as u8,
            )
        };
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(bar_x, leg_y - 5.0), Vec2::new(14.0, 10.0)),
            1_u8,
            c,
        );
        bar_x += 16.0;
    }
}

fn desenhar_scatter(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.points.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    // Legenda superior indicando o Eixo Y (ordenadas) em Escala Logarítmica (Log10)
    let leg_text = format!(
        "🏷️ {}: [●] {} | {}: {} | {}: {}",
        loc.t("chart_legend_title"),
        loc.t("chart_legend_scatter"),
        loc.t("chart_axis_x"),
        loc.t("stat_scale_time"),
        loc.t("chart_axis_y"),
        loc.t("chart_file_size_log")
    );
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let top_y = rect.top() + 28.0;
    let base_y = rect.bottom() - 35.0;
    let base_x = rect.left() + 68.0;
    let plot_w = (rect.right() - 18.0 - base_x).max(40.0);
    let plot_h = (base_y - top_y).max(40.0);

    // Título do Eixo das Ordenadas (Y) rotacionado 90° no sentido anti-horário (-PI/2 rad) e centralizado
    let y_axis_title = loc.t("chart_file_size_log");
    let y_title_font = FontId::proportional(10.5);
    let y_title_color = theme.text_accent;
    let galley = painter.layout_no_wrap(y_axis_title, y_title_font, y_title_color);
    let y_axis_center = Pos2::new(rect.left() + 12.0, (top_y + base_y) * 0.5);
    let unrotated_top_left = y_axis_center - galley.size() * 0.5;
    let rotated_title = egui::epaint::TextShape::new(unrotated_top_left, galley, y_title_color)
        .with_angle_and_anchor(-PI / 2.0, egui::Align2::CENTER_CENTER);
    painter.add(egui::Shape::Text(rotated_title));

    // Limites logarítmicos de base 10 para o eixo das ordenadas (Y)
    let (log_min, log_max) = if data.secondary_values.len() >= 2
        && data.secondary_values[1] > data.secondary_values[0]
    {
        (data.secondary_values[0], data.secondary_values[1])
    } else {
        let tamanhos: Vec<f64> = data.points.iter().map(|p| p.1).collect();
        calcular_limites_log10_kb(&tamanhos)
    };
    let log_span = (log_max - log_min).max(1.0) as f32;
    let log_min_f = log_min as f32;

    let map_log_y = |log_val: f32| -> f32 {
        let norm = ((log_val - log_min_f) / log_span).clamp(0.0, 1.0);
        base_y - norm * plot_h
    };

    let dec_min = log_min.floor() as i32;
    let dec_max = log_max.ceil() as i32;

    // Sub-grade logarítmica menor (m = 2..=9 em cada década 10^k)
    for k in dec_min..dec_max {
        for m in 2..=9 {
            let sub_log = k as f32 + (m as f32).log10();
            if sub_log > log_min_f && sub_log < log_max as f32 {
                let y_sub = map_log_y(sub_log);
                painter.line_segment(
                    [Pos2::new(base_x, y_sub), Pos2::new(base_x + plot_w, y_sub)],
                    Stroke::new(0.4, theme.grid_line_subtle),
                );
                painter.line_segment(
                    [Pos2::new(base_x - 3.0, y_sub), Pos2::new(base_x, y_sub)],
                    Stroke::new(0.8, theme.axis_line),
                );
            }
        }
    }

    // Grade horizontal principal e rótulos logarítmicos do eixo Y (10^k KB)
    for k in dec_min..=dec_max {
        let y = map_log_y(k as f32);
        painter.line_segment(
            [Pos2::new(base_x, y), Pos2::new(base_x + plot_w, y)],
            Stroke::new(0.7, theme.grid_line),
        );
        painter.line_segment(
            [Pos2::new(base_x - 5.0, y), Pos2::new(base_x, y)],
            Stroke::new(1.1, theme.axis_line),
        );
        painter.text(
            Pos2::new(base_x - 7.0, y),
            egui::Align2::RIGHT_CENTER,
            rotulo_decada_log10_kb(k),
            FontId::proportional(9.5),
            theme.text_secondary,
        );
    }

    // Eixo das Ordenadas (Y) e Abscissas (X)
    painter.line_segment(
        [Pos2::new(base_x, base_y), Pos2::new(base_x + plot_w, base_y)],
        Stroke::new(1.3, theme.axis_line),
    );
    painter.line_segment(
        [Pos2::new(base_x, top_y), Pos2::new(base_x, base_y)],
        Stroke::new(1.3, theme.axis_line),
    );

    let max_x = if data.max_x > 0.0 { data.max_x as f32 } else { 24.0 };

    // Rótulos do eixo X (abscissas) de acordo com a escala ativa
    if !data.labels.is_empty() {
        let n_labels = data.labels.len();
        let step = (n_labels / 6).max(1);
        for idx in (0..n_labels).step_by(step) {
            let x = base_x + (idx as f32 / (n_labels - 1).max(1) as f32) * plot_w;
            painter.line_segment(
                [Pos2::new(x, base_y), Pos2::new(x, base_y + 4.0)],
                Stroke::new(1.0, theme.axis_line),
            );
            painter.text(
                Pos2::new(x, base_y + 14.0),
                egui::Align2::CENTER_CENTER,
                &data.labels[idx],
                FontId::proportional(10.0),
                theme.text_secondary,
            );
        }
    }

    // Indicador da escala ativa
    let scale_text = if !data.time_unit_label.is_empty() {
        loc.t_args("stat_scale_prefix", &[&data.time_unit_label])
    } else {
        loc.t_args("stat_scale_prefix", &[&loc.t("stat_scale_time")])
    };
    painter.text(
        Pos2::new(base_x + plot_w, rect.top() + 8.0),
        egui::Align2::RIGHT_CENTER,
        scale_text,
        FontId::proportional(11.0),
        theme.text_accent,
    );

    // Pontos de dispersão calculados pelo método logarítmico (log10) no eixo das ordenadas (Y)
    let circle_stroke_color = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };
    for (idx, &(h, size)) in data.points.iter().enumerate() {
        let x = base_x + ((h as f32 / max_x).clamp(0.0, 1.0)) * plot_w;
        let log_y = data
            .values
            .get(idx)
            .copied()
            .unwrap_or_else(|| calcular_log10_kb(size)) as f32;
        let y = map_log_y(log_y).clamp(top_y + 4.0, base_y - 4.0);
        let cor = CORES_PALETA[idx % CORES_PALETA.len()];
        painter.circle_filled(
            Pos2::new(x, y),
            4.2,
            Color32::from_rgba_premultiplied(cor.r(), cor.g(), cor.b(), 195),
        );
        painter.circle_stroke(
            Pos2::new(x, y),
            4.2,
            Stroke::new(1.0, circle_stroke_color),
        );
    }
}

// Diagrama de Relações e Associações Agrupadas (Sankey / Fluxos)
fn desenhar_sankey(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.flows.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_relations"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let leg_text = format!(
        "🏷️ {}: [{}]  ──►  [{}]  ──►  [{}]",
        loc.t("chart_legend_sankey_flow"),
        loc.t("chart_sankey_legend_origin"),
        loc.t("chart_sankey_legend_flow"),
        loc.t("chart_sankey_legend_destination"),
    );
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let left_x = rect.left() + 140.0;
    let right_x = rect.right() - 140.0;
    let top_y = rect.top() + 35.0;
    let bottom_y = rect.bottom() - 25.0;
    let available_h = bottom_y - top_y;

    // Coleta todas as origens e destinos únicos
    let mut origens: Vec<String> = Vec::new();
    let mut destinos: Vec<String> = Vec::new();
    let mut total_por_origem: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut total_por_destino: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut total_fluxo: f64 = 0.0;

    for (o, d, v) in &data.flows {
        if !origens.contains(o) {
            origens.push(o.clone());
        }
        if !destinos.contains(d) {
            destinos.push(d.clone());
        }
        *total_por_origem.entry(o.clone()).or_insert(0.0) += v;
        *total_por_destino.entry(d.clone()).or_insert(0.0) += v;
        total_fluxo += v;
    }

    if total_fluxo <= 0.0 {
        return;
    }

    // Ordenação determinística e estável de origens e destinos com critério de desempate alfabético.
    // Garante estabilidade geométrica absoluta, eliminando 100% de qualquer tremor, oscilação ou flicker
    // nas curvas de Bézier e nas caixas de nós em todos os diagramas Sankey.
    origens.sort_by(|a, b| {
        let va = total_por_origem.get(a).copied().unwrap_or(0.0);
        let vb = total_por_origem.get(b).copied().unwrap_or(0.0);
        vb.partial_cmp(&va)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });

    destinos.sort_by(|a, b| {
        let va = total_por_destino.get(a).copied().unwrap_or(0.0);
        let vb = total_por_destino.get(b).copied().unwrap_or(0.0);
        vb.partial_cmp(&va)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });

    // Posições verticais dos nós de origem (esquerda)
    let n_orig = origens.len().max(1);
    let step_orig = available_h / (n_orig as f32);
    let mut orig_y_map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for (i, orig) in origens.iter().enumerate() {
        let cy = (top_y + (i as f32 + 0.5) * step_orig).round();
        orig_y_map.insert(orig.clone(), cy);
    }

    // Posições verticais dos nós de destino (direita)
    let n_dest = destinos.len().max(1);
    let step_dest = available_h / (n_dest as f32);
    let mut dest_y_map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for (i, dest) in destinos.iter().enumerate() {
        let cy = (top_y + (i as f32 + 0.5) * step_dest).round();
        dest_y_map.insert(dest.clone(), cy);
    }

    // Fitas relacionais em curvas de Bézier cúbicas suaves e perfeitamente ancoradas
    for (idx, (origem, destino, val)) in data.flows.iter().enumerate() {
        let y1 = orig_y_map.get(origem).copied().unwrap_or(top_y);
        let y2 = dest_y_map.get(destino).copied().unwrap_or(top_y);
        let ribbon_w = ((val / total_fluxo) as f32 * 26.0).clamp(2.0, 16.0);
        let cor = CORES_PALETA[idx % CORES_PALETA.len()];

        let p1 = Pos2::new(left_x.round(), y1);
        let p2 = Pos2::new(right_x.round(), y2);
        let c1 = Pos2::new((left_x + (right_x - left_x) * 0.45).round(), y1);
        let c2 = Pos2::new((right_x - (right_x - left_x) * 0.45).round(), y2);

        let shape = egui::epaint::CubicBezierShape::from_points_stroke(
            [p1, c1, c2, p2],
            false,
            Color32::TRANSPARENT,
            Stroke::new(ribbon_w, Color32::from_rgba_premultiplied(cor.r(), cor.g(), cor.b(), 140)),
        );
        painter.add(shape);
    }

    let node_stroke_color = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };

    // Nós de Origem (esquerda)
    for (i, orig) in origens.iter().enumerate() {
        let cy = orig_y_map.get(orig).copied().unwrap_or(top_y);
        let total = total_por_origem.get(orig).copied().unwrap_or(0.0);
        let cor = CORES_PALETA[i % CORES_PALETA.len()];

        let box_rect = Rect::from_center_size(Pos2::new(left_x - 60.0, cy), Vec2::new(110.0, 24.0));
        painter.rect_filled(box_rect, 4_u8, Color32::from_rgba_premultiplied(cor.r(), cor.g(), cor.b(), 180));
        painter.rect_stroke(box_rect, 4_u8, Stroke::new(1.2, node_stroke_color), egui::StrokeKind::Inside);

        painter.text(
            Pos2::new(left_x - 60.0, cy),
            egui::Align2::CENTER_CENTER,
            format!("{} ({:.0})", orig, total),
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    }

    // Nós de Destino (direita)
    for (i, dest) in destinos.iter().enumerate() {
        let cy = dest_y_map.get(dest).copied().unwrap_or(top_y);
        let total = total_por_destino.get(dest).copied().unwrap_or(0.0);
        let cor = CORES_PALETA[(i + 3) % CORES_PALETA.len()];

        let box_rect = Rect::from_center_size(Pos2::new(right_x + 60.0, cy), Vec2::new(110.0, 24.0));
        painter.rect_filled(box_rect, 4_u8, Color32::from_rgba_premultiplied(cor.r(), cor.g(), cor.b(), 180));
        painter.rect_stroke(box_rect, 4_u8, Stroke::new(1.2, node_stroke_color), egui::StrokeKind::Inside);

        painter.text(
            Pos2::new(right_x + 60.0, cy),
            egui::Align2::CENTER_CENTER,
            format!("{} ({:.0})", dest, total),
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    }
}

/// Boxplot de Tamanhos de Arquivo: reporta a quantidade total por tipo de extensão em relação à
/// soma total do tamanho de todos os arquivos daquele tipo de extensão, acompanhada da distribuição
/// estatística (quartis, mediana e limites mín./máx.) dos tamanhos por extensão.
fn desenhar_boxplot(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    let total_arqs: usize = data.boxplot_totais.iter().map(|t| t.1).sum();
    let soma_global_kb: f64 = data.boxplot_totais.iter().map(|t| t.2).sum();

    let leg_text = format!(
        "🏷️ {}: [◼] {}  |  [─] {}  |  [┬] {}  |  [◆] {}",
        loc.t("chart_legend_title"),
        loc.t("stat_iqr"),
        loc.t("stat_median"),
        loc.t("stat_whiskers"),
        loc.t("chart_legend_boxplot_qty_sum"),
    );
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 3.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.0),
        theme.text_accent,
    );

    let box_stroke_color = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };

    if !data.boxplot_groups.is_empty() {
        let base_y = rect.bottom() - 48.0;
        let top_y = rect.top() + 44.0;
        let base_x = rect.left() + 62.0;
        let right_x = rect.right() - 68.0;
        let plot_w = (right_x - base_x).max(40.0);
        let plot_h = (base_y - top_y).max(40.0);

        // Legenda de resumo transferida para o canto direito da área de plotagem
        let resumo_text = format!(
            "{}: {}  |  {}: {} {}  |  Σ {}",
            loc.t("chart_extensions_label"),
            data.boxplot_groups.len(),
            loc.t("stat_total"),
            total_arqs,
            loc.t("chart_files_short"),
            formatar_tamanho_kb(soma_global_kb),
        );
        painter.text(
            Pos2::new(right_x, rect.top() + 21.0),
            egui::Align2::RIGHT_TOP,
            resumo_text,
            FontId::proportional(10.5),
            theme.text_secondary,
        );

        let global_max = data.boxplot_groups.iter().map(|g| g.5).fold(1.0, f64::max);
        let global_min = data
            .boxplot_groups
            .iter()
            .map(|g| g.1)
            .fold(f64::INFINITY, f64::min)
            .max(0.0);
        let global_min = if global_min.is_finite() { global_min } else { 0.0 };

        // Transformação matemática logarítmica (log10(x + 1)) para evitar achatamento de caixas
        // quando existem extensões em diferentes ordens de grandeza (KB, MB, GB)
        let log_min = if global_min <= 1.0 {
            0.0
        } else {
            ((global_min + 1.0).log10() * 0.92).max(0.0)
        };
        let log_max = (((global_max.max(1.0) + 1.0).log10()) * 1.06).max(log_min + 0.5);
        let log_range = (log_max - log_min).max(0.25);

        let max_soma_kb = data
            .boxplot_totais
            .iter()
            .map(|t| t.2)
            .fold(global_max.max(1.0), f64::max)
            .max(1.0);
        let log_soma_max = ((max_soma_kb + 1.0).log10() * 1.05).max(0.5);

        let map_y = |val: f64| -> f32 {
            let lv = (val.max(0.0) + 1.0).log10();
            let norm = ((lv - log_min) / log_range).clamp(0.0, 1.0) as f32;
            base_y - norm * plot_h
        };
        let map_soma_y = |soma: f64| -> f32 {
            let ls = (soma.max(0.0) + 1.0).log10();
            let norm = (ls / log_soma_max).clamp(0.0, 1.0) as f32;
            base_y - norm * plot_h
        };

        // Eixo Y esquerdo (tamanho individual Tukey/Log), eixo Y direito (soma total por extensão) e eixo X
        painter.line_segment(
            [Pos2::new(base_x, top_y), Pos2::new(base_x, base_y)],
            Stroke::new(1.2, theme.axis_line),
        );
        painter.line_segment(
            [Pos2::new(right_x, top_y), Pos2::new(right_x, base_y)],
            Stroke::new(1.0, theme.axis_line),
        );
        painter.line_segment(
            [Pos2::new(base_x, base_y), Pos2::new(right_x, base_y)],
            Stroke::new(1.2, theme.axis_line),
        );

        for step in 0..=4 {
            let frac = step as f64 / 4.0;
            let lv = log_min + log_range * frac;
            let val = (10.0_f64.powf(lv) - 1.0).max(0.0);
            let y = base_y - (frac as f32) * plot_h;
            painter.line_segment(
                [Pos2::new(base_x, y), Pos2::new(right_x, y)],
                Stroke::new(0.6, theme.grid_line),
            );
            painter.text(
                Pos2::new(base_x - 6.0, y),
                egui::Align2::RIGHT_CENTER,
                formatar_tamanho_kb(val),
                FontId::proportional(9.5),
                theme.text_muted,
            );

            // Escala direita: Soma Total do Tamanho por extensão (em progressão logarítmica)
            let val_soma = (10.0_f64.powf(log_soma_max * frac) - 1.0).max(0.0);
            painter.text(
                Pos2::new(right_x + 6.0, y),
                egui::Align2::LEFT_CENTER,
                format!("Σ {}", formatar_tamanho_kb(val_soma)),
                FontId::proportional(9.0),
                Color32::from_rgb(129, 212, 250),
            );
        }

        let n = data.boxplot_groups.len();
        let col_w = plot_w / (n as f32).max(1.0);
        let max_chars = ((col_w / 6.0).floor() as usize).clamp(3, 14);
        let mut pts_soma = Vec::with_capacity(n);

        for (idx, (ext, min, q1, med, q3, max)) in data.boxplot_groups.iter().enumerate() {
            let cx = base_x + (idx as f32 + 0.5) * col_w;
            let color = CORES_PALETA[idx % CORES_PALETA.len()];
            let (qtd_ext, soma_ext_kb) = data
                .boxplot_totais
                .get(idx)
                .map(|(_, q, s)| (*q, *s))
                .unwrap_or((1, *med));

            let y_min = map_y(*min);
            let y_q1_raw = map_y(*q1);
            let y_med = map_y(*med);
            let y_q3_raw = map_y(*q3);
            let y_max = map_y(*max);

            // Garante altura visual mínima (5 px) para a caixa interquartil quando Q1 ≈ Q3
            let (y_q3, y_q1) = if (y_q1_raw - y_q3_raw).abs() < 5.0 {
                (
                    (y_med - 2.5).clamp(top_y, base_y),
                    (y_med + 2.5).clamp(top_y, base_y),
                )
            } else {
                (y_q3_raw.min(y_q1_raw), y_q1_raw.max(y_q3_raw))
            };

            let y_soma = map_soma_y(soma_ext_kb);
            pts_soma.push((Pos2::new(cx, y_soma), qtd_ext, soma_ext_kb));

            let box_half_w = (col_w * 0.28).clamp(4.0, 24.0);

            // Coluna translúcida ao fundo indicando a soma total de tamanhos da extensão
            let soma_rect = Rect::from_x_y_ranges(
                cx - box_half_w * 1.15..=cx + box_half_w * 1.15,
                y_soma..=base_y,
            );
            painter.rect_filled(
                soma_rect,
                3_u8,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 38),
            );

            // Bigodes de Tukey (cercas de 1.5 * IQR)
            painter.line_segment(
                [Pos2::new(cx, y_min), Pos2::new(cx, y_max)],
                Stroke::new(1.5, theme.text_muted),
            );
            painter.line_segment(
                [Pos2::new(cx - box_half_w * 0.6, y_min), Pos2::new(cx + box_half_w * 0.6, y_min)],
                Stroke::new(1.5, theme.text_muted),
            );
            painter.line_segment(
                [Pos2::new(cx - box_half_w * 0.6, y_max), Pos2::new(cx + box_half_w * 0.6, y_max)],
                Stroke::new(1.5, theme.text_muted),
            );

            // Caixa IQR (Q1 a Q3 interpolados)
            let box_rect = Rect::from_x_y_ranges(cx - box_half_w..=cx + box_half_w, y_q3..=y_q1);
            painter.rect_filled(
                box_rect,
                2_u8,
                Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 185),
            );
            painter.rect_stroke(
                box_rect,
                2_u8,
                Stroke::new(1.2, box_stroke_color),
                egui::StrokeKind::Inside,
            );

            // Linha da Mediana e ponto central de referência estatística
            painter.line_segment(
                [Pos2::new(cx - box_half_w, y_med), Pos2::new(cx + box_half_w, y_med)],
                Stroke::new(2.5, Color32::from_rgb(255, 235, 59)),
            );
            painter.circle_filled(Pos2::new(cx, y_med), 2.3, Color32::from_rgb(255, 235, 59));

            // Rótulos no eixo X: 1) Extensão, 2) Quantidade total vs. Soma total, 3) Mediana
            painter.text(
                Pos2::new(cx, base_y + 11.0),
                egui::Align2::CENTER_CENTER,
                abreviar_rotulo(ext, max_chars),
                FontId::proportional(11.0),
                theme.text_primary,
            );
            let qtd_soma_txt = if col_w >= 72.0 {
                format!("{} {} | Σ {}", qtd_ext, loc.t("chart_files_short"), formatar_tamanho_kb(soma_ext_kb))
            } else {
                format!("{}x|{}", qtd_ext, formatar_tamanho_kb(soma_ext_kb))
            };
            painter.text(
                Pos2::new(cx, base_y + 24.0),
                egui::Align2::CENTER_CENTER,
                qtd_soma_txt,
                FontId::proportional(9.5),
                Color32::from_rgb(129, 212, 250),
            );
            let med_txt = format!("{}: {}", loc.t("stat_median_short"), formatar_tamanho_kb(*med));
            painter.text(
                Pos2::new(cx, base_y + 36.0),
                egui::Align2::CENTER_CENTER,
                med_txt,
                FontId::proportional(9.0),
                Color32::from_rgb(255, 215, 64),
            );
        }

        // Linha e marcadores [◆] conectando a relação (Quantidade Total × Soma Total) por extensão
        for w in pts_soma.windows(2) {
            painter.line_segment(
                [w[0].0, w[1].0],
                Stroke::new(1.6, Color32::from_rgb(100, 181, 246)),
            );
        }
        for (pt, qtd_ext, soma_ext_kb) in &pts_soma {
            let d = 4.5;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(pt.x, pt.y - d),
                    Pos2::new(pt.x + d, pt.y),
                    Pos2::new(pt.x, pt.y + d),
                    Pos2::new(pt.x - d, pt.y),
                ],
                Color32::from_rgb(41, 182, 246),
                Stroke::new(1.0, box_stroke_color),
            ));
            if col_w >= 48.0 {
                painter.text(
                    Pos2::new(pt.x, (pt.y - 7.0).max(top_y - 10.0)),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{} ({})", qtd_ext, formatar_tamanho_kb(*soma_ext_kb)),
                    FontId::proportional(9.0),
                    Color32::from_rgb(179, 229, 252),
                );
            }
        }
    } else if let Some((min, q1, med, q3, max)) = data.boxplot {
        let center_x = rect.center().x;
        let base_y = rect.bottom() - 30.0;
        let top_y = rect.top() + 35.0;
        let total_h = base_y - top_y;
        let range = (max - min).max(1.0);

        let map_y = |val: f64| -> f32 { base_y - (((val - min) / range) as f32) * total_h };

        let y_min = map_y(min);
        let y_q1 = map_y(q1);
        let y_med = map_y(med);
        let y_q3 = map_y(q3);
        let y_max = map_y(max);

        // Bigodes
        painter.line_segment(
            [Pos2::new(center_x, y_min), Pos2::new(center_x, y_max)],
            Stroke::new(1.5, theme.text_muted),
        );
        painter.line_segment(
            [
                Pos2::new(center_x - 15.0, y_min),
                Pos2::new(center_x + 15.0, y_min),
            ],
            Stroke::new(1.5, theme.text_muted),
        );
        painter.line_segment(
            [
                Pos2::new(center_x - 15.0, y_max),
                Pos2::new(center_x + 15.0, y_max),
            ],
            Stroke::new(1.5, theme.text_muted),
        );

        // Caixa interquartil
        let box_rect = Rect::from_x_y_ranges(center_x - 30.0..=center_x + 30.0, y_q3..=y_q1);
        painter.rect_filled(box_rect, 2_u8, Color32::from_rgb(21, 101, 192));
        painter.rect_stroke(
            box_rect,
            2_u8,
            Stroke::new(1.5, box_stroke_color),
            egui::StrokeKind::Inside,
        );

        // Linha da mediana
        painter.line_segment(
            [
                Pos2::new(center_x - 30.0, y_med),
                Pos2::new(center_x + 30.0, y_med),
            ],
            Stroke::new(2.5, Color32::from_rgb(255, 179, 0)),
        );

        // Rótulos de texto
        painter.text(
            Pos2::new(center_x + 38.0, y_max),
            egui::Align2::LEFT_CENTER,
            format!("{}: {:.1} KB", loc.t("stat_max"), max),
            FontId::proportional(11.0),
            theme.text_primary,
        );
        painter.text(
            Pos2::new(center_x + 38.0, y_q3),
            egui::Align2::LEFT_CENTER,
            format!("Q3: {:.1} KB", q3),
            FontId::proportional(11.0),
            theme.text_secondary,
        );
        painter.text(
            Pos2::new(center_x + 38.0, y_med),
            egui::Align2::LEFT_CENTER,
            format!("{}: {:.1} KB", loc.t("stat_median"), med),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );
        painter.text(
            Pos2::new(center_x + 38.0, y_q1),
            egui::Align2::LEFT_CENTER,
            format!("Q1: {:.1} KB", q1),
            FontId::proportional(11.0),
            theme.text_secondary,
        );
        painter.text(
            Pos2::new(center_x + 38.0, y_min),
            egui::Align2::LEFT_CENTER,
            format!("{}: {:.1} KB", loc.t("stat_min"), min),
            FontId::proportional(11.0),
            theme.text_primary,
        );
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_insufficient_boxplot_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
    }
}

fn desenhar_radar(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.labels.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let leg_text = format!("🏷️ {}: {}", loc.t("chart_legend_title"), loc.t("chart_legend_radar_ops"));
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let center = Pos2::new(rect.center().x, rect.center().y + 12.0);
    let radius = (rect.height().min(rect.width()) / 2.0 - 45.0).max(30.0);
    let n = data.labels.len().max(3);

    // Círculos concêntricos e eixos
    for r_step in [0.25, 0.5, 0.75, 1.0] {
        painter.circle_stroke(
            center,
            radius * r_step,
            Stroke::new(0.8, theme.grid_line),
        );
    }

    let mut points = Vec::new();
    let max_v = data.values.iter().copied().fold(0.0, f64::max).max(1.0);
    let total_v: f64 = data.values.iter().sum();

    for i in 0..n {
        let angle = -PI / 2.0 + (i as f32 / n as f32) * 2.0 * PI;
        let spoke_end = Pos2::new(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        );
        painter.line_segment(
            [center, spoke_end],
            Stroke::new(1.0, theme.grid_line),
        );

        let val = data.values.get(i).copied().unwrap_or(0.0);
        let pct = if total_v > 0.0 { (val / total_v) * 100.0 } else { 0.0 };

        if let Some(label) = data.labels.get(i) {
            let label_pos = Pos2::new(
                center.x + angle.cos() * (radius + 18.0),
                center.y + angle.sin() * (radius + 18.0),
            );
            let trad_label = loc.traduzir_tipo_operacao(label);
            let texto_label = format!("{}\n{:.1}%", trad_label, pct);
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                texto_label,
                FontId::proportional(10.5),
                theme.text_secondary,
            );
        }

        let r_val = ((val / max_v) as f32 * radius).max(4.0);
        points.push(Pos2::new(
            center.x + angle.cos() * r_val,
            center.y + angle.sin() * r_val,
        ));
    }

    if points.len() >= 3 {
        painter.add(egui::Shape::convex_polygon(
            points.clone(),
            Color32::from_rgba_premultiplied(46, 125, 50, 90),
            Stroke::new(2.0, Color32::from_rgb(76, 175, 80)),
        ));
        let pt_color = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };
        for pt in points {
            painter.circle_filled(pt, 3.5, pt_color);
        }
    }
}

fn desenhar_timeline(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.timeline_items.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_events_timeline"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let op_lanes_loc = [
        loc.t("op_added"),
        loc.t("op_modified"),
        loc.t("op_deleted"),
        loc.t("op_renamed"),
        loc.t("op_moved"),
        loc.t("op_scanned"),
    ];

    // Top Legend
    let leg_title = format!("🏷️ {}:", loc.t("chart_legend_timeline"));
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_title,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let mut cur_x = rect.left() + 175.0;
    for (idx, name) in op_lanes_loc.iter().enumerate() {
        let color = CORES_PALETA[idx % CORES_PALETA.len()];
        painter.circle_filled(Pos2::new(cur_x, rect.top() + 10.0), 3.5, color);
        painter.text(
            Pos2::new(cur_x + 6.0, rect.top() + 10.0),
            egui::Align2::LEFT_CENTER,
            name,
            FontId::proportional(10.0),
            theme.text_secondary,
        );
        cur_x += name.len() as f32 * 6.0 + 16.0;
    }

    let left_x = rect.left() + 140.0;
    let right_x = rect.right() - 25.0;
    let base_y = rect.bottom() - 36.0;
    let top_y = rect.top() + 35.0;
    let timeline_w = right_x - left_x;
    let timeline_h = base_y - top_y;
    let lane_h = timeline_h / (op_lanes_loc.len() as f32);

    // Eixo das Ordenadas (Y) completo com linha e seta
    painter.line_segment(
        [Pos2::new(left_x, top_y), Pos2::new(left_x, base_y)],
        Stroke::new(1.5, theme.axis_line),
    );
    painter.arrow(
        Pos2::new(left_x, top_y + 8.0),
        Vec2::new(0.0, -8.0),
        Stroke::new(1.5, theme.axis_line),
    );
    painter.text(
        Pos2::new(left_x - 10.0, top_y - 12.0),
        egui::Align2::RIGHT_CENTER,
        loc.t("chart_axis_operations_qty"),
        FontId::proportional(11.0),
        theme.text_accent,
    );

    // Eixo das Abscissas (X) completo com linha e seta
    painter.line_segment(
        [Pos2::new(left_x, base_y), Pos2::new(right_x, base_y)],
        Stroke::new(1.5, theme.axis_line),
    );
    painter.arrow(
        Pos2::new(right_x - 8.0, base_y),
        Vec2::new(8.0, 0.0),
        Stroke::new(1.5, theme.axis_line),
    );
    painter.text(
        Pos2::new(left_x + timeline_w / 2.0, base_y + 24.0),
        egui::Align2::CENTER_CENTER,
        loc.t("chart_axis_timeline_events"),
        FontId::proportional(11.0),
        theme.text_accent,
    );

    for (idx, lane_name) in op_lanes_loc.iter().enumerate() {
        let lane_y = top_y + (idx as f32 + 0.5) * lane_h;
        let color = CORES_PALETA[idx % CORES_PALETA.len()];

        // Traço de tick no Eixo Y
        painter.line_segment(
            [Pos2::new(left_x - 5.0, lane_y), Pos2::new(left_x, lane_y)],
            Stroke::new(1.0, theme.axis_line),
        );

        // Grade horizontal discreta
        painter.line_segment(
            [Pos2::new(left_x, lane_y), Pos2::new(right_x, lane_y)],
            Stroke::new(0.6, theme.grid_line),
        );

        let total_op = data
            .pairs
            .iter()
            .find(|(k, _)| {
                crate::core::types::OperationType::from_raw(k)
                    == crate::core::types::OperationType::from_raw(lane_name)
            })
            .map(|(_, v)| *v as usize)
            .unwrap_or_else(|| {
                data.timeline_items
                    .iter()
                    .filter(|(_, op, _, _)| {
                        crate::core::types::OperationType::from_raw(op)
                            == crate::core::types::OperationType::from_raw(lane_name)
                    })
                    .count()
            });

        painter.text(
            Pos2::new(left_x - 9.0, lane_y),
            egui::Align2::RIGHT_CENTER,
            format!("{} ({})", lane_name, total_op),
            FontId::proportional(11.0),
            color,
        );
    }

    // Marcações temporais no Eixo X (Abscissas)
    if !data.timeline_items.is_empty() {
        let start_ts = &data.timeline_items.first().unwrap().0;
        let end_ts = &data.timeline_items.last().unwrap().0;
        let mid_ts = &data.timeline_items[data.timeline_items.len() / 2].0;
        let fmt_ts = |ts: &str| -> String {
            if ts.len() >= 19 {
                ts[11..19].to_string()
            } else {
                ts.to_string()
            }
        };

        let ticks = [
            (0.02f32, fmt_ts(start_ts)),
            (0.50f32, fmt_ts(mid_ts)),
            (0.96f32, fmt_ts(end_ts)),
        ];

        for (pct, label) in ticks {
            let x = left_x + pct * timeline_w;
            painter.line_segment(
                [Pos2::new(x, base_y), Pos2::new(x, base_y + 4.0)],
                Stroke::new(1.0, theme.axis_line),
            );
            painter.text(
                Pos2::new(x, base_y + 11.0),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(9.5),
                theme.text_secondary,
            );
        }
    }

    let dot_stroke_color = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };

    // Dispersão dos eventos/operações ao longo da linha do tempo
    let n = data.timeline_items.len();
    for (i, (_ts, op, nome, _size)) in data.timeline_items.iter().enumerate() {
        let op_type = crate::core::types::OperationType::from_raw(op);
        let op_idx = match op_type {
            crate::core::types::OperationType::Adicionado => 0,
            crate::core::types::OperationType::Modificado => 1,
            crate::core::types::OperationType::Excluido => 2,
            crate::core::types::OperationType::Renomeado => 3,
            crate::core::types::OperationType::Movido => 4,
            crate::core::types::OperationType::Escaneado => 5,
        };
        let lane_y = top_y + (op_idx as f32 + 0.5) * lane_h;
        let x = (left_x + 10.0 + (i as f32 / n.max(1) as f32) * (timeline_w - 20.0))
            .clamp(left_x + 6.0, right_x - 10.0);
        let color = CORES_PALETA[op_idx % CORES_PALETA.len()];

        painter.circle_filled(Pos2::new(x, lane_y), 4.0, color);
        painter.circle_stroke(Pos2::new(x, lane_y), 4.0, Stroke::new(1.0, dot_stroke_color));

        if n <= 15 {
            painter.text(
                Pos2::new(x, lane_y - 8.0),
                egui::Align2::CENTER_BOTTOM,
                nome,
                FontId::proportional(9.0),
                theme.text_secondary,
            );
        }
    }
}

// 1.1 e 1.2: Treemap por Tipos: partição recursiva 2D cobrindo 100% da área disponível com legenda
fn desenhar_treemap(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.treemap_items.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data_treemap"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let total: f64 = data.treemap_items.iter().map(|item| item.1).sum();
    if total <= 0.0 {
        return;
    }

    // Faixa Superior de Legenda com identificação clara de extensões e proporções
    let legend_y = rect.top() + 4.0;
    let mut cur_x = rect.left() + 4.0;

    let legend_title = format!("🏷️ {}:", loc.t("chart_legend_treemap"));
    painter.text(
        Pos2::new(cur_x, legend_y),
        egui::Align2::LEFT_TOP,
        &legend_title,
        FontId::proportional(11.5),
        theme.text_accent,
    );
    cur_x += legend_title.len() as f32 * 6.5 + 18.0;

    for (label, val, color) in data.treemap_items.iter().take(8) {
        let pct = (val / total * 100.0) as f32;
        let badge_txt = format!("{}: {:.0} ({:.1}%)", label, val, pct);
        if cur_x + 85.0 > rect.right() - 10.0 {
            break;
        }
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(cur_x, legend_y + 2.0), Vec2::new(10.0, 10.0)),
            2_u8,
            *color,
        );
        painter.text(
            Pos2::new(cur_x + 14.0, legend_y),
            egui::Align2::LEFT_TOP,
            &badge_txt,
            FontId::proportional(10.5),
            theme.text_secondary,
        );
        cur_x += badge_txt.len() as f32 * 6.0 + 20.0;
    }

    // Área total de plotagem: ocupa todo o espaço restante disponível
    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top() + 26.0),
        rect.max,
    );

    if plot_rect.width() <= 5.0 || plot_rect.height() <= 5.0 {
        return;
    }

    let mut layout_cells = Vec::with_capacity(data.treemap_items.len());
    layout_treemap_recursive(&data.treemap_items, plot_rect, &mut layout_cells);

    for (cell, label, val, color) in layout_cells {
        let inner = cell.shrink(1.5);
        painter.rect_filled(inner, 4_u8, color);
        painter.rect_stroke(
            inner,
            4_u8,
            Stroke::new(1.0, theme.border_card),
            egui::StrokeKind::Inside,
        );

        let pct = (val / total * 100.0) as f32;
        if inner.width() >= 55.0 && inner.height() >= 30.0 {
            let text = format!("{}\n{:.0} ({:.1}%)", label, val, pct);
            painter.text(
                inner.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::proportional(10.5),
                Color32::WHITE,
            );
        } else if inner.width() >= 38.0 && inner.height() >= 20.0 {
            let text = format!("{}\n{:.0}", label, val);
            painter.text(
                inner.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::proportional(9.5),
                Color32::WHITE,
            );
        } else if inner.width() >= 24.0 && inner.height() >= 14.0 {
            painter.text(
                inner.center(),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(9.0),
                Color32::WHITE,
            );
        }
    }
}

// Algoritmo de partição binária 2D recursiva (tiling exato a 100% sem sobras e sem lacunas)
fn layout_treemap_recursive(
    items: &[(String, f64, Color32)],
    rect: Rect,
    results: &mut Vec<(Rect, String, f64, Color32)>,
) {
    if items.is_empty() {
        return;
    }
    if items.len() == 1 {
        results.push((rect, items[0].0.clone(), items[0].1, items[0].2));
        return;
    }
    let total: f64 = items.iter().map(|it| it.1).sum();
    if total <= 0.0 {
        return;
    }
    let half = total / 2.0;
    let mut acc = 0.0;
    let mut best_idx = 1;
    let mut best_diff = f64::MAX;
    for (i, it) in items[..items.len() - 1].iter().enumerate() {
        acc += it.1;
        let diff = (acc - half).abs();
        if diff < best_diff {
            best_diff = diff;
            best_idx = i + 1;
        }
    }
    let left_items = &items[..best_idx];
    let right_items = &items[best_idx..];
    let left_weight: f64 = left_items.iter().map(|it| it.1).sum();
    let frac = (left_weight / total) as f32;

    if rect.width() >= rect.height() {
        // Divide horizontalmente (ao longo do eixo X)
        let w1 = (rect.width() * frac).clamp(1.0, rect.width() - 1.0);
        let r1 = Rect::from_min_size(rect.min, Vec2::new(w1, rect.height()));
        let r2 = Rect::from_min_size(
            Pos2::new(rect.min.x + w1, rect.min.y),
            Vec2::new(rect.width() - w1, rect.height()),
        );
        layout_treemap_recursive(left_items, r1, results);
        layout_treemap_recursive(right_items, r2, results);
    } else {
        // Divide verticalmente (ao longo do eixo Y)
        let h1 = (rect.height() * frac).clamp(1.0, rect.height() - 1.0);
        let r1 = Rect::from_min_size(rect.min, Vec2::new(rect.width(), h1));
        let r2 = Rect::from_min_size(
            Pos2::new(rect.min.x, rect.min.y + h1),
            Vec2::new(rect.width(), rect.height() - h1),
        );
        layout_treemap_recursive(left_items, r1, results);
        layout_treemap_recursive(right_items, r2, results);
    }
}

fn desenhar_linha(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.multi_series.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data_timeline"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let leg_text = format!("🏷️ {}:", loc.t("chart_legend_line_events"));
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let left_x = rect.left() + 40.0;
    let right_x = rect.right() - 20.0;
    let base_y = rect.bottom() - 30.0;
    let top_y = rect.top() + 35.0;
    let plot_w = right_x - left_x;
    let plot_h = base_y - top_y;

    painter.line_segment(
        [Pos2::new(left_x, base_y), Pos2::new(right_x, base_y)],
        Stroke::new(1.0, theme.axis_line),
    );
    painter.line_segment(
        [Pos2::new(left_x, top_y), Pos2::new(left_x, base_y)],
        Stroke::new(1.0, theme.axis_line),
    );

    for step in 1..=4 {
        let y = base_y - (step as f32 / 4.0) * plot_h;
        painter.line_segment(
            [Pos2::new(left_x, y), Pos2::new(right_x, y)],
            Stroke::new(0.6, theme.grid_line),
        );
    }

    let mut max_val: f64 = 1.0;
    for (_, vals, _) in &data.multi_series {
        for &v in vals {
            if v > max_val {
                max_val = v;
            }
        }
    }

    // Indicador da escala ativa no topo direito
    let scale_text = if !data.time_unit_label.is_empty() {
        loc.t_args("stat_scale_prefix", &[&data.time_unit_label])
    } else {
        loc.t_args("stat_scale_prefix", &[&loc.t("stat_scale_time")])
    };
    painter.text(
        Pos2::new(right_x, top_y - 15.0),
        egui::Align2::RIGHT_CENTER,
        scale_text,
        FontId::proportional(11.0),
        theme.text_accent,
    );

    // Centralização das legendas das operações/evento no topo da área de plotagem
    let font_leg = FontId::proportional(10.5);
    let mut item_widths = Vec::with_capacity(data.multi_series.len());
    let mut trad_nomes = Vec::with_capacity(data.multi_series.len());
    let item_gap = 14.0;

    for (nome, _, _) in &data.multi_series {
        let trad = loc.traduzir_tipo_operacao(nome);
        let text_w = painter
            .layout_no_wrap(trad.clone(), font_leg.clone(), theme.text_secondary)
            .size()
            .x;
        let item_w = 14.0 + text_w;
        item_widths.push(item_w);
        trad_nomes.push(trad);
    }

    let total_legends_w: f32 = item_widths.iter().sum::<f32>()
        + (item_widths.len().saturating_sub(1) as f32 * item_gap);
    let plot_center_x = (left_x + right_x) * 0.5;
    let mut cur_leg_x = (plot_center_x - (total_legends_w * 0.5)).max(left_x + 60.0);
    let leg_y = rect.top() + 10.0;

    for (s_idx, (_, _, color)) in data.multi_series.iter().enumerate() {
        let trad_nome = &trad_nomes[s_idx];
        let item_w = item_widths[s_idx];
        let circle_x = cur_leg_x + 4.0;
        painter.circle_filled(Pos2::new(circle_x, leg_y), 4.0, *color);
        painter.text(
            Pos2::new(circle_x + 8.0, leg_y),
            egui::Align2::LEFT_CENTER,
            trad_nome,
            font_leg.clone(),
            theme.text_secondary,
        );
        cur_leg_x += item_w + item_gap;
    }

    for (_s_idx, (_nome, vals, color)) in data.multi_series.iter().enumerate() {
        let pts_count = vals.len().max(2);
        let mut pts = Vec::new();
        for (h, &val) in vals.iter().enumerate() {
            let x = left_x + (h as f32 / (pts_count - 1) as f32) * plot_w;
            let y = base_y - ((val / max_val) as f32) * plot_h;
            pts.push(Pos2::new(x, y));
        }

        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(2.0, *color));
        }

        for (h, &pt) in pts.iter().enumerate() {
            if vals.get(h).copied().unwrap_or(0.0) > 0.0 {
                painter.circle_filled(pt, 3.0, *color);
            }
        }
    }

    // Rótulos do eixo X (abscissas) dinâmicos conforme data.labels
    if !data.labels.is_empty() {
        let n_labels = data.labels.len();
        let step = (n_labels / 6).max(1);
        for idx in (0..n_labels).step_by(step) {
            let x = left_x + (idx as f32 / (n_labels - 1).max(1) as f32) * plot_w;
            painter.line_segment(
                [Pos2::new(x, base_y), Pos2::new(x, base_y + 4.0)],
                Stroke::new(1.0, theme.axis_line),
            );
            painter.text(
                Pos2::new(x, base_y + 14.0),
                egui::Align2::CENTER_CENTER,
                &data.labels[idx],
                FontId::proportional(10.0),
                theme.text_secondary,
            );
        }
    }
}

fn desenhar_dotplot(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.dotplot_groups.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data_dotplot"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    // Identificação do gráfico no topo
    let leg_title = format!("🏷️ {}: {}", loc.t("chart_legend_title"), loc.t("chart_legend_dotplot"));
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 3.0),
        egui::Align2::LEFT_TOP,
        leg_title,
        FontId::proportional(11.0),
        theme.text_accent,
    );

    // Reserva faixa à direita da área de plotagem para a legenda "Escala por Tamanho"
    let legend_panel_w = (rect.width() * 0.24).clamp(170.0, 215.0);
    let left_x = rect.left() + 140.0;
    let right_x = (rect.right() - legend_panel_w - 14.0).max(left_x + 120.0);
    let base_y = rect.bottom() - 38.0;
    let top_y = rect.top() + 26.0;
    let plot_w = (right_x - left_x).max(40.0);
    let plot_h = (base_y - top_y).max(40.0);
    let n_groups = data.dotplot_groups.len();
    let row_h = plot_h / (n_groups as f32).max(1.0);

    // Eixo das Ordenadas (Y) completo com linha e seta
    painter.line_segment(
        [Pos2::new(left_x, top_y), Pos2::new(left_x, base_y)],
        Stroke::new(1.5, theme.axis_line),
    );
    painter.arrow(
        Pos2::new(left_x, top_y + 8.0),
        Vec2::new(0.0, -8.0),
        Stroke::new(1.5, theme.axis_line),
    );

    // Título do eixo das ordenadas rotacionado 90 graus no sentido anti-horário (-PI/2 rad)
    // e centralizado verticalmente em relação ao seu eixo (top_y..=base_y)
    let y_axis_title = loc.t("chart_axis_extensions_qty");
    let y_title_font = FontId::proportional(11.0);
    let y_title_color = theme.text_accent;
    let galley = painter.layout_no_wrap(y_axis_title, y_title_font, y_title_color);
    let y_axis_center = Pos2::new(rect.left() + 14.0, (top_y + base_y) * 0.5);
    let unrotated_top_left = y_axis_center - galley.size() * 0.5;
    let rotated_title = egui::epaint::TextShape::new(unrotated_top_left, galley, y_title_color)
        .with_angle_and_anchor(-PI / 2.0, egui::Align2::CENTER_CENTER);
    painter.add(egui::Shape::Text(rotated_title));

    // Eixo das Abscissas (X) completo com linha e seta
    painter.line_segment(
        [Pos2::new(left_x, base_y), Pos2::new(right_x, base_y)],
        Stroke::new(1.5, theme.axis_line),
    );
    painter.arrow(
        Pos2::new(right_x - 8.0, base_y),
        Vec2::new(8.0, 0.0),
        Stroke::new(1.5, theme.axis_line),
    );
    painter.text(
        Pos2::new(left_x + plot_w / 2.0, base_y + 26.0),
        egui::Align2::CENTER_CENTER,
        loc.t("chart_file_size_log"),
        FontId::proportional(11.0),
        theme.text_accent,
    );

    // Legenda referente a "Escala por Tamanho" transferida para o lado direito da área de plotagem
    let leg_left = right_x + 14.0;
    let leg_right = (rect.right() - 4.0).max(leg_left + 140.0);
    let leg_box_h = 128.0_f32.min(plot_h);
    let leg_top = top_y + ((plot_h - leg_box_h) * 0.5).max(0.0);
    let leg_rect = Rect::from_min_max(
        Pos2::new(leg_left, leg_top),
        Pos2::new(leg_right, leg_top + leg_box_h),
    );
    painter.rect_filled(leg_rect, 6_u8, theme.panel_bg);
    painter.rect_stroke(
        leg_rect,
        6_u8,
        Stroke::new(1.0, theme.panel_border),
        egui::StrokeKind::Inside,
    );

    let inner_x = leg_rect.left() + 10.0;
    let mut cur_y = leg_rect.top() + 14.0;

    let scale_lbl = format!("📏 {}:", loc.t("chart_legend_dotplot_size"));
    painter.text(
        Pos2::new(inner_x, cur_y),
        egui::Align2::LEFT_CENTER,
        scale_lbl,
        FontId::proportional(10.5),
        theme.text_accent,
    );
    cur_y += 24.0;

    // Amostra 1: Círculo colorido preenchido (<= 1 MB)
    let sample_fill_color = Color32::from_rgb(76, 175, 80);
    let dot_small_stroke = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };
    painter.circle_filled(Pos2::new(inner_x + 6.0, cur_y), 4.5, sample_fill_color);
    painter.circle_stroke(
        Pos2::new(inner_x + 6.0, cur_y),
        4.5,
        Stroke::new(0.8, dot_small_stroke),
    );
    painter.text(
        Pos2::new(inner_x + 18.0, cur_y),
        egui::Align2::LEFT_CENTER,
        loc.t("chart_legend_dotplot_filled"),
        FontId::proportional(10.0),
        theme.text_secondary,
    );
    cur_y += 22.0;

    // Amostra 2: Círculo vazado com contorno colorido (> 1 MB)
    let sample_hollow_color = Color32::from_rgb(255, 152, 0);
    painter.circle_stroke(
        Pos2::new(inner_x + 6.0, cur_y),
        5.5,
        Stroke::new(1.8, sample_hollow_color),
    );
    painter.text(
        Pos2::new(inner_x + 18.0, cur_y),
        egui::Align2::LEFT_CENTER,
        loc.t("chart_legend_dotplot_hollow"),
        FontId::proportional(10.0),
        theme.text_secondary,
    );
    cur_y += 24.0;

    // Amostra 3: Escala visual de raios proporcionais (1 KB -> 1 MB -> 1 GB)
    painter.circle_filled(Pos2::new(inner_x + 4.0, cur_y), 2.5, sample_fill_color);
    painter.circle_filled(Pos2::new(inner_x + 15.0, cur_y), 4.5, sample_fill_color);
    painter.circle_stroke(
        Pos2::new(inner_x + 29.0, cur_y),
        6.5,
        Stroke::new(1.6, sample_hollow_color),
    );
    painter.text(
        Pos2::new(inner_x + 42.0, cur_y),
        egui::Align2::LEFT_CENTER,
        "1 KB → 1 GB",
        FontId::proportional(9.5),
        theme.text_secondary,
    );
    cur_y += 22.0;

    // Rótulo de proporcionalidade
    let prop_galley = painter.layout(
        format!("({})", loc.t("chart_legend_dotplot_proportional")),
        FontId::proportional(9.5),
        theme.text_muted,
        (leg_rect.width() - 16.0).max(80.0),
    );
    painter.galley(
        Pos2::new(inner_x, cur_y - 6.0),
        prop_galley,
        theme.text_muted,
    );

    // Escala logarítmica de base 10 no eixo das abscissas (X):
    // Sub-grade logarítmica menor (m = 2..=9 em cada década 10^k) e grade principal das décadas (10^0 a 10^6 KB)
    let log_min_x = 0.0_f32;
    let log_max_x = 6.0_f32;
    let log_span_x = (log_max_x - log_min_x).max(1.0);
    let map_log_x = |lv: f32| -> f32 {
        let norm = ((lv - log_min_x) / log_span_x).clamp(0.0, 1.0);
        left_x + norm * plot_w
    };

    for k in 0..6 {
        let x_maj = map_log_x(k as f32);
        painter.line_segment(
            [Pos2::new(x_maj, top_y), Pos2::new(x_maj, base_y)],
            Stroke::new(0.6, theme.grid_line),
        );
        for m in 2..=9 {
            let sub_log = k as f32 + (m as f32).log10();
            let x_sub = map_log_x(sub_log);
            painter.line_segment(
                [Pos2::new(x_sub, top_y), Pos2::new(x_sub, base_y)],
                Stroke::new(0.35, theme.grid_line_subtle),
            );
            painter.line_segment(
                [Pos2::new(x_sub, base_y), Pos2::new(x_sub, base_y + 2.5)],
                Stroke::new(0.7, theme.axis_line),
            );
        }
    }

    for (idx, (cat, values)) in data.dotplot_groups.iter().enumerate() {
        let row_y = top_y + (idx as f32 + 0.5) * row_h;
        let color = CORES_PALETA[idx % CORES_PALETA.len()];

        // Traço de tick no Eixo Y
        painter.line_segment(
            [Pos2::new(left_x - 5.0, row_y), Pos2::new(left_x, row_y)],
            Stroke::new(1.0, theme.axis_line),
        );

        // Identificação da extensão e quantidade de arquivos
        painter.text(
            Pos2::new(left_x - 9.0, row_y),
            egui::Align2::RIGHT_CENTER,
            format!("{} ({})", cat, values.len()),
            FontId::proportional(11.0),
            theme.text_primary,
        );
        painter.line_segment(
            [Pos2::new(left_x, row_y), Pos2::new(right_x, row_y)],
            Stroke::new(0.6, theme.grid_line),
        );

        for (v_idx, &val) in values.iter().enumerate() {
            // Cálculo estritamente pelo método logarítmico de base 10 no eixo das abscissas (X)
            let log_v = calcular_log10_kb(val.max(1.0)) as f32;
            let progress = ((log_v - log_min_x) / log_span_x).clamp(0.0, 1.0);
            // Raio proporcional ao logaritmo do tamanho entre 2.5 e 7.0 px
            let r = 2.5 + progress * 4.5;
            let offset_y = if v_idx % 2 == 0 { -2.0 } else { 2.0 };
            
            // Clamping estrito para não extrapolar os limites da área de plotagem
            let x = map_log_x(log_v).clamp(left_x + r + 2.0, right_x - r - 2.0);
            let y = (row_y + offset_y).clamp(top_y + r + 2.0, base_y - r - 2.0);

            if val <= 1024.0 {
                // Arquivos compactos/pequenos (<= 1 MB): círculo colorido preenchido
                painter.circle_filled(
                    Pos2::new(x, y),
                    r,
                    Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 190),
                );
                painter.circle_stroke(
                    Pos2::new(x, y),
                    r,
                    Stroke::new(0.8, dot_small_stroke),
                );
            } else {
                // Arquivos maiores (> 1 MB): círculo vazado com contorno colorido
                painter.circle_stroke(
                    Pos2::new(x, y),
                    r,
                    Stroke::new(1.8, color),
                );
                painter.circle_stroke(
                    Pos2::new(x, y),
                    r - 1.0,
                    Stroke::new(0.6, Color32::from_rgba_premultiplied(255, 255, 255, 70)),
                );
            }
        }
    }

    for k in 0..=6 {
        let x = map_log_x(k as f32);
        // Traço de tick principal da década logarítmica no Eixo X
        painter.line_segment(
            [Pos2::new(x, base_y), Pos2::new(x, base_y + 4.5)],
            Stroke::new(1.1, theme.axis_line),
        );
        painter.text(
            Pos2::new(x, base_y + 13.5),
            egui::Align2::CENTER_CENTER,
            rotulo_decada_log10_kb(k),
            FontId::proportional(9.5),
            theme.text_secondary,
        );
    }
}

fn desenhar_arvore_diretorios(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.tree_nodes.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_no_data_tree"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let leg_text = format!(
        "🏷️ {}: 📂 {}  |  📁 {}  |  {}",
        loc.t("chart_legend_title"),
        loc.t("chart_tree_root"),
        loc.t("chart_tree_subfolder"),
        loc.t("chart_tree_metrics")
    );
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 2.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let mut cur_y = rect.top() + 24.0;

    for (idx, (path, depth, count, size_mb)) in data.tree_nodes.iter().enumerate() {
        let indent = rect.left() + 16.0 + (*depth as f32 * 20.0);
        let connector = if *depth == 0 {
            "📂 "
        } else if idx + 1 < data.tree_nodes.len() && data.tree_nodes[idx + 1].1 == *depth {
            "├── 📁 "
        } else {
            "└── 📁 "
        };

        let cor_texto = match depth % 4 {
            0 => Color32::from_rgb(144, 202, 249),
            1 => Color32::from_rgb(129, 199, 132),
            2 => Color32::from_rgb(255, 213, 79),
            _ => Color32::from_rgb(206, 147, 216),
        };

        painter.text(
            Pos2::new(indent, cur_y),
            egui::Align2::LEFT_CENTER,
            format!("{}{}", connector, path),
            FontId::proportional(11.5),
            cor_texto,
        );

        let badge = format!("[{} {}, {:.2} MB]", count, loc.t("chart_files_short"), size_mb);
        painter.text(
            Pos2::new(rect.right() - 20.0, cur_y),
            egui::Align2::RIGHT_CENTER,
            badge,
            FontId::proportional(10.5),
            Color32::from_rgb(255, 215, 64),
        );

        // Linha divisória sutil
        painter.line_segment(
            [Pos2::new(indent, cur_y + 10.0), Pos2::new(rect.right() - 20.0, cur_y + 10.0)],
            Stroke::new(0.5, theme.grid_line_subtle),
        );

        cur_y += 22.0;
    }
}

fn desenhar_radar_multidimensional(painter: &egui::Painter, rect: Rect, data: &ChartData, loc: &Localizer, theme: &ChartTheme) {
    if data.radar_dimensions.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            loc.t("chart_insufficient_security_data"),
            FontId::proportional(14.0),
            theme.text_muted,
        );
        return;
    }

    let leg_text = format!("🏷️ {}: {}", loc.t("chart_legend_title"), loc.t("chart_legend_radar_multi"));
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        leg_text,
        FontId::proportional(11.5),
        theme.text_accent,
    );

    let center = Pos2::new(rect.center().x, rect.center().y + 12.0);
    let radius = (rect.height().min(rect.width()) / 2.0 - 50.0).max(35.0);
    let n = data.radar_dimensions.len().max(3);

    // Círculos concêntricos e níveis de pontuação
    for step in [0.25, 0.50, 0.75, 1.0] {
        let mut ring_pts = Vec::new();
        for i in 0..n {
            let angle = -PI / 2.0 + (i as f32 / n as f32) * 2.0 * PI;
            ring_pts.push(Pos2::new(
                center.x + angle.cos() * radius * step,
                center.y + angle.sin() * radius * step,
            ));
        }
        ring_pts.push(ring_pts[0]);
        for w in ring_pts.windows(2) {
            painter.line_segment(
                [w[0], w[1]],
                Stroke::new(0.8, theme.grid_line),
            );
        }
        painter.text(
            Pos2::new(center.x + 4.0, center.y - radius * step),
            egui::Align2::LEFT_CENTER,
            format!("{:.0}%", step * 100.0),
            FontId::proportional(9.0),
            theme.text_muted,
        );
    }

    // Calcula os vértices de cada dimensão
    let mut points = Vec::new();
    for (i, (_dim_name, score)) in data.radar_dimensions.iter().enumerate() {
        let angle = -PI / 2.0 + (i as f32 / n as f32) * 2.0 * PI;
        let r_val = (*score as f32 / 100.0).clamp(0.05, 1.0) * radius;
        points.push(Pos2::new(
            center.x + angle.cos() * r_val,
            center.y + angle.sin() * r_val,
        ));
    }

    // Setores poligonais coloridos com cores diferentes para cada conjunto de dados monitorados
    for i in 0..n {
        let next_i = (i + 1) % n;
        let cor = CORES_PALETA[i % CORES_PALETA.len()];
        let p_center = center;
        let p1 = points[i];
        let p2 = points[next_i];

        painter.add(egui::Shape::convex_polygon(
            vec![p_center, p1, p2],
            Color32::from_rgba_premultiplied(cor.r(), cor.g(), cor.b(), 65),
            Stroke::NONE,
        ));

        painter.line_segment(
            [p1, p2],
            Stroke::new(2.5, cor),
        );
    }

    let vertex_stroke_color = if theme.dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };

    // Raios, rótulos e marcadores individuais com a cor distintiva de cada dimensão
    for (i, (dim_name, score)) in data.radar_dimensions.iter().enumerate() {
        let angle = -PI / 2.0 + (i as f32 / n as f32) * 2.0 * PI;
        let cor = CORES_PALETA[i % CORES_PALETA.len()];

        let spoke_end = Pos2::new(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        );
        painter.line_segment(
            [center, spoke_end],
            Stroke::new(1.2, Color32::from_rgba_premultiplied(cor.r(), cor.g(), cor.b(), 120)),
        );

        let label_pos = Pos2::new(
            center.x + angle.cos() * (radius + 22.0),
            center.y + angle.sin() * (radius + 22.0),
        );
        let label_text = if *score > 0.0 && *score < 1.0 {
            format!("{}\n{:.1}%", dim_name, score)
        } else {
            format!("{}\n{:.0}%", dim_name, score)
        };
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            label_text,
            FontId::proportional(10.5),
            cor,
        );

        painter.circle_filled(points[i], 5.0, cor);
        painter.circle_stroke(points[i], 5.0, Stroke::new(1.5, vertex_stroke_color));
    }
}
