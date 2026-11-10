// Módulo de Menus Contextuais da Planilha (ContextMenuController)
// Implementa menus de contexto acionados pelo botão direito para Linhas, Colunas e Células/Intervalos.
// As opções não fecham o menu ao clicar: o menu permanece aberto enquanto o ponteiro do mouse
// estiver sobre a região do menu e fecha automaticamente ao retirar o ponteiro de cima dele.

use crate::core::types::OperationType;
use crate::locale::Localizer;
use egui::{RichText, Ui};

#[derive(Debug, Clone, PartialEq)]
pub enum GridContextAction {
    OpenFile(String),
    ShowInExplorer(String),
    CopyText(String),
    CopyTsv(String),
    CopyHash(String),
    FilterOperation(OperationType),
    FilterExtension(String),
    Quarantine(String),
    AutoFitColumn(String),
    AutoFitAllColumns,
    HideColumn(String),
    UnhideAllColumns,
    HideSelectedRows,
    UnhideAllRows,
    DeleteSelectedRows,
    DuplicateSelectedRows,
    InsertRowAbove,
    InsertRowBelow,
    ClearSelectedContent,
    SortColumn { key: String, ascending: bool },
    ToggleFreezeColumn(usize),
    ToggleFreezeRow(usize),
    TogglePinColumn(String),
    TogglePinRow(usize),
    SelectAll,
}

pub struct ContextMenuController;

impl ContextMenuController {
    /// Renderiza menu de contexto de cabeçalho de coluna traduzido dinamicamente em tempo de execução
    pub fn show_column_menu_localized(
        ui: &mut Ui,
        loc: &Localizer,
        col_key: &str,
        col_title: &str,
        is_pinned: bool,
    ) -> Option<GridContextAction> {
        let mut action = None;
        ui.set_min_width(240.0);

        ui.label(RichText::new(loc.t_args("ctx_column_header_fmt", &[col_title])).strong());
        ui.separator();

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "▲",
            loc.t("ctx_sort_asc"),
        )
        .clicked()
        {
            action = Some(GridContextAction::SortColumn {
                key: col_key.to_string(),
                ascending: true,
            });
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "▼",
            loc.t("ctx_sort_desc"),
        )
        .clicked()
        {
            action = Some(GridContextAction::SortColumn {
                key: col_key.to_string(),
                ascending: false,
            });
        }

        ui.separator();

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "↔️",
            loc.t("ctx_autofit_col"),
        )
        .clicked()
        {
            action = Some(GridContextAction::AutoFitColumn(col_key.to_string()));
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "↔️",
            loc.t("ctx_autofit_all"),
        )
        .clicked()
        {
            action = Some(GridContextAction::AutoFitAllColumns);
        }

        ui.separator();

        let (pin_emoji, pin_label) = if is_pinned {
            ("🔓", loc.t("ctx_unpin_column"))
        } else {
            ("📌", loc.t("ctx_pin_column"))
        };
        if crate::ui::widgets::render_centered_emoji_menu_button(ui, pin_emoji, pin_label)
            .clicked()
        {
            action = Some(GridContextAction::TogglePinColumn(col_key.to_string()));
        }

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "👁️",
            loc.t("ctx_hide_column"),
        )
        .clicked()
        {
            action = Some(GridContextAction::HideColumn(col_key.to_string()));
        }

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "👁️‍🗨️",
            loc.t("ctx_unhide_all_columns"),
        )
        .clicked()
        {
            action = Some(GridContextAction::UnhideAllColumns);
        }

        action
    }

    /// Renderiza menu de contexto de cabeçalho de coluna (compatibilidade legada)
    pub fn show_column_menu(
        ui: &mut Ui,
        col_key: &str,
        col_title: &str,
        logical_index: usize,
    ) -> Option<GridContextAction> {
        let loc = Localizer::new();
        let _ = logical_index;
        Self::show_column_menu_localized(ui, &loc, col_key, col_title, false)
    }

    /// Renderiza menu de contexto de cabeçalho de linha traduzido dinamicamente em tempo de execução
    pub fn show_row_menu_localized(
        ui: &mut Ui,
        loc: &Localizer,
        row_idx: usize,
        is_pinned: bool,
    ) -> Option<GridContextAction> {
        let mut action = None;
        ui.set_min_width(220.0);

        let row_num_str = (row_idx + 1).to_string();
        ui.label(RichText::new(loc.t_args("row_item_fmt", &[&row_num_str])).strong());
        ui.separator();

        let (pin_emoji, pin_label) = if is_pinned {
            ("🔓", loc.t("ctx_unpin_row"))
        } else {
            ("📌", loc.t("ctx_pin_row"))
        };
        if crate::ui::widgets::render_centered_emoji_menu_button(ui, pin_emoji, pin_label)
            .clicked()
        {
            action = Some(GridContextAction::TogglePinRow(row_idx));
        }

        ui.separator();

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "➕",
            loc.t("ctx_insert_row_above"),
        )
        .clicked()
        {
            action = Some(GridContextAction::InsertRowAbove);
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "➕",
            loc.t("ctx_insert_row_below"),
        )
        .clicked()
        {
            action = Some(GridContextAction::InsertRowBelow);
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "📋",
            loc.t("ctx_duplicate_row"),
        )
        .clicked()
        {
            action = Some(GridContextAction::DuplicateSelectedRows);
        }

        ui.separator();

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "🗑️",
            loc.t("ctx_delete_selected_row"),
        )
        .clicked()
        {
            action = Some(GridContextAction::DeleteSelectedRows);
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "🧹",
            loc.t("ctx_clear_row_content"),
        )
        .clicked()
        {
            action = Some(GridContextAction::ClearSelectedContent);
        }

        ui.separator();

        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "👁️",
            loc.t("ctx_hide_selected_row"),
        )
        .clicked()
        {
            action = Some(GridContextAction::HideSelectedRows);
        }
        if crate::ui::widgets::render_centered_emoji_menu_button(
            ui,
            "👁️‍🗨️",
            loc.t("ctx_unhide_all_rows"),
        )
        .clicked()
        {
            action = Some(GridContextAction::UnhideAllRows);
        }

        action
    }

    /// Renderiza menu de contexto de cabeçalho de linha (compatibilidade legada)
    pub fn show_row_menu(ui: &mut Ui, row_idx: usize) -> Option<GridContextAction> {
        let loc = Localizer::new();
        Self::show_row_menu_localized(ui, &loc, row_idx, false)
    }
}
