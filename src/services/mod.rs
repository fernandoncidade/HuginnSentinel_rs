pub mod base_evento;
pub mod db_writer;
pub mod diretorio_scanner;
pub mod gerenciador_colunas;
pub mod gerenciador_tabela;
pub mod metadados;
pub mod observador;

pub use base_evento::BaseEvento;
pub use db_writer::DbWriter;
pub use diretorio_scanner::{DiretorioScanner, ScanProgress};
pub use gerenciador_colunas::{ColunaInfo, GerenciadorColunas};
pub use gerenciador_tabela::{
    calcular_cor_texto_ideal, color_to_hex, parse_hex_color, GerenciadorTabela,
};
pub use observador::Observador;
