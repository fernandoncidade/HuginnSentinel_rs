use chrono::Local;

pub struct AdministradorCalendario;

impl AdministradorCalendario {
    pub fn hoje_str() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    pub fn inicio_do_dia_str(data: &str) -> String {
        format!("{} 00:00:00", data)
    }

    pub fn fim_do_dia_str(data: &str) -> String {
        format!("{} 23:59:59", data)
    }
}
