use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Adicionado,
    Modificado,
    Excluido,
    Renomeado,
    Movido,
    Escaneado,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Adicionado => "adicionado",
            OperationType::Modificado => "modificado",
            OperationType::Excluido => "excluido",
            OperationType::Renomeado => "renomeado",
            OperationType::Movido => "movido",
            OperationType::Escaneado => "escaneado",
        }
    }

    pub fn display_pt(&self) -> &'static str {
        match self {
            OperationType::Adicionado => "Adicionado",
            OperationType::Modificado => "Modificado",
            OperationType::Excluido => "Excluído",
            OperationType::Renomeado => "Renomeado",
            OperationType::Movido => "Movido",
            OperationType::Escaneado => "Escaneado",
        }
    }

    pub fn from_raw(s: &str) -> Self {
        let clean = s.trim().to_lowercase();
        if clean.contains("adic") || clean.contains("add") || clean.contains("created") {
            OperationType::Adicionado
        } else if clean.contains("mod") || clean.contains("edit") || clean.contains("chang") {
            OperationType::Modificado
        } else if clean.contains("excl") || clean.contains("del") || clean.contains("remov") {
            OperationType::Excluido
        } else if clean.contains("renom") || clean.contains("rename") {
            OperationType::Renomeado
        } else if clean.contains("mov") || clean.contains("move") {
            OperationType::Movido
        } else if clean.contains("escan") || clean.contains("scan") {
            OperationType::Escaneado
        } else {
            OperationType::Modificado
        }
    }

    pub fn default_color_hex(&self) -> (&'static str, &'static str) {
        // (background_hex, foreground_hex)
        match self {
            OperationType::Adicionado => ("#2e7d32", "#ffffff"), // Green
            OperationType::Modificado => ("#1565c0", "#ffffff"), // Blue
            OperationType::Excluido => ("#c62828", "#ffffff"),   // Red
            OperationType::Renomeado => ("#6a1b9a", "#ffffff"),  // Purple
            OperationType::Movido => ("#ef6c00", "#ffffff"),     // Orange
            OperationType::Escaneado => ("#37474f", "#ffffff"),  // Slate Blue/Grey
        }
    }

    pub fn all() -> &'static [OperationType] {
        &[
            OperationType::Adicionado,
            OperationType::Modificado,
            OperationType::Excluido,
            OperationType::Renomeado,
            OperationType::Movido,
            OperationType::Escaneado,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Baixo,
    Medio,
    Alto,
    Critico,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Baixo => "Baixo",
            RiskLevel::Medio => "Médio",
            RiskLevel::Alto => "Alto",
            RiskLevel::Critico => "Crítico",
        }
    }

    pub fn from_score(score: i32) -> Self {
        if score >= 85 {
            RiskLevel::Critico
        } else if score >= 60 {
            RiskLevel::Alto
        } else if score >= 30 {
            RiskLevel::Medio
        } else {
            RiskLevel::Baixo
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s = s.trim().to_lowercase();
        if s.contains("crit") {
            RiskLevel::Critico
        } else if s.contains("alt") || s.contains("high") {
            RiskLevel::Alto
        } else if s.contains("med") {
            RiskLevel::Medio
        } else {
            RiskLevel::Baixo
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            RiskLevel::Baixo => "#4caf50",
            RiskLevel::Medio => "#ff9800",
            RiskLevel::Alto => "#f44336",
            RiskLevel::Critico => "#b71c1c",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileCategory {
    CodigoFonte,
    Imagem,
    Audio,
    Video,
    Documento,
    Planilha,
    Apresentacao,
    BancoDados,
    Executavel,
    Temporario,
    Compactado,
    Backup,
    Log,
    Config,
    Estruturado,
    Generico,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileCategory::CodigoFonte => "Código Fonte",
            FileCategory::Imagem => "Imagem",
            FileCategory::Audio => "Áudio",
            FileCategory::Video => "Vídeo",
            FileCategory::Documento => "Documento",
            FileCategory::Planilha => "Planilha",
            FileCategory::Apresentacao => "Apresentação",
            FileCategory::BancoDados => "Banco de Dados",
            FileCategory::Executavel => "Executável",
            FileCategory::Temporario => "Temporário",
            FileCategory::Compactado => "Compactado",
            FileCategory::Backup => "Backup",
            FileCategory::Log => "Log",
            FileCategory::Config => "Configuração",
            FileCategory::Estruturado => "Dados Estruturados",
            FileCategory::Generico => "Arquivo Genérico",
        }
    }
}
