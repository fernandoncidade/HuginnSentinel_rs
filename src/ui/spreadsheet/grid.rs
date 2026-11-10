// Módulo Principal da Planilha Eletrônica (SpreadsheetGrid)
// Orquestra apresentação, seleção, dimensões, navegação, menus de contexto e renderização virtualizada.

use egui::{Color32, FontId, Ui};

use super::cell_selection::{CellSelectionModel, SelectionBehaviorConfig};
use super::col_header::ColumnHeader;
use super::grid_layout::GridLayout;
use super::row_col_manager::RowColumnManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridDragMode {
    #[default]
    None,
    Cells { anchor_row: usize, anchor_col: usize },
    Rows { anchor_row: usize },
    Cols { anchor_col: usize },
}

pub struct SpreadsheetGrid {
    pub selection: CellSelectionModel,
    pub selection_config: SelectionBehaviorConfig,
    pub layout: GridLayout,
    pub manager: RowColumnManager,
    pub row_pin_input: String,
    pub scroll_offset: egui::Vec2,
    pub drag_mode: GridDragMode,
    pub request_scroll_to_selection: bool,
}

impl Default for SpreadsheetGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl SpreadsheetGrid {
    pub fn new() -> Self {
        Self {
            selection: CellSelectionModel::new(),
            selection_config: SelectionBehaviorConfig::default(),
            layout: GridLayout::new(),
            manager: RowColumnManager::new(),
            row_pin_input: String::new(),
            scroll_offset: egui::Vec2::ZERO,
            drag_mode: GridDragMode::None,
            request_scroll_to_selection: false,
        }
    }

    /// Instancia a planilha restaurando as configurações salvas de linhas e colunas fixas e de seleção
    pub fn new_persistent() -> Self {
        let mut grid = Self::new();
        grid.layout.carregar_configuracao_painel_fixo();
        grid.carregar_configuracao_selecao();
        grid
    }

    /// Salva a configuração de comportamento de seleção em `%LOCALAPPDATA%\HuginnSentinel`
    pub fn salvar_configuracao_selecao(&self) {
        let p = crate::utils::paths::obter_caminho_persistente()
            .join("selection_behavior_config.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.selection_config) {
            let _ = std::fs::write(p, json);
        }
    }

    /// Carrega a configuração de comportamento de seleção se existir
    pub fn carregar_configuracao_selecao(&mut self) {
        let p = crate::utils::paths::obter_caminho_persistente()
            .join("selection_behavior_config.json");
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(cfg) = serde_json::from_slice::<SelectionBehaviorConfig>(&bytes) {
                self.selection_config = cfg;
            }
        }
    }

    /// Executa o cálculo dinâmico de AutoFit para a largura da coluna medindo o texto real no egui
    pub fn autofit_coluna(
        &mut self,
        ui: &Ui,
        col_key: &str,
        col_title: &str,
        sample_cell_values: &[String],
    ) {
        let font_id = FontId::proportional(12.0);
        let header_font = FontId::proportional(11.5);

        // Mede largura do cabeçalho
        let header_text = format!("{} ▲", col_title);
        let header_galley = ui
            .painter()
            .layout_no_wrap(header_text, header_font, Color32::WHITE);
        let mut max_width = header_galley.size().x + 28.0;

        // Mede amostra de até 50 células
        for val in sample_cell_values.iter().take(50) {
            let cell_galley =
                ui.painter()
                    .layout_no_wrap(val.clone(), font_id.clone(), Color32::WHITE);
            let w = cell_galley.size().x + 20.0;
            if w > max_width {
                max_width = w;
            }
        }

        self.layout.set_col_width(col_key, max_width);
    }

    /// Executa AutoFit em todas as colunas visíveis
    pub fn autofit_todas_colunas<F>(
        &mut self,
        ui: &Ui,
        columns: &[ColumnHeader],
        mut get_cell_sample: F,
    ) where
        F: FnMut(&str) -> Vec<String>,
    {
        for col in columns {
            if col.visible && !self.manager.is_col_hidden(&col.key) {
                let samples = get_cell_sample(&col.key);
                self.autofit_coluna(ui, &col.key, &col.title, &samples);
            }
        }
    }
}
