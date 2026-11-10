use egui::{Color32, FontId, Id, Pos2, Rect, RichText, ScrollArea, Stroke, Ui, Vec2};
use std::collections::BTreeSet;

use super::app::HuginnApp;
use super::spreadsheet::cell_selection::CellSelectionModel;
use super::spreadsheet::grid_renderer::truncate_text;
use super::spreadsheet::{
    ColumnHeader, ContextMenuController, GridContextAction, GridDragMode, GridNavigation,
    GridRenderer, SelectAllButton,
};
use crate::core::types::OperationType;
use crate::locale::Localizer;
use crate::services::gerenciador_colunas::ColunaInfo;
use crate::services::GerenciadorTabela;
use crate::utils::app_association::{abrir_arquivo_com_associado, selecionar_no_explorer};

pub fn formatar_celula_evento(
    loc: &crate::locale::Localizer,
    ev: &crate::core::event::EventModel,
    col_key: &str,
) -> String {
    match col_key {
        "tipo_operacao" => loc.traduzir_tipo_operacao(ev.op_type().as_str()),
        "tipo" => {
            let t_lower = ev.tipo.trim().to_lowercase();
            let attrs_lower = ev.atributos.to_lowercase();
            let is_folder = t_lower == "folder"
                || t_lower == "pasta"
                || t_lower == "directory"
                || t_lower == "pasta de arquivos"
                || ev.mime == "inode/directory"
                || attrs_lower.starts_with("folder")
                || attrs_lower.starts_with("pasta");
            if is_folder {
                loc.t("folder")
            } else {
                let ext_nome = crate::services::metadados::extensao_tipo_preferencial(
                    std::path::Path::new(&ev.nome),
                );
                if !ext_nome.is_empty() {
                    format!(".{}", ext_nome.trim_start_matches('.'))
                } else {
                    loc.traduzir_tipo_arquivo(&ev.tipo)
                }
            }
        }
        "atributos" => loc.traduzir_atributos(&ev.atributos),
        "protegido" => loc.traduzir_booleano(&ev.protegido),
        "baseline_status" | "security_baseline_status" => {
            loc.traduzir_baseline_status(&ev.baseline_status)
        }
        "quarantine_status" | "security_quarantine_status" => {
            loc.traduzir_quarantine_status(&ev.quarantine_status)
        }
        "alerta_resumo" | "security_alert_summary" => loc.traduzir_alerta(&ev.alerta_resumo),
        "session_label" | "security_session_label" => {
            loc.traduzir_session_label(&ev.session_label)
        }
        "diff_resumo" | "security_diff_summary" => loc.traduzir_diff_resumo(&ev.diff_resumo),
        "actor_process" | "security_actor_process" => {
            loc.traduzir_actor_process(&ev.actor_process)
        }
        "risk_level" | "alerta_nivel" | "security_risk_level" => {
            loc.traduzir_risco(&ev.risk_level)
        }
        _ => ev.get_field(col_key),
    }
}

struct GridFrameInteractions {
    acao_contexto: Option<GridContextAction>,
    redimensionamentos: Vec<(&'static str, f32)>,
    autofit_solicitado: Option<String>,
    select_all_clicked: bool,
}

impl GridFrameInteractions {
    fn new() -> Self {
        Self {
            acao_contexto: None,
            redimensionamentos: Vec::new(),
            autofit_solicitado: None,
            select_all_clicked: false,
        }
    }
}

#[inline]
fn map_scroll_slot_to_row_idx(slot: usize, excluded_sorted: &[usize]) -> usize {
    let mut r = slot;
    for &ex in excluded_sorted {
        if ex <= r {
            r += 1;
        } else {
            break;
        }
    }
    r
}

#[allow(clippy::too_many_arguments)]
fn render_header_cell_at(
    ui: &mut Ui,
    rect: Rect,
    col: &ColumnHeader,
    col_info: &ColunaInfo,
    is_pinned_col: bool,
    total_filtrados: usize,
    loc: &Localizer,
    selection: &mut CellSelectionModel,
    selection_config: &crate::ui::spreadsheet::cell_selection::SelectionBehaviorConfig,
    interactions: &mut GridFrameInteractions,
    drag_mode: &mut GridDragMode,
) {
    let col_idx = col.logical_index;
    let is_col_selected = selection.is_col_selected(col_idx);
    let hdr_id = Id::unique(("grid_col_hdr_cell", col.key.as_str()));
    let response = ui.interact(rect, hdr_id, egui::Sense::click_and_drag());

    let display_title = if is_pinned_col {
        format!("📌 {}", col.title)
    } else {
        col.title.clone()
    };

    GridRenderer::render_column_header(
        ui,
        rect,
        &display_title,
        is_col_selected,
        col.is_sorted,
        col.sort_ascending,
        response.hovered(),
        Some(selection_config.cor_selecao_color32()),
    );

    // Alça interativa de redimensionamento na borda direita
    let edge_rect = Rect::from_min_max(
        Pos2::new(rect.right() - 4.0, rect.top()),
        Pos2::new(rect.right() + 4.0, rect.bottom()),
    );
    let edge_id = Id::unique(("resize_grid_col", col.key.as_str()));
    let edge_resp = ui.interact(edge_rect, edge_id, egui::Sense::click_and_drag());

    if edge_resp.hovered() || edge_resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
    }

    if edge_resp.dragged() {
        let delta = edge_resp.drag_delta().x;
        if delta.abs() > 0.0 {
            interactions
                .redimensionamentos
                .push((col_info.key, (col.width + delta).max(40.0)));
        }
    }

    if edge_resp.double_clicked() {
        interactions.autofit_solicitado = Some(col.key.clone());
    }

    let over_resize = edge_resp.hovered() || edge_resp.dragged();
    let primary_down = ui.input(|i| i.pointer.primary_down());

    if (response.drag_started()
        || (response.hovered() && primary_down)
        || response.is_pointer_button_down_on())
        && !over_resize
        && *drag_mode == GridDragMode::None
    {
        *drag_mode = GridDragMode::Cols { anchor_col: col_idx };
        let shift = ui.input(|i| i.modifiers.shift);
        let ctrl = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
        selection.select_col(col_idx, shift, ctrl, total_filtrados);
    }

    if response.clicked() && !over_resize && !response.dragged() {
        let shift = ui.input(|i| i.modifiers.shift);
        let ctrl = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
        selection.select_col(col_idx, shift, ctrl, total_filtrados);
    }

    crate::ui::widgets::menu_contexto_persistente(&response, |ui| {
        if let Some(act) = ContextMenuController::show_column_menu_localized(
            ui,
            loc,
            &col.key,
            &col.title,
            is_pinned_col,
        ) {
            interactions.acao_contexto = Some(act);
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_row_header_at(
    ui: &mut Ui,
    rect: Rect,
    row_idx: usize,
    is_pinned_row: bool,
    total_cols: usize,
    loc: &Localizer,
    selection: &mut CellSelectionModel,
    selection_config: &crate::ui::spreadsheet::cell_selection::SelectionBehaviorConfig,
    linha_selecionada: &mut Option<usize>,
    interactions: &mut GridFrameInteractions,
    drag_mode: &mut GridDragMode,
) {
    let rh_id = Id::unique(("grid_row_hdr_cell", row_idx));
    let rh_resp = ui.interact(rect, rh_id, egui::Sense::click_and_drag());
    let is_row_selected = selection.is_row_selected(row_idx);
    let is_row_active = selection.active_cell.map(|c| c.row) == Some(row_idx);

    GridRenderer::render_row_header(
        ui,
        rect,
        row_idx + 1,
        is_row_selected,
        is_row_active,
        rh_resp.hovered(),
        Some(selection_config.cor_selecao_color32()),
    );

    if is_pinned_row {
        let pin_accent = if ui.visuals().dark_mode {
            Color32::from_rgb(90, 170, 250)
        } else {
            Color32::from_rgb(24, 100, 196)
        };
        let bar_rect = Rect::from_min_max(
            rect.min,
            Pos2::new((rect.min.x + 3.0).min(rect.max.x), rect.max.y),
        );
        ui.painter().rect_filled(bar_rect, 0_u8, pin_accent);
    }

    let primary_down = ui.input(|i| i.pointer.primary_down());

    if (rh_resp.drag_started()
        || (rh_resp.hovered() && primary_down)
        || rh_resp.is_pointer_button_down_on())
        && *drag_mode == GridDragMode::None
    {
        *drag_mode = GridDragMode::Rows { anchor_row: row_idx };
        let shift = ui.input(|i| i.modifiers.shift);
        let ctrl = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
        selection.select_row(row_idx, shift, ctrl, total_cols);
        *linha_selecionada = Some(row_idx);
    }

    if rh_resp.clicked() && !rh_resp.dragged() {
        let shift = ui.input(|i| i.modifiers.shift);
        let ctrl = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
        selection.select_row(row_idx, shift, ctrl, total_cols);
        *linha_selecionada = Some(row_idx);
    }

    crate::ui::widgets::menu_contexto_persistente(&rh_resp, |ui| {
        if let Some(act) =
            ContextMenuController::show_row_menu_localized(ui, loc, row_idx, is_pinned_row)
        {
            interactions.acao_contexto = Some(act);
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_data_cell_at(
    ui: &mut Ui,
    cell_rect: Rect,
    row_idx: usize,
    is_pinned_row: bool,
    col: &ColumnHeader,
    is_pinned_col: bool,
    ev: &crate::core::event::EventModel,
    all_headers: &[ColumnHeader],
    loc: &Localizer,
    gerenciador_tabela: &GerenciadorTabela,
    selection: &mut CellSelectionModel,
    selection_config: &crate::ui::spreadsheet::cell_selection::SelectionBehaviorConfig,
    linha_selecionada: &mut Option<usize>,
    interactions: &mut GridFrameInteractions,
    drag_mode: &mut GridDragMode,
) {
    let col_idx = col.logical_index;
    let text = formatar_celula_evento(loc, ev, &col.key);

    let cell_id = Id::unique(("grid_data_cell", row_idx, col.key.as_str()));
    let cell_resp = ui.interact(cell_rect, cell_id, egui::Sense::click_and_drag());

    let is_cell_selected = selection.is_cell_selected(row_idx, col_idx);
    let is_active_cell = selection.is_active_cell(row_idx, col_idx);
    let dark_mode = ui.visuals().dark_mode;
    let is_hovered = cell_resp.hovered() && !is_cell_selected;
    let is_even = row_idx % 2 == 0;

    // Fundo base da planilha adaptado dinamicamente ao tema do Windows 11 e painéis fixos
    let base_bg = if is_cell_selected {
        selection_config.cor_selecao_fundo_celula(dark_mode)
    } else if is_hovered {
        if dark_mode {
            Color32::from_rgb(16, 18, 24)
        } else {
            Color32::from_rgb(230, 240, 252)
        }
    } else if is_pinned_row || is_pinned_col {
        if dark_mode {
            if is_even {
                Color32::from_rgb(29, 35, 45)
            } else {
                Color32::from_rgb(34, 41, 52)
            }
        } else if is_even {
            Color32::from_rgb(245, 249, 255)
        } else {
            Color32::from_rgb(239, 244, 252)
        }
    } else if is_even {
        if dark_mode {
            Color32::from_rgb(32, 34, 38)
        } else {
            Color32::from_rgb(255, 255, 255)
        }
    } else if dark_mode {
        Color32::from_rgb(38, 42, 48)
    } else {
        Color32::from_rgb(247, 248, 250)
    };

    ui.painter().rect_filled(cell_rect, 0_u8, base_bg);
    GridRenderer::draw_grid_lines(ui, cell_rect);

    if is_cell_selected && !is_active_cell {
        GridRenderer::draw_range_selection(
            ui,
            cell_rect,
            selection_config.cor_selecao_preenchimento(),
            selection_config.cor_selecao_color32(),
        );
    }

    if is_hovered {
        let hover_stroke = if dark_mode {
            Color32::from_rgb(58, 66, 84)
        } else {
            Color32::from_rgb(175, 200, 235)
        };
        ui.painter().rect_stroke(
            cell_rect,
            0_u8,
            Stroke::new(1.0, hover_stroke),
            egui::StrokeKind::Inside,
        );
    }

    let col_title = &col.title;
    cell_resp.clone().on_hover_ui(|ui| {
        ui.label(
            RichText::new(col_title)
                .strong()
                .size(12.0)
                .color(if dark_mode {
                    Color32::from_rgb(100, 180, 246)
                } else {
                    Color32::from_rgb(24, 100, 190)
                }),
        );
        ui.label(RichText::new(&text).size(12.0));
    });

    // Renderização do Conteúdo da Célula
    let deve_colorir = gerenciador_tabela.coluna_deve_colorir(&col.key);
    if deve_colorir {
        let (chip_bg, chip_fg) = gerenciador_tabela.obter_cores_operacao(ev.op_type());
        let chip_rect = cell_rect.shrink2(Vec2::new(3.0, 2.5));
        ui.painter().rect_filled(chip_rect, 3_u8, chip_bg);
        if col.key == "tipo_operacao" {
            ui.painter().text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                &text,
                FontId::proportional(11.0),
                chip_fg,
            );
        } else {
            ui.painter().text(
                Pos2::new(chip_rect.left() + 6.0, chip_rect.center().y),
                egui::Align2::LEFT_CENTER,
                truncate_text(&text, (col.width / 7.2) as usize),
                FontId::proportional(12.0),
                chip_fg,
            );
        }
    } else {
        let text_color = if is_cell_selected {
            selection_config.cor_selecao_texto(dark_mode)
        } else if is_hovered {
            if dark_mode {
                Color32::from_rgb(245, 250, 255)
            } else {
                Color32::from_rgb(10, 20, 40)
            }
        } else if dark_mode {
            Color32::from_rgb(230, 235, 245)
        } else {
            Color32::from_rgb(20, 24, 32)
        };
        ui.painter().text(
            Pos2::new(cell_rect.left() + 6.0, cell_rect.center().y),
            egui::Align2::LEFT_CENTER,
            truncate_text(&text, (col.width / 7.2) as usize),
            FontId::proportional(12.0),
            text_color,
        );
    }

    if is_active_cell {
        GridRenderer::draw_active_cell_indicator(
            ui,
            cell_rect,
            selection_config.cor_selecao_color32(),
        );
    }

    let primary_down = ui.input(|i| i.pointer.primary_down());

    if (cell_resp.drag_started()
        || (cell_resp.hovered() && primary_down)
        || cell_resp.is_pointer_button_down_on())
        && *drag_mode == GridDragMode::None
    {
        *drag_mode = GridDragMode::Cells {
            anchor_row: row_idx,
            anchor_col: col_idx,
        };
        let shift = ui.input(|i| i.modifiers.shift);
        if shift {
            selection.expand_to(row_idx, col_idx);
        } else {
            selection.select_cell(row_idx, col_idx);
        }
        *linha_selecionada = Some(row_idx);
    }

    if cell_resp.clicked() && !cell_resp.dragged() {
        let shift = ui.input(|i| i.modifiers.shift);
        if shift {
            selection.expand_to(row_idx, col_idx);
        } else {
            selection.select_cell(row_idx, col_idx);
        }
        *linha_selecionada = Some(row_idx);
    }

    if cell_resp.double_clicked() {
        interactions.acao_contexto = Some(GridContextAction::OpenFile(ev.full_path()));
    }

    crate::ui::widgets::menu_contexto_persistente(&cell_resp, |ui| {
        ui.set_min_width(235.0);
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "📄",
            loc.t("ctx_open_default_app"),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::OpenFile(ev.full_path()));
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "📂",
            loc.t("ctx_show_in_explorer"),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::ShowInExplorer(ev.full_path()));
        }
        ui.separator();
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "📋",
            loc.t("ctx_copy_cell_value"),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::CopyText(text.clone()));
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "📑",
            loc.t("ctx_copy_row_tsv"),
        )
        .clicked()
        {
            let tsv = all_headers
                .iter()
                .map(|c| formatar_celula_evento(loc, ev, &c.key))
                .collect::<Vec<_>>()
                .join("\t");
            interactions.acao_contexto = Some(GridContextAction::CopyTsv(tsv));
        }
        if !ev.sha256_atual.is_empty() {
            if crate::ui::widgets::render_centered_emoji_menu_button(
                ui,
                "🔑",
                loc.t("ctx_copy_sha256"),
            )
            .clicked()
            {
                interactions.acao_contexto =
                    Some(GridContextAction::CopyHash(ev.sha256_atual.clone()));
            }
        }
        ui.separator();
        let (pin_r_emoji, pin_r_txt) = if is_pinned_row {
            ("🔓", loc.t("ctx_unpin_row"))
        } else {
            ("📌", loc.t("ctx_pin_row"))
        };
        if crate::ui::widgets::render_centered_emoji_menu_button(ui, pin_r_emoji, pin_r_txt)
            .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::TogglePinRow(row_idx));
        }
        let (pin_c_emoji, pin_c_txt) = if is_pinned_col {
            ("🔓", loc.t("ctx_unpin_column"))
        } else {
            ("📌", loc.t("ctx_pin_column"))
        };
        if crate::ui::widgets::render_centered_emoji_menu_button(ui, pin_c_emoji, pin_c_txt)
            .clicked()
        {
            interactions.acao_contexto =
                Some(GridContextAction::TogglePinColumn(col.key.clone()));
        }
        ui.separator();
        let op_label = loc.traduzir_tipo_operacao(ev.op_type().as_str());
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "🔍",
            loc.t_args("ctx_filter_only_op_fmt", &[&op_label]),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::FilterOperation(ev.op_type()));
        }
        let ext = ev.nome.rsplit('.').next().unwrap_or("").to_lowercase();
        if !ext.is_empty() && ext != ev.nome.to_lowercase() {
            if crate::ui::widgets::render_centered_emoji_menu_button(
                ui,
                "🏷️",
                loc.t_args("ctx_filter_by_ext_fmt", &[&ext]),
            )
            .clicked()
            {
                interactions.acao_contexto = Some(GridContextAction::FilterExtension(ext));
            }
        }
        ui.separator();
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "➕",
            loc.t("ctx_insert_row"),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::InsertRowBelow);
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "🗑️",
            loc.t("ctx_delete_selected_row"),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::DeleteSelectedRows);
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "🔒",
            loc.t("ctx_move_to_quarantine"),
        )
        .clicked()
        {
            interactions.acao_contexto = Some(GridContextAction::Quarantine(ev.full_path()));
        }
    });
}

fn map_screen_pos_to_grid_col(
    p_x: f32,
    outer_rect: Rect,
    split_x: f32,
    fixed_rh_w: f32,
    scroll_rh_w: f32,
    fixed_col_count: usize,
    fixed_col_offsets: &[f32],
    scroll_col_offsets: &[f32],
    colunas_headers: &[ColumnHeader],
    scroll_x: f32,
) -> usize {
    if colunas_headers.is_empty() {
        return 0;
    }
    // Caso 1: Cursor está na área de colunas fixas à esquerda
    if p_x < split_x {
        let rel_x = p_x - (outer_rect.left() + fixed_rh_w);
        if rel_x <= 0.0 || fixed_col_count == 0 {
            return colunas_headers[0].logical_index;
        }
        for (f_idx, &offset) in fixed_col_offsets.iter().enumerate() {
            let width = colunas_headers[f_idx].width;
            if rel_x >= offset && rel_x < offset + width {
                return colunas_headers[f_idx].logical_index;
            }
        }
        if fixed_col_count > 0 {
            return colunas_headers[fixed_col_count - 1].logical_index;
        }
    }

    // Caso 2: Cursor está na área de colunas roláveis à direita
    let rel_x = p_x - split_x + scroll_x - scroll_rh_w;
    if rel_x <= 0.0 {
        return colunas_headers[fixed_col_count.min(colunas_headers.len() - 1)].logical_index;
    }
    for (s_idx, &offset) in scroll_col_offsets.iter().enumerate() {
        let global_idx = fixed_col_count + s_idx;
        if global_idx < colunas_headers.len() {
            let width = colunas_headers[global_idx].width;
            if rel_x >= offset && rel_x < offset + width {
                return colunas_headers[global_idx].logical_index;
            }
        }
    }
    colunas_headers.last().map(|c| c.logical_index).unwrap_or(0)
}

fn map_screen_pos_to_grid_row(
    p_y: f32,
    outer_rect: Rect,
    split_y: f32,
    base_top_hdr_h: f32,
    scroll_top_hdr_h: f32,
    row_height: f32,
    visible_pinned_rows: &[usize],
    excluded_scroll_rows: &[usize],
    scrollable_row_count: usize,
    scroll_y: f32,
    total_filtrados: usize,
) -> usize {
    if total_filtrados == 0 {
        return 0;
    }
    // Caso 1: Cursor está na área superior de linhas fixadas
    if p_y < split_y {
        let rel_y = p_y - (outer_rect.top() + base_top_hdr_h);
        if rel_y <= 0.0 || visible_pinned_rows.is_empty() {
            if let Some(&first) = visible_pinned_rows.first() {
                return first;
            }
        } else {
            let slot = (rel_y / row_height).floor() as usize;
            if slot < visible_pinned_rows.len() {
                return visible_pinned_rows[slot];
            }
            if let Some(&last) = visible_pinned_rows.last() {
                return last;
            }
        }
    }

    // Caso 2: Cursor está na área rolável
    let rel_y = p_y - split_y + scroll_y - scroll_top_hdr_h;
    if rel_y <= 0.0 {
        return map_scroll_slot_to_row_idx(0, excluded_scroll_rows)
            .min(total_filtrados.saturating_sub(1));
    }
    let slot = (rel_y / row_height).floor() as usize;
    if slot < scrollable_row_count {
        return map_scroll_slot_to_row_idx(slot, excluded_scroll_rows)
            .min(total_filtrados.saturating_sub(1));
    }
    total_filtrados.saturating_sub(1)
}

pub fn renderizar_tabela_eventos(app: &mut HuginnApp, ui: &mut Ui) {
    // Garante sincronização entre índices filtrados e eventos antes de renderizar em O(1)
    if app.filtros_sujos
        || app
            .indices_filtrados
            .last()
            .map_or(false, |&i| i >= app.eventos.len())
    {
        app.atualizar_indices_filtrados();
    }

    if !ui.input(|i| i.pointer.primary_down()) {
        app.spreadsheet_grid.drag_mode = GridDragMode::None;
    } else if app.spreadsheet_grid.drag_mode != GridDragMode::None {
        ui.ctx().request_repaint();
    }

    let colunas_base: Vec<ColunaInfo> = app
        .gerenciador_colunas
        .colunas_visiveis()
        .into_iter()
        .filter(|c| !app.spreadsheet_grid.manager.is_col_hidden(c.key))
        .cloned()
        .collect();

    if colunas_base.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(app.loc.t("no_visible_columns_configured"))
                    .size(14.0)
                    .color(ui.visuals().weak_text_color()),
            );
        });
        return;
    }

    // Particiona colunas em Fixas (à esquerda, não rolam horizontalmente) e Roláveis (à direita)
    let mut fixed_col_infos: Vec<ColunaInfo> = Vec::new();
    let mut scroll_col_infos: Vec<ColunaInfo> = Vec::new();
    for (orig_idx, c) in colunas_base.iter().cloned().enumerate() {
        if app
            .spreadsheet_grid
            .layout
            .is_col_effectively_pinned(orig_idx, c.key)
        {
            fixed_col_infos.push(c);
        } else {
            scroll_col_infos.push(c);
        }
    }
    let fixed_col_count = fixed_col_infos.len();
    let mut colunas_info = fixed_col_infos;
    colunas_info.extend(scroll_col_infos);

    let total_eventos = app.eventos.len();
    let total_filtrados = app.indices_filtrados.len();
    let row_height = 26.0;
    let header_height = 34.0;
    let row_header_width = 52.0;

    // Constrói os cabeçalhos de coluna respeitando a partição visual [fixas | roláveis]
    let mut colunas_headers: Vec<ColumnHeader> = Vec::with_capacity(colunas_info.len());
    for (idx, c) in colunas_info.iter().enumerate() {
        let mut h = ColumnHeader::new(
            idx,
            c.key,
            &app.loc.get_text(c.translation_key),
            c.largura_padrao,
        );
        h.is_sorted = app.gerenciador_tabela.coluna_ordenacao.as_deref() == Some(c.key);
        h.sort_ascending = app.gerenciador_tabela.ordem_ascendente;
        colunas_headers.push(h);
    }

    // --- Processamento de Navegação por Teclado (Excel-like) ---
    if GridNavigation::handle_keyboard(
        ui,
        &mut app.spreadsheet_grid.selection,
        total_filtrados,
        colunas_headers.len(),
        18,
    ) {
        if app.spreadsheet_grid.selection_config.deve_acompanhar_rolagem() {
            app.spreadsheet_grid.request_scroll_to_selection = true;
        }
    }

    if let Some(ac) = app.spreadsheet_grid.selection.active_cell {
        app.linha_selecionada = Some(ac.row);
    }

    // Coleta linhas de dados fixadas (0-based) válidas dentro de `0..total_filtrados`
    let mut pinned_rows_set: BTreeSet<usize> = BTreeSet::new();
    for r in 0..app.spreadsheet_grid.layout.frozen_rows.min(total_filtrados) {
        if !app.spreadsheet_grid.manager.is_row_hidden(r) {
            pinned_rows_set.insert(r);
        }
    }
    for &r in &app.spreadsheet_grid.layout.pinned_rows {
        if r < total_filtrados && !app.spreadsheet_grid.manager.is_row_hidden(r) {
            pinned_rows_set.insert(r);
        }
    }
    let pinned_data_rows: Vec<usize> = pinned_rows_set.into_iter().collect();

    // --- Barra de Ferramentas Superior da Planilha ---
    let mut acao_autofit_todas = false;
    let mut acao_toggle_freeze = false;

    ui.horizontal(|ui| {
        let pct = if total_eventos > 0 {
            (total_filtrados as f32 / total_eventos as f32) * 100.0
        } else {
            0.0
        };

        let dark_mode = ui.visuals().dark_mode;

        let (badge_bg, badge_stroke_col, badge_text_col) = if dark_mode {
            (
                Color32::from_rgb(26, 30, 40),
                Color32::from_rgb(52, 60, 80),
                Color32::from_rgb(220, 230, 245),
            )
        } else {
            (
                Color32::from_rgb(240, 244, 250),
                Color32::from_rgb(208, 216, 228),
                Color32::from_rgb(28, 36, 50),
            )
        };

        let emoji_font = FontId::proportional(13.0);
        let text_font = FontId::proportional(11.5);
        let emoji_str = "📊";
        let total_filt_str = total_filtrados.to_string();
        let total_ev_str = total_eventos.to_string();
        let pct_str = format!("{:.1}", pct);
        let text_str =
            app.loc
                .t_args("showing_events_count", &[&total_filt_str, &total_ev_str, &pct_str]);

        let emoji_galley =
            ui.painter()
                .layout_no_wrap(emoji_str.to_string(), emoji_font, badge_text_col);
        let text_galley = ui.painter().layout_no_wrap(text_str, text_font, badge_text_col);

        let spacing = 6.0;
        let emoji_w = emoji_galley.size().x;
        let total_content_w = emoji_w + spacing + text_galley.size().x;
        let badge_w = (total_content_w + 22.0).max(210.0);

        let badge_rect = ui.allocate_space(Vec2::new(badge_w, 26.0)).1;
        ui.painter().rect_filled(badge_rect, 4_u8, badge_bg);
        ui.painter().rect_stroke(
            badge_rect,
            4_u8,
            Stroke::new(1.0, badge_stroke_col),
            egui::StrokeKind::Inside,
        );

        let center_y = badge_rect.center().y;
        let start_x = badge_rect.center().x - (total_content_w / 2.0);

        let emoji_mesh_center_y = if emoji_galley.mesh_bounds.is_positive() {
            emoji_galley.mesh_bounds.center().y
        } else {
            emoji_galley.rect.center().y
        };
        let emoji_pos = Pos2::new(start_x, center_y - emoji_mesh_center_y);
        ui.painter().galley(emoji_pos, emoji_galley, badge_text_col);

        let text_mesh_center_y = if text_galley.mesh_bounds.is_positive() {
            text_galley.mesh_bounds.center().y
        } else {
            text_galley.rect.center().y
        };
        let text_x = start_x + emoji_w + spacing;
        let text_pos = Pos2::new(text_x, center_y - text_mesh_center_y);
        ui.painter().galley(text_pos, text_galley, badge_text_col);

        ui.add_space(8.0);

        if app.spreadsheet_grid.selection.is_all_selected {
            let label_col = if dark_mode {
                Color32::from_rgb(120, 240, 150)
            } else {
                Color32::from_rgb(16, 124, 65)
            };
            let total_str = total_filtrados.to_string();
            ui.label(
                RichText::new(app.loc.t_args("all_rows_selected_label", &[&total_str]))
                    .size(11.5)
                    .color(label_col),
            );
        } else if let Some(ac) = app.spreadsheet_grid.selection.active_cell {
            if ac.col < colunas_headers.len() {
                let col_name = &colunas_headers[ac.col].title;
                let label_col = if dark_mode {
                    Color32::from_rgb(120, 240, 150)
                } else {
                    Color32::from_rgb(16, 124, 65)
                };
                let row_str = (ac.row + 1).to_string();
                ui.label(
                    RichText::new(app.loc.t_args("cell_selected_label", &[col_name, &row_str]))
                        .size(11.5)
                        .color(label_col),
                );
            }
        }

        // Indicador compacto de painéis fixos quando linhas/colunas personalizadas estão fixas
        if !pinned_data_rows.is_empty() || fixed_col_count > 0 {
            ui.add_space(6.0);
            let total_fixed_r =
                pinned_data_rows.len() + usize::from(app.spreadsheet_grid.layout.freeze_header_row);
            let total_fixed_c =
                fixed_col_count + usize::from(app.spreadsheet_grid.layout.freeze_row_headers);
            let r_s = total_fixed_r.to_string();
            let c_s = total_fixed_c.to_string();
            let pin_col = if dark_mode {
                Color32::from_rgb(130, 200, 255)
            } else {
                Color32::from_rgb(24, 100, 190)
            };
            ui.label(
                RichText::new(app.loc.t_args("pinned_status_badge", &[&r_s, &c_s]))
                    .size(11.5)
                    .color(pin_col),
            );
        }

        ui.add_space(8.0);

        if app.gerenciador_tabela.is_ordenacao_habilitada() {
            if let Some(ref col_key) = app.gerenciador_tabela.coluna_ordenacao {
                let col_name = colunas_headers
                    .iter()
                    .find(|c| c.key == *col_key)
                    .map(|c| c.title.as_str())
                    .unwrap_or(col_key.as_str());
                let seta = if app.gerenciador_tabela.ordem_ascendente {
                    app.loc.t("ascending")
                } else {
                    app.loc.t("descending")
                };
                let ord_col = if dark_mode {
                    Color32::from_rgb(100, 180, 246)
                } else {
                    Color32::from_rgb(24, 100, 190)
                };
                let ord_txt = app.loc.t_args("sorted_order_prefix", &[col_name, &seta]);
                ui.label(RichText::new(ord_txt).size(11.5).color(ord_col));

                if crate::ui::widgets::render_centered_emoji_button(ui, "✖", "")
                    .on_hover_text(app.loc.t("clear_sorting"))
                    .clicked()
                {
                    app.gerenciador_tabela.coluna_ordenacao = None;
                    app.filtros_sujos = true;
                }
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if app.spreadsheet_grid.selection.active_cell.is_some()
                || app.spreadsheet_grid.selection.is_all_selected
            {
                if crate::ui::widgets::render_centered_emoji_button(
                    ui,
                    "✖",
                    app.loc.t("deselect_selection"),
                )
                .clicked()
                {
                    app.spreadsheet_grid.selection.clear();
                    app.linha_selecionada = None;
                }
            }

            if crate::ui::widgets::render_centered_emoji_button(
                ui,
                "↔️",
                app.loc.t("autofit_button"),
            )
            .on_hover_text(app.loc.t("autofit_tooltip"))
            .clicked()
            {
                acao_autofit_todas = true;
            }

            let freeze_label = if fixed_col_count > 0 {
                app.loc.t("unfreeze_column")
            } else {
                app.loc.t("freeze_first_column")
            };
            if crate::ui::widgets::render_centered_emoji_button(ui, "📌", freeze_label)
                .on_hover_text(app.loc.t("freeze_tooltip"))
                .clicked()
            {
                acao_toggle_freeze = true;
            }

            if crate::ui::widgets::render_centered_emoji_button(
                ui,
                "🔄",
                app.loc.t("invert_order"),
            )
            .on_hover_text(app.loc.t("invert_tooltip"))
            .clicked()
            {
                // Determina qual coluna deve ser ordenada e invertida:
                // 1. Verifica se há uma coluna inteira selecionada através de seu cabeçalho (selected_cols)
                // 2. Ou se há uma célula ativa apontando para a coluna
                let col_idx_opt = app
                    .spreadsheet_grid
                    .selection
                    .selected_cols
                    .iter()
                    .min()
                    .copied()
                    .or_else(|| app.spreadsheet_grid.selection.active_cell.map(|ac| ac.col));

                if let Some(col_idx) = col_idx_opt {
                    if let Some(col_header) = colunas_headers.get(col_idx) {
                        let target_key = col_header.key.clone();
                        if app.gerenciador_tabela.coluna_ordenacao.as_deref() == Some(&target_key)
                            && app.gerenciador_tabela.is_ordenacao_habilitada()
                        {
                            // A coluna selecionada já é a coluna de ordenação ativa: inverte entre ascendente e descendente
                            app.gerenciador_tabela.inverter_ordem();
                        } else {
                            // Nova coluna selecionada pelo cabeçalho: define-a e ordena em ascendente
                            app.gerenciador_tabela.definir_ordenacao_coluna(target_key, true);
                        }
                    }
                } else if app.gerenciador_tabela.coluna_ordenacao.is_some() {
                    // Nenhuma coluna selecionada, mas há coluna de ordenação ativa: inverte sua direção
                    app.gerenciador_tabela.inverter_ordem();
                } else {
                    // Sem coluna prévia: define timestamp e inverte para ascendente
                    app.gerenciador_tabela.definir_ordenacao_coluna("timestamp", true);
                }
                app.filtros_sujos = true;
            }
        });
    });

    if acao_autofit_todas {
        app.gerenciador_colunas
            .autofit_todas_colunas_ao_cabecalho(&app.loc);
    }

    if acao_toggle_freeze {
        if fixed_col_count > 0 {
            app.spreadsheet_grid.layout.frozen_cols = 0;
            app.spreadsheet_grid.layout.pinned_cols.clear();
        } else if let Some(first_col) = colunas_base.first() {
            app.spreadsheet_grid
                .layout
                .set_col_pinned(first_col.key, true);
        }
        app.spreadsheet_grid
            .layout
            .salvar_configuracao_painel_fixo();
    }

    ui.add_space(3.0);

    // =========================================================================
    // RENDERIZAÇÃO EM 4 QUADRANTES SINCRONIZADOS (LINHAS E COLUNAS FIXAS 2D)
    // =========================================================================
    let avail = ui.available_size();
    let table_w = if avail.x.is_finite() && avail.x > 80.0 {
        avail.x
    } else {
        900.0
    };
    let table_h = if avail.y.is_finite() && avail.y > 80.0 {
        avail.y
    } else {
        520.0
    };
    let (outer_rect, _) =
        ui.allocate_exact_size(Vec2::new(table_w, table_h), egui::Sense::hover());

    let col_header_fixed = app.spreadsheet_grid.layout.freeze_header_row;
    let row_num_fixed = app.spreadsheet_grid.layout.freeze_row_headers;

    let fixed_rh_w = if row_num_fixed { row_header_width } else { 0.0 };
    let scroll_rh_w = if !row_num_fixed { row_header_width } else { 0.0 };

    let fixed_cols_w: f32 = colunas_headers[..fixed_col_count]
        .iter()
        .map(|c| c.width)
        .sum();
    let scroll_cols_w: f32 = colunas_headers[fixed_col_count..]
        .iter()
        .map(|c| c.width)
        .sum();

    let raw_fixed_left_w = fixed_rh_w + fixed_cols_w;
    let fixed_left_pane_w = if fixed_col_count == colunas_headers.len() && scroll_rh_w == 0.0 {
        raw_fixed_left_w.min(table_w)
    } else {
        raw_fixed_left_w.min((table_w - 80.0).max(0.0))
    };

    let base_top_hdr_h = if col_header_fixed { header_height } else { 0.0 };
    let scroll_top_hdr_h = if !col_header_fixed { header_height } else { 0.0 };

    let max_pinned_fit =
        (((table_h - 80.0 - base_top_hdr_h).max(0.0)) / row_height).floor() as usize;
    let visible_pinned_rows: &[usize] =
        &pinned_data_rows[..pinned_data_rows.len().min(max_pinned_fit)];
    let fixed_top_pane_h = base_top_hdr_h + (visible_pinned_rows.len() as f32 * row_height);

    // Conjunto ordenado de linhas excluídas do corpo rolável (linhas fixas no topo + linhas ocultas)
    let mut excluded_set: BTreeSet<usize> = visible_pinned_rows.iter().copied().collect();
    for &hr in &app.spreadsheet_grid.manager.hidden_rows {
        if hr < total_filtrados {
            excluded_set.insert(hr);
        }
    }
    let excluded_scroll_rows: Vec<usize> = excluded_set.into_iter().collect();
    let scrollable_row_count = total_filtrados.saturating_sub(excluded_scroll_rows.len());

    let scroll_content_w = scroll_rh_w + scroll_cols_w;
    let scroll_content_h = scroll_top_hdr_h + (scrollable_row_count as f32 * row_height);

    let split_x = (outer_rect.left() + fixed_left_pane_w).min(outer_rect.right());
    let split_y = (outer_rect.top() + fixed_top_pane_h).min(outer_rect.bottom());

    let tl_rect = Rect::from_min_max(outer_rect.min, Pos2::new(split_x, split_y));
    let tr_rect = Rect::from_min_max(
        Pos2::new(split_x, outer_rect.top()),
        Pos2::new(outer_rect.right(), split_y),
    );
    let bl_rect = Rect::from_min_max(
        Pos2::new(outer_rect.left(), split_y),
        Pos2::new(split_x, outer_rect.bottom()),
    );
    let br_rect = Rect::from_min_max(Pos2::new(split_x, split_y), outer_rect.max);

    // Pré-calcula offsets horizontais cumulativos para colunas fixas e roláveis
    let mut fixed_col_offsets: Vec<f32> = Vec::with_capacity(fixed_col_count);
    let mut acc_x = 0.0;
    for c in &colunas_headers[..fixed_col_count] {
        fixed_col_offsets.push(acc_x);
        acc_x += c.width;
    }

    let mut scroll_col_offsets: Vec<f32> =
        Vec::with_capacity(colunas_headers.len() - fixed_col_count);
    acc_x = 0.0;
    for c in &colunas_headers[fixed_col_count..] {
        scroll_col_offsets.push(acc_x);
        acc_x += c.width;
    }

    // Encaminha roda do mouse quando o cursor está sobre painéis fixos (BL ou TR)
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let wheel_delta = ui.input(|i| i.smooth_scroll_delta);
    let mut override_scroll_x: Option<f32> = None;
    let mut override_scroll_y: Option<f32> = None;
    if let Some(pos) = pointer_pos {
        if bl_rect.contains(pos) && wheel_delta.y.abs() > 0.01 {
            let max_y = (scroll_content_h - br_rect.height()).max(0.0);
            override_scroll_y =
                Some((app.spreadsheet_grid.scroll_offset.y - wheel_delta.y).clamp(0.0, max_y));
        } else if tr_rect.contains(pos)
            && (wheel_delta.x.abs() > 0.01 || wheel_delta.y.abs() > 0.01)
        {
            let dx = if wheel_delta.x.abs() > 0.01 {
                wheel_delta.x
            } else {
                wheel_delta.y
            };
            let max_x = (scroll_content_w - br_rect.width()).max(0.0);
            override_scroll_x =
                Some((app.spreadsheet_grid.scroll_offset.x - dx).clamp(0.0, max_x));
        }
    }

    // Auto-Scroll: quando o modo 'Acompanhar a Seleção' estiver ativo, mantém a célula/linha selecionada visível
    if app.spreadsheet_grid.selection_config.deve_acompanhar_rolagem()
        && app.spreadsheet_grid.request_scroll_to_selection
    {
        app.spreadsheet_grid.request_scroll_to_selection = false;
        let target_cell = app.spreadsheet_grid.selection.active_cell
            .or_else(|| app.linha_selecionada.map(|r| crate::ui::spreadsheet::cell_selection::CellCoord::new(r, 0)));

        if let Some(target) = target_cell {
            // Ajuste vertical (linhas)
            if !app.spreadsheet_grid.layout.is_row_pinned(target.row) && target.row < total_filtrados {
                let slot = target.row.saturating_sub(
                    excluded_scroll_rows.iter().filter(|&&r| r < target.row).count()
                );
                let row_top = slot as f32 * row_height;
                let row_bottom = row_top + row_height;
                let cur_y = override_scroll_y.unwrap_or(app.spreadsheet_grid.scroll_offset.y);
                let avail_h = (br_rect.height() - scroll_top_hdr_h).max(row_height);

                if row_top < cur_y {
                    override_scroll_y = Some(row_top);
                } else if row_bottom > cur_y + avail_h {
                    override_scroll_y = Some((row_bottom - avail_h).max(0.0));
                }
            }

            // Ajuste horizontal (colunas)
            if target.col >= fixed_col_count && target.col < colunas_headers.len() {
                let s_idx = target.col - fixed_col_count;
                if s_idx < scroll_col_offsets.len() {
                    let col_left = scroll_col_offsets[s_idx];
                    let col_w = colunas_headers[target.col].width;
                    let col_right = col_left + col_w;
                    let cur_x = override_scroll_x.unwrap_or(app.spreadsheet_grid.scroll_offset.x);
                    let avail_w = (br_rect.width() - scroll_rh_w).max(col_w);

                    if col_left < cur_x {
                        override_scroll_x = Some(col_left);
                    } else if col_right > cur_x + avail_w {
                        override_scroll_x = Some((col_right - avail_w).max(0.0));
                    }
                }
            }
        }
    }

    // Sincronização em tempo real de seleção interativa por arraste contínuo (Células, Colunas e Linhas)
    let total_cols = colunas_headers.len();
    let primary_down = ui.input(|i| i.pointer.primary_down());
    let pointer_pos = ui.input(|i| i.pointer.latest_pos().or(i.pointer.hover_pos()));

    if !primary_down {
        app.spreadsheet_grid.drag_mode = GridDragMode::None;
    } else if app.spreadsheet_grid.drag_mode != GridDragMode::None {
        if let Some(pos) = pointer_pos {
            match app.spreadsheet_grid.drag_mode {
                GridDragMode::Cols { anchor_col } => {
                    let cur_col = map_screen_pos_to_grid_col(
                        pos.x,
                        outer_rect,
                        split_x,
                        fixed_rh_w,
                        scroll_rh_w,
                        fixed_col_count,
                        &fixed_col_offsets,
                        &scroll_col_offsets,
                        &colunas_headers,
                        app.spreadsheet_grid.scroll_offset.x,
                    );
                    app.spreadsheet_grid
                        .selection
                        .select_col_range(anchor_col, cur_col, total_filtrados);
                }
                GridDragMode::Rows { anchor_row } => {
                    let cur_row = map_screen_pos_to_grid_row(
                        pos.y,
                        outer_rect,
                        split_y,
                        base_top_hdr_h,
                        scroll_top_hdr_h,
                        row_height,
                        visible_pinned_rows,
                        &excluded_scroll_rows,
                        scrollable_row_count,
                        app.spreadsheet_grid.scroll_offset.y,
                        total_filtrados,
                    );
                    app.spreadsheet_grid
                        .selection
                        .select_row_range(anchor_row, cur_row, total_cols);
                    app.linha_selecionada = Some(cur_row);
                }
                GridDragMode::Cells {
                    anchor_row,
                    anchor_col,
                } => {
                    let cur_col = map_screen_pos_to_grid_col(
                        pos.x,
                        outer_rect,
                        split_x,
                        fixed_rh_w,
                        scroll_rh_w,
                        fixed_col_count,
                        &fixed_col_offsets,
                        &scroll_col_offsets,
                        &colunas_headers,
                        app.spreadsheet_grid.scroll_offset.x,
                    );
                    let cur_row = map_screen_pos_to_grid_row(
                        pos.y,
                        outer_rect,
                        split_y,
                        base_top_hdr_h,
                        scroll_top_hdr_h,
                        row_height,
                        visible_pinned_rows,
                        &excluded_scroll_rows,
                        scrollable_row_count,
                        app.spreadsheet_grid.scroll_offset.y,
                        total_filtrados,
                    );
                    app.spreadsheet_grid.selection.anchor_cell = Some(
                        crate::ui::spreadsheet::cell_selection::CellCoord::new(
                            anchor_row, anchor_col,
                        ),
                    );
                    app.spreadsheet_grid.selection.expand_to(cur_row, cur_col);
                    app.linha_selecionada = Some(cur_row);
                }
                GridDragMode::None => {}
            }
            ui.ctx().request_repaint();
        }
    }

    let mut interactions = GridFrameInteractions::new();

    // -------------------------------------------------------------------------
    // QUADRANTE 1: Inferior Direito (BR) — Área Principal Rolável (2D)
    // -------------------------------------------------------------------------
    let mut scroll_x = app.spreadsheet_grid.scroll_offset.x;
    let mut scroll_y = app.spreadsheet_grid.scroll_offset.y;
    let mut vp_width = br_rect.width();
    let mut vp_height = br_rect.height();

    if br_rect.width() > 1.0 && br_rect.height() > 1.0 {
        let mut br_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(br_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        br_ui.set_clip_rect(br_rect);

        let mut scroll_area = ScrollArea::both()
            .id_salt("linceu_audit_grid_scroll_2d")
            .auto_shrink([false, false]);
        if let Some(ox) = override_scroll_x {
            scroll_area = scroll_area.horizontal_scroll_offset(ox);
        }
        if let Some(oy) = override_scroll_y {
            scroll_area = scroll_area.vertical_scroll_offset(oy);
        }

        let br_out = scroll_area.show_viewport(&mut br_ui, |vp_ui, viewport| {
            scroll_x = viewport.min.x;
            scroll_y = viewport.min.y;
            vp_width = viewport.width();
            vp_height = viewport.height();

            let (canvas_rect, _) = vp_ui.allocate_exact_size(
                Vec2::new(scroll_content_w.max(1.0), scroll_content_h.max(1.0)),
                egui::Sense::hover(),
            );

            // Se a 1ª linha de cabeçalho NÃO estiver fixa, renderiza no topo do canvas rolável
            if !col_header_fixed && scroll_y <= header_height + row_height {
                let hdr_y = canvas_rect.top();
                if !row_num_fixed {
                    let corner_r = Rect::from_min_size(
                        Pos2::new(canvas_rect.left(), hdr_y),
                        Vec2::new(row_header_width, header_height),
                    );
                    let btn = SelectAllButton::with_color(
                        app.spreadsheet_grid.selection.is_all_selected,
                        Some(app.spreadsheet_grid.selection_config.cor_selecao_color32()),
                    );
                    if btn.show_at_rect(vp_ui, corner_r).clicked() {
                        interactions.select_all_clicked = true;
                    }
                }
                for (s_idx, col) in colunas_headers[fixed_col_count..].iter().enumerate() {
                    let local_x = scroll_rh_w + scroll_col_offsets[s_idx];
                    if local_x + col.width < scroll_x - 32.0
                        || local_x > scroll_x + vp_width + 32.0
                    {
                        continue;
                    }
                    let c_rect = Rect::from_min_size(
                        Pos2::new(canvas_rect.left() + local_x, hdr_y),
                        Vec2::new(col.width, header_height),
                    );
                    render_header_cell_at(
                        vp_ui,
                        c_rect,
                        col,
                        &colunas_info[col.logical_index],
                        false,
                        total_filtrados,
                        &app.loc,
                        &mut app.spreadsheet_grid.selection,
                        &app.spreadsheet_grid.selection_config,
                        &mut interactions,
                        &mut app.spreadsheet_grid.drag_mode,
                    );
                }
            }

            // Virtualização O(visíveis) das linhas não fixadas
            let data_offset_y = (scroll_y - scroll_top_hdr_h).max(0.0);
            let first_slot = ((data_offset_y / row_height).floor() as usize).saturating_sub(2);
            let vis_count = ((vp_height / row_height).ceil() as usize) + 5;
            let last_slot = (first_slot + vis_count).min(scrollable_row_count);

            for slot in first_slot..last_slot {
                let row_idx = map_scroll_slot_to_row_idx(slot, &excluded_scroll_rows);
                let Some(&ev_idx) = app.indices_filtrados.get(row_idx) else {
                    continue;
                };
                let Some(ev) = app.eventos.get(ev_idx) else {
                    continue;
                };

                let row_y = canvas_rect.top() + scroll_top_hdr_h + (slot as f32 * row_height);

                if !row_num_fixed && scroll_x <= row_header_width + 32.0 {
                    let rh_rect = Rect::from_min_size(
                        Pos2::new(canvas_rect.left(), row_y),
                        Vec2::new(row_header_width, row_height),
                    );
                    render_row_header_at(
                        vp_ui,
                        rh_rect,
                        row_idx,
                        false,
                        total_cols,
                        &app.loc,
                        &mut app.spreadsheet_grid.selection,
                        &app.spreadsheet_grid.selection_config,
                        &mut app.linha_selecionada,
                        &mut interactions,
                        &mut app.spreadsheet_grid.drag_mode,
                    );
                }

                for (s_idx, col) in colunas_headers[fixed_col_count..].iter().enumerate() {
                    let local_x = scroll_rh_w + scroll_col_offsets[s_idx];
                    if local_x + col.width < scroll_x - 32.0
                        || local_x > scroll_x + vp_width + 32.0
                    {
                        continue;
                    }
                    let cell_rect = Rect::from_min_size(
                        Pos2::new(canvas_rect.left() + local_x, row_y),
                        Vec2::new(col.width, row_height),
                    );
                    render_data_cell_at(
                        vp_ui,
                        cell_rect,
                        row_idx,
                        false,
                        col,
                        false,
                        ev,
                        &colunas_headers,
                        &app.loc,
                        &app.gerenciador_tabela,
                        &mut app.spreadsheet_grid.selection,
                        &app.spreadsheet_grid.selection_config,
                        &mut app.linha_selecionada,
                        &mut interactions,
                        &mut app.spreadsheet_grid.drag_mode,
                    );
                }
            }
        });

        let cur_offset = br_out.state.clamped_offset();
        scroll_x = cur_offset.x;
        scroll_y = cur_offset.y;
        vp_width = br_out.inner_rect.width();
        vp_height = br_out.inner_rect.height();
        app.spreadsheet_grid.scroll_offset = cur_offset;
    }

    // -------------------------------------------------------------------------
    // QUADRANTE 2: Superior Direito (TR) — Linhas Fixas × Colunas Roláveis
    // -------------------------------------------------------------------------
    if fixed_top_pane_h > 0.5 && tr_rect.width() > 1.0 {
        let tr_clip = Rect::from_min_max(
            tr_rect.min,
            Pos2::new((split_x + vp_width).min(tr_rect.max.x), split_y),
        );
        let mut tr_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(tr_clip)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        tr_ui.set_clip_rect(tr_clip);

        let origin_x = split_x - scroll_x;
        let mut cur_y = outer_rect.top();

        if col_header_fixed {
            if !row_num_fixed && scroll_x <= row_header_width + 32.0 {
                let corner_r = Rect::from_min_size(
                    Pos2::new(origin_x, cur_y),
                    Vec2::new(row_header_width, header_height),
                );
                let btn = SelectAllButton::with_color(
                    app.spreadsheet_grid.selection.is_all_selected,
                    Some(app.spreadsheet_grid.selection_config.cor_selecao_color32()),
                );
                if btn.show_at_rect(&mut tr_ui, corner_r).clicked() {
                    interactions.select_all_clicked = true;
                }
            }

            for (s_idx, col) in colunas_headers[fixed_col_count..].iter().enumerate() {
                let local_x = scroll_rh_w + scroll_col_offsets[s_idx];
                if local_x + col.width < scroll_x - 32.0 || local_x > scroll_x + vp_width + 32.0 {
                    continue;
                }
                let c_rect = Rect::from_min_size(
                    Pos2::new(origin_x + local_x, cur_y),
                    Vec2::new(col.width, header_height),
                );
                render_header_cell_at(
                    &mut tr_ui,
                    c_rect,
                    col,
                    &colunas_info[col.logical_index],
                    false,
                    total_filtrados,
                    &app.loc,
                    &mut app.spreadsheet_grid.selection,
                    &app.spreadsheet_grid.selection_config,
                    &mut interactions,
                    &mut app.spreadsheet_grid.drag_mode,
                );
            }
            cur_y += header_height;
        }

        for &row_idx in visible_pinned_rows {
            if let Some(&ev_idx) = app.indices_filtrados.get(row_idx) {
                if let Some(ev) = app.eventos.get(ev_idx) {
                    if !row_num_fixed && scroll_x <= row_header_width + 32.0 {
                        let rh_rect = Rect::from_min_size(
                            Pos2::new(origin_x, cur_y),
                            Vec2::new(row_header_width, row_height),
                        );
                        render_row_header_at(
                            &mut tr_ui,
                            rh_rect,
                            row_idx,
                            true,
                            total_cols,
                            &app.loc,
                            &mut app.spreadsheet_grid.selection,
                            &app.spreadsheet_grid.selection_config,
                            &mut app.linha_selecionada,
                            &mut interactions,
                            &mut app.spreadsheet_grid.drag_mode,
                        );
                    }

                    for (s_idx, col) in colunas_headers[fixed_col_count..].iter().enumerate() {
                        let local_x = scroll_rh_w + scroll_col_offsets[s_idx];
                        if local_x + col.width < scroll_x - 32.0
                            || local_x > scroll_x + vp_width + 32.0
                        {
                            continue;
                        }
                        let cell_rect = Rect::from_min_size(
                            Pos2::new(origin_x + local_x, cur_y),
                            Vec2::new(col.width, row_height),
                        );
                        render_data_cell_at(
                            &mut tr_ui,
                            cell_rect,
                            row_idx,
                            true,
                            col,
                            false,
                            ev,
                            &colunas_headers,
                            &app.loc,
                            &app.gerenciador_tabela,
                            &mut app.spreadsheet_grid.selection,
                            &app.spreadsheet_grid.selection_config,
                            &mut app.linha_selecionada,
                            &mut interactions,
                            &mut app.spreadsheet_grid.drag_mode,
                        );
                    }
                }
            }
            cur_y += row_height;
        }
    }

    // -------------------------------------------------------------------------
    // QUADRANTE 3: Inferior Esquerdo (BL) — Linhas Roláveis × Colunas Fixas
    // -------------------------------------------------------------------------
    if fixed_left_pane_h_or_w_active(fixed_left_pane_w, bl_rect.height()) {
        let bl_clip = Rect::from_min_max(
            bl_rect.min,
            Pos2::new(split_x, (split_y + vp_height).min(bl_rect.max.y)),
        );
        let mut bl_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bl_clip)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        bl_ui.set_clip_rect(bl_clip);

        let origin_y = split_y - scroll_y;

        if !col_header_fixed && scroll_y <= header_height + row_height {
            if row_num_fixed {
                let corner_r = Rect::from_min_size(
                    Pos2::new(outer_rect.left(), origin_y),
                    Vec2::new(row_header_width, header_height),
                );
                let btn = SelectAllButton::with_color(
                    app.spreadsheet_grid.selection.is_all_selected,
                    Some(app.spreadsheet_grid.selection_config.cor_selecao_color32()),
                );
                if btn.show_at_rect(&mut bl_ui, corner_r).clicked() {
                    interactions.select_all_clicked = true;
                }
            }

            for (f_idx, col) in colunas_headers[..fixed_col_count].iter().enumerate() {
                let c_rect = Rect::from_min_size(
                    Pos2::new(outer_rect.left() + fixed_rh_w + fixed_col_offsets[f_idx], origin_y),
                    Vec2::new(col.width, header_height),
                );
                render_header_cell_at(
                    &mut bl_ui,
                    c_rect,
                    col,
                    &colunas_info[col.logical_index],
                    true,
                    total_filtrados,
                    &app.loc,
                    &mut app.spreadsheet_grid.selection,
                    &app.spreadsheet_grid.selection_config,
                    &mut interactions,
                    &mut app.spreadsheet_grid.drag_mode,
                );
            }
        }

        let data_offset_y = (scroll_y - scroll_top_hdr_h).max(0.0);
        let first_slot = ((data_offset_y / row_height).floor() as usize).saturating_sub(2);
        let vis_count = ((vp_height / row_height).ceil() as usize) + 5;
        let last_slot = (first_slot + vis_count).min(scrollable_row_count);

        for slot in first_slot..last_slot {
            let row_idx = map_scroll_slot_to_row_idx(slot, &excluded_scroll_rows);
            let Some(&ev_idx) = app.indices_filtrados.get(row_idx) else {
                continue;
            };
            let Some(ev) = app.eventos.get(ev_idx) else {
                continue;
            };

            let row_y = origin_y + scroll_top_hdr_h + (slot as f32 * row_height);

            if row_num_fixed {
                let rh_rect = Rect::from_min_size(
                    Pos2::new(outer_rect.left(), row_y),
                    Vec2::new(row_header_width, row_height),
                );
                render_row_header_at(
                    &mut bl_ui,
                    rh_rect,
                    row_idx,
                    false,
                    total_cols,
                    &app.loc,
                    &mut app.spreadsheet_grid.selection,
                    &app.spreadsheet_grid.selection_config,
                    &mut app.linha_selecionada,
                    &mut interactions,
                    &mut app.spreadsheet_grid.drag_mode,
                );
            }

            for (f_idx, col) in colunas_headers[..fixed_col_count].iter().enumerate() {
                let cell_rect = Rect::from_min_size(
                    Pos2::new(outer_rect.left() + fixed_rh_w + fixed_col_offsets[f_idx], row_y),
                    Vec2::new(col.width, row_height),
                );
                render_data_cell_at(
                    &mut bl_ui,
                    cell_rect,
                    row_idx,
                    false,
                    col,
                    true,
                    ev,
                    &colunas_headers,
                    &app.loc,
                    &app.gerenciador_tabela,
                    &mut app.spreadsheet_grid.selection,
                    &app.spreadsheet_grid.selection_config,
                    &mut app.linha_selecionada,
                    &mut interactions,
                    &mut app.spreadsheet_grid.drag_mode,
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // QUADRANTE 4: Superior Esquerdo (TL) — Canto Duplamente Fixo
    // -------------------------------------------------------------------------
    if fixed_top_pane_h > 0.5 && fixed_left_pane_w > 0.5 {
        let mut tl_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(tl_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        tl_ui.set_clip_rect(tl_rect);

        let mut cur_y = outer_rect.top();
        if col_header_fixed {
            if row_num_fixed {
                let corner_r = Rect::from_min_size(
                    Pos2::new(outer_rect.left(), cur_y),
                    Vec2::new(row_header_width, header_height),
                );
                let btn = SelectAllButton::with_color(
                    app.spreadsheet_grid.selection.is_all_selected,
                    Some(app.spreadsheet_grid.selection_config.cor_selecao_color32()),
                );
                if btn.show_at_rect(&mut tl_ui, corner_r).clicked() {
                    interactions.select_all_clicked = true;
                }
            }

            for (f_idx, col) in colunas_headers[..fixed_col_count].iter().enumerate() {
                let c_rect = Rect::from_min_size(
                    Pos2::new(outer_rect.left() + fixed_rh_w + fixed_col_offsets[f_idx], cur_y),
                    Vec2::new(col.width, header_height),
                );
                render_header_cell_at(
                    &mut tl_ui,
                    c_rect,
                    col,
                    &colunas_info[col.logical_index],
                    true,
                    total_filtrados,
                    &app.loc,
                    &mut app.spreadsheet_grid.selection,
                    &app.spreadsheet_grid.selection_config,
                    &mut interactions,
                    &mut app.spreadsheet_grid.drag_mode,
                );
            }
            cur_y += header_height;
        }

        for &row_idx in visible_pinned_rows {
            if let Some(&ev_idx) = app.indices_filtrados.get(row_idx) {
                if let Some(ev) = app.eventos.get(ev_idx) {
                    if row_num_fixed {
                        let rh_rect = Rect::from_min_size(
                            Pos2::new(outer_rect.left(), cur_y),
                            Vec2::new(row_header_width, row_height),
                        );
                        render_row_header_at(
                            &mut tl_ui,
                            rh_rect,
                            row_idx,
                            true,
                            total_cols,
                            &app.loc,
                            &mut app.spreadsheet_grid.selection,
                            &app.spreadsheet_grid.selection_config,
                            &mut app.linha_selecionada,
                            &mut interactions,
                            &mut app.spreadsheet_grid.drag_mode,
                        );
                    }

                    for (f_idx, col) in colunas_headers[..fixed_col_count].iter().enumerate() {
                        let cell_rect = Rect::from_min_size(
                            Pos2::new(
                                outer_rect.left() + fixed_rh_w + fixed_col_offsets[f_idx],
                                cur_y,
                            ),
                            Vec2::new(col.width, row_height),
                        );
                        render_data_cell_at(
                            &mut tl_ui,
                            cell_rect,
                            row_idx,
                            true,
                            col,
                            true,
                            ev,
                            &colunas_headers,
                            &app.loc,
                            &app.gerenciador_tabela,
                            &mut app.spreadsheet_grid.selection,
                            &app.spreadsheet_grid.selection_config,
                            &mut app.linha_selecionada,
                            &mut interactions,
                            &mut app.spreadsheet_grid.drag_mode,
                        );
                    }
                }
            }
            cur_y += row_height;
        }
    }

    // -------------------------------------------------------------------------
    // Linhas divisórias visuais dos painéis congelados (estilo Excel / Sheets)
    // -------------------------------------------------------------------------
    let dark_mode = ui.visuals().dark_mode;
    let accent_divider = if dark_mode {
        Color32::from_rgb(76, 146, 232)
    } else {
        Color32::from_rgb(36, 108, 198)
    };
    let subtle_divider = if dark_mode {
        Color32::from_rgb(68, 74, 90)
    } else {
        Color32::from_rgb(185, 192, 204)
    };

    if fixed_left_pane_w > 0.5 && (fixed_col_count > 0 || scroll_x > 0.5) {
        let stroke = if fixed_col_count > 0 {
            Stroke::new(1.8, accent_divider)
        } else {
            Stroke::new(1.2, subtle_divider)
        };
        let y_end = (split_y + vp_height).min(outer_rect.bottom());
        ui.painter().line_segment(
            [Pos2::new(split_x, outer_rect.top()), Pos2::new(split_x, y_end)],
            stroke,
        );
    }

    if fixed_top_pane_h > 0.5 && (!visible_pinned_rows.is_empty() || scroll_y > 0.5) {
        let stroke = if !visible_pinned_rows.is_empty() {
            Stroke::new(1.8, accent_divider)
        } else {
            Stroke::new(1.2, subtle_divider)
        };
        let x_end = (split_x + vp_width).min(outer_rect.right());
        ui.painter().line_segment(
            [Pos2::new(outer_rect.left(), split_y), Pos2::new(x_end, split_y)],
            stroke,
        );
    }

    // -------------------------------------------------------------------------
    // Aplicação das interações coletadas no frame
    // -------------------------------------------------------------------------
    if interactions.select_all_clicked {
        if app.spreadsheet_grid.selection.is_all_selected {
            app.spreadsheet_grid.selection.clear();
            app.linha_selecionada = None;
        } else {
            app.spreadsheet_grid.selection.select_all();
        }
    }

    for (k, nova_largura) in interactions.redimensionamentos {
        app.gerenciador_colunas.set_largura(k, nova_largura);
    }

    if let Some(key) = interactions.autofit_solicitado {
        app.gerenciador_colunas
            .autofit_coluna_ao_cabecalho(&key, &app.loc);
    }

    if let Some(act) = interactions.acao_contexto {
        match act {
            GridContextAction::OpenFile(path) => {
                abrir_arquivo_com_associado(&path);
            }
            GridContextAction::ShowInExplorer(path) => {
                selecionar_no_explorer(&path);
            }
            GridContextAction::CopyText(txt) => {
                ui.ctx().copy_text(txt);
            }
            GridContextAction::CopyTsv(tsv) => {
                ui.ctx().copy_text(tsv);
            }
            GridContextAction::CopyHash(h) => {
                ui.ctx().copy_text(h);
            }
            GridContextAction::Quarantine(path) => {
                let motivo = app.loc.t("manual_quarantine_reason");
                if let Some(qitem) = app
                    .security_manager
                    .quarentena()
                    .isolar_arquivo(std::path::Path::new(&path), &motivo)
                {
                    if let Some(r) = app.linha_selecionada {
                        if let Some(&ev_idx) = app.indices_filtrados.get(r) {
                            if let Some(ev) = app.eventos.get_mut(ev_idx) {
                                ev.quarantine_status = "captured".to_string();
                                ev.evidence_id = qitem.evidence_id;
                            }
                        }
                    }
                }
            }
            GridContextAction::FilterOperation(op) => {
                app.filtros.op_adicionado = op == OperationType::Adicionado;
                app.filtros.op_modificado = op == OperationType::Modificado;
                app.filtros.op_excluido = op == OperationType::Excluido;
                app.filtros.op_renomeado = op == OperationType::Renomeado;
                app.filtros.op_movido = op == OperationType::Movido;
                app.filtros.op_escaneado = op == OperationType::Escaneado;
                app.filtros_sujos = true;
            }
            GridContextAction::FilterExtension(ext) => {
                app.filtros.filtro_extensao = ext;
                app.filtros_sujos = true;
            }
            GridContextAction::AutoFitColumn(key) => {
                app.gerenciador_colunas
                    .autofit_coluna_ao_cabecalho(&key, &app.loc);
            }
            GridContextAction::AutoFitAllColumns => {
                app.gerenciador_colunas
                    .autofit_todas_colunas_ao_cabecalho(&app.loc);
            }
            GridContextAction::HideColumn(key) => {
                app.spreadsheet_grid.manager.hide_col(&key);
                app.gerenciador_colunas.set_visibilidade(&key, false);
                app.filtros_sujos = true;
            }
            GridContextAction::UnhideAllColumns => {
                app.spreadsheet_grid.manager.unhide_all_cols();
                for c in app.gerenciador_colunas.colunas().to_vec() {
                    app.gerenciador_colunas.set_visibilidade(c.key, true);
                }
                app.filtros_sujos = true;
            }
            GridContextAction::HideSelectedRows => {
                if let Some(r) = app.linha_selecionada {
                    app.spreadsheet_grid.manager.hide_row(r);
                }
            }
            GridContextAction::UnhideAllRows => {
                app.spreadsheet_grid.manager.unhide_all_rows();
            }
            GridContextAction::DeleteSelectedRows => {
                if let Some(r) = app.linha_selecionada {
                    if r < app.indices_filtrados.len() {
                        let ev_idx = app.indices_filtrados[r];
                        if ev_idx < app.eventos.len() {
                            app.eventos.remove(ev_idx);
                            app.spreadsheet_grid.selection.on_row_deleted(r);
                            app.spreadsheet_grid.layout.on_row_deleted(r);
                            app.linha_selecionada = None;
                            app.filtros_sujos = true;
                        }
                    }
                }
            }
            GridContextAction::DuplicateSelectedRows => {
                if let Some(r) = app.linha_selecionada {
                    if r < app.indices_filtrados.len() {
                        let ev_idx = app.indices_filtrados[r];
                        if let Some(ev) = app.eventos.get(ev_idx).cloned() {
                            let mut clone = ev;
                            clone.id = None;
                            let saved = app.base_evento.registrar_evento(clone);
                            app.eventos.insert(ev_idx + 1, saved);
                            app.spreadsheet_grid.selection.on_row_inserted(r + 1);
                            app.spreadsheet_grid.layout.on_row_inserted(r + 1);
                            app.filtros_sujos = true;
                        }
                    }
                }
            }
            GridContextAction::InsertRowAbove | GridContextAction::InsertRowBelow => {
                let target_row = app.linha_selecionada.unwrap_or(0);
                let insert_idx = if act == GridContextAction::InsertRowAbove {
                    target_row
                } else {
                    target_row + 1
                };
                let placeholder = crate::core::event::EventModel::new(
                    OperationType::Escaneado,
                    "registro_manual.log",
                    "C:\\Manual",
                );
                let saved = app.base_evento.registrar_evento(placeholder);
                if insert_idx <= app.eventos.len() {
                    app.eventos.insert(insert_idx, saved);
                } else {
                    app.eventos.push(saved);
                }
                app.spreadsheet_grid.selection.on_row_inserted(insert_idx);
                app.spreadsheet_grid.layout.on_row_inserted(insert_idx);
                app.linha_selecionada = Some(insert_idx);
                app.filtros_sujos = true;
            }
            GridContextAction::ClearSelectedContent => {
                if let Some(r) = app.linha_selecionada {
                    if r < app.indices_filtrados.len() {
                        let ev_idx = app.indices_filtrados[r];
                        if let Some(ev) = app.eventos.get_mut(ev_idx) {
                            ev.alerta_resumo.clear();
                            ev.diff_resumo.clear();
                        }
                    }
                }
            }
            GridContextAction::SortColumn { key, ascending } => {
                app.gerenciador_tabela.coluna_ordenacao = Some(key);
                app.gerenciador_tabela.ordem_ascendente = ascending;
                app.filtros_sujos = true;
            }
            GridContextAction::ToggleFreezeColumn(count) => {
                app.spreadsheet_grid.layout.toggle_freeze_cols(count);
                app.spreadsheet_grid
                    .layout
                    .salvar_configuracao_painel_fixo();
            }
            GridContextAction::ToggleFreezeRow(count) => {
                app.spreadsheet_grid.layout.toggle_freeze_rows(count);
                app.spreadsheet_grid
                    .layout
                    .salvar_configuracao_painel_fixo();
            }
            GridContextAction::TogglePinColumn(col_key) => {
                if let Some((orig_idx, _)) =
                    colunas_base.iter().enumerate().find(|(_, c)| c.key == col_key)
                {
                    let currently_pinned = app
                        .spreadsheet_grid
                        .layout
                        .is_col_effectively_pinned(orig_idx, &col_key);
                    if currently_pinned && orig_idx < app.spreadsheet_grid.layout.frozen_cols {
                        for (idx, c_info) in colunas_base
                            .iter()
                            .enumerate()
                            .take(app.spreadsheet_grid.layout.frozen_cols)
                        {
                            if idx != orig_idx {
                                app.spreadsheet_grid
                                    .layout
                                    .set_col_pinned(c_info.key, true);
                            }
                        }
                        app.spreadsheet_grid.layout.frozen_cols = 0;
                    }
                    app.spreadsheet_grid
                        .layout
                        .set_col_pinned(&col_key, !currently_pinned);
                    app.spreadsheet_grid
                        .layout
                        .salvar_configuracao_painel_fixo();
                }
            }
            GridContextAction::TogglePinRow(row_idx) => {
                app.spreadsheet_grid.layout.toggle_pin_row(row_idx);
                app.spreadsheet_grid
                    .layout
                    .salvar_configuracao_painel_fixo();
            }
            GridContextAction::SelectAll => {
                app.spreadsheet_grid.selection.select_all();
            }
        }
    }
}

#[inline]
fn fixed_left_pane_h_or_w_active(fixed_left_w: f32, bl_h: f32) -> bool {
    fixed_left_w > 0.5 && bl_h > 1.0
}
