use std::io::Read;
use std::path::Path;

use crate::services::metadados::executables::{
    extrair_pe_version_string, verificar_pe_assinado,
};
use crate::services::metadados::extractor::abrir_arquivo_compartilhado;
use crate::services::metadados::extensao_tipo_preferencial;
use crate::services::metadados::types::{is_executable_ext, is_font_ext, is_system_ext};

#[derive(Debug, Clone)]
pub struct ExecutableIntelResult {
    pub is_executable: bool,
    pub is_signed: bool,
    pub publisher: String,
    pub signature_state: String,
    pub script_risk: String,
    pub sensitive_directory: bool,
}

pub fn analisar_executavel(caminho: &Path, autor_fallback: &str) -> ExecutableIntelResult {
    let ext = extensao_tipo_preferencial(caminho);
    let dir_lower = caminho
        .parent()
        .map(|p| p.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let sensitive_directory = dir_lower.contains("windows\\system32")
        || dir_lower.contains("windows\\syswow64")
        || dir_lower.contains("program files")
        || dir_lower.contains("appdata\\roaming\\microsoft\\windows\\start menu\\programs\\startup")
        || dir_lower.contains("appdata\\local\\temp")
        || dir_lower.ends_with("\\temp")
        || dir_lower.contains("/tmp");

    let is_script = matches!(
        ext.as_str(),
        "ps1" | "psm1" | "psd1" | "bat" | "cmd" | "vbs" | "vbe" | "wsf" | "wsh" | "hta" | "js"
            | "jse" | "sh" | "bash" | "py" | "rb" | "pl" | "lua"
    );
    let is_exec = is_executable_ext(&ext) || is_system_ext(&ext) || is_font_ext(&ext) || is_script;

    let mut script_risk = "none".to_string();
    if is_script && caminho.exists() && caminho.is_file() {
        script_risk = avaliar_risco_script(caminho);
    }

    let mut is_signed = false;
    let mut publisher = String::new();

    if (is_executable_ext(&ext) || is_system_ext(&ext)) && caminho.exists() && caminho.is_file() {
        if let Ok(mut file) = abrir_arquivo_compartilhado(caminho) {
            let mut buf = vec![0u8; 262_144];
            if let Ok(n) = file.read(&mut buf) {
                buf.truncate(n);
                publisher = extrair_pe_version_string(&buf, "CompanyName")
                    .or_else(|| extrair_pe_version_string(&buf, "ProductName"))
                    .unwrap_or_default();
                is_signed = verificar_pe_assinado(&buf);
            }
        }

        #[cfg(target_os = "windows")]
        if matches!(ext.as_str(), "exe" | "dll" | "sys" | "msi" | "ocx" | "cpl" | "scr") {
            if verificar_wintrust(caminho) {
                is_signed = true;
            }
        }
    }

    if publisher.is_empty() {
        if !autor_fallback.is_empty() {
            publisher = autor_fallback.to_string();
        } else {
            publisher = std::env::var("USERNAME")
                .or_else(|_| std::env::var("USER"))
                .unwrap_or_else(|_| "Sistema".to_string());
        }
    }

    let signature_state = if !is_exec {
        "not_applicable".to_string()
    } else if is_signed {
        "signed".to_string()
    } else {
        "unsigned".to_string()
    };

    ExecutableIntelResult {
        is_executable: is_exec,
        is_signed,
        publisher,
        signature_state,
        script_risk,
        sensitive_directory,
    }
}

fn avaliar_risco_script(caminho: &Path) -> String {
    let Ok(mut file) = abrir_arquivo_compartilhado(caminho) else {
        return "low".to_string();
    };
    let mut buf = vec![0u8; 65536];
    let Ok(n) = file.read(&mut buf) else {
        return "low".to_string();
    };
    let content = String::from_utf8_lossy(&buf[..n]).to_lowercase();

    let high_indicators = [
        "frombase64string",
        "-encodedcommand",
        "-enc ",
        "invoke-expression",
        "iex(",
        "downloadstring",
        "downloadfile",
        "wscript.shell",
        "createremotethread",
        "virtualalloc",
        "regsvr32 /s /n /u",
        "certutil -urlcache",
        "bitsadmin /transfer",
        "vssadmin delete shadows",
        "wbadmin delete catalog",
        "bcdedit /set",
    ];
    if high_indicators.iter().any(|ind| content.contains(ind)) {
        return "high".to_string();
    }

    let medium_indicators = [
        "invoke-webrequest",
        "curl ",
        "wget ",
        "net user ",
        "net localgroup ",
        "schtasks ",
        "reg add ",
        "reg delete ",
        "bypass",
        "hidden",
    ];
    if medium_indicators.iter().any(|ind| content.contains(ind)) {
        return "medium".to_string();
    }

    "low".to_string()
}

#[cfg(target_os = "windows")]
fn verificar_wintrust(caminho: &Path) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let path_w: Vec<u16> = OsStr::new(caminho)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut policy_guid = windows_sys::core::GUID {
        data1: 0x00aac56b,
        data2: 0xcd44,
        data3: 0x11d0,
        data4: [0x8c, 0xc2, 0x00, 0xc0, 0x4f, 0xc2, 0x95, 0xee],
    };

    let mut file_info = windows_sys::Win32::Security::WinTrust::WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<windows_sys::Win32::Security::WinTrust::WINTRUST_FILE_INFO>()
            as u32,
        pcwszFilePath: path_w.as_ptr(),
        hFile: std::ptr::null_mut(),
        pgKnownSubject: std::ptr::null_mut(),
    };

    let mut trust_data = windows_sys::Win32::Security::WinTrust::WINTRUST_DATA {
        cbStruct: std::mem::size_of::<windows_sys::Win32::Security::WinTrust::WINTRUST_DATA>()
            as u32,
        pPolicyCallbackData: std::ptr::null_mut(),
        pSIPClientData: std::ptr::null_mut(),
        dwUIChoice: 2,          // WTD_UI_NONE
        fdwRevocationChecks: 0, // WTD_REVOKE_NONE
        dwUnionChoice: 1,       // WTD_CHOICE_FILE
        Anonymous: windows_sys::Win32::Security::WinTrust::WINTRUST_DATA_0 {
            pFile: &mut file_info,
        },
        dwStateAction: 1, // WTD_STATEACTION_VERIFY
        hWVTStateData: std::ptr::null_mut(),
        pwszURLReference: std::ptr::null_mut(),
        dwProvFlags: 0x00000100 | 0x00000010, // WTD_SAFER_FLAG | WTD_CACHE_ONLY_URL_RETRIEVAL
        dwUIContext: 0,
        pSignatureSettings: std::ptr::null_mut(),
    };

    unsafe {
        let status = windows_sys::Win32::Security::WinTrust::WinVerifyTrust(
            std::ptr::null_mut(),
            &mut policy_guid,
            &mut trust_data as *mut _ as _,
        );

        trust_data.dwStateAction = 2; // WTD_STATEACTION_CLOSE
        windows_sys::Win32::Security::WinTrust::WinVerifyTrust(
            std::ptr::null_mut(),
            &mut policy_guid,
            &mut trust_data as *mut _ as _,
        );

        status == 0
    }
}
