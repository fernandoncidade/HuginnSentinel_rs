#![windows_subsystem = "windows"]

use eframe::NativeOptions;
use std::path::PathBuf;

use huginn_rs::ui::HuginnApp;
use huginn_rs::utils::icons::carregar_icone_janela;
use huginn_rs::utils::logger::get_logger;

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        windows_sys::Win32::System::Console::FreeConsole();
    }

    get_logger().info("Iniciando Huginn Sentinel Desktop (Rust/egui)...");

    let args: Vec<String> = std::env::args().collect();
    let pasta_inicial = if args.len() > 1 {
        let p = PathBuf::from(&args[1]);
        if p.exists() && p.is_dir() {
            Some(p)
        } else {
            None
        }
    } else {
        None
    };

    let mut native_options = NativeOptions::default();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Huginn Sentinel — Auditoria & Monitoramento de Arquivos")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([960.0, 600.0])
        .with_visible(false)
        .with_active(true);

    #[cfg(target_os = "windows")]
    {
        unsafe {
            let cx = windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows_sys::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
            );
            let cy = windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows_sys::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
            );
            if cx > 0 && cy > 0 {
                let x = ((cx as f32 - 1280.0) * 0.5).max(0.0);
                let y = ((cy as f32 - 800.0) * 0.5).max(0.0);
                viewport = viewport.with_position([x, y]);
            }
        }
    }

    if let Some(icon) = carregar_icone_janela() {
        viewport = viewport.with_icon(icon);
    }
    native_options.viewport = viewport;

    eframe::run_native(
        "Huginn Sentinel",
        native_options,
        Box::new(|_cc| Ok(Box::new(HuginnApp::new(pasta_inicial)))),
    )
}
