use super::paths::resolver_caminho_asset;
use std::path::PathBuf;
use std::sync::OnceLock;

static ICON_DATA_CACHE: OnceLock<Option<egui::IconData>> = OnceLock::new();

pub fn obter_caminho_icone(nome_icone: &str) -> Option<PathBuf> {
    if let Some(p) = resolver_caminho_asset(nome_icone) {
        return Some(p);
    }
    let in_icons = format!("images/icons/{}", nome_icone);
    if let Some(p) = resolver_caminho_asset(&in_icons) {
        return Some(p);
    }
    let in_imgs = format!("images/imgs/{}", nome_icone);
    if let Some(p) = resolver_caminho_asset(&in_imgs) {
        return Some(p);
    }
    None
}

pub fn carregar_icone_janela() -> Option<egui::IconData> {
    ICON_DATA_CACHE
        .get_or_init(|| {
            if let Some(path) = obter_caminho_icone("huginn.ico") {
                if let Ok(img) = image::open(&path) {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    return Some(egui::IconData {
                        rgba: rgba.into_raw(),
                        width,
                        height,
                    });
                }
            }
            if let Ok(img) = image::load_from_memory(include_bytes!("../assets/images/icons/huginn.ico")) {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                return Some(egui::IconData {
                    rgba: rgba.into_raw(),
                    width,
                    height,
                });
            }
            None
        })
        .clone()
}
