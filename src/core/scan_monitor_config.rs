use serde::{Deserialize, Serialize};

use crate::utils::paths::obter_caminho_persistente;

/// Ação padrão ao acionar o botão ou menu de escaneamento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScanActionOption {
    /// Apenas realizar o escaneamento do diretório selecionado
    #[default]
    ApenasEscanear,
    /// Iniciar o monitoramento assim que terminar o escaneamento
    EscanearEMonitorar,
}

/// Ação padrão ao acionar o botão ou menu de monitoramento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MonitorActionOption {
    /// Apenas realizar o monitoramento em tempo real
    #[default]
    ApenasMonitorar,
    /// Escanear primeiro e depois iniciar o monitoramento
    EscanearEDepoisMonitorar,
}

/// Configuração de comportamento e avisos para escaneamento e monitoramento
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanMonitorConfig {
    /// Se ativado, exibe mensagem/pergunta logo após o usuário clicar em "🔍 Escanear"
    pub perguntar_ao_escanear: bool,
    /// Ação padrão de escaneamento
    pub acao_escaneamento: ScanActionOption,

    /// Se ativado, exibe mensagem/pergunta logo após o usuário clicar em "▶️ Iniciar Monitoramento"
    pub perguntar_ao_monitorar: bool,
    /// Ação padrão de monitoramento
    pub acao_monitoramento: MonitorActionOption,
}

impl Default for ScanMonitorConfig {
    fn default() -> Self {
        Self {
            perguntar_ao_escanear: true,
            acao_escaneamento: ScanActionOption::ApenasEscanear,
            perguntar_ao_monitorar: true,
            acao_monitoramento: MonitorActionOption::ApenasMonitorar,
        }
    }
}

impl ScanMonitorConfig {
    /// Salva as configurações de escaneamento e monitoramento em `%LOCALAPPDATA%\HuginnSentinel\scan_monitor_config.json`
    pub fn salvar(&self) {
        let p = obter_caminho_persistente().join("scan_monitor_config.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, json);
        }
    }

    /// Carrega as configurações de escaneamento e monitoramento ou retorna os valores padrão
    pub fn carregar() -> Self {
        let p = obter_caminho_persistente().join("scan_monitor_config.json");
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(cfg) = serde_json::from_slice::<Self>(&bytes) {
                return cfg;
            }
        }
        Self::default()
    }
}

