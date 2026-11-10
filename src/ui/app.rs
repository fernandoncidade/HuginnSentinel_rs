use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{Color32, ProgressBar, RichText};
use std::path::PathBuf;
use std::sync::Arc;

use std::time::Duration;

use super::dialogs::{
    AboutDialog, AdvancedFiltersDialog, AlertCenterDialog, CleanupDialog, ColorDialog, ColumnDialog,
    EvidenceDialog, HelpDialog, LimpezaAlvo, ManualDialog, MultiDirectoryDialog, NotificationConfigDialog,
    RecoveryDialog, SessionCenterDialog, StatisticsDialog,
};
use super::filter_panel::renderizar_painel_filtros;
use super::menu_bar::renderizar_menu_bar;
use super::table_view::renderizar_tabela_eventos;
use super::toolbar::renderizar_toolbar;
use crate::core::event::EventModel;
use crate::core::types::OperationType;
use crate::data::charts::TipoGrafico;
use crate::locale::Localizer;
use crate::security::SecurityManager;
use crate::services::{
    BaseEvento, DiretorioScanner, GerenciadorColunas, GerenciadorTabela, Observador, ScanProgress,
};
use crate::tools::filters::{ContadoresEventos, FiltroEstado};
use crate::utils::logger::get_logger;

pub struct HuginnApp {
    pub loc: Localizer,
    pub base_evento: Arc<BaseEvento>,
    pub security_manager: Arc<SecurityManager>,
    pub gerenciador_colunas: GerenciadorColunas,
    pub gerenciador_tabela: GerenciadorTabela,
    pub filtros: FiltroEstado,

    pub diretorio_ativo: Option<PathBuf>,
    pub observador: Option<Observador>,
    pub scanner: Option<DiretorioScanner>,

    pub eventos: Vec<EventModel>,
    pub event_tx: Sender<EventModel>,
    pub event_rx: Receiver<EventModel>,

    pub progress_tx: Sender<ScanProgress>,
    pub progress_rx: Receiver<ScanProgress>,
    pub progresso_atual: Option<ScanProgress>,

    // Cache de filtragem e virtualização de alto desempenho
    pub indices_filtrados: Vec<usize>,
    pub contadores_cache: ContadoresEventos,
    pub filtros_sujos: bool,
    pub ultimo_update_filtro: std::time::Instant,
    pub linha_selecionada: Option<usize>,

    // Opções de exportação avançadas e histórico de salvamento
    pub export_colunas_ativas: bool,
    pub export_filtros_ativos: bool,
    pub export_selecao: bool,
    pub ultimo_salvamento: Option<(PathBuf, String)>,

    // Diálogos modais e painéis auxiliares
    pub dialog_about: AboutDialog,
    pub dialog_help: HelpDialog,
    pub dialog_manual: ManualDialog,
    pub dialog_colors: ColorDialog,
    pub dialog_columns: ColumnDialog,
    pub dialog_stats: StatisticsDialog,
    pub dialog_alerts: AlertCenterDialog,
    pub dialog_sessions: SessionCenterDialog,
    pub dialog_recovery: RecoveryDialog,
    pub dialog_multi_dir: MultiDirectoryDialog,
    pub dialog_evidence: EvidenceDialog,
    pub dialog_notify_config: NotificationConfigDialog,
    pub dialog_advanced_filters: AdvancedFiltersDialog,
    pub dialog_cleanup: CleanupDialog,
    pub spreadsheet_grid: crate::ui::spreadsheet::SpreadsheetGrid,
    pub modo_escuro_atual: bool,
    pub janela_visivel: bool,

    // Configurações de fluxo de trabalho para Escaneamento e Monitoramento
    pub scan_monitor_config: crate::core::ScanMonitorConfig,
    pub monitorar_apos_escaneamento: bool,
    pub prompt_escanear_aberto: bool,
    pub prompt_monitorar_aberto: bool,
    pub prompt_escanear_lembrar: bool,
    pub prompt_monitorar_lembrar: bool,

    // Configurações e diálogos de personalização da cor de seleção de células
    pub dialogo_cor_selecao_aberto: bool,
    pub cor_selecao_temp: [u8; 3],
    pub prompt_confirmar_aplicar_cor: bool,
    pub prompt_confirmar_restaurar_cor: bool,
}

impl Default for HuginnApp {
    fn default() -> Self {
        Self::new(None)
    }
}

impl HuginnApp {
    pub fn new(diretorio_inicial: Option<PathBuf>) -> Self {
        let inicio_boot = std::time::Instant::now();
        let (event_tx, event_rx) = unbounded::<EventModel>();
        let (progress_tx, progress_rx) = unbounded::<ScanProgress>();

        let base_evento = Arc::new(BaseEvento::new(None));
        let security_manager = Arc::new(SecurityManager::new(None));

        // Carrega absolutamente todos os eventos registrados no banco SQLite local na inicialização
        let eventos = base_evento.carregar_todos_eventos();
        let total_carregados = eventos.len();

        let mut app = Self {
            loc: Localizer::new(),
            base_evento,
            security_manager,
            gerenciador_colunas: GerenciadorColunas::new(),
            gerenciador_tabela: GerenciadorTabela::new(),
            filtros: FiltroEstado::default(),

            diretorio_ativo: diretorio_inicial.clone(),
            observador: None,
            scanner: None,

            eventos,
            event_tx,
            event_rx,

            progress_tx,
            progress_rx,
            progresso_atual: None,

            indices_filtrados: Vec::new(),
            contadores_cache: ContadoresEventos::default(),
            filtros_sujos: true,
            ultimo_update_filtro: std::time::Instant::now(),
            linha_selecionada: None,

            export_colunas_ativas: false,
            export_filtros_ativos: false,
            export_selecao: false,
            ultimo_salvamento: None,

            dialog_about: AboutDialog::default(),
            dialog_help: HelpDialog::default(),
            dialog_manual: ManualDialog::default(),
            dialog_colors: ColorDialog::default(),
            dialog_columns: ColumnDialog::default(),
            dialog_stats: StatisticsDialog::default(),
            dialog_alerts: AlertCenterDialog::default(),
            dialog_sessions: SessionCenterDialog::default(),
            dialog_recovery: RecoveryDialog::default(),
            dialog_multi_dir: MultiDirectoryDialog::default(),
            dialog_evidence: EvidenceDialog::default(),
            dialog_notify_config: NotificationConfigDialog::default(),
            dialog_advanced_filters: AdvancedFiltersDialog::default(),
            dialog_cleanup: CleanupDialog::default(),
            spreadsheet_grid: crate::ui::spreadsheet::SpreadsheetGrid::new_persistent(),
            modo_escuro_atual: crate::utils::detectar_modo_escuro_windows(),
            janela_visivel: false,
            scan_monitor_config: crate::core::ScanMonitorConfig::carregar(),
            monitorar_apos_escaneamento: false,
            prompt_escanear_aberto: false,
            prompt_monitorar_aberto: false,
            prompt_escanear_lembrar: false,
            prompt_monitorar_lembrar: false,
            dialogo_cor_selecao_aberto: false,
            cor_selecao_temp: crate::ui::spreadsheet::cell_selection::COR_SELECAO_PADRAO,
            prompt_confirmar_aplicar_cor: false,
            prompt_confirmar_restaurar_cor: false,
        };

        app.atualizar_indices_filtrados();

        if let Some(ref dir) = diretorio_inicial {
            app.security_manager.configurar_diretorio_primario(dir);
        }

        get_logger().info(&format!(
            "Aplicação inicializada em {} ms ({} eventos carregados do histórico).",
            inicio_boot.elapsed().as_millis(),
            total_carregados
        ));

        app
    }

    pub fn selecionar_diretorio(&mut self) -> Option<PathBuf> {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.parar_monitoramento();
            self.parar_escaneamento();
            self.diretorio_ativo = Some(path.clone());
            self.security_manager.configurar_diretorio_primario(&path);
            Some(path)
        } else {
            None
        }
    }

    pub fn iniciar_observacao(&mut self, dir: PathBuf) {
        self.diretorio_ativo = Some(dir.clone());
        self.security_manager.configurar_diretorio_primario(&dir);
        match Observador::novo(dir, self.security_manager.clone(), self.event_tx.clone()) {
            Ok(obs) => {
                self.observador = Some(obs);
            }
            Err(e) => {
                get_logger().error(&format!("Falha ao iniciar observador: {}", e));
            }
        }
    }

    pub fn iniciar_monitoramento(&mut self) {
        if self.scanner.is_some() {
            self.parar_escaneamento();
        }
        let dir = match self.diretorio_ativo.clone() {
            Some(d) => d,
            None => match self.selecionar_diretorio() {
                Some(d) => d,
                None => return,
            },
        };
        self.iniciar_observacao(dir);
    }

    pub fn parar_monitoramento(&mut self) {
        if let Some(mut obs) = self.observador.take() {
            obs.parar();
        }
    }

    pub fn alternar_pausa(&mut self) {
        if let Some(ref obs) = self.observador {
            if obs.is_pausado() {
                obs.retomar();
            } else {
                obs.pausar();
            }
        }
        if let Some(ref sc) = self.scanner {
            if sc.is_pausado() {
                sc.retomar();
            } else {
                sc.pausar();
            }
        }
    }

    pub fn is_pausado(&self) -> bool {
        self.observador
            .as_ref()
            .map(|o| o.is_pausado())
            .unwrap_or(false)
            || self
                .scanner
                .as_ref()
                .map(|s| s.is_pausado())
                .unwrap_or(false)
    }

    pub fn iniciar_escaneamento(&mut self) {
        // Interrompe qualquer monitoramento em tempo real ativo para que eventos
        // ao vivo (ex.: Modificado, Adicionado) não contaminem a sessão de varredura.
        self.parar_monitoramento();

        let dir = match self.diretorio_ativo.clone() {
            Some(d) => d,
            None => match self.selecionar_diretorio() {
                Some(d) => d,
                None => return,
            },
        };

        if let Some(sc) = self.scanner.take() {
            sc.cancelar();
        }

        // Descarta quaisquer notificações residuais enfileiradas pelo Observador
        while self.event_rx.try_recv().is_ok() {}

        self.security_manager.configurar_diretorio_primario(&dir);

        self.scanner = Some(DiretorioScanner::iniciar_scan(
            dir,
            self.security_manager.clone(),
            self.event_tx.clone(),
            self.progress_tx.clone(),
        ));
    }

    pub fn parar_escaneamento(&mut self) {
        self.monitorar_apos_escaneamento = false;
        if let Some(sc) = self.scanner.take() {
            sc.cancelar();
        }
        self.progresso_atual = None;
    }

    pub fn salvar_configuracao_scan_monitor(&self) {
        self.scan_monitor_config.salvar();
    }

    pub fn acionar_botao_escanear(&mut self) {
        if self.scanner.is_some() {
            self.parar_escaneamento();
        } else if self.scan_monitor_config.perguntar_ao_escanear {
            self.prompt_escanear_aberto = true;
            self.prompt_escanear_lembrar = false;
        } else {
            match self.scan_monitor_config.acao_escaneamento {
                crate::core::ScanActionOption::EscanearEMonitorar => {
                    self.executar_escanear_e_monitorar();
                }
                crate::core::ScanActionOption::ApenasEscanear => {
                    self.executar_apenas_escaneamento();
                }
            }
        }
    }

    pub fn acionar_botao_monitorar(&mut self) {
        if self.observador.is_some() {
            self.parar_monitoramento();
        } else if self.scan_monitor_config.perguntar_ao_monitorar {
            self.prompt_monitorar_aberto = true;
            self.prompt_monitorar_lembrar = false;
        } else {
            match self.scan_monitor_config.acao_monitoramento {
                crate::core::MonitorActionOption::EscanearEDepoisMonitorar => {
                    self.executar_escanear_e_depois_monitorar();
                }
                crate::core::MonitorActionOption::ApenasMonitorar => {
                    self.executar_apenas_monitoramento();
                }
            }
        }
    }

    pub fn executar_apenas_escaneamento(&mut self) {
        self.monitorar_apos_escaneamento = false;
        self.iniciar_escaneamento();
    }

    pub fn executar_escanear_e_monitorar(&mut self) {
        self.monitorar_apos_escaneamento = true;
        self.iniciar_escaneamento();
    }

    pub fn executar_apenas_monitoramento(&mut self) {
        self.monitorar_apos_escaneamento = false;
        self.iniciar_monitoramento();
    }

    pub fn executar_escanear_e_depois_monitorar(&mut self) {
        self.monitorar_apos_escaneamento = true;
        self.iniciar_escaneamento();
    }

    pub fn abrir_dialogo_limpeza(&mut self) {
        self.dialog_cleanup.abrir_selecao();
    }

    pub fn abrir_dialogo_limpeza_alvo(&mut self, alvo: LimpezaAlvo) {
        self.dialog_cleanup.abrir_alvo(alvo);
    }

    pub fn limpar_dados(&mut self) {
        self.abrir_dialogo_limpeza();
    }

    pub fn limpar_apenas_tabela_ui(&mut self) {
        self.eventos.clear();
        self.indices_filtrados.clear();
        self.contadores_cache = crate::tools::filters::ContadoresEventos::default();
        self.filtros_sujos = true;
        self.ultimo_salvamento = None;
    }

    pub fn resetar_caches_dialogos_seguranca(&mut self) {
        self.dialog_recovery.carregado = false;
        self.dialog_multi_dir.carregado = false;
        self.dialog_alerts.cache_alertas.clear();
        self.dialog_alerts.ultimo_total_eventos = usize::MAX;
        self.dialog_sessions.cache_sessoes.clear();
        self.dialog_sessions.ultimo_total_eventos = usize::MAX;
        self.dialog_evidence.ultimo_total_eventos = usize::MAX;
    }

    pub fn executar_limpeza(&mut self, alvo: LimpezaAlvo) -> Result<(), String> {
        match alvo {
            LimpezaAlvo::TabelaUI => {
                self.limpar_apenas_tabela_ui();
                Ok(())
            }
            LimpezaAlvo::BancoDados => {
                if self.scanner.is_some() {
                    self.parar_escaneamento();
                }
                if self.observador.is_some() {
                    self.parar_monitoramento();
                }
                self.base_evento.limpar_banco_eventos()?;
                self.limpar_apenas_tabela_ui();
                Ok(())
            }
            LimpezaAlvo::BancoSeguranca => {
                self.security_manager.limpar_banco_seguranca()?;
                self.resetar_caches_dialogos_seguranca();
                Ok(())
            }
            LimpezaAlvo::Ambos => {
                if self.scanner.is_some() {
                    self.parar_escaneamento();
                }
                if self.observador.is_some() {
                    self.parar_monitoramento();
                }
                let res_ev = self.base_evento.limpar_banco_eventos();
                let res_sec = self.security_manager.limpar_banco_seguranca();
                self.limpar_apenas_tabela_ui();
                self.resetar_caches_dialogos_seguranca();

                if let Err(e) = res_ev {
                    return Err(format!("{}: {}", self.loc.t("clear_main_db_failed"), e));
                }
                if let Err(e) = res_sec {
                    return Err(format!("{}: {}", self.loc.t("clear_security_db_failed"), e));
                }
                Ok(())
            }
        }
    }

    pub fn abrir_salvar_como(&mut self) {
        let default_name = format!(
            "huginn_eventos_{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter(self.loc.t("filter_csv"), &["csv"])
            .add_filter(self.loc.t("filter_json"), &["json"])
            .add_filter(self.loc.t("filter_html"), &["html"])
            .add_filter("TXT (*.txt)", &["txt"])
            .add_filter("XML (*.xml)", &["xml"])
            .save_file()
        {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("csv")
                .to_lowercase();
            self.exportar_para_caminho(&path, &ext);
            self.ultimo_salvamento = Some((path, ext));
        }
    }

    pub fn salvar_dados(&mut self) {
        if let Some((ref path, ref ext)) = self.ultimo_salvamento.clone() {
            self.exportar_para_caminho(path, ext);
        } else {
            self.abrir_salvar_como();
        }
    }

    pub fn exportar_dados(&self) {
        let default_name = format!(
            "huginn_eventos_{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter(self.loc.t("filter_csv"), &["csv"])
            .add_filter(self.loc.t("filter_json"), &["json"])
            .add_filter(self.loc.t("filter_html"), &["html"])
            .add_filter("TXT (*.txt)", &["txt"])
            .add_filter("XML (*.xml)", &["xml"])
            .save_file()
        {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("csv")
                .to_lowercase();
            self.exportar_para_caminho(&path, &ext);
        }
    }

    pub fn resetar_opcoes_exportacao(&mut self) {
        self.export_colunas_ativas = false;
        self.export_filtros_ativos = false;
        self.export_selecao = false;
    }

    pub fn exportar_para_caminho(&self, path: &std::path::Path, formato: &str) {
        let eventos_alvo: Vec<&EventModel> = if self.export_filtros_ativos || self.export_selecao {
            self.indices_filtrados
                .iter()
                .filter_map(|&i| self.eventos.get(i))
                .collect()
        } else {
            self.eventos.iter().collect()
        };

        let colunas = if self.export_colunas_ativas {
            self.gerenciador_colunas.colunas_visiveis()
        } else {
            self.gerenciador_colunas.colunas().iter().collect()
        };

        match formato {
            "json" => {
                let json_data: Vec<serde_json::Value> = eventos_alvo
                    .iter()
                    .map(|ev| {
                        let mut map = serde_json::Map::new();
                        for col in &colunas {
                            map.insert(
                                col.key.to_string(),
                                serde_json::Value::String(ev.get_field(col.key)),
                            );
                        }
                        serde_json::Value::Object(map)
                    })
                    .collect();
                if let Ok(json_str) = serde_json::to_string_pretty(&json_data) {
                    let _ = std::fs::write(path, json_str);
                }
            }
            "html" => {
                let mut html = String::from("<!DOCTYPE html><html><head><meta charset='utf-8'><title>Huginn Sentinel - Relatório de Eventos</title><style>table{border-collapse:collapse;width:100%;font-family:sans-serif;font-size:12px;}th,td{border:1px solid #ddd;padding:6px;text-align:left;}tr:nth-child(even){background:#f9f9f9;}th{background:#333;color:#fff;}</style></head><body><h2>Huginn Sentinel — Relatório de Auditoria</h2><table><thead><tr>");
                for col in &colunas {
                    html.push_str(&format!(
                        "<th>{}</th>",
                        self.loc.get_text(col.translation_key)
                    ));
                }
                html.push_str("</tr></thead><tbody>");
                for ev in &eventos_alvo {
                    html.push_str("<tr>");
                    for col in &colunas {
                        html.push_str(&format!("<td>{}</td>", ev.get_field(col.key)));
                    }
                    html.push_str("</tr>");
                }
                html.push_str("</tbody></table></body></html>");
                let _ = std::fs::write(path, html);
            }
            "txt" => {
                let mut txt = String::new();
                let headers: Vec<String> = colunas
                    .iter()
                    .map(|c| self.loc.get_text(c.translation_key))
                    .collect();
                txt.push_str(&headers.join("\t"));
                txt.push('\n');
                for ev in &eventos_alvo {
                    let row: Vec<String> = colunas.iter().map(|c| ev.get_field(c.key)).collect();
                    txt.push_str(&row.join("\t"));
                    txt.push('\n');
                }
                let _ = std::fs::write(path, txt);
            }
            "xml" => {
                let mut xml =
                    String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<huginn_events>\n");
                for ev in &eventos_alvo {
                    xml.push_str("  <event>\n");
                    for col in &colunas {
                        let val = ev
                            .get_field(col.key)
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;");
                        xml.push_str(&format!("    <{}>{}</{}>\n", col.key, val, col.key));
                    }
                    xml.push_str("  </event>\n");
                }
                xml.push_str("</huginn_events>\n");
                let _ = std::fs::write(path, xml);
            }
            _ => {
                // CSV
                let mut csv = String::new();
                let headers: Vec<String> = colunas
                    .iter()
                    .map(|c| {
                        format!(
                            "\"{}\"",
                            self.loc.get_text(c.translation_key).replace('"', "\"\"")
                        )
                    })
                    .collect();
                csv.push_str(&headers.join(","));
                csv.push('\n');
                for ev in &eventos_alvo {
                    let row: Vec<String> = colunas
                        .iter()
                        .map(|c| format!("\"{}\"", ev.get_field(c.key).replace('"', "\"\"")))
                        .collect();
                    csv.push_str(&row.join(","));
                    csv.push('\n');
                }
                let _ = std::fs::write(path, csv);
            }
        }
    }

    pub fn alternar_analise_diretorio(&mut self) {
        if self.scanner.is_some() {
            self.parar_escaneamento();
        } else if self.observador.is_some() {
            self.parar_monitoramento();
        } else {
            self.iniciar_escaneamento();
        }
    }

    pub fn limpar_banco_seguranca(&mut self) {
        self.abrir_dialogo_limpeza_alvo(LimpezaAlvo::BancoSeguranca);
    }

    pub fn selecionar_todas_colunas(&mut self) {
        for col in self.gerenciador_colunas.colunas().to_vec() {
            self.gerenciador_colunas.set_visibilidade(col.key, true);
        }
        self.gerenciador_colunas.salvar_configuracoes();
        self.filtros_sujos = true;
    }

    pub fn resetar_colunas(&mut self) {
        self.gerenciador_colunas.resetar_padrao();
        self.filtros_sujos = true;
    }

    pub fn selecionar_todas_cores_colunas(&mut self) {
        let keys: Vec<&str> = self
            .gerenciador_colunas
            .colunas()
            .iter()
            .map(|c| c.key)
            .collect();
        self.gerenciador_tabela
            .selecionar_todas_colunas_colorir(&keys);
    }

    pub fn resetar_cores_colunas(&mut self) {
        self.gerenciador_tabela.resetar_colunas_colorir();
    }

    pub fn alternar_ordenacao_linhas(&mut self) {
        let habilitada = self.gerenciador_tabela.alternar_ordenacao_linhas();
        if habilitada {
            // Se ativou a ordenação e há uma coluna selecionada pelo cabeçalho ou célula, adota essa coluna
            let col_idx_opt = self
                .spreadsheet_grid
                .selection
                .selected_cols
                .iter()
                .min()
                .copied()
                .or_else(|| self.spreadsheet_grid.selection.active_cell.map(|ac| ac.col));

            if let Some(col_idx) = col_idx_opt {
                let cols_visiveis = self.gerenciador_colunas.colunas_visiveis();
                if let Some(col) = cols_visiveis.get(col_idx) {
                    self.gerenciador_tabela.coluna_ordenacao = Some(col.key.to_string());
                }
            } else if self.gerenciador_tabela.coluna_ordenacao.is_none() {
                self.gerenciador_tabela.coluna_ordenacao = Some("timestamp".to_string());
            }
        }
        self.filtros_sujos = true;
    }

    pub fn processar_mensagens_background(&mut self, ctx: &egui::Context) {
        // Drenagem de alto desempenho com orçamento de tempo (até 4ms ou 512 eventos por lote)
        // preserva 60 FPS contínuos, esvazia filas com fluidez e não trava as UIs secundárias
        let start_time = std::time::Instant::now();
        let mut buffer = Vec::with_capacity(512);
        while let Ok(evento) = self.event_rx.try_recv() {
            // Se um escaneamento estiver ativamente em andamento, garante estritamente que
            // apenas eventos do tipo Escaneado sejam aceitos e integrados ao histórico.
            if self.scanner.is_some() && evento.op_type() != OperationType::Escaneado {
                continue;
            }
            buffer.push(evento);
            if buffer.len() >= 512 && start_time.elapsed() >= Duration::from_millis(4) {
                break;
            }
        }

        if !buffer.is_empty() {
            let persistidos = self.base_evento.registrar_eventos_lote(buffer);
            let start_idx = self.eventos.len();
            self.eventos.extend(persistidos);

            // Verifica se a ordenação atual é a ordem natural (timestamp decrescente) ou se não há ordenação customizada
            let ordenacao_natural = match self.gerenciador_tabela.coluna_ordenacao.as_deref() {
                None => true,
                Some("timestamp") => !self.gerenciador_tabela.ordem_ascendente,
                _ => false,
            };

            if !self.gerenciador_tabela.ordenacao_habilitada || ordenacao_natural {
                // Atualização incremental ultrarrápida: não re-filtra todo o histórico existente nem reordena
                let q = if self.filtros.texto_busca.is_empty() {
                    None
                } else {
                    Some(self.filtros.texto_busca.to_lowercase())
                };
                let exts: Vec<String> = if self.filtros.filtro_extensao.is_empty() {
                    Vec::new()
                } else {
                    self.filtros
                        .filtro_extensao
                        .split(',')
                        .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                        .filter(|s| !s.is_empty())
                        .collect()
                };

                let mut novos_indices = Vec::new();
                for i in start_idx..self.eventos.len() {
                    let ev = &self.eventos[i];
                    self.contadores_cache.total += 1;
                    match ev.op_type() {
                        OperationType::Adicionado => self.contadores_cache.adicionados += 1,
                        OperationType::Modificado => self.contadores_cache.modificados += 1,
                        OperationType::Excluido => self.contadores_cache.excluidos += 1,
                        OperationType::Renomeado => self.contadores_cache.renomeados += 1,
                        OperationType::Movido => self.contadores_cache.movidos += 1,
                        OperationType::Escaneado => self.contadores_cache.escaneados += 1,
                    }
                    if self.filtro_corresponde(ev, q.as_deref(), &exts) {
                        novos_indices.push(i);
                    }
                }

                if !novos_indices.is_empty() {
                    let shift = novos_indices.len();
                    // Se a opção de acompanhar seleção estiver ativa, desloca os índices selecionados para manter os mesmos registros
                    if self.spreadsheet_grid.selection_config.deve_acompanhar_linhas() {
                        if let Some(ref mut l) = self.linha_selecionada {
                            *l += shift;
                        }
                    }
                    if self.spreadsheet_grid.selection_config.deve_acompanhar_celulas() {
                        self.spreadsheet_grid.selection.shift_rows(shift);
                    }
                    if self.spreadsheet_grid.selection_config.deve_acompanhar_rolagem()
                        && (self.linha_selecionada.is_some()
                            || self.spreadsheet_grid.selection.active_cell.is_some())
                    {
                        self.spreadsheet_grid.request_scroll_to_selection = true;
                    }

                    // Prepend em bloco O(N) com alocação direta em vez de splice quadrático
                    let mut novo_indices = Vec::with_capacity(novos_indices.len() + self.indices_filtrados.len());
                    novo_indices.extend(novos_indices.into_iter().rev());
                    novo_indices.append(&mut self.indices_filtrados);
                    self.indices_filtrados = novo_indices;
                }
            } else {
                // Se ordenação personalizada estiver ativa (ex: por nome, tamanho), marca para reordenar
                self.filtros_sujos = true;
            }

            // Se ainda houver mais eventos na fila, agenda repintura cadenciada a 60 FPS
            // para não saturar a thread principal nem concorrer com as UIs secundárias
            if !self.event_rx.is_empty() {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }

        // Recebe progresso do scanner
        let mut recebeu_prog = false;
        while let Ok(prog) = self.progress_rx.try_recv() {
            if prog.concluido {
                self.scanner = None;
                if self.monitorar_apos_escaneamento {
                    self.monitorar_apos_escaneamento = false;
                    if let Some(dir) = self.diretorio_ativo.clone() {
                        crate::utils::logger::get_logger().info(
                            "Escaneamento concluído com sucesso. Iniciando monitoramento em tempo real..."
                        );
                        self.iniciar_observacao(dir);
                    }
                }
            }
            self.progresso_atual = Some(prog);
            recebeu_prog = true;
        }
        if recebeu_prog {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    pub fn atualizar_indices_filtrados(&mut self) {
        let agora = std::time::Instant::now();
        if self.scanner.is_none() || agora.duration_since(self.ultimo_update_filtro).as_millis() >= 100 {
            self.contadores_cache = self.filtros.calcular_contadores(&self.eventos);
            self.ultimo_update_filtro = agora;
        }

        let q = if self.filtros.texto_busca.is_empty() {
            None
        } else {
            Some(self.filtros.texto_busca.to_lowercase())
        };

        let exts: Vec<String> = if self.filtros.filtro_extensao.is_empty() {
            Vec::new()
        } else {
            self.filtros
                .filtro_extensao
                .split(',')
                .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        };

        // Ordem decrescente padrão (mais recentes no topo)
        let mut indices: Vec<usize> = (0..self.eventos.len())
            .rev()
            .filter(|&idx| {
                let ev = &self.eventos[idx];
                self.filtro_corresponde(ev, q.as_deref(), &exts)
            })
            .collect();

        // Ordenação dinâmica configurada pelo usuário (quando ordenação automática habilitada)
        if self.gerenciador_tabela.ordenacao_habilitada {
            let ordenacao_natural = match self.gerenciador_tabela.coluna_ordenacao.as_deref() {
                None => true,
                Some("timestamp") => !self.gerenciador_tabela.ordem_ascendente,
                _ => false,
            };
            if !ordenacao_natural {
                if let Some(ref col_key) = self.gerenciador_tabela.coluna_ordenacao {
                    let asc = self.gerenciador_tabela.ordem_ascendente;
                    let eventos = &self.eventos;
                    indices.sort_by(|&a, &b| {
                        let val_a = eventos[a].get_field(col_key);
                        let val_b = eventos[b].get_field(col_key);
                        let cmp = if let (Ok(num_a), Ok(num_b)) =
                            (val_a.trim().parse::<f64>(), val_b.trim().parse::<f64>())
                        {
                            num_a
                                .partial_cmp(&num_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            val_a.cmp(&val_b)
                        };
                        if asc {
                            cmp
                        } else {
                            cmp.reverse()
                        }
                    });
                }
            }
        }

        self.indices_filtrados = indices;
        self.filtros_sujos = false;
    }

    fn filtro_corresponde(&self, ev: &EventModel, q: Option<&str>, exts: &[String]) -> bool {
        let op_ok = match ev.op_type() {
            OperationType::Adicionado => self.filtros.op_adicionado,
            OperationType::Modificado => self.filtros.op_modificado,
            OperationType::Excluido => self.filtros.op_excluido,
            OperationType::Renomeado => self.filtros.op_renomeado,
            OperationType::Movido => self.filtros.op_movido,
            OperationType::Escaneado => self.filtros.op_escaneado,
        };
        if !op_ok {
            return false;
        }

        if let Some(query) = q {
            let matches_name = ev.nome.to_lowercase().contains(query);
            let matches_dir = ev.dir_atual.to_lowercase().contains(query);
            let matches_prev = ev.dir_anterior.to_lowercase().contains(query);
            let matches_type = ev.tipo.to_lowercase().contains(query);
            let matches_hash = ev.sha256_atual.to_lowercase().contains(query);

            if !matches_name && !matches_dir && !matches_prev && !matches_type && !matches_hash {
                return false;
            }
        }

        if !exts.is_empty() {
            let ev_ext = ev.nome.rsplit('.').next().unwrap_or("").to_lowercase();
            if !exts.contains(&ev_ext) {
                return false;
            }
        }

        if self.filtros.filtro_risco != "Todos" && !self.filtros.filtro_risco.is_empty() {
            if !ev
                .risk_level
                .eq_ignore_ascii_case(&self.filtros.filtro_risco)
            {
                return false;
            }
        }

        let ignorar_data = match ev.op_type() {
            OperationType::Movido => self.filtros.ignorar_data_mover,
            OperationType::Renomeado => self.filtros.ignorar_data_renomeados,
            OperationType::Adicionado => self.filtros.ignorar_data_adicionados,
            OperationType::Excluido => self.filtros.ignorar_data_excluidos,
            OperationType::Modificado => self.filtros.ignorar_data_modificados,
            OperationType::Escaneado => self.filtros.ignorar_data_escaneados,
        };

        if !ignorar_data {
            if !self.filtros.data_inicio.is_empty() && ev.timestamp < self.filtros.data_inicio {
                return false;
            }
            if !self.filtros.data_fim.is_empty() && ev.timestamp > self.filtros.data_fim {
                return false;
            }
        }

        true
    }

    pub fn abrir_dialogo_ajuda(&mut self) {
        self.dialog_help.abrir();
    }

    pub fn abrir_dialogo_sobre(&mut self) {
        self.dialog_about.abrir();
    }

    pub fn abrir_dialogo_manual(&mut self) {
        self.dialog_manual.abrir();
    }

    pub fn abrir_dialogo_cores(&mut self) {
        self.dialog_colors.abrir();
    }

    pub fn abrir_dialogo_colunas(&mut self) {
        self.dialog_columns.abrir();
    }

    pub fn abrir_dialogo_estatisticas(&mut self) {
        self.dialog_stats.abrir();
    }

    pub fn abrir_dialogo_estatisticas_com_grafico(&mut self, tipo: TipoGrafico) {
        self.dialog_stats.abrir_com_grafico(tipo);
    }

    pub fn abrir_dialogo_alertas(&mut self) {
        self.dialog_alerts.abrir();
    }

    pub fn abrir_dialogo_sessoes(&mut self) {
        self.dialog_sessions.abrir();
    }

    pub fn abrir_dialogo_recuperacao(&mut self) {
        self.dialog_recovery.abrir();
    }

    pub fn abrir_dialogo_multi_dir(&mut self) {
        self.dialog_multi_dir.abrir();
    }

    pub fn abrir_dialogo_evidencias(&mut self) {
        self.dialog_evidence.abrir();
    }

    pub fn abrir_dialogo_notificacoes(&mut self) {
        self.dialog_notify_config.abrir();
    }

    pub fn abrir_dialogo_popup_alertas(&mut self) {
        self.dialog_notify_config.abrir_alerts();
    }

    pub fn abrir_dialogo_popup_sessoes(&mut self) {
        self.dialog_notify_config.abrir_sessions();
    }

    pub fn abrir_dialogo_popup_quarentena(&mut self) {
        self.dialog_notify_config.abrir_quarantine();
    }

    pub fn abrir_dialogo_notificacoes_resumo(&mut self) {
        self.dialog_notify_config.abrir_summary();
    }

    pub fn abrir_dialogo_filtros_avancados(&mut self) {
        self.dialog_advanced_filters.abrir();
    }

    pub fn renderizar_prompts_confirmacao(&mut self, ctx: &egui::Context) {
        if self.prompt_escanear_aberto {
            egui::Window::new(format!("🔍 {}", self.loc.t("scan_prompt_title")))
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(420.0);
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(self.loc.t("scan_prompt_question"))
                            .size(13.0)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    ui.vertical(|ui| {
                        if ui
                            .button(format!("▶ {}", self.loc.t("scan_and_monitor_btn")))
                            .clicked()
                        {
                            if self.prompt_escanear_lembrar {
                                self.scan_monitor_config.perguntar_ao_escanear = false;
                                self.scan_monitor_config.acao_escaneamento =
                                    crate::core::ScanActionOption::EscanearEMonitorar;
                                self.salvar_configuracao_scan_monitor();
                            }
                            self.prompt_escanear_aberto = false;
                            self.executar_escanear_e_monitorar();
                        }

                        ui.add_space(6.0);

                        if ui
                            .button(format!("🔍 {}", self.loc.t("scan_only_btn")))
                            .clicked()
                        {
                            if self.prompt_escanear_lembrar {
                                self.scan_monitor_config.perguntar_ao_escanear = false;
                                self.scan_monitor_config.acao_escaneamento =
                                    crate::core::ScanActionOption::ApenasEscanear;
                                self.salvar_configuracao_scan_monitor();
                            }
                            self.prompt_escanear_aberto = false;
                            self.executar_apenas_escaneamento();
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.prompt_escanear_lembrar,
                            self.loc.t("dont_ask_again"),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(self.loc.t("cancel")).clicked() {
                                self.prompt_escanear_aberto = false;
                            }
                        });
                    });
                });
        }

        if self.prompt_monitorar_aberto {
            egui::Window::new(format!("▶ {}", self.loc.t("monitor_prompt_title")))
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(420.0);
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(self.loc.t("monitor_prompt_question"))
                            .size(13.0)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    ui.vertical(|ui| {
                        if ui
                            .button(format!("🔍 {}", self.loc.t("scan_first_then_monitor_btn")))
                            .clicked()
                        {
                            if self.prompt_monitorar_lembrar {
                                self.scan_monitor_config.perguntar_ao_monitorar = false;
                                self.scan_monitor_config.acao_monitoramento =
                                    crate::core::MonitorActionOption::EscanearEDepoisMonitorar;
                                self.salvar_configuracao_scan_monitor();
                            }
                            self.prompt_monitorar_aberto = false;
                            self.executar_escanear_e_depois_monitorar();
                        }

                        ui.add_space(6.0);

                        if ui
                            .button(format!("▶ {}", self.loc.t("monitor_only_btn")))
                            .clicked()
                        {
                            if self.prompt_monitorar_lembrar {
                                self.scan_monitor_config.perguntar_ao_monitorar = false;
                                self.scan_monitor_config.acao_monitoramento =
                                    crate::core::MonitorActionOption::ApenasMonitorar;
                                self.salvar_configuracao_scan_monitor();
                            }
                            self.prompt_monitorar_aberto = false;
                            self.executar_apenas_monitoramento();
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.prompt_monitorar_lembrar,
                            self.loc.t("dont_ask_again"),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(self.loc.t("cancel")).clicked() {
                                self.prompt_monitorar_aberto = false;
                            }
                        });
                    });
                });
        }

        // Diálogo Completo de Personalização da Cor de Seleção das Células
        if self.dialogo_cor_selecao_aberto {
            egui::Window::new(format!("🎨 {}", self.loc.t("selection_color_dialog_title")))
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(460.0);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(self.loc.t("selection_color_dialog_desc"))
                            .size(12.5),
                    );
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 1. Seletor de cores nativo com amostra e código Hexadecimal
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(self.loc.t("select_color_label")).strong());
                        let mut cur_sel_c = egui::Color32::from_rgb(
                            self.cor_selecao_temp[0],
                            self.cor_selecao_temp[1],
                            self.cor_selecao_temp[2],
                        );
                        let color_resp = crate::ui::widgets::render_custom_color_edit_button(
                            ui,
                            ui.scope_id().with("sel_color_edit_btn"),
                            &mut cur_sel_c,
                            &self.loc,
                        );
                        if color_resp.changed() {
                            self.cor_selecao_temp = [cur_sel_c.r(), cur_sel_c.g(), cur_sel_c.b()];
                        }

                        // Realce em torno da paleta de cor (retângulo) associada a string "Cor de Seleção:" ao passar o ponteiro do mouse (hover)
                        if color_resp.hovered() {
                            let highlight_rect = color_resp.rect.expand(2.5);
                            let highlight_color = if ui.visuals().dark_mode {
                                egui::Color32::from_rgb(100, 190, 255)
                            } else {
                                egui::Color32::from_rgb(20, 110, 230)
                            };
                            ui.painter().rect_stroke(
                                highlight_rect,
                                3_u8,
                                egui::Stroke::new(2.0, highlight_color),
                                egui::StrokeKind::Outside,
                            );
                        }

                        let hex_code = format!(
                            "#{:02X}{:02X}{:02X}",
                            self.cor_selecao_temp[0],
                            self.cor_selecao_temp[1],
                            self.cor_selecao_temp[2]
                        );
                        ui.monospace(hex_code);
                    });

                    ui.add_space(10.0);

                    // 2. Paleta de Cores Recomendadas (Presets Rápidos)
                    ui.label(egui::RichText::new(self.loc.t("recommended_palette_label")).strong());
                    ui.add_space(4.0);

                    let presets: [(&str, [u8; 3]); 7] = [
                        ("color_preset_green", [16, 124, 65]),
                        ("color_preset_blue", [24, 100, 190]),
                        ("color_preset_teal", [0, 140, 160]),
                        ("color_preset_purple", [115, 45, 150]),
                        ("color_preset_orange", [210, 105, 30]),
                        ("color_preset_red", [180, 40, 40]),
                        ("color_preset_gray", [70, 80, 95]),
                    ];

                    ui.horizontal_wrapped(|ui| {
                        for (key, rgb) in presets {
                            let label = self.loc.t(key);
                            let color32 = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                            let is_current = self.cor_selecao_temp == rgb;

                            let btn_text = if is_current {
                                format!("✔ {}", label)
                            } else {
                                label.to_string()
                            };

                            let text_color = if is_current {
                                if ui.visuals().dark_mode {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::BLACK
                                }
                            } else {
                                ui.visuals().text_color()
                            };

                            let mut rich_text = egui::RichText::new(btn_text)
                                .size(11.5)
                                .color(text_color);
                            if is_current {
                                rich_text = rich_text.strong();
                            }

                            let resp = ui.button(rich_text);

                            // Pinta um pequeno indicador de cor no botão
                            let rect = resp.rect;
                            let swatch_rect = egui::Rect::from_min_size(
                                egui::Pos2::new(rect.left() + 3.0, rect.top() + 3.0),
                                egui::Vec2::new(4.0, rect.height() - 6.0),
                            );
                            ui.painter().rect_filled(swatch_rect, 1_u8, color32);

                            if resp.clicked() {
                                self.cor_selecao_temp = rgb;
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 3. Área de Visualização Prévia da Grade
                    ui.label(egui::RichText::new(self.loc.t("selection_color_preview_label")).strong());
                    ui.add_space(4.0);

                    let mut preview_cfg = self.spreadsheet_grid.selection_config.clone();
                    preview_cfg.cor_selecao = self.cor_selecao_temp;
                    let dark_mode = ui.visuals().dark_mode;

                    let preview_h = 32.0;
                    let (preview_rect, _) = ui.allocate_exact_size(
                        egui::Vec2::new(ui.available_width(), preview_h),
                        egui::Sense::hover(),
                    );

                    let half_w = preview_rect.width() / 2.0;
                    let cell1_rect = egui::Rect::from_min_size(
                        preview_rect.min,
                        egui::Vec2::new(half_w, preview_h),
                    );
                    let cell2_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(preview_rect.min.x + half_w, preview_rect.min.y),
                        egui::Vec2::new(half_w, preview_h),
                    );

                    // Célula 1: Não selecionada
                    let c1_bg = if dark_mode {
                        egui::Color32::from_rgb(32, 34, 38)
                    } else {
                        egui::Color32::from_rgb(255, 255, 255)
                    };
                    ui.painter().rect_filled(cell1_rect, 0_u8, c1_bg);
                    crate::ui::spreadsheet::GridRenderer::draw_grid_lines(ui, cell1_rect);
                    ui.painter().text(
                        cell1_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.loc.t("preview_unselected_cell"),
                        egui::FontId::proportional(12.0),
                        ui.visuals().text_color(),
                    );

                    // Célula 2: Selecionada com a cor em teste
                    let c2_bg = preview_cfg.cor_selecao_fundo_celula(dark_mode);
                    let c2_fg = preview_cfg.cor_selecao_texto(dark_mode);
                    ui.painter().rect_filled(cell2_rect, 0_u8, c2_bg);
                    crate::ui::spreadsheet::GridRenderer::draw_active_cell_indicator(
                        ui,
                        cell2_rect,
                        preview_cfg.cor_selecao_color32(),
                    );
                    ui.painter().text(
                        cell2_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.loc.t("preview_selected_cell"),
                        egui::FontId::proportional(12.0),
                        c2_fg,
                    );

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 4. Botões de Ação com apenas os emojis centralizados em relação ao eixo vertical da caixa do botão
                    ui.horizontal(|ui| {
                        if crate::ui::widgets::render_centered_emoji_button(
                            ui,
                            "💾",
                            self.loc.t("apply_and_save_color_btn"),
                        )
                        .clicked()
                        {
                            self.prompt_confirmar_aplicar_cor = true;
                        }

                        if crate::ui::widgets::render_centered_emoji_button(
                            ui,
                            "🔄",
                            self.loc.t("reset_selection_color_btn"),
                        )
                        .clicked()
                        {
                            self.prompt_confirmar_restaurar_cor = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if crate::ui::widgets::render_centered_emoji_button(
                                ui,
                                "❌",
                                self.loc.t("cancel"),
                            )
                            .clicked()
                            {
                                self.cor_selecao_temp = self.spreadsheet_grid.selection_config.cor_selecao;
                                self.dialogo_cor_selecao_aberto = false;
                            }
                        });
                    });
                });
        }

        // Janela de Confirmação: Salvar e Aplicar Nova Cor de Seleção
        if self.prompt_confirmar_aplicar_cor {
            egui::Window::new(format!("💾 {}", self.loc.t("confirm_apply_color_title")))
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(380.0);
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(self.loc.t("confirm_apply_color_msg"))
                            .size(13.0)
                            .strong(),
                    );
                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(format!("✅ {}", self.loc.t("yes")))
                            .clicked()
                        {
                            self.spreadsheet_grid.selection_config.cor_selecao = self.cor_selecao_temp;
                            self.spreadsheet_grid.salvar_configuracao_selecao();
                            self.prompt_confirmar_aplicar_cor = false;
                            self.dialogo_cor_selecao_aberto = false;
                        }

                        ui.add_space(8.0);

                        if ui
                            .button(format!("❌ {}", self.loc.t("no")))
                            .clicked()
                        {
                            self.prompt_confirmar_aplicar_cor = false;
                        }
                    });
                });
        }

        // Janela de Confirmação: Restaurar Cor Padrão de Seleção (Verde Excel)
        if self.prompt_confirmar_restaurar_cor {
            egui::Window::new(format!("🔄 {}", self.loc.t("confirm_reset_color_title")))
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(380.0);
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(self.loc.t("confirm_reset_color_msg"))
                            .size(13.0)
                            .strong(),
                    );
                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(format!("✅ {}", self.loc.t("yes")))
                            .clicked()
                        {
                            self.spreadsheet_grid.selection_config.restaurar_cor_padrao();
                            self.cor_selecao_temp = self.spreadsheet_grid.selection_config.cor_selecao;
                            self.spreadsheet_grid.salvar_configuracao_selecao();
                            self.prompt_confirmar_restaurar_cor = false;
                            self.dialogo_cor_selecao_aberto = false;
                        }

                        ui.add_space(8.0);

                        if ui
                            .button(format!("❌ {}", self.loc.t("no")))
                            .clicked()
                        {
                            self.prompt_confirmar_restaurar_cor = false;
                        }
                    });
                });
        }
    }
}

impl eframe::App for HuginnApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Detecção e Ajuste Dinâmico em Tempo de Execução do Tema do Windows 11 (Claro <-> Escuro)
        let dark_mode_windows = crate::utils::detectar_modo_escuro_windows();
        if self.modo_escuro_atual != dark_mode_windows || ui.visuals().dark_mode != dark_mode_windows {
            self.modo_escuro_atual = dark_mode_windows;
            crate::ui::widgets::aplicar_tema_visuals(&ctx, dark_mode_windows);
            ctx.request_repaint();
        }

        // Revela a janela apenas após o primeiro quadro estar renderizado com o tema aplicado,
        // centralizando perfeitamente a janela no display principal antes de torná-la visível
        if !self.janela_visivel {
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(&ctx) {
                ctx.send_viewport_cmd(cmd);
            }
            self.janela_visivel = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }

        // Configuração de Estilo Global: Garante que emojis (📋, ⚙️, 🗑️, ⏸️, ▶️, 💾) não estourem os limites dos botões e widgets
        ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.5);
        ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 5.0);
        let cur_interact_y = ui.style_mut().spacing.interact_size.y;
        ui.style_mut().spacing.interact_size.y = cur_interact_y.max(26.0);
        ui.style_mut().spacing.menu_margin = egui::Margin::same(6);

        self.processar_mensagens_background(&ctx);
        crate::ui::widgets::iniciar_rastreamento_menus_frame(&ctx);

        // Tratamento de Atalhos Globais de Teclado
        let mut acao_ajuda = false;
        let mut acao_manual = false;
        let mut acao_sobre = false;
        let mut acao_selecionar_dir = false;
        let mut acao_iniciar_parar = false;
        let mut acao_pausar = false;
        let mut acao_salvar_como = false;
        let mut acao_salvar = false;
        let mut acao_estatisticas = false;
        let mut acao_limpar = false;
        let mut acao_sair = false;
        let mut acao_selecionar_todas_colunas = false;
        let mut acao_selecionar_todas_cores = false;
        let mut acao_alternar_ordenacao = false;

        ui.input(|i| {
            let ctrl = i.modifiers.command || i.modifiers.ctrl;
            let shift = i.modifiers.shift;

            if i.key_pressed(egui::Key::F1) {
                acao_ajuda = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::M) {
                acao_manual = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::I) {
                acao_sobre = true;
            }
            if ctrl && shift && i.key_pressed(egui::Key::O) {
                acao_selecionar_dir = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::R) {
                acao_iniciar_parar = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::P) {
                acao_pausar = true;
            }
            if ctrl && shift && i.key_pressed(egui::Key::S) {
                acao_salvar_como = true;
            } else if ctrl && !shift && i.key_pressed(egui::Key::S) {
                acao_salvar = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::G) {
                acao_estatisticas = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::L) {
                acao_limpar = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::Q) {
                acao_sair = true;
            }
            if ctrl && shift && i.key_pressed(egui::Key::A) {
                acao_selecionar_todas_cores = true;
            } else if ctrl && !shift && i.key_pressed(egui::Key::A) {
                acao_selecionar_todas_colunas = true;
            }
            if ctrl && !shift && i.key_pressed(egui::Key::O) {
                acao_alternar_ordenacao = true;
            }
        });

        if acao_ajuda {
            self.abrir_dialogo_ajuda();
        }
        if acao_manual {
            self.abrir_dialogo_manual();
        }
        if acao_sobre {
            self.abrir_dialogo_sobre();
        }
        if acao_selecionar_dir {
            self.selecionar_diretorio();
        }
        if acao_iniciar_parar {
            self.alternar_analise_diretorio();
        }
        if acao_pausar {
            self.alternar_pausa();
        }
        if acao_salvar_como {
            self.abrir_salvar_como();
        }
        if acao_salvar {
            self.salvar_dados();
        }
        if acao_estatisticas {
            self.abrir_dialogo_estatisticas();
        }
        if acao_limpar {
            self.limpar_dados();
        }
        if acao_sair {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if acao_selecionar_todas_colunas {
            self.selecionar_todas_colunas();
        }
        if acao_selecionar_todas_cores {
            self.selecionar_todas_cores_colunas();
        }
        if acao_alternar_ordenacao {
            self.alternar_ordenacao_linhas();
        }

        let tempo_desde_filtro = self.ultimo_update_filtro.elapsed();
        let deve_atualizar_filtro = self.filtros_sujos
            && (tempo_desde_filtro >= Duration::from_millis(50) || self.event_rx.is_empty());
        if deve_atualizar_filtro {
            self.atualizar_indices_filtrados();
            self.ultimo_update_filtro = std::time::Instant::now();
        }

        // Repintura cadenciada a 60 FPS (~16ms) se houver eventos pendentes,
        // repintura suave (33ms) durante monitoramento/escaneamento ativo, ou a cada 200ms em repouso.
        // Evita saturação da thread principal e elimina concorrência com as janelas secundárias.
        if !self.event_rx.is_empty() || !self.progress_rx.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.observador.is_some() || self.scanner.is_some() {
            ctx.request_repaint_after(Duration::from_millis(33));
        } else {
            ctx.request_repaint_after(Duration::from_millis(200));
        }

        // Painel Superior: Menu Bar + Toolbar
        egui::Panel::top("top_panel").show(ui, |ui| {
            renderizar_menu_bar(self, ui);
            renderizar_toolbar(self, ui);
        });

        // Painel Inferior: Barra de Status, Progresso e Gerenciador Z-Order de Janelas Modeless
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(ref prog) = self.progresso_atual {
                    if !prog.concluido {
                        ui.label(self.loc.t("scan_label"));
                        ui.add(ProgressBar::new(prog.progresso_pct / 100.0).text(format!(
                            "{:.1}% ({}/{})",
                            prog.progresso_pct, prog.itens_escaneados, prog.total_estimado
                        )));
                        ui.label(
                            RichText::new(&prog.item_atual)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                }

                let total_ev_str = self.eventos.len().to_string();
                ui.label(self.loc.t_args("events_recorded_count", &[&total_ev_str]));

                let status_monitor = if self.observador.is_some() {
                    if self.is_pausado() {
                        RichText::new(format!("● {}", self.loc.t("status_paused")))
                            .color(Color32::from_rgb(255, 179, 0))
                    } else {
                        RichText::new(format!("● {}", self.loc.t("status_active")))
                            .color(Color32::from_rgb(76, 175, 80))
                    }
                } else {
                    RichText::new(format!("○ {}", self.loc.t("status_stopped")))
                        .color(ui.visuals().weak_text_color())
                };
                ui.label(status_monitor);

                ui.separator();

                // Gerenciador Modeless Z-Order / Barra de Tarefas de Janelas Ativas
                let mut janelas_ativas = Vec::new();
                if self.dialog_advanced_filters.aberto {
                    janelas_ativas.push(("🔍", self.loc.t("advanced_filters"), 0));
                }
                if self.dialog_stats.aberto {
                    janelas_ativas.push(("📊", self.loc.t("statistics"), 1));
                }
                if self.dialog_alerts.aberto {
                    janelas_ativas.push(("🛡️", self.loc.t("alert_center"), 2));
                }
                if self.dialog_sessions.aberto {
                    janelas_ativas.push(("⏱️", self.loc.t("sessions"), 3));
                }
                if self.dialog_columns.aberto {
                    janelas_ativas.push(("📑", self.loc.t("configure_columns"), 4));
                }
                if self.dialog_colors.aberto {
                    janelas_ativas.push(("🎨", self.loc.t("colors_config"), 5));
                }
                if self.dialog_recovery.aberto {
                    janelas_ativas.push(("🔒", self.loc.t("recovery"), 6));
                }
                if self.dialog_multi_dir.aberto {
                    janelas_ativas.push(("📁", self.loc.t("multi_dir"), 7));
                }
                if self.dialog_evidence.aberto {
                    janelas_ativas.push(("📦", self.loc.t("evidence_export"), 8));
                }
                if self.dialog_notify_config.alerts_popup.aberto {
                    janelas_ativas.push(("⚠️", self.loc.t("security_alert_popup_notifications"), 9));
                }
                if self.dialog_notify_config.sessions_popup.aberto {
                    janelas_ativas.push(("⏱️", self.loc.t("security_session_popup_notifications"), 13));
                }
                if self.dialog_notify_config.quarantine_popup.aberto {
                    janelas_ativas.push(("🔒", self.loc.t("security_quarantine_popup_notifications"), 14));
                }
                if self.dialog_notify_config.summary_popup.aberto {
                    janelas_ativas.push(("📢", self.loc.t("security_summary_notifications"), 15));
                }
                if self.dialog_manual.aberto {
                    janelas_ativas.push(("📖", self.loc.t("manual"), 10));
                }
                if self.dialog_help.aberto {
                    janelas_ativas.push(("❓", self.loc.t("help"), 11));
                }
                if self.dialog_about.aberto {
                    janelas_ativas.push(("ℹ️", self.loc.t("about"), 12));
                }
                if self.dialog_cleanup.aberto {
                    janelas_ativas.push(("🗑️", self.loc.t("cleanup_system_title"), 16));
                }

                if !janelas_ativas.is_empty() {
                    let qtd_janelas = janelas_ativas.len();
                    let qtd_str = qtd_janelas.to_string();
                    let label_janelas = if qtd_janelas == 1 {
                        self.loc.t_args("active_windows_count_one", &[&qtd_str])
                    } else {
                        self.loc.t_args("active_windows_count_many", &[&qtd_str])
                    };
                    ui.label(
                        RichText::new(label_janelas)
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                    for (emoji, label, id) in janelas_ativas {
                        if crate::ui::widgets::render_centered_emoji_button(ui, emoji, &label).clicked() {
                            match id {
                                0 => {
                                    self.dialog_advanced_filters.abrir();
                                }
                                1 => {
                                    self.dialog_stats.abrir();
                                }
                                2 => {
                                    self.dialog_alerts.abrir();
                                }
                                3 => {
                                    self.dialog_sessions.abrir();
                                }
                                4 => {
                                    self.dialog_columns.abrir();
                                }
                                5 => {
                                    self.dialog_colors.abrir();
                                }
                                6 => {
                                    self.dialog_recovery.abrir();
                                }
                                7 => {
                                    self.dialog_multi_dir.abrir();
                                }
                                8 => {
                                    self.dialog_evidence.abrir();
                                }
                                9 => {
                                    self.dialog_notify_config.abrir_alerts();
                                }
                                10 => {
                                    self.dialog_manual.abrir();
                                }
                                11 => {
                                    self.dialog_help.abrir();
                                }
                                12 => {
                                    self.dialog_about.abrir();
                                }
                                13 => {
                                    self.dialog_notify_config.abrir_sessions();
                                }
                                14 => {
                                    self.dialog_notify_config.abrir_quarantine();
                                }
                                15 => {
                                    self.dialog_notify_config.abrir_summary();
                                }
                                16 => {
                                    self.dialog_cleanup.abrir_selecao();
                                }
                                _ => {}
                            }
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let lang_str = self
                        .loc
                        .t_args("language_status", &[&self.loc.idioma_atual()]);
                    ui.label(RichText::new(lang_str).size(11.0).color(ui.visuals().weak_text_color()));
                    let local_first_color = if ui.visuals().dark_mode {
                        Color32::from_rgb(100, 181, 246)
                    } else {
                        Color32::from_rgb(25, 118, 210)
                    };
                    ui.label(
                        RichText::new(self.loc.t("local_first_status"))
                            .size(11.0)
                            .color(local_first_color),
                    );
                });
            });
        });

        // Painel Esquerdo: Filtros e Contadores
        egui::Panel::left("filter_side_panel")
            .default_size(220.0)
            .size_range(180.0..=320.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    renderizar_painel_filtros(self, ui);
                });
            });

        // Painel Central: Tabela Virtual de Eventos
        egui::CentralPanel::default().show(ui, |ui| {
            renderizar_tabela_eventos(self, ui);
        });

        // Exibição dos diálogos modais independentes
        self.dialog_about.mostrar(&ctx, &self.loc);
        self.dialog_help.mostrar(&ctx, &self.loc);
        self.dialog_manual.mostrar(&ctx, &self.loc);
        self.dialog_colors
            .mostrar(&ctx, &self.loc, &mut self.gerenciador_tabela);
        self.dialog_columns
            .mostrar(&ctx, &self.loc, &mut self.gerenciador_colunas);
        self.dialog_stats.mostrar(&ctx, &self.loc, &self.eventos);
        self.dialog_alerts.mostrar(&ctx, &self.loc, &self.eventos);
        self.dialog_sessions.mostrar(&ctx, &self.loc, &self.eventos);
        self.dialog_recovery
            .mostrar(&ctx, &self.loc, &self.security_manager);
        self.dialog_multi_dir
            .mostrar(&ctx, &self.loc, &self.security_manager.multi_dir());
        self.dialog_evidence
            .mostrar(&ctx, &self.loc, &self.security_manager, &self.eventos);
        self.dialog_notify_config.mostrar(&ctx, &self.loc);
        self.dialog_advanced_filters.mostrar(
            &ctx,
            &self.loc,
            &mut self.filtros,
            &mut self.filtros_sujos,
            &self.contadores_cache,
            &self.gerenciador_tabela,
        );

        // Exibição do diálogo do sistema de limpeza de dados
        let mut acao_limpeza = None;
        self.dialog_cleanup.mostrar(&ctx, &self.loc, |alvo| {
            acao_limpeza = Some(alvo);
        });
        if let Some(alvo) = acao_limpeza {
            let res = self.executar_limpeza(alvo);
            self.dialog_cleanup.processar_resultado(alvo, res);
        }

        // Exibição dos prompts modais interativos de confirmação de fluxo
        self.renderizar_prompts_confirmacao(&ctx);

        crate::ui::widgets::finalizar_rastreamento_menus_frame(&ctx);
    }
}
