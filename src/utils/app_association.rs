use std::path::Path;
use std::process::Command;

pub fn abrir_arquivo_com_associado(caminho: &str) -> bool {
    let path = Path::new(caminho);
    if !path.exists() {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let wide_path: Vec<u16> = OsStr::new(caminho)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let wide_open: Vec<u16> = OsStr::new("open")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let res = windows_sys::Win32::UI::Shell::ShellExecuteW(
                0 as _,
                wide_open.as_ptr(),
                wide_path.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL as i32,
            );
            if (res as isize) > 32 {
                return true;
            }
        }
    }

    // Fallback: spawn command
    #[cfg(target_os = "windows")]
    {
        if Command::new("cmd")
            .args(["/c", "start", "", caminho])
            .spawn()
            .is_ok()
        {
            return true;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if Command::new("xdg-open").arg(caminho).spawn().is_ok() {
            return true;
        }
    }

    false
}

pub fn selecionar_no_explorer(caminho: &str) -> bool {
    let path = Path::new(caminho);
    if !path.exists() {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,\"{}\"", caminho))
            .spawn()
            .is_ok()
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = path.parent() {
            Command::new("xdg-open").arg(parent).spawn().is_ok()
        } else {
            false
        }
    }
}
