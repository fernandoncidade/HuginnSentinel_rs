// Módulo de Cabeçalho de Linha (RowHeader) — Padrão Excel
// Exibe numeração sequencial (1, 2, 3...), suporta seleção de linha inteira e redimensionamento de altura.

#[derive(Debug, Clone, PartialEq)]
pub struct RowHeader {
    pub logical_index: usize,
    pub display_number: usize, // 1-based (Excel)
    pub height: f32,
    pub visible: bool,
    pub is_selected: bool,
    pub is_active: bool,
}

impl RowHeader {
    pub fn new(logical_index: usize, height: f32) -> Self {
        Self {
            logical_index,
            display_number: logical_index + 1,
            height: height.max(18.0),
            visible: true,
            is_selected: false,
            is_active: false,
        }
    }
}
