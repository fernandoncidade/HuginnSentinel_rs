pub mod localizer;
pub mod manual;

pub use localizer::{Localizer, IDIOMAS_SUPORTADOS};
pub use manual::{obter_secoes_manual, ManualDetails, ManualSection};
