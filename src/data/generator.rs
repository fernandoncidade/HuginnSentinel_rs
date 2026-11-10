use chrono::NaiveDateTime;
use egui::Color32;
use std::collections::{BTreeMap, HashMap};

use crate::core::event::EventModel;
use crate::locale::Localizer;

pub const CORES_PALETA_GERADOR: &[Color32] = &[
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

#[derive(Debug, Clone)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub secondary_values: Vec<f64>,
    pub pairs: Vec<(String, f64)>,
    pub matrix_7x24: Vec<Vec<f64>>,
    pub points: Vec<(f64, f64)>,                    // (x, y)
    pub flows: Vec<(String, String, f64)>,          // (origem, destino, valor)
    pub boxplot: Option<(f64, f64, f64, f64, f64)>, // (min, q1, median, q3, max)

    // Estruturas de dados especializadas para renderização robusta
    pub timeline_items: Vec<(String, String, String, f64)>, // (timestamp, operacao, nome, size_kb)
    pub treemap_items: Vec<(String, f64, Color32)>,         // (categoria, valor, cor)
    pub multi_series: Vec<(String, Vec<f64>, Color32)>,     // (nome_serie, valores_por_label, cor)
    pub dotplot_groups: Vec<(String, Vec<f64>)>,            // (grupo, valores)
    pub tree_nodes: Vec<(String, usize, usize, f64)>, // (caminho_dir, nivel_depth, arqs, total_mb)
    pub radar_dimensions: Vec<(String, f64)>,         // (dimensao, score 0..100)
    pub boxplot_groups: Vec<(String, f64, f64, f64, f64, f64)>, // (ext, min, q1, median, q3, max)
    pub boxplot_totais: Vec<(String, usize, f64)>, // (ext, quantidade_total, soma_tamanho_kb) alinhado a boxplot_groups
    pub time_unit_label: String,
    pub max_x: f64,
}

impl Default for ChartData {
    fn default() -> Self {
        Self {
            labels: Vec::new(),
            values: Vec::new(),
            secondary_values: Vec::new(),
            pairs: Vec::new(),
            matrix_7x24: vec![vec![0.0; 24]; 7],
            points: Vec::new(),
            flows: Vec::new(),
            boxplot: None,
            timeline_items: Vec::new(),
            treemap_items: Vec::new(),
            multi_series: Vec::new(),
            dotplot_groups: Vec::new(),
            tree_nodes: Vec::new(),
            radar_dimensions: Vec::new(),
            boxplot_groups: Vec::new(),
            boxplot_totais: Vec::new(),
            time_unit_label: String::new(),
            max_x: 24.0,
        }
    }
}

impl ChartData {
    /// Retorna a quantidade total de extensões presentes nos dados antes de qualquer filtro de limite.
    pub fn total_extensoes(&self) -> usize {
        if !self.boxplot_groups.is_empty() {
            self.boxplot_groups.len()
        } else {
            self.labels.len()
        }
    }

    /// Aplica um limite máximo de itens plotados (`Some(10..=100)` ou `None` para "Todos")
    /// preservando as `N` extensões com maior quantidade de arquivos.
    pub fn com_limite_extensoes(&self, limite: Option<usize>) -> ChartData {
        let Some(max_n) = limite else {
            return self.clone();
        };
        let mut filtrado = self.clone();

        if !filtrado.boxplot_groups.is_empty() {
            filtrado.boxplot_groups.truncate(max_n);
            filtrado.boxplot_totais.truncate(max_n);
        } else {
            filtrado.labels.truncate(max_n);
            filtrado.values.truncate(max_n);
            filtrado.pairs.truncate(max_n);
            if !filtrado.secondary_values.is_empty() {
                // Recalcula o percentual acumulado de Pareto (0% a 100%) sobre os itens selecionados
                let total_sel: f64 = filtrado.values.iter().sum();
                let mut acum = 0.0;
                let mut novo_sec = Vec::with_capacity(filtrado.values.len());
                for &v in &filtrado.values {
                    acum += v;
                    novo_sec.push(if total_sel > 0.0 {
                        (acum / total_sel) * 100.0
                    } else {
                        0.0
                    });
                }
                filtrado.secondary_values = novo_sec;
            }
        }

        filtrado
    }

    /// Retorna a quantidade total de itens de origem únicos presentes nos fluxos do diagrama Sankey.
    pub fn total_origens_sankey(&self) -> usize {
        let mut origens: Vec<&str> = Vec::new();
        for (o, _, _) in &self.flows {
            if !origens.contains(&o.as_str()) {
                origens.push(o.as_str());
            }
        }
        origens.len()
    }

    /// Retorna a quantidade total de itens de destino únicos presentes nos fluxos do diagrama Sankey.
    pub fn total_destinos_sankey(&self) -> usize {
        let mut destinos: Vec<&str> = Vec::new();
        for (_, d, _) in &self.flows {
            if !destinos.contains(&d.as_str()) {
                destinos.push(d.as_str());
            }
        }
        destinos.len()
    }

    /// Retorna a maior contagem entre nós de origem e nós de destino do diagrama Sankey.
    pub fn max_nos_sankey(&self) -> usize {
        self.total_origens_sankey().max(self.total_destinos_sankey())
    }

    /// Aplica um filtro de tipos de operações/eventos selecionados (via checkboxes) e um limite de quantidade no Destino.
    /// Utilizado especificamente pelos diagramas "Sankey: Operação → Tipo de Arquivo" e "Sankey: Operação → Diretório",
    /// onde o usuário escolhe via checkboxes quais operações quer plotar ("Adicionado", "Modificado", etc.).
    pub fn com_filtro_operacoes_e_limite_destino(
        &self,
        operacoes_selecionadas: &[crate::core::types::OperationType],
        limite_destino: Option<usize>,
        loc: &Localizer,
    ) -> ChartData {
        if operacoes_selecionadas.is_empty() {
            let mut vazio = self.clone();
            vazio.flows.clear();
            return vazio;
        }

        // 1. Mapeia as operações selecionadas para seus nomes traduzidos na UI
        let ops_traduzidas: std::collections::HashSet<String> = operacoes_selecionadas
            .iter()
            .map(|op| loc.traduzir_tipo_operacao(op.as_str()))
            .collect();

        let fluxos_origem: Vec<(String, String, f64)> = self
            .flows
            .iter()
            .filter(|(o, _, _)| ops_traduzidas.contains(o))
            .cloned()
            .collect();

        // 2. Filtra destinos se limite_destino for Some
        let mut fluxos_finais: Vec<(String, String, f64)> = if let Some(max_dest) = limite_destino {
            let mut vol_destino: std::collections::HashMap<String, f64> =
                std::collections::HashMap::new();
            for (_, d, v) in &fluxos_origem {
                *vol_destino.entry(d.clone()).or_insert(0.0) += v;
            }
            let mut ordenados: Vec<(String, f64)> = vol_destino.into_iter().collect();
            // Ordenação 100% determinística com critério de desempate alfabético
            // Previne categoricamente qualquer tremor, oscilação ou pulo visual entre frames
            ordenados.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            let top_dest: std::collections::HashSet<String> =
                ordenados.into_iter().take(max_dest).map(|(d, _)| d).collect();

            fluxos_origem
                .into_iter()
                .filter(|(_, d, _)| top_dest.contains(d))
                .collect()
        } else {
            fluxos_origem
        };

        // 3. Ordenação 100% determinística e estável dos fluxos finais
        fluxos_finais.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        let mut filtrado = self.clone();
        filtrado.flows = fluxos_finais;
        filtrado
    }

    /// Aplica um filtro opcional por Tipo de Operação/Evento e um limite de quantidade no Destino.
    /// Utilizado especificamente pelos diagramas "Sankey: Operação → Tipo de Arquivo" e "Sankey: Operação → Diretório",
    /// onde o usuário escolhe a operação específica ("Adicionado", "Modificado", etc. ou "Todas as Operações").
    pub fn com_filtro_operacao_e_limite_destino(
        &self,
        filtro_op: Option<crate::core::types::OperationType>,
        limite_destino: Option<usize>,
        loc: &Localizer,
    ) -> ChartData {
        match filtro_op {
            Some(op) => self.com_filtro_operacoes_e_limite_destino(&[op], limite_destino, loc),
            None => self.com_filtro_operacoes_e_limite_destino(
                crate::core::types::OperationType::all(),
                limite_destino,
                loc,
            ),
        }
    }

    /// Aplica limites independentes de itens plotados tanto para origem quanto para destino
    /// nos diagramas Sankey (`Some(10..=100)` ou `None` para "Todos").
    pub fn com_limite_sankey_duplo(
        &self,
        limite_origem: Option<usize>,
        limite_destino: Option<usize>,
    ) -> ChartData {
        if limite_origem.is_none() && limite_destino.is_none() {
            return self.clone();
        }

        // 1. Filtra origens se limite_origem for Some
        let fluxos_origem: Vec<(String, String, f64)> = if let Some(max_orig) = limite_origem {
            let mut vol_origem: std::collections::HashMap<String, f64> =
                std::collections::HashMap::new();
            for (o, _, v) in &self.flows {
                *vol_origem.entry(o.clone()).or_insert(0.0) += v;
            }
            let mut ordenados: Vec<(String, f64)> = vol_origem.into_iter().collect();
            ordenados.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            let top_orig: std::collections::HashSet<String> =
                ordenados.into_iter().take(max_orig).map(|(o, _)| o).collect();

            self.flows
                .iter()
                .filter(|(o, _, _)| top_orig.contains(o))
                .cloned()
                .collect()
        } else {
            self.flows.clone()
        };

        // 2. Filtra destinos se limite_destino for Some
        let mut fluxos_finais: Vec<(String, String, f64)> = if let Some(max_dest) = limite_destino {
            let mut vol_destino: std::collections::HashMap<String, f64> =
                std::collections::HashMap::new();
            for (_, d, v) in &fluxos_origem {
                *vol_destino.entry(d.clone()).or_insert(0.0) += v;
            }
            let mut ordenados: Vec<(String, f64)> = vol_destino.into_iter().collect();
            ordenados.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            let top_dest: std::collections::HashSet<String> =
                ordenados.into_iter().take(max_dest).map(|(d, _)| d).collect();

            fluxos_origem
                .into_iter()
                .filter(|(_, d, _)| top_dest.contains(d))
                .collect()
        } else {
            fluxos_origem
        };

        fluxos_finais.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        let mut filtrado = self.clone();
        filtrado.flows = fluxos_finais;
        filtrado
    }

    /// Aplica um limite máximo de itens plotados nos diagramas Sankey (`Some(10..=100)` ou `None` para "Todos"),
    /// preservando as `N` extensões ou diretórios de destino com maior volume de eventos.
    pub fn com_limite_sankey(&self, limite: Option<usize>) -> ChartData {
        self.com_limite_sankey_duplo(None, limite)
    }
}

/// Calcula o valor em escala logarítmica de base 10 para um tamanho em KB (`log10(max(kb, 0.1))`).
pub fn calcular_log10_kb(size_kb: f64) -> f64 {
    size_kb.max(0.1).log10()
}

/// Calcula os limites logarítmicos (em décadas inteiras de base 10) para um conjunto de tamanhos em KB.
pub fn calcular_limites_log10_kb(tamanhos_kb: &[f64]) -> (f64, f64) {
    if tamanhos_kb.is_empty() {
        return (0.0, 6.0);
    }
    let min_kb = tamanhos_kb
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
        .max(0.1);
    let max_kb = tamanhos_kb
        .iter()
        .copied()
        .fold(0.1_f64, f64::max)
        .max(1.0);

    let log_min = if min_kb < 1.0 { -1.0 } else { 0.0 };
    let log_max = (max_kb.log10().ceil()).clamp(3.0, 6.0).max(log_min + 2.0);
    (log_min, log_max)
}

/// Calcula o percentil `p` (0.0 a 1.0) usando interpolação linear contínua (método Hyndman-Fan Tipo 7)
/// sobre uma fatia já ordenada em ordem crescente.
pub fn calcular_percentil_interpolado(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let p = p.clamp(0.0, 1.0);
    let pos = p * (sorted.len() - 1) as f64;
    let lower = pos.floor() as usize;
    let upper = pos.ceil() as usize;
    let weight = pos - lower as f64;
    sorted[lower] * (1.0 - weight) + sorted[upper] * weight
}

/// Calcula as 5 estatísticas do Boxplot de Tukey:
/// `(whisker_inferior, Q1, mediana, Q3, whisker_superior)`
/// com quartis interpolados continuamente e bigodes limitados pelas cercas de Tukey
/// `[Q1 - 1.5 * IQR, Q3 + 1.5 * IQR]` usando os valores adjacentes reais da amostra.
pub fn calcular_estatisticas_boxplot_tukey(sorted: &[f64]) -> (f64, f64, f64, f64, f64) {
    if sorted.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let q1 = calcular_percentil_interpolado(sorted, 0.25);
    let med = calcular_percentil_interpolado(sorted, 0.50);
    let q3 = calcular_percentil_interpolado(sorted, 0.75);
    let iqr = (q3 - q1).max(0.0);
    let cerca_inf = q1 - 1.5 * iqr;
    let cerca_sup = q3 + 1.5 * iqr;

    let min_adj = sorted
        .iter()
        .copied()
        .find(|&v| v >= cerca_inf)
        .unwrap_or(sorted[0]);
    let max_adj = sorted
        .iter()
        .rev()
        .copied()
        .find(|&v| v <= cerca_sup)
        .unwrap_or(*sorted.last().unwrap());

    (min_adj, q1, med, q3, max_adj)
}

fn encurtar_diretorio(dir: &str) -> String {
    let p = dir.replace('/', "\\");
    let parts: Vec<&str> = p.split('\\').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 2 {
        if dir.is_empty() {
            "Raiz".to_string()
        } else {
            dir.to_string()
        }
    } else {
        format!("..\\{}\\{}", parts[parts.len() - 2], parts[parts.len() - 1])
    }
}

pub fn extrair_extensao_nome(nome: &str) -> String {
    let p = std::path::Path::new(nome);
    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
        if !ext.is_empty() {
            return format!(".{}", ext.to_lowercase());
        }
    }
    "(sem ext)".to_string()
}

pub fn extrair_extensao_nome_loc(nome: &str, loc: &Localizer) -> String {
    let p = std::path::Path::new(nome);
    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
        if !ext.is_empty() {
            return format!(".{}", ext.to_lowercase());
        }
    }
    loc.t("stat_no_extension")
}

/// Indica se o evento se refere a um diretório (e não a um arquivo monitorado).
fn evento_eh_diretorio(ev: &EventModel) -> bool {
    let tipo = ev.tipo.to_lowercase();
    tipo.contains("pasta")
        || tipo.contains("diretório")
        || tipo.contains("diretorio")
        || tipo.contains("directory")
        || tipo.contains("folder")
}

/// Contabiliza a quantidade total de eventos de arquivos por extensão, abrangendo TODAS as
/// extensões monitoradas (sem truncamento). Retorna pares ordenados de forma decrescente pela
/// quantidade e, em caso de empate, alfabeticamente pela extensão.
fn contar_por_extensao(eventos: &[EventModel], loc: &Localizer) -> Vec<(String, f64)> {
    let mut contagem: HashMap<String, f64> = HashMap::new();
    for ev in eventos.iter().filter(|ev| !evento_eh_diretorio(ev)) {
        let ext = extrair_extensao_nome_loc(&ev.nome, loc);
        *contagem.entry(ext).or_insert(0.0) += 1.0;
    }

    let mut pairs: Vec<(String, f64)> = contagem.into_iter().collect();
    pairs.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    pairs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeScaleUnit {
    Segundos,
    Minutos,
    Horas,
    Dias,
}

pub fn determinar_escala_temporal(eventos: &[EventModel]) -> (i64, i64, TimeScaleUnit, usize, Vec<String>) {
    let mut ts_list = Vec::new();
    for ev in eventos {
        if let Ok(epoch) = ev.timestamp.parse::<i64>() {
            ts_list.push(epoch);
        } else if let Ok(dt) = NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%dT%H:%M:%S"))
        {
            ts_list.push(dt.and_utc().timestamp());
        }
    }

    if ts_list.is_empty() {
        let labels = (0..60).map(|s| format!("{:02}s", s)).collect();
        return (0, 60, TimeScaleUnit::Segundos, 60, labels);
    }

    ts_list.sort_unstable();
    let t_min = ts_list[0];
    let t_max = *ts_list.last().unwrap();
    let delta = (t_max - t_min).max(0);

    if delta < 60 {
        // Escala em segundos (0..60s)
        let labels: Vec<String> = (0..60).map(|s| format!("{:02}s", s)).collect();
        (t_min, t_min + 60, TimeScaleUnit::Segundos, 60, labels)
    } else if delta < 3600 {
        // Escala em minutos (0..60m)
        let total_mins = (((delta as f64) / 60.0).ceil() as usize).clamp(60, 120);
        let labels: Vec<String> = (0..total_mins).map(|m| format!("{:02}m", m)).collect();
        (t_min, t_min + (total_mins as i64 * 60), TimeScaleUnit::Minutos, total_mins, labels)
    } else if delta < 86400 {
        // Escala em horas (0..24h)
        let labels: Vec<String> = (0..24).map(|h| format!("{:02}h", h)).collect();
        (t_min, t_min + 86400, TimeScaleUnit::Horas, 24, labels)
    } else {
        // Escala em dias
        let total_days = (((delta as f64) / 86400.0).ceil() as usize).max(2);
        let labels: Vec<String> = (0..total_days).map(|d| format!("D{:02}", d + 1)).collect();
        (t_min, t_min + (total_days as i64 * 86400), TimeScaleUnit::Dias, total_days, labels)
    }
}

pub struct GeradorEstatisticas;

impl GeradorEstatisticas {
    // 1. Pizza de Operações
    pub fn gerar_pizza_operacoes(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_pizza_operacoes_loc(eventos, &loc)
    }

    pub fn gerar_pizza_operacoes_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut contagem: HashMap<String, f64> = HashMap::new();
        for ev in eventos {
            let label = loc.traduzir_tipo_operacao(ev.op_type().as_str());
            *contagem.entry(label).or_insert(0.0) += 1.0;
        }

        let mut pairs: Vec<(String, f64)> = contagem.into_iter().collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let labels = pairs.iter().map(|p| p.0.clone()).collect();
        let values = pairs.iter().map(|p| p.1).collect();

        ChartData {
            labels,
            values,
            pairs,
            ..Default::default()
        }
    }

    // 2. Barras por Tipo de Arquivo: quantidade total por tipo de extensão, de TODAS as extensões monitoradas
    pub fn gerar_barras_tipos(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_barras_tipos_loc(eventos, &loc)
    }

    pub fn gerar_barras_tipos_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        // Sem truncamento: todas as extensões de arquivos monitorados são reportadas
        let pairs = contar_por_extensao(eventos, loc);

        let labels = pairs.iter().map(|p| p.0.clone()).collect();
        let values = pairs.iter().map(|p| p.1).collect();

        ChartData {
            labels,
            values,
            pairs,
            ..Default::default()
        }
    }

    // 3. Linha do Tempo de Operações (Timeline)
    pub fn gerar_timeline_operacoes_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut ev_sorted: Vec<&EventModel> = eventos.iter().collect();
        ev_sorted.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Amostra até 300 eventos cronológicos
        let step = (ev_sorted.len() / 300).max(1);
        let mut timeline_items = Vec::new();
        let mut contagem_ops: HashMap<String, f64> = HashMap::new();

        for (idx, ev) in ev_sorted.iter().enumerate() {
            let op_label = loc.traduzir_tipo_operacao(ev.op_type().as_str());
            *contagem_ops.entry(op_label.clone()).or_insert(0.0) += 1.0;

            if idx % step == 0 || ev.op_type() == crate::core::types::OperationType::Movido {
                timeline_items.push((ev.timestamp.clone(), op_label, ev.nome.clone(), ev.size_kb));
            }
        }

        let mut pairs: Vec<(String, f64)> = contagem_ops.into_iter().collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let labels = pairs.iter().map(|p| p.0.clone()).collect();
        let values = pairs.iter().map(|p| p.1).collect();

        ChartData {
            labels,
            values,
            pairs,
            timeline_items,
            ..Default::default()
        }
    }

    // 4. Treemap por Tipos: reporta por tipos de extensão de arquivo
    pub fn gerar_treemap_tipos_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut contagem: HashMap<String, f64> = HashMap::new();
        for ev in eventos {
            let ext = extrair_extensao_nome_loc(&ev.nome, loc);
            *contagem.entry(ext).or_insert(0.0) += 1.0;
        }

        let mut pairs: Vec<(String, f64)> = contagem.into_iter().collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let max_items = 24;
        let final_pairs = if pairs.len() > max_items {
            let (top, rest) = pairs.split_at(max_items - 1);
            let mut list: Vec<(String, f64)> = top.to_vec();
            let rest_sum: f64 = rest.iter().map(|p| p.1).sum();
            if rest_sum > 0.0 {
                list.push((loc.t("stat_others"), rest_sum));
            }
            list
        } else {
            pairs
        };

        let mut treemap_items = Vec::with_capacity(final_pairs.len());
        for (i, (label, val)) in final_pairs.iter().enumerate() {
            let color = CORES_PALETA_GERADOR[i % CORES_PALETA_GERADOR.len()];
            treemap_items.push((label.clone(), *val, color));
        }

        let labels = final_pairs.iter().map(|p| p.0.clone()).collect();
        let values = final_pairs.iter().map(|p| p.1).collect();

        ChartData {
            labels,
            values,
            pairs: final_pairs,
            treemap_items,
            ..Default::default()
        }
    }

    // 5. Histograma por Horários
    pub fn gerar_histograma_horarios(eventos: &[EventModel]) -> ChartData {
        let mut horas = vec![0.0; 24];
        for ev in eventos {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%d %H:%M:%S") {
                let h = dt.format("%H").to_string().parse::<usize>().unwrap_or(0);
                if h < 24 {
                    horas[h] += 1.0;
                }
            }
        }

        let labels = (0..24).map(|h| format!("{:02}h", h)).collect();

        ChartData {
            labels,
            values: horas,
            ..Default::default()
        }
    }

    // 6. Pareto de Extensões: quantidade total por extensão (TODAS as extensões) com base na técnica de Pareto
    pub fn gerar_pareto_extensoes(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_pareto_extensoes_loc(eventos, &loc)
    }

    pub fn gerar_pareto_extensoes_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        // Ordenação decrescente obrigatória para a análise de Pareto (sem truncamento)
        let pairs = contar_por_extensao(eventos, loc);

        // O percentual acumulado é calculado sobre o total geral de todas as extensões
        let total: f64 = pairs.iter().map(|p| p.1).sum();
        let mut acumulado = 0.0;
        let mut pct_acumulado = Vec::with_capacity(pairs.len());

        for p in &pairs {
            acumulado += p.1;
            pct_acumulado.push(if total > 0.0 {
                (acumulado / total) * 100.0
            } else {
                0.0
            });
        }

        let labels = pairs.iter().map(|p| p.0.clone()).collect();
        let values = pairs.iter().map(|p| p.1).collect();

        ChartData {
            labels,
            values,
            secondary_values: pct_acumulado,
            pairs,
            ..Default::default()
        }
    }

    // 7. Evolução de Eventos: Escala Dinâmica (Segundos -> Minutos -> Horas -> Dias)
    pub fn gerar_linha_eventos_hora_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let (t_min, _, scale, num_buckets, labels) = determinar_escala_temporal(eventos);
        let mut matriz_ops = vec![vec![0.0; num_buckets]; 6]; // 6 operações
        let mut totais = vec![0.0; num_buckets];

        for ev in eventos {
            let t_ev_opt = if let Ok(epoch) = ev.timestamp.parse::<i64>() {
                Some(epoch)
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%dT%H:%M:%S"))
            {
                Some(dt.and_utc().timestamp())
            } else {
                None
            };

            if let Some(t_ev) = t_ev_opt {
                let bucket = match scale {
                    TimeScaleUnit::Segundos => {
                        ((t_ev - t_min).clamp(0, (num_buckets - 1) as i64)) as usize
                    }
                    TimeScaleUnit::Minutos => {
                        (((t_ev - t_min) / 60).clamp(0, (num_buckets - 1) as i64)) as usize
                    }
                    TimeScaleUnit::Horas => {
                        (((t_ev - t_min) / 3600).clamp(0, (num_buckets - 1) as i64)) as usize
                    }
                    TimeScaleUnit::Dias => {
                        (((t_ev - t_min) / 86400).clamp(0, (num_buckets - 1) as i64)) as usize
                    }
                };

                if bucket < num_buckets {
                    totais[bucket] += 1.0;
                    let op_idx = match ev.op_type() {
                        crate::core::types::OperationType::Adicionado => 0,
                        crate::core::types::OperationType::Modificado => 1,
                        crate::core::types::OperationType::Excluido => 2,
                        crate::core::types::OperationType::Renomeado => 3,
                        crate::core::types::OperationType::Movido => 4,
                        crate::core::types::OperationType::Escaneado => 5,
                    };
                    matriz_ops[op_idx][bucket] += 1.0;
                }
            }
        }

        let ops_info = [
            (loc.t("op_added"), CORES_PALETA_GERADOR[0]),
            (loc.t("op_modified"), CORES_PALETA_GERADOR[4]),
            (loc.t("op_deleted"), CORES_PALETA_GERADOR[2]),
            (loc.t("op_renamed"), CORES_PALETA_GERADOR[3]),
            (loc.t("op_moved"), CORES_PALETA_GERADOR[1]),
            (loc.t("op_scanned"), CORES_PALETA_GERADOR[5]),
        ];

        let mut multi_series = Vec::new();
        for (idx, (nome_op, cor)) in ops_info.iter().enumerate() {
            multi_series.push((nome_op.clone(), matriz_ops[idx].clone(), *cor));
        }

        let time_unit_label = match scale {
            TimeScaleUnit::Segundos => loc.t("scale_seconds"),
            TimeScaleUnit::Minutos => loc.t("scale_minutes"),
            TimeScaleUnit::Horas => loc.t("scale_hours"),
            TimeScaleUnit::Dias => loc.t("scale_days"),
        };

        ChartData {
            labels,
            values: totais.clone(),
            secondary_values: totais,
            multi_series,
            time_unit_label,
            max_x: (num_buckets as f64).max(1.0),
            ..Default::default()
        }
    }

    // 8. Boxplot de Tamanhos de Arquivos: reporta, para TODAS as extensões monitoradas, a quantidade total
    // de arquivos de cada extensão em relação à soma total do tamanho dos arquivos daquela extensão,
    // acompanhada da distribuição estatística (quartis) dos tamanhos individuais.
    pub fn gerar_boxplot_tamanhos(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_boxplot_tamanhos_loc(eventos, &loc)
    }

    pub fn gerar_boxplot_tamanhos_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut map_ext_sizes: HashMap<String, Vec<f64>> = HashMap::new();
        let mut global_sizes: Vec<f64> = Vec::new();

        for ev in eventos.iter().filter(|ev| !evento_eh_diretorio(ev)) {
            let size = ev.size_kb.max(0.0);
            global_sizes.push(size);
            let ext = extrair_extensao_nome_loc(&ev.nome, loc);
            map_ext_sizes.entry(ext).or_default().push(size);
        }

        global_sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let boxplot = if !global_sizes.is_empty() {
            Some(calcular_estatisticas_boxplot_tukey(&global_sizes))
        } else {
            None
        };

        let mut groups: Vec<(String, Vec<f64>, f64)> = map_ext_sizes
            .into_iter()
            .map(|(ext, sizes)| {
                let soma: f64 = sizes.iter().sum();
                (ext, sizes, soma)
            })
            .collect();
        // Ordena primariamente pela quantidade total de arquivos da extensão (desc.), depois pela soma total
        // do tamanho (desc.) e pelo nome da extensão. Sem truncamento: todas as extensões são reportadas.
        groups.sort_by(|a, b| {
            b.1.len()
                .cmp(&a.1.len())
                .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.0.cmp(&b.0))
        });

        let mut boxplot_groups = Vec::with_capacity(groups.len());
        let mut boxplot_totais = Vec::with_capacity(groups.len());
        for (ext, mut sizes, soma) in groups {
            sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let n = sizes.len();
            if n > 0 {
                let (min_w, q1, med, q3, max_w) = calcular_estatisticas_boxplot_tukey(&sizes);
                boxplot_groups.push((ext.clone(), min_w, q1, med, q3, max_w));
                boxplot_totais.push((ext, n, soma));
            }
        }

        let labels = if boxplot.is_some() {
            vec![
                "Mínimo".to_string(),
                "Q1 (25%)".to_string(),
                "Mediana".to_string(),
                "Q3 (75%)".to_string(),
                "Máximo".to_string(),
            ]
        } else {
            Vec::new()
        };

        ChartData {
            labels,
            values: if let Some((min, q1, med, q3, max)) = boxplot {
                vec![min, q1, med, q3, max]
            } else {
                Vec::new()
            },
            boxplot,
            boxplot_groups,
            boxplot_totais,
            ..Default::default()
        }
    }

    // 9. Radar de Operações (6 eixos)
    pub fn gerar_radar_operacoes_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut contagem = [0.0; 6];
        for ev in eventos {
            match ev.op_type() {
                crate::core::types::OperationType::Adicionado => contagem[0] += 1.0,
                crate::core::types::OperationType::Modificado => contagem[1] += 1.0,
                crate::core::types::OperationType::Excluido => contagem[2] += 1.0,
                crate::core::types::OperationType::Renomeado => contagem[3] += 1.0,
                crate::core::types::OperationType::Movido => contagem[4] += 1.0,
                crate::core::types::OperationType::Escaneado => contagem[5] += 1.0,
            }
        }

        let max_val = contagem.iter().copied().fold(0.0, f64::max).max(1.0);
        let ops = [
            loc.t("op_added"),
            loc.t("op_modified"),
            loc.t("op_deleted"),
            loc.t("op_renamed"),
            loc.t("op_moved"),
            loc.t("op_scanned"),
        ];

        let mut radar_dimensions = Vec::new();
        let mut labels = Vec::new();
        let mut values = Vec::new();

        for i in 0..6 {
            let score = (contagem[i] / max_val) * 100.0;
            radar_dimensions.push((ops[i].clone(), score));
            labels.push(ops[i].clone());
            values.push(contagem[i]);
        }

        ChartData {
            labels,
            values,
            radar_dimensions,
            ..Default::default()
        }
    }

    // 10. Heatmap Dia x Hora
    pub fn gerar_heatmap_dia_hora(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_heatmap_dia_hora_loc(eventos, &loc)
    }

    pub fn gerar_heatmap_dia_hora_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut mat = vec![vec![0.0; 24]; 7];
        for ev in eventos {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%d %H:%M:%S") {
                use chrono::Datelike;
                let d = dt.weekday().num_days_from_monday() as usize;
                let h = dt.format("%H").to_string().parse::<usize>().unwrap_or(0);
                if d < 7 && h < 24 {
                    mat[d][h] += 1.0;
                }
            }
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
        let mut labels = Vec::with_capacity(7 * 24);
        let mut values = Vec::with_capacity(7 * 24);
        for d in 0..7 {
            for h in 0..24 {
                labels.push(format!("{} {:02}h", dias[d], h));
                values.push(mat[d][h]);
            }
        }

        ChartData {
            labels,
            values,
            matrix_7x24: mat,
            ..Default::default()
        }
    }

    // 11. Scatter Tamanho x Tempo (Escala Dinâmica no eixo X; Escala Logarítmica base 10 no eixo Y das ordenadas)
    pub fn gerar_scatter_tamanho_hora(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_scatter_tamanho_hora_loc(eventos, &loc)
    }

    pub fn gerar_scatter_tamanho_hora_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let (t_min, _, scale, num_buckets, labels) = determinar_escala_temporal(eventos);
        let mut points = Vec::new();
        let mut valores_log_y = Vec::new();
        let mut tamanhos_kb = Vec::new();

        for ev in eventos {
            let t_ev_opt = if let Ok(epoch) = ev.timestamp.parse::<i64>() {
                Some(epoch)
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&ev.timestamp, "%Y-%m-%dT%H:%M:%S"))
            {
                Some(dt.and_utc().timestamp())
            } else {
                None
            };

            if let Some(t_ev) = t_ev_opt {
                let x = match scale {
                    TimeScaleUnit::Segundos => (t_ev - t_min).clamp(0, 60) as f64,
                    TimeScaleUnit::Minutos => {
                        ((t_ev - t_min) as f64 / 60.0).clamp(0.0, num_buckets as f64)
                    }
                    TimeScaleUnit::Horas => {
                        ((t_ev - t_min) as f64 / 3600.0).clamp(0.0, 24.0)
                    }
                    TimeScaleUnit::Dias => ((t_ev - t_min) as f64 / 86400.0).max(0.0),
                };
                let size = ev.size_kb.max(0.1);
                let log_y = calcular_log10_kb(size);
                points.push((x, size));
                valores_log_y.push(log_y);
                tamanhos_kb.push(size);
            }
        }

        let (log_min, log_max) = calcular_limites_log10_kb(&tamanhos_kb);

        let max_x = match scale {
            TimeScaleUnit::Segundos => 60.0,
            TimeScaleUnit::Minutos => (num_buckets as f64).max(60.0),
            TimeScaleUnit::Horas => 24.0,
            TimeScaleUnit::Dias => (num_buckets as f64).max(2.0),
        };

        let unit_name = match scale {
            TimeScaleUnit::Segundos => loc.t("scale_seconds"),
            TimeScaleUnit::Minutos => loc.t("scale_minutes"),
            TimeScaleUnit::Horas => loc.t("scale_hours"),
            TimeScaleUnit::Dias => loc.t("scale_days"),
        };

        ChartData {
            points,
            values: valores_log_y,
            secondary_values: vec![log_min, log_max],
            labels,
            time_unit_label: unit_name,
            max_x,
            ..Default::default()
        }
    }

    // 12. Relação: Operação → Tipos de Extensão de Arquivo
    pub fn gerar_sankey_operacao_tipo(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_sankey_operacao_tipo_loc(eventos, &loc)
    }

    pub fn gerar_sankey_operacao_tipo_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut fluxos: HashMap<(String, String), f64> = HashMap::new();
        for ev in eventos {
            let op = loc.traduzir_tipo_operacao(ev.op_type().as_str());
            let ext = extrair_extensao_nome_loc(&ev.nome, loc);
            *fluxos.entry((op, ext)).or_insert(0.0) += 1.0;
        }

        let mut flows: Vec<(String, String, f64)> =
            fluxos.into_iter().map(|((o, d), v)| (o, d, v)).collect();
        flows.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        // Garante que todas as operações presentes nos eventos (especialmente Movido)
        // estejam representadas nos fluxos do diagrama Sankey
        let mut flows_selecionados: Vec<(String, String, f64)> = Vec::new();
        let mut ops_incluidas: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 1ª passada: garante o fluxo de maior volume de cada operação presente
        for f in &flows {
            if ops_incluidas.insert(f.0.clone()) {
                flows_selecionados.push(f.clone());
            }
        }

        // 2ª passada: completa com os maiores fluxos restantes
        for f in &flows {
            if !flows_selecionados.iter().any(|s| s.0 == f.0 && s.1 == f.1) {
                flows_selecionados.push(f.clone());
            }
        }
        flows_selecionados.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        ChartData {
            flows: flows_selecionados,
            ..Default::default()
        }
    }

    // 13. Radar Multidimensional de Risco (operações separadas e eixos forenses)
    pub fn gerar_radar_multidimensional_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let total = eventos.len() as f64;
        if total == 0.0 {
            return ChartData::default();
        }

        let mut add_count = 0.0;
        let mut scan_count = 0.0;
        let mut del_count = 0.0;
        let mut mod_count = 0.0;
        let mut mov_count = 0.0;
        let mut ren_count = 0.0;
        let mut bin_count = 0.0;
        let mut baseline_alerts = 0.0;
        let mut high_risk_count = 0.0;
        let mut actor_count = 0.0;

        for ev in eventos {
            match ev.op_type() {
                crate::core::types::OperationType::Adicionado => add_count += 1.0,
                crate::core::types::OperationType::Escaneado => scan_count += 1.0,
                crate::core::types::OperationType::Excluido => del_count += 1.0,
                crate::core::types::OperationType::Modificado => mod_count += 1.0,
                crate::core::types::OperationType::Movido => mov_count += 1.0,
                crate::core::types::OperationType::Renomeado => ren_count += 1.0,
            }

            if ev.tipo.contains("Executável")
                || ev.tipo.contains("Binário")
                || ev.binary_file_b > 0.0
            {
                bin_count += 1.0;
            }

            if ev.baseline_status.contains("MODIFICADO") || ev.baseline_status.contains("NOVO") {
                baseline_alerts += 1.0;
            }

            if ev.risk_score >= 35
                || ev.risk_level.contains("Alto")
                || ev.risk_level.contains("Crítico")
            {
                high_risk_count += 1.0;
            }

            if !ev.actor_user.is_empty() && ev.actor_user != "Sistema" {
                actor_count += 1.0;
            }
        }

        let d_add = (add_count / total) * 100.0;
        let d_scan = (scan_count / total) * 100.0;
        let d_del = (del_count / total) * 100.0;
        let d_mod = (mod_count / total) * 100.0;
        let d_mov = (mov_count / total) * 100.0;
        let d_ren = (ren_count / total) * 100.0;

        let d_bin = (bin_count / total) * 100.0;
        let d_base = (baseline_alerts / total) * 100.0;
        let d_risk = (high_risk_count / total) * 100.0;
        let d_actor = (actor_count / total) * 100.0;

        let radar_dimensions = vec![
            (loc.t("op_added"), d_add),
            (loc.t("op_scanned"), d_scan),
            (loc.t("op_deleted"), d_del),
            (loc.t("op_modified"), d_mod),
            (loc.t("op_moved"), d_mov),
            (loc.t("op_renamed"), d_ren),
            (loc.t("radar_dim_executables"), d_bin),
            (loc.t("radar_dim_baseline_alerts"), d_base),
            (loc.t("radar_dim_high_risk"), d_risk),
            (loc.t("radar_dim_traceability"), d_actor),
        ];

        let labels = radar_dimensions.iter().map(|d| d.0.clone()).collect();
        let values = radar_dimensions.iter().map(|d| d.1).collect();

        ChartData {
            labels,
            values,
            radar_dimensions,
            ..Default::default()
        }
    }

    // 14. Dotplot de Distribuição de Eventos por Extensão e Tamanho (Escala Logarítmica base 10 no eixo X das abscissas)
    pub fn gerar_dotplot_distribuicao_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut grupos: HashMap<String, Vec<f64>> = HashMap::new();
        for ev in eventos {
            let cat = extrair_extensao_nome_loc(&ev.nome, loc);
            let size = ev.size_kb.max(0.1);
            grupos.entry(cat).or_default().push(size);
        }

        let mut dotplot_groups: Vec<(String, Vec<f64>)> = grupos.into_iter().collect();
        dotplot_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
        dotplot_groups.truncate(8);

        let mut todos_tamanhos = Vec::new();
        let mut medias_log_x = Vec::with_capacity(dotplot_groups.len());

        for (_, list) in &mut dotplot_groups {
            list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            list.truncate(60); // Amostra até 60 pontos por grupo
            let soma_log: f64 = list.iter().map(|&sz| calcular_log10_kb(sz)).sum();
            let media_log = if !list.is_empty() {
                soma_log / (list.len() as f64)
            } else {
                0.0
            };
            medias_log_x.push(media_log);
            todos_tamanhos.extend(list.iter().copied());
        }

        let (log_min, log_max) = calcular_limites_log10_kb(&todos_tamanhos);
        let labels = dotplot_groups.iter().map(|g| g.0.clone()).collect();

        ChartData {
            labels,
            values: medias_log_x,
            secondary_values: vec![log_min, log_max],
            dotplot_groups,
            ..Default::default()
        }
    }

    // 15. Relação: Operação → Diretórios
    pub fn gerar_sankey_evento_diretorio_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut fluxos: HashMap<(String, String), f64> = HashMap::new();
        for ev in eventos {
            let op = loc.traduzir_tipo_operacao(ev.op_type().as_str());
            let raw_dir = if !ev.dir_atual.trim().is_empty() {
                &ev.dir_atual
            } else if !ev.dir_anterior.trim().is_empty() {
                &ev.dir_anterior
            } else {
                ""
            };
            let dir = encurtar_diretorio(raw_dir);
            *fluxos.entry((op, dir)).or_insert(0.0) += 1.0;
        }

        let mut flows: Vec<(String, String, f64)> =
            fluxos.into_iter().map(|((o, d), v)| (o, d, v)).collect();
        flows.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        // Garante que todas as operações presentes nos eventos (especialmente Movido)
        // estejam representadas nos fluxos do diagrama Sankey
        let mut flows_selecionados: Vec<(String, String, f64)> = Vec::new();
        let mut ops_incluidas: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 1ª passada: garante o fluxo de maior volume de cada operação presente
        for f in &flows {
            if ops_incluidas.insert(f.0.clone()) {
                flows_selecionados.push(f.clone());
            }
        }

        // 2ª passada: completa com os maiores fluxos restantes
        for f in &flows {
            if !flows_selecionados.iter().any(|s| s.0 == f.0 && s.1 == f.1) {
                flows_selecionados.push(f.clone());
            }
        }
        flows_selecionados.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        ChartData {
            flows: flows_selecionados,
            ..Default::default()
        }
    }

    // 16. Relação: Tipo de Arquivo → Diretório
    pub fn gerar_sankey_tipo_diretorio(eventos: &[EventModel]) -> ChartData {
        let loc = Localizer::new();
        Self::gerar_sankey_tipo_diretorio_loc(eventos, &loc)
    }

    pub fn gerar_sankey_tipo_diretorio_loc(eventos: &[EventModel], loc: &Localizer) -> ChartData {
        let mut fluxos: HashMap<(String, String), f64> = HashMap::new();
        for ev in eventos {
            let raw_dir = if !ev.dir_atual.trim().is_empty() {
                &ev.dir_atual
            } else if !ev.dir_anterior.trim().is_empty() {
                &ev.dir_anterior
            } else {
                ""
            };
            let dir = encurtar_diretorio(raw_dir);
            let ext = extrair_extensao_nome_loc(&ev.nome, loc);
            *fluxos.entry((ext, dir)).or_insert(0.0) += 1.0;
        }

        let mut flows: Vec<(String, String, f64)> =
            fluxos.into_iter().map(|((o, d), v)| (o, d, v)).collect();
        flows.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });

        ChartData {
            flows,
            ..Default::default()
        }
    }

    // 17. Árvore de Diretórios: reporta toda a estrutura de diretórios, a árvore completa
    pub fn gerar_arvore_diretorios_loc(eventos: &[EventModel], _loc: &Localizer) -> ChartData {
        let mut dir_stats: BTreeMap<String, (usize, f64)> = BTreeMap::new();

        for ev in eventos {
            let dir = ev.dir_atual.trim().replace('/', "\\");
            let norm_dir = dir.trim_end_matches('\\').to_string();
            if !norm_dir.is_empty() {
                let e = dir_stats.entry(norm_dir.clone()).or_insert((0, 0.0));
                e.0 += 1;
                e.1 += ev.size_mb;

                // Registra diretórios pais até a raiz para garantir a árvore completa
                let mut current = norm_dir.as_str();
                while let Some(idx) = current.rfind('\\') {
                    let parent = &current[..idx];
                    if parent.is_empty() || parent.ends_with(':') {
                        let root = if parent.ends_with(':') {
                            format!("{}\\", parent)
                        } else {
                            "\\".to_string()
                        };
                        dir_stats.entry(root).or_insert((0, 0.0));
                        break;
                    } else {
                        dir_stats.entry(parent.to_string()).or_insert((0, 0.0));
                        current = parent;
                    }
                }
            }
        }

        // Constrói a lista hierárquica completa sem truncamento
        let mut tree_nodes = Vec::new();
        for (dir_path, (direct_count, direct_mb)) in &dir_stats {
            let clean = dir_path.trim_end_matches('\\');
            let depth = if clean.contains('\\') {
                clean.split('\\').filter(|s| !s.is_empty()).count().saturating_sub(1)
            } else {
                0
            };

            let folder_name = if let Some(last) = clean.split('\\').filter(|s| !s.is_empty()).last() {
                if clean.ends_with(':') || last.ends_with(':') {
                    format!("{}\\", last)
                } else {
                    last.to_string()
                }
            } else {
                dir_path.clone()
            };

            // Calcula contagem e tamanho recursivo agregando descendentes
            let prefix = if dir_path.ends_with('\\') {
                dir_path.clone()
            } else {
                format!("{}\\", dir_path)
            };

            let mut total_count = *direct_count;
            let mut total_mb = *direct_mb;
            for (other_path, (c, m)) in &dir_stats {
                if other_path != dir_path && other_path.starts_with(&prefix) {
                    total_count += *c;
                    total_mb += *m;
                }
            }

            tree_nodes.push((folder_name, depth, total_count, total_mb));
        }

        ChartData {
            tree_nodes,
            ..Default::default()
        }
    }
}
