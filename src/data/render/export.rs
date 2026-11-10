use image::{Rgba, RgbaImage};
use std::f32::consts::PI;
use std::path::Path;

use super::super::charts::TipoGrafico;
use super::super::generator::{calcular_limites_log10_kb, calcular_log10_kb, ChartData};
use crate::core::types::OperationType;

const CORES_PALETA_EXPORT: &[Rgba<u8>] = &[
    Rgba([46, 125, 50, 255]),  // Verde Floresta
    Rgba([21, 101, 192, 255]), // Azul Safira
    Rgba([198, 40, 40, 255]),  // Vermelho Rubi
    Rgba([106, 27, 154, 255]), // Roxo Ametista
    Rgba([239, 108, 0, 255]),  // Laranja Âmbar
    Rgba([0, 137, 123, 255]),  // Ciano Esmeralda
    Rgba([92, 107, 115, 255]), // Ardósia
    Rgba([124, 77, 255, 255]), // Violeta Neon
    Rgba([255, 179, 0, 255]),  // Amarelo Dourado
    Rgba([233, 30, 99, 255]),  // Rosa Escarlate
];

pub fn exportar_grafico_png(
    tipo: TipoGrafico,
    data: &ChartData,
    caminho_saida: &Path,
    largura: u32,
    altura: u32,
) -> Result<(), String> {
    let mut img = RgbaImage::new(largura, altura);
    let bg_color = Rgba([24, 26, 31, 255]);
    let border_color = Rgba([45, 48, 58, 255]);

    for pixel in img.pixels_mut() {
        *pixel = bg_color;
    }

    // Moldura exterior
    for x in 0..largura {
        img.put_pixel(x, 0, border_color);
        img.put_pixel(x, altura - 1, border_color);
    }
    for y in 0..altura {
        img.put_pixel(0, y, border_color);
        img.put_pixel(largura - 1, y, border_color);
    }

    let margin = 40i32;
    let x0 = margin;
    let y0 = margin;
    let x1 = (largura as i32) - margin;
    let y1 = (altura as i32) - margin;

    match tipo {
        TipoGrafico::PizzaOperacoes => render_pie(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::BarrasTipos | TipoGrafico::HistogramaHorarios => {
            render_bars(&mut img, data, x0, y0, x1, y1)
        }
        TipoGrafico::ParetoExtensoes => render_pareto(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::TimelineOperacoes => render_timeline(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::TreemapTipos => render_treemap(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::LinhaEventosHora => render_line(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::BoxplotTamanhos => render_boxplot(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::RadarOperacoes | TipoGrafico::RadarMultidimensional => {
            render_radar(&mut img, data, x0, y0, x1, y1)
        }
        TipoGrafico::HeatmapDiaHora => render_heatmap(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::ScatterTamanhoHora => render_scatter(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::DotplotDistribuicao => render_dotplot(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::SankeyOperacaoTipo
        | TipoGrafico::SankeyEventoDiretorio
        | TipoGrafico::SankeyTipoDiretorio => render_sankey(&mut img, data, x0, y0, x1, y1),
        TipoGrafico::ArvoreDiretorios => render_tree(&mut img, data, x0, y0, x1, y1),
    }

    img.save(caminho_saida)
        .map_err(|e| format!("Falha ao salvar imagem PNG: {}", e))?;

    Ok(())
}

fn fill_rect(img: &mut RgbaImage, x_min: i32, y_min: i32, x_max: i32, y_max: i32, color: Rgba<u8>) {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let x_start = x_min.clamp(0, w);
    let x_end = x_max.clamp(0, w);
    let y_start = y_min.clamp(0, h);
    let y_end = y_max.clamp(0, h);

    for y in y_start..y_end {
        for x in x_start..x_end {
            img.put_pixel(x as u32, y as u32, color);
        }
    }
}

fn draw_line_bresenham(img: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut curr_x = x0;
    let mut curr_y = y0;
    let w = img.width() as i32;
    let h = img.height() as i32;

    loop {
        if curr_x >= 0 && curr_x < w && curr_y >= 0 && curr_y < h {
            img.put_pixel(curr_x as u32, curr_y as u32, color);
        }
        if curr_x == x1 && curr_y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            curr_x += sx;
        }
        if e2 <= dx {
            err += dx;
            curr_y += sy;
        }
    }
}

fn draw_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Rgba<u8>) {
    let w = img.width() as i32;
    let h = img.height() as i32;

    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r {
                if x >= 0 && x < w && y >= 0 && y < h {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

fn draw_circle_hollow(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Rgba<u8>) {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let inner_r = (r - 1).max(0);

    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            let dist_sq = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if dist_sq <= r * r && dist_sq >= inner_r * inner_r {
                if x >= 0 && x < w && y >= 0 && y < h {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

fn render_pie(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let total: f64 = data.values.iter().sum();
    if total <= 0.0 {
        return;
    }

    let cx = (x0 + x1) / 2 - 80;
    let cy = (y0 + y1) / 2;
    let radius = ((y1 - y0).min(x1 - x0) / 2 - 20).max(30);

    let mut start_angle = -PI / 2.0;
    let w = img.width() as i32;
    let h = img.height() as i32;

    for (idx, &val) in data.values.iter().enumerate() {
        let sweep = ((val / total) as f32) * 2.0 * PI;
        let end_angle = start_angle + sweep;
        let color = CORES_PALETA_EXPORT[idx % CORES_PALETA_EXPORT.len()];

        for y in (cy - radius)..=(cy + radius) {
            for x in (cx - radius)..=(cx + radius) {
                let dx = (x - cx) as f32;
                let dy = (y - cy) as f32;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= (radius * radius) as f32 {
                    let mut a = dy.atan2(dx);
                    if a < -PI / 2.0 {
                        a += 2.0 * PI;
                    }
                    let mut norm_start = start_angle;
                    let mut norm_end = end_angle;
                    if norm_start < -PI / 2.0 {
                        norm_start += 2.0 * PI;
                        norm_end += 2.0 * PI;
                    }
                    if (a >= norm_start && a <= norm_end)
                        || (a + 2.0 * PI >= norm_start && a + 2.0 * PI <= norm_end)
                    {
                        if x >= 0 && x < w && y >= 0 && y < h {
                            img.put_pixel(x as u32, y as u32, color);
                        }
                    }
                }
            }
        }

        start_angle = end_angle;
    }

    // Legenda lateral
    let leg_x = cx + radius + 40;
    let mut leg_y = y0 + 20;
    for (idx, _) in data.labels.iter().enumerate().take(8) {
        let color = CORES_PALETA_EXPORT[idx % CORES_PALETA_EXPORT.len()];
        fill_rect(img, leg_x, leg_y, leg_x + 14, leg_y + 14, color);
        leg_y += 24;
    }
}

fn render_bars(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    if data.values.is_empty() {
        return;
    }

    let n = data.values.len() as i32;
    let plot_w = x1 - x0;
    let plot_h = y1 - y0 - 30;
    let bar_w = ((plot_w / n) - 8).max(4);
    let max_v = data.values.iter().copied().fold(1.0, f64::max);

    // Linha de base
    draw_line_bresenham(img, x0, y1 - 20, x1, y1 - 20, Rgba([120, 120, 120, 255]));

    for (i, &v) in data.values.iter().enumerate() {
        let h_bar = (((v / max_v) as f32) * (plot_h as f32)) as i32;
        let bx0 = x0 + (i as i32) * (plot_w / n) + 4;
        let by0 = y1 - 20 - h_bar;
        let bx1 = bx0 + bar_w;
        let by1 = y1 - 20;
        let color = CORES_PALETA_EXPORT[i % CORES_PALETA_EXPORT.len()];

        fill_rect(img, bx0, by0, bx1, by1, color);
    }
}

fn render_pareto(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    render_bars(img, data, x0, y0, x1, y1);

    if !data.secondary_values.is_empty() {
        let n = data.secondary_values.len() as i32;
        let plot_w = x1 - x0;
        let plot_h = y1 - y0 - 30;
        let mut prev_pt: Option<(i32, i32)> = None;

        for (i, &pct) in data.secondary_values.iter().enumerate() {
            let px = x0 + (i as i32) * (plot_w / n) + (plot_w / (2 * n));
            let py = y1 - 20 - ((pct as f32 / 100.0) * (plot_h as f32)) as i32;

            if let Some((x_prev, y_prev)) = prev_pt {
                draw_line_bresenham(img, x_prev, y_prev, px, py, Rgba([255, 179, 0, 255]));
            }
            draw_circle(img, px, py, 4, Rgba([255, 215, 0, 255]));
            prev_pt = Some((px, py));
        }
    }
}

fn render_timeline(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let left_x = x0 + 40;
    let right_x = x1 - 20;
    let top_y = y0 + 20;
    let bottom_y = y1 - 20;

    // Eixos X e Y
    draw_line_bresenham(img, left_x, bottom_y, left_x, top_y, Rgba([180, 190, 205, 255]));
    draw_line_bresenham(img, left_x, bottom_y, right_x, bottom_y, Rgba([180, 190, 205, 255]));

    let lanes = 6i32;
    let lane_h = (bottom_y - top_y) / lanes.max(1);

    for i in 0..lanes {
        let ly = top_y + i * lane_h + lane_h / 2;
        draw_line_bresenham(img, left_x - 4, ly, left_x, ly, Rgba([120, 125, 135, 255]));
        draw_line_bresenham(img, left_x, ly, right_x, ly, Rgba([45, 48, 58, 255]));
    }

    let n = data.timeline_items.len() as i32;
    let plot_w = right_x - left_x;
    for (idx, (_, op, _, _)) in data.timeline_items.iter().enumerate() {
        let op_type = OperationType::from_raw(op);
        let lane_idx = match op_type {
            OperationType::Adicionado => 0,
            OperationType::Modificado => 1,
            OperationType::Excluido => 2,
            OperationType::Renomeado => 3,
            OperationType::Movido => 4,
            OperationType::Escaneado => 5,
        };
        let ly = top_y + lane_idx * lane_h + lane_h / 2;
        let lx = (left_x + ((idx as i32) * plot_w) / n.max(1)).clamp(left_x + 4, right_x - 4);
        let color = CORES_PALETA_EXPORT[(lane_idx as usize) % CORES_PALETA_EXPORT.len()];
        draw_circle(img, lx, ly, 4, color);
    }
}

fn render_treemap(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let total: f64 = data.treemap_items.iter().map(|item| item.1).sum();
    if total <= 0.0 || data.treemap_items.is_empty() {
        return;
    }
    layout_export_treemap_recursive(&data.treemap_items, x0, y0, x1, y1, img);
}

fn layout_export_treemap_recursive(
    items: &[(String, f64, egui::Color32)],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    img: &mut RgbaImage,
) {
    if items.is_empty() || x1 <= x0 + 1 || y1 <= y0 + 1 {
        return;
    }
    if items.len() == 1 {
        let val = items[0].1;
        let color = CORES_PALETA_EXPORT[((val * 7.0) as usize) % CORES_PALETA_EXPORT.len()];
        fill_rect(img, x0 + 1, y0 + 1, x1 - 1, y1 - 1, color);
        return;
    }
    let total: f64 = items.iter().map(|it| it.1).sum();
    if total <= 0.0 {
        return;
    }
    let half: f64 = total / 2.0;
    let mut acc: f64 = 0.0;
    let mut best_idx = 1;
    let mut best_diff: f64 = f64::MAX;
    for (i, it) in items[..items.len() - 1].iter().enumerate() {
        acc += it.1;
        let diff: f64 = (acc - half).abs();
        if diff < best_diff {
            best_diff = diff;
            best_idx = i + 1;
        }
    }
    let left_items = &items[..best_idx];
    let right_items = &items[best_idx..];
    let left_weight: f64 = left_items.iter().map(|it| it.1).sum();
    let frac = left_weight / total;

    let w = x1 - x0;
    let h = y1 - y0;

    if w >= h {
        let split_w = (((w as f64) * frac).round() as i32).clamp(1, w - 1);
        layout_export_treemap_recursive(left_items, x0, y0, x0 + split_w, y1, img);
        layout_export_treemap_recursive(right_items, x0 + split_w, y0, x1, y1, img);
    } else {
        let split_h = (((h as f64) * frac).round() as i32).clamp(1, h - 1);
        layout_export_treemap_recursive(left_items, x0, y0, x1, y0 + split_h, img);
        layout_export_treemap_recursive(right_items, x0, y0 + split_h, x1, y1, img);
    }
}

fn render_line(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let plot_w = x1 - x0;
    let plot_h = y1 - y0;

    // Eixos
    draw_line_bresenham(img, x0, y1, x1, y1, Rgba([100, 100, 100, 255]));
    draw_line_bresenham(img, x0, y0, x0, y1, Rgba([100, 100, 100, 255]));

    let mut max_v = 1.0;
    for (_, vals, _) in &data.multi_series {
        for &v in vals {
            if v > max_v {
                max_v = v;
            }
        }
    }

    for (s_idx, (_, vals, _)) in data.multi_series.iter().enumerate() {
        let color = CORES_PALETA_EXPORT[s_idx % CORES_PALETA_EXPORT.len()];
        let mut prev_pt: Option<(i32, i32)> = None;
        let pts_count = vals.len().max(2);

        for (h, &val) in vals.iter().enumerate() {
            let px = x0 + ((h as i32) * plot_w) / ((pts_count - 1) as i32).max(1);
            let py = y1 - (((val / max_v) as f32) * (plot_h as f32)) as i32;

            if let Some((x_prev, y_prev)) = prev_pt {
                draw_line_bresenham(img, x_prev, y_prev, px, py, color);
            }
            draw_circle(img, px, py, 3, color);
            prev_pt = Some((px, py));
        }
    }
}

fn render_boxplot(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    if !data.boxplot_groups.is_empty() {
        let n = data.boxplot_groups.len() as i32;
        let plot_w = x1 - x0;
        let plot_h = (y1 - y0) as f32;
        let col_w = plot_w / n.max(1);

        let global_max = data.boxplot_groups.iter().map(|g| g.5).fold(1.0, f64::max);
        let global_min = data
            .boxplot_groups
            .iter()
            .map(|g| g.1)
            .fold(f64::INFINITY, f64::min)
            .max(0.0);
        let global_min = if global_min.is_finite() { global_min } else { 0.0 };

        let log_min = if global_min <= 1.0 {
            0.0
        } else {
            ((global_min + 1.0).log10() * 0.92).max(0.0)
        };
        let log_max = (((global_max.max(1.0) + 1.0).log10()) * 1.06).max(log_min + 0.5);
        let log_range = (log_max - log_min).max(0.25);

        let map_y = |v: f64| -> i32 {
            let lv = (v.max(0.0) + 1.0).log10();
            let norm = ((lv - log_min) / log_range).clamp(0.0, 1.0) as f32;
            y1 - (norm * plot_h) as i32
        };

        draw_line_bresenham(img, x0, y1, x1, y1, Rgba([100, 100, 100, 255]));
        draw_line_bresenham(img, x0, y0, x0, y1, Rgba([100, 100, 100, 255]));

        for (idx, (_ext, min, q1, med, q3, max)) in data.boxplot_groups.iter().enumerate() {
            let cx = x0 + (idx as i32) * col_w + col_w / 2;
            let color = CORES_PALETA_EXPORT[idx % CORES_PALETA_EXPORT.len()];

            let y_min = map_y(*min);
            let y_q1_raw = map_y(*q1);
            let y_med = map_y(*med);
            let y_q3_raw = map_y(*q3);
            let y_max = map_y(*max);

            let (y_q3, y_q1) = if (y_q1_raw - y_q3_raw).abs() < 4 {
                ((y_med - 2).max(y0), (y_med + 2).min(y1))
            } else {
                (y_q3_raw.min(y_q1_raw), y_q1_raw.max(y_q3_raw))
            };

            let half_w = (col_w / 4).clamp(4, 25);

            draw_line_bresenham(img, cx, y_max, cx, y_min, Rgba([200, 200, 200, 255]));
            draw_line_bresenham(img, cx - half_w / 2, y_max, cx + half_w / 2, y_max, Rgba([200, 200, 200, 255]));
            draw_line_bresenham(img, cx - half_w / 2, y_min, cx + half_w / 2, y_min, Rgba([200, 200, 200, 255]));

            fill_rect(img, cx - half_w, y_q3, cx + half_w, y_q1, color);
            draw_line_bresenham(img, cx - half_w, y_med, cx + half_w, y_med, Rgba([255, 235, 59, 255]));
        }
    } else if let Some((min, q1, med, q3, max)) = data.boxplot {
        let cx = (x0 + x1) / 2;
        let range = (max - min).max(1.0);
        let plot_h = (y1 - y0) as f32;

        let map_y = |v: f64| -> i32 { y1 - (((v - min) / range) as f32 * plot_h) as i32 };

        let y_min = map_y(min);
        let y_q1 = map_y(q1);
        let y_med = map_y(med);
        let y_q3 = map_y(q3);
        let y_max = map_y(max);

        draw_line_bresenham(img, cx, y_max, cx, y_min, Rgba([200, 200, 200, 255]));
        draw_line_bresenham(
            img,
            cx - 20,
            y_max,
            cx + 20,
            y_max,
            Rgba([200, 200, 200, 255]),
        );
        draw_line_bresenham(
            img,
            cx - 20,
            y_min,
            cx + 20,
            y_min,
            Rgba([200, 200, 200, 255]),
        );

        fill_rect(img, cx - 40, y_q3, cx + 40, y_q1, Rgba([21, 101, 192, 255]));
        draw_line_bresenham(
            img,
            cx - 40,
            y_med,
            cx + 40,
            y_med,
            Rgba([255, 179, 0, 255]),
        );
    }
}

fn render_radar(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let cx = (x0 + x1) / 2;
    let cy = (y0 + y1) / 2;
    let radius = ((y1 - y0).min(x1 - x0) / 2 - 20).max(30);

    let n = if !data.radar_dimensions.is_empty() {
        data.radar_dimensions.len()
    } else {
        data.labels.len().max(3)
    };

    for i in 0..n {
        let angle = -PI / 2.0 + (i as f32 / n as f32) * 2.0 * PI;
        let sx = cx + (angle.cos() * (radius as f32)) as i32;
        let sy = cy + (angle.sin() * (radius as f32)) as i32;
        let spoke_color = CORES_PALETA_EXPORT[i % CORES_PALETA_EXPORT.len()];
        draw_line_bresenham(img, cx, cy, sx, sy, spoke_color);
    }

    let mut pts = Vec::new();
    for (i, (_, score)) in data.radar_dimensions.iter().enumerate() {
        let angle = -PI / 2.0 + (i as f32 / n as f32) * 2.0 * PI;
        let r_val = (*score as f32 / 100.0) * (radius as f32);
        let px = cx + (angle.cos() * r_val) as i32;
        let py = cy + (angle.sin() * r_val) as i32;
        pts.push((px, py));
    }

    for i in 0..pts.len() {
        let next = (i + 1) % pts.len();
        let color = CORES_PALETA_EXPORT[i % CORES_PALETA_EXPORT.len()];
        draw_line_bresenham(
            img,
            pts[i].0,
            pts[i].1,
            pts[next].0,
            pts[next].1,
            color,
        );
        draw_circle(img, pts[i].0, pts[i].1, 5, color);
    }
}

fn render_heatmap(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let cell_w = (x1 - x0) / 24;
    let cell_h = (y1 - y0) / 7;

    let mut max_v = 1.0;
    for d in 0..7 {
        for h in 0..24 {
            if data.matrix_7x24[d][h] > max_v {
                max_v = data.matrix_7x24[d][h];
            }
        }
    }

    for d in 0..7 {
        for h in 0..24 {
            let val = data.matrix_7x24[d][h];
            let t = (val / max_v) as f32;
            let color = Rgba([
                (24.0 + t * 200.0) as u8,
                (26.0 + t * 100.0) as u8,
                (31.0 + t * 40.0) as u8,
                255,
            ]);

            let cx0 = x0 + (h as i32) * cell_w;
            let cy0 = y0 + (d as i32) * cell_h;
            fill_rect(
                img,
                cx0 + 1,
                cy0 + 1,
                cx0 + cell_w - 1,
                cy0 + cell_h - 1,
                color,
            );
        }
    }
}

fn render_scatter(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    draw_line_bresenham(img, x0, y1, x1, y1, Rgba([180, 190, 205, 255]));
    draw_line_bresenham(img, x0, y0, x0, y1, Rgba([180, 190, 205, 255]));

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
    let plot_h = (y1 - y0).max(1) as f32;
    let plot_w = (x1 - x0).max(1) as f32;

    for k in (log_min.floor() as i32)..=(log_max.ceil() as i32) {
        let norm = ((k as f32 - log_min_f) / log_span).clamp(0.0, 1.0);
        let gy = y1 - (norm * plot_h) as i32;
        draw_line_bresenham(img, x0, gy, x1, gy, Rgba([45, 48, 58, 255]));
    }

    let max_x = if data.max_x > 0.0 { data.max_x as f32 } else { 24.0 };
    for (idx, &(h, sz)) in data.points.iter().enumerate() {
        let px = x0 + (((h as f32 / max_x).clamp(0.0, 1.0)) * plot_w) as i32;
        let log_y = data
            .values
            .get(idx)
            .copied()
            .unwrap_or_else(|| calcular_log10_kb(sz)) as f32;
        let norm_y = ((log_y - log_min_f) / log_span).clamp(0.0, 1.0);
        let py = (y1 - (norm_y * plot_h) as i32).clamp(y0 + 4, y1 - 4);
        let color = CORES_PALETA_EXPORT[idx % CORES_PALETA_EXPORT.len()];
        draw_circle(img, px, py, 4, color);
    }
}

fn render_dotplot(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let left_x = x0 + 50;
    let right_x = (x1 - 120).max(left_x + 80);
    let top_y = y0 + 20;
    let bottom_y = y1 - 20;

    draw_line_bresenham(img, left_x, bottom_y, right_x, bottom_y, Rgba([180, 190, 205, 255]));
    draw_line_bresenham(img, left_x, top_y, left_x, bottom_y, Rgba([180, 190, 205, 255]));

    // Indicador vertical do título do eixo das ordenadas (centralizado em relação ao eixo Y)
    let y_mid = (top_y + bottom_y) / 2;
    draw_line_bresenham(img, x0 + 14, y_mid - 35, x0 + 14, y_mid + 35, Rgba([100, 181, 246, 255]));

    // Painel de legenda "Escala por Tamanho" ao lado direito da área de plotagem
    let leg_x0 = right_x + 14;
    let leg_x1 = (x1 - 6).max(leg_x0 + 60);
    let leg_y0 = (y_mid - 42).max(top_y);
    let leg_y1 = (y_mid + 42).min(bottom_y);
    fill_rect(img, leg_x0, leg_y0, leg_x1, leg_y1, Rgba([30, 33, 41, 255]));
    draw_circle(img, leg_x0 + 14, leg_y0 + 24, 5, Rgba([76, 175, 80, 255]));
    draw_circle_hollow(img, leg_x0 + 14, leg_y0 + 52, 6, Rgba([255, 152, 0, 255]));

    let n_groups = data.dotplot_groups.len() as i32;
    if n_groups == 0 {
        return;
    }
    let plot_w = right_x - left_x;
    let row_h = (bottom_y - top_y) / n_groups.max(1);

    for k in 0..=6 {
        let gx = left_x + ((k as f32 / 6.0) * (plot_w as f32)) as i32;
        draw_line_bresenham(img, gx, top_y, gx, bottom_y, Rgba([40, 44, 54, 255]));
    }

    for (idx, (_cat, values)) in data.dotplot_groups.iter().enumerate() {
        let row_y = top_y + (idx as i32) * row_h + row_h / 2;
        let color = CORES_PALETA_EXPORT[idx % CORES_PALETA_EXPORT.len()];

        draw_line_bresenham(img, left_x - 4, row_y, left_x, row_y, Rgba([120, 125, 135, 255]));
        draw_line_bresenham(img, left_x, row_y, right_x, row_y, Rgba([45, 48, 58, 255]));

        for &val in values {
            let log_v = calcular_log10_kb(val.max(1.0));
            let progress = (log_v / 6.0).clamp(0.0, 1.0) as f32;
            let r = (2.5 + progress * 4.0) as i32;
            let px = (left_x + (progress * (plot_w as f32)) as i32).clamp(left_x + r + 1, right_x - r - 1);
            let py = row_y.clamp(top_y + r + 1, bottom_y - r - 1);
            if val <= 1024.0 {
                draw_circle(img, px, py, r, color);
            } else {
                draw_circle_hollow(img, px, py, r, color);
            }
        }
    }
}

fn render_sankey(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut cur_y = y0 + 10;
    for (idx, (_, _, _)) in data.flows.iter().enumerate() {
        if cur_y + 16 >= y1 {
            break;
        }
        let color = CORES_PALETA_EXPORT[idx % CORES_PALETA_EXPORT.len()];
        let p_end = (cur_y + 16).min(y1);
        fill_rect(img, x0 + 40, cur_y, x0 + 60, p_end, color);
        draw_line_bresenham(img, x0 + 60, cur_y + 8, x1 - 60, cur_y + 8, color);
        fill_rect(img, x1 - 60, cur_y, x1 - 40, p_end, color);
        cur_y += 24;
    }
}

fn render_tree(img: &mut RgbaImage, data: &ChartData, x0: i32, y0: i32, x1: i32, y1: i32) {
    let mut cy = y0 + 15;
    for (_path, depth, _count, _mb) in &data.tree_nodes {
        if cy + 12 >= y1 {
            break;
        }
        let ix = x0 + 20 + (*depth as i32) * 20;
        fill_rect(img, ix, cy, ix + 12, cy + 12, Rgba([144, 202, 249, 255]));
        draw_line_bresenham(
            img,
            ix + 16,
            cy + 6,
            x1 - 30,
            cy + 6,
            Rgba([60, 64, 76, 255]),
        );
        cy += 20;
    }
}
