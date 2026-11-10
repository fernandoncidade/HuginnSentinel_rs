#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoGrafico {
    PizzaOperacoes,
    BarrasTipos,
    TimelineOperacoes,
    TreemapTipos,
    HistogramaHorarios,
    ParetoExtensoes,
    LinhaEventosHora,
    BoxplotTamanhos,
    RadarOperacoes,
    HeatmapDiaHora,
    ScatterTamanhoHora,
    SankeyOperacaoTipo,
    RadarMultidimensional,
    DotplotDistribuicao,
    SankeyEventoDiretorio,
    SankeyTipoDiretorio,
    ArvoreDiretorios,
}

impl TipoGrafico {
    pub fn all() -> &'static [TipoGrafico] {
        &[
            TipoGrafico::PizzaOperacoes,
            TipoGrafico::BarrasTipos,
            TipoGrafico::TimelineOperacoes,
            TipoGrafico::TreemapTipos,
            TipoGrafico::HistogramaHorarios,
            TipoGrafico::ParetoExtensoes,
            TipoGrafico::LinhaEventosHora,
            TipoGrafico::BoxplotTamanhos,
            TipoGrafico::RadarOperacoes,
            TipoGrafico::HeatmapDiaHora,
            TipoGrafico::ScatterTamanhoHora,
            TipoGrafico::SankeyOperacaoTipo,
            TipoGrafico::RadarMultidimensional,
            TipoGrafico::DotplotDistribuicao,
            TipoGrafico::SankeyEventoDiretorio,
            TipoGrafico::SankeyTipoDiretorio,
            TipoGrafico::ArvoreDiretorios,
        ]
    }

    pub fn titulo(&self, loc: &crate::locale::Localizer) -> String {
        let key = match self {
            TipoGrafico::PizzaOperacoes => "chart_pizza_operacoes_title",
            TipoGrafico::BarrasTipos => "chart_barras_tipos_title",
            TipoGrafico::TimelineOperacoes => "chart_timeline_operacoes_title",
            TipoGrafico::TreemapTipos => "chart_treemap_tipos_title",
            TipoGrafico::HistogramaHorarios => "chart_histograma_horarios_title",
            TipoGrafico::ParetoExtensoes => "chart_pareto_extensoes_title",
            TipoGrafico::LinhaEventosHora => "chart_linha_eventos_hora_title",
            TipoGrafico::BoxplotTamanhos => "chart_boxplot_tamanhos_title",
            TipoGrafico::RadarOperacoes => "chart_radar_operacoes_title",
            TipoGrafico::HeatmapDiaHora => "chart_heatmap_dia_hora_title",
            TipoGrafico::ScatterTamanhoHora => "chart_scatter_tamanho_hora_title",
            TipoGrafico::SankeyOperacaoTipo => "chart_sankey_operacao_tipo_title",
            TipoGrafico::RadarMultidimensional => "chart_radar_multidimensional_title",
            TipoGrafico::DotplotDistribuicao => "chart_dotplot_distribuicao_title",
            TipoGrafico::SankeyEventoDiretorio => "chart_sankey_evento_diretorio_title",
            TipoGrafico::SankeyTipoDiretorio => "chart_sankey_tipo_diretorio_title",
            TipoGrafico::ArvoreDiretorios => "chart_arvore_diretorios_title",
        };
        loc.t(key)
    }

    pub fn descricao(&self, loc: &crate::locale::Localizer) -> String {
        let key = match self {
            TipoGrafico::PizzaOperacoes => "chart_pizza_operacoes_desc",
            TipoGrafico::BarrasTipos => "chart_barras_tipos_desc",
            TipoGrafico::TimelineOperacoes => "chart_timeline_operacoes_desc",
            TipoGrafico::TreemapTipos => "chart_treemap_tipos_desc",
            TipoGrafico::HistogramaHorarios => "chart_histograma_horarios_desc",
            TipoGrafico::ParetoExtensoes => "chart_pareto_extensoes_desc",
            TipoGrafico::LinhaEventosHora => "chart_linha_eventos_hora_desc",
            TipoGrafico::BoxplotTamanhos => "chart_boxplot_tamanhos_desc",
            TipoGrafico::RadarOperacoes => "chart_radar_operacoes_desc",
            TipoGrafico::HeatmapDiaHora => "chart_heatmap_dia_hora_desc",
            TipoGrafico::ScatterTamanhoHora => "chart_scatter_tamanho_hora_desc",
            TipoGrafico::SankeyOperacaoTipo => "chart_sankey_operacao_tipo_desc",
            TipoGrafico::RadarMultidimensional => "chart_radar_multidimensional_desc",
            TipoGrafico::DotplotDistribuicao => "chart_dotplot_distribuicao_desc",
            TipoGrafico::SankeyEventoDiretorio => "chart_sankey_evento_diretorio_desc",
            TipoGrafico::SankeyTipoDiretorio => "chart_sankey_tipo_diretorio_desc",
            TipoGrafico::ArvoreDiretorios => "chart_arvore_diretorios_desc",
        };
        loc.t(key)
    }

    pub fn titulo_pt(&self) -> &'static str {
        match self {
            TipoGrafico::PizzaOperacoes => "Pizza de Operações",
            TipoGrafico::BarrasTipos => "Barras por Tipo de Arquivo",
            TipoGrafico::TimelineOperacoes => "Linha do Tempo de Operações",
            TipoGrafico::TreemapTipos => "Treemap por Tipos",
            TipoGrafico::HistogramaHorarios => "Histograma por Horários",
            TipoGrafico::ParetoExtensoes => "Gráfico de Pareto de Extensões",
            TipoGrafico::LinhaEventosHora => "Evolução de Eventos por Hora",
            TipoGrafico::BoxplotTamanhos => "Boxplot de Tamanhos de Arquivo",
            TipoGrafico::RadarOperacoes => "Radar de Operações",
            TipoGrafico::HeatmapDiaHora => "Heatmap Dia da Semana x Hora",
            TipoGrafico::ScatterTamanhoHora => "Dispersão Tamanho x Hora",
            TipoGrafico::SankeyOperacaoTipo => "Sankey: Operação → Tipo de Arquivo",
            TipoGrafico::RadarMultidimensional => "Radar Multidimensional de Risco",
            TipoGrafico::DotplotDistribuicao => "Dotplot de Distribuição de Eventos",
            TipoGrafico::SankeyEventoDiretorio => "Sankey: Operação → Diretório",
            TipoGrafico::SankeyTipoDiretorio => "Sankey: Tipo de Arquivo → Diretório",
            TipoGrafico::ArvoreDiretorios => "Árvore Hierárquica de Diretórios",
        }
    }

    pub fn descricao_pt(&self) -> &'static str {
        match self {
            TipoGrafico::PizzaOperacoes => {
                "Distribuição percentual das operações monitoradas (adicionado, modificado, etc.)."
            }
            TipoGrafico::BarrasTipos => {
                "Quantidade total por tipo de extensão, de todas as extensões de arquivos monitorados."
            }
            TipoGrafico::TimelineOperacoes => {
                "Frequência de ocorrência de eventos ao longo do tempo cronológico."
            }
            TipoGrafico::TreemapTipos => {
                "Visualização em mosaico proporcional por volume de tipos de arquivo."
            }
            TipoGrafico::HistogramaHorarios => {
                "Concentração horária de alterações no sistema de arquivos de 0h a 23h."
            }
            TipoGrafico::ParetoExtensoes => {
                "Quantidade total por tipo de extensão, de todas as extensões monitoradas com base na técnica de Pareto."
            }
            TipoGrafico::LinhaEventosHora => {
                "Tendência temporal contínua da intensidade de escrita e leitura."
            }
            TipoGrafico::BoxplotTamanhos => {
                "Quantidade total por tipo de extensão em relação à soma total do tamanho dos arquivos daquela extensão."
            }
            TipoGrafico::RadarOperacoes => "Mapeamento radial da intensidade por tipo de operação.",
            TipoGrafico::HeatmapDiaHora => {
                "Matriz de calor da atividade semanal por dia e horário."
            }
            TipoGrafico::ScatterTamanhoHora => {
                "Correlação bidimensional entre hora de alteração e tamanho do arquivo."
            }
            TipoGrafico::SankeyOperacaoTipo => {
                "Fluxo volumétrico ligando cada operação às categorias de arquivos envolvidas."
            }
            TipoGrafico::RadarMultidimensional => {
                "Comparativo radial de atributos de segurança e volume."
            }
            TipoGrafico::DotplotDistribuicao => {
                "Dispersão pontual de densidade de arquivos por escopo."
            }
            TipoGrafico::SankeyEventoDiretorio => {
                "Fluxo conectando tipos de operação aos diretórios afetados."
            }
            TipoGrafico::SankeyTipoDiretorio => {
                "Fluxo ligando categorias de arquivo aos diretórios de destino."
            }
            TipoGrafico::ArvoreDiretorios => {
                "Hierarquia estrutural das pastas monitoradas e contagem de itens."
            }
        }
    }
}
