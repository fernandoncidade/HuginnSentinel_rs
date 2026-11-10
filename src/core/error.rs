use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    Notify(notify::Error),
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "Erro de E/S: {}", e),
            AppError::Database(e) => write!(f, "Erro de Banco de Dados: {}", e),
            AppError::Serialization(e) => write!(f, "Erro de Serialização: {}", e),
            AppError::Notify(e) => write!(f, "Erro de Monitoramento de Arquivos: {}", e),
            AppError::Generic(e) => write!(f, "Erro: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err)
    }
}

impl From<notify::Error> for AppError {
    fn from(err: notify::Error) -> Self {
        AppError::Notify(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Generic(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Generic(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
