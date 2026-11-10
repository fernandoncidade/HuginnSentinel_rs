use std::sync::OnceLock;

pub struct AtribuicaoForense;

static CONTEXTO_CACHE: OnceLock<(String, String, i32)> = OnceLock::new();

impl AtribuicaoForense {
    pub fn capturar_contexto() -> (String, String, i32) {
        CONTEXTO_CACHE
            .get_or_init(|| {
                let user = std::env::var("USERNAME")
                    .or_else(|_| std::env::var("USER"))
                    .unwrap_or_else(|_| "desconhecido".to_string());

                let pid = std::process::id() as i32;

                let process_name = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
                    .unwrap_or_else(|| "huginnsentinel.exe".to_string());

                (user, process_name, pid)
            })
            .clone()
    }
}
