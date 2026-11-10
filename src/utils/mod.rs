pub mod app_association;
pub mod icons;
pub mod logger;
pub mod paths;
pub mod theme;

pub use app_association::{abrir_arquivo_com_associado, selecionar_no_explorer};
pub use icons::{carregar_icone_janela, obter_caminho_icone};
pub use logger::{get_logger, LogManager};
pub use paths::{
    obter_caminho_db_eventos, obter_caminho_db_seguranca, obter_caminho_persistente,
    obter_diretorio_base, resolver_caminho_asset,
};
pub use theme::detectar_modo_escuro_windows;
