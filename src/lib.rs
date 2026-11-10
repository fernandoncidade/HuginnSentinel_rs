pub mod core;
pub mod data;
pub mod locale;
pub mod security;
pub mod services;
pub mod tools;
pub mod ui;
pub mod utils;

pub use core::event::EventModel;
pub use core::types::{FileCategory, OperationType, RiskLevel};
pub use locale::Localizer;
pub use security::SecurityManager;
pub use services::BaseEvento;
pub use ui::HuginnApp;

// Alias de conveniência para suportar tanto locale quanto language
pub use locale as language;
