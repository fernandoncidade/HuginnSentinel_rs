// Detecção Dinâmica de Tema Windows 11 (Claro / Escuro)
// Consulta a chave do registro do Windows:
// HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize -> AppsUseLightTheme
// Utiliza cache atômico com cadência de 1,5s para eliminar contenção e chamadas repetitivas de I/O ao registro

#[cfg(target_os = "windows")]
static CACHE_MODO_ESCURO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
#[cfg(target_os = "windows")]
static ULTIMA_CONSULTA_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(target_os = "windows")]
pub fn detectar_modo_escuro_windows() -> bool {
    use std::ptr::null_mut;
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    };

    let agora_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let ultima = ULTIMA_CONSULTA_MS.load(Ordering::Relaxed);
    if ultima > 0 && agora_ms.saturating_sub(ultima) < 1500 {
        return CACHE_MODO_ESCURO.load(Ordering::Relaxed);
    }

    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    let mut hkey: HKEY = null_mut();
    let resultado = unsafe {
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        ) == 0
        {
            let mut val: u32 = 0;
            let mut val_len: u32 = std::mem::size_of::<u32>() as u32;
            let status = RegQueryValueExW(
                hkey,
                value_name.as_ptr(),
                null_mut(),
                null_mut(),
                &mut val as *mut u32 as *mut u8,
                &mut val_len,
            );
            RegCloseKey(hkey);
            if status == 0 {
                // AppsUseLightTheme == 1 -> Claro; == 0 -> Escuro
                val == 0
            } else {
                true
            }
        } else {
            true
        }
    };

    CACHE_MODO_ESCURO.store(resultado, Ordering::Relaxed);
    ULTIMA_CONSULTA_MS.store(agora_ms, Ordering::Relaxed);
    resultado
}

#[cfg(not(target_os = "windows"))]
pub fn detectar_modo_escuro_windows() -> bool {
    true
}
