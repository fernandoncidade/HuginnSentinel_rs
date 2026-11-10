use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ExecutableMetadata {
    pub formato: String,
    pub publisher: String,
    pub assinado: bool,
    pub binary_file_b: f64,
    pub binary_file_kb: f64,
    pub binary_file_mb: f64,
    pub binary_file_gb: f64,
    pub binary_file_tb: f64,
}

pub fn extrair_executavel(caminho: &Path, file_size: u64) -> Option<ExecutableMetadata> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let read_len = file_size.min(524288) as usize;
    let mut buf = vec![0u8; read_len];
    let bytes_read = file.read(&mut buf).ok()?;
    buf.truncate(bytes_read);

    // Em binários PE grandes, a seção .rsrc (com VS_VERSION_INFO) costuma ficar no final do arquivo
    if file_size > (read_len as u64) {
        let tail_len = 524288.min((file_size - read_len as u64) as usize);
        if file.seek(SeekFrom::End(-(tail_len as i64))).is_ok() {
            let mut tail = vec![0u8; tail_len];
            if let Ok(tn) = file.read(&mut tail) {
                buf.extend_from_slice(&tail[..tn]);
            }
        }
    }

    let formato = if bytes_read >= 2 && &buf[0..2] == b"MZ" {
        "PE (Windows Executable)".to_string()
    } else if bytes_read >= 4 && &buf[0..4] == b"\x7FELF" {
        "ELF (Linux Binary)".to_string()
    } else if bytes_read >= 4
        && (&buf[0..4] == b"\xFE\xED\xFA\xCE" || &buf[0..4] == b"\xCF\xFA\xED\xFE")
    {
        "Mach-O (macOS Binary)".to_string()
    } else {
        "Binário".to_string()
    };

    let mut publisher = extrair_pe_version_string(&buf, "CompanyName")
        .or_else(|| extrair_pe_version_string(&buf, "ProductName"))
        .or_else(|| extrair_pe_version_string(&buf, "FileDescription"))
        .unwrap_or_default();

    if publisher.is_empty() {
        if let Some(ascii_pub) = extrair_ascii_company(&buf) {
            publisher = ascii_pub;
        }
    }

    let assinado = verificar_pe_assinado(&buf);

    let b = file_size as f64;
    let kb = b / 1024.0;
    let mb = kb / 1024.0;
    let gb = mb / 1024.0;
    let tb = gb / 1024.0;

    Some(ExecutableMetadata {
        formato,
        publisher,
        assinado,
        binary_file_b: b,
        binary_file_kb: kb,
        binary_file_mb: mb,
        binary_file_gb: gb,
        binary_file_tb: tb,
    })
}

/// Procura chave UTF-16LE (como `CompanyName\0`) dentro do recurso `VS_VERSION_INFO` de um PE
/// e retorna o valor UTF-16LE correspondente.
pub fn extrair_pe_version_string(buf: &[u8], key: &str) -> Option<String> {
    let mut pattern = Vec::with_capacity((key.len() + 1) * 2);
    for u in key.encode_utf16() {
        let le = u.to_le_bytes();
        pattern.push(le[0]);
        pattern.push(le[1]);
    }
    pattern.push(0);
    pattern.push(0);

    let mut search_from = 0usize;
    while search_from + pattern.len() < buf.len() {
        let Some(rel_pos) = buf[search_from..]
            .windows(pattern.len())
            .position(|w| w == pattern.as_slice())
        else {
            break;
        };
        let pos = search_from + rel_pos;
        let mut val_start = pos + pattern.len();

        // Em estruturas String do VS_VERSION_INFO, o valor é alinhado em 32 bits (zeros de padding)
        while val_start + 2 <= buf.len() && buf[val_start] == 0 && buf[val_start + 1] == 0 {
            val_start += 2;
            if val_start > pos + pattern.len() + 16 {
                break;
            }
        }

        let mut u16_chars = Vec::new();
        let mut cur = val_start;
        while cur + 2 <= buf.len() && u16_chars.len() < 128 {
            let code = u16::from_le_bytes([buf[cur], buf[cur + 1]]);
            if code == 0 {
                break;
            }
            u16_chars.push(code);
            cur += 2;
        }

        if !u16_chars.is_empty() {
            let candidate = String::from_utf16_lossy(&u16_chars).trim().to_string();
            if candidate.len() >= 2
                && candidate
                    .chars()
                    .all(|c| !c.is_control() && c != '\u{FFFD}')
            {
                return Some(candidate);
            }
        }

        search_from = pos + pattern.len();
    }

    None
}

fn extrair_ascii_company(buf: &[u8]) -> Option<String> {
    let key = b"CompanyName";
    let pos = buf.windows(key.len()).position(|w| w == key)?;
    let start = pos + key.len();
    let end = (start + 96).min(buf.len());
    let slice = &buf[start..end];
    let clean: String = slice
        .iter()
        .skip_while(|&&b| b == 0 || b == b' ' || b == b':' || b == b'=')
        .take_while(|&&b| (32..=126).contains(&b))
        .map(|&b| b as char)
        .collect();
    let trimmed = clean.trim();
    if trimmed.len() >= 2 {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub fn verificar_pe_assinado(buf: &[u8]) -> bool {
    if buf.len() < 0x40 || &buf[0..2] != b"MZ" {
        return false;
    }
    let pe_offset = u32::from_le_bytes([buf[0x3C], buf[0x3D], buf[0x3E], buf[0x3F]]) as usize;
    if pe_offset + 24 > buf.len() || &buf[pe_offset..pe_offset + 4] != b"PE\0\0" {
        return false;
    }
    let opt_hdr = pe_offset + 24;
    if opt_hdr + 2 > buf.len() {
        return false;
    }
    let magic = u16::from_le_bytes([buf[opt_hdr], buf[opt_hdr + 1]]);
    // DataDirectory[4] (IMAGE_DIRECTORY_ENTRY_SECURITY)
    // PE32 (0x10B): offset 96 + 4*8 = 128 a partir do OptionalHeader
    // PE32+ (0x20B): offset 112 + 4*8 = 144 a partir do OptionalHeader
    let sec_dir_off = match magic {
        0x10B => opt_hdr + 128,
        0x20B => opt_hdr + 144,
        _ => return false,
    };
    if sec_dir_off + 8 <= buf.len() {
        let cert_rva = u32::from_le_bytes([
            buf[sec_dir_off],
            buf[sec_dir_off + 1],
            buf[sec_dir_off + 2],
            buf[sec_dir_off + 3],
        ]);
        let cert_size = u32::from_le_bytes([
            buf[sec_dir_off + 4],
            buf[sec_dir_off + 5],
            buf[sec_dir_off + 6],
            buf[sec_dir_off + 7],
        ]);
        if cert_rva > 0 && cert_size > 0 {
            return true;
        }
    }
    false
}
