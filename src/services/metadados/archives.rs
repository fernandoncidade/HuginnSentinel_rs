use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::types::extensao_tipo_preferencial;

#[derive(Debug, Clone, Default)]
pub struct ArchiveMetadata {
    pub arquivos: String,
    pub protegido: bool,
    pub unzipped_b: f64,
    pub unzipped_kb: f64,
    pub unzipped_mb: f64,
    pub unzipped_gb: f64,
    pub unzipped_tb: f64,
}

impl ArchiveMetadata {
    fn from_counts(file_count: usize, total_uncompressed: u64, protegido: bool) -> Self {
        let b = total_uncompressed as f64;
        let kb = b / 1024.0;
        let mb = kb / 1024.0;
        let gb = mb / 1024.0;
        let tb = gb / 1024.0;
        Self {
            arquivos: file_count.max(1).to_string(),
            protegido,
            unzipped_b: b,
            unzipped_kb: kb,
            unzipped_mb: mb,
            unzipped_gb: gb,
            unzipped_tb: tb,
        }
    }
}

pub fn extrair_arquivo_compactado(caminho: &Path) -> Option<ArchiveMetadata> {
    extrair_arquivo_compactado_com_ext(caminho, "")
}

pub fn extrair_arquivo_compactado_com_ext(
    caminho: &Path,
    ext_override: &str,
) -> Option<ArchiveMetadata> {
    let file_size = std::fs::metadata(caminho).map(|m| m.len()).ok()?;
    if file_size == 0 {
        return Some(ArchiveMetadata::from_counts(0, 0, false));
    }

    let ext = if !ext_override.is_empty() {
        ext_override.trim_start_matches('.').to_lowercase()
    } else {
        extensao_tipo_preferencial(caminho)
    };

    // 1. Tenta como arquivo ZIP (cobre .zip, .jar, .war, .apk, .ipa, .whl, .nupkg, .vsix, .cbz, etc.)
    if let Ok(file) = super::extractor::abrir_arquivo_compartilhado(caminho) {
        if let Ok(mut zip) = zip::ZipArchive::new(file) {
            let mut file_count = 0usize;
            let mut total_uncompressed: u64 = 0;
            let mut protegido = false;

            for i in 0..zip.len() {
                if let Ok(entry) = zip.by_index_raw(i) {
                    if entry.encrypted() {
                        protegido = true;
                    }
                    if !entry.is_dir() {
                        file_count += 1;
                        total_uncompressed = total_uncompressed.saturating_add(entry.size());
                    }
                }
            }

            if total_uncompressed == 0 {
                total_uncompressed = file_size;
            }
            return Some(ArchiveMetadata::from_counts(
                file_count.max(zip.len().min(1)),
                total_uncompressed,
                protegido,
            ));
        }
    }

    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut hdr = [0u8; 512];
    let n = file.read(&mut hdr).unwrap_or(0);

    // 2. GZIP (.gz, .tgz, .tar.gz, .sql.gz, .pb.gz)
    if (n >= 10 && hdr[0] == 0x1F && hdr[1] == 0x8B)
        || matches!(ext.as_str(), "gz" | "tgz" | "tar.gz" | "sql.gz" | "pb.gz" | "taz")
    {
        let mut isize_val = 0u64;
        if file_size >= 18 && file.seek(SeekFrom::End(-4)).is_ok() {
            let mut ib = [0u8; 4];
            if file.read_exact(&mut ib).is_ok() {
                isize_val = u32::from_le_bytes(ib) as u64;
            }
        }
        if isize_val < file_size || isize_val > file_size.saturating_mul(25) {
            isize_val = (file_size as f64 * 2.8) as u64;
        }
        let est_files = if ext.contains("tar") || ext == "tgz" || ext == "taz" {
            ((isize_val / 16384) as usize).max(1)
        } else {
            1
        };
        return Some(ArchiveMetadata::from_counts(est_files, isize_val, false));
    }

    // 3. TAR (.tar, .cbt)
    if ext == "tar" || ext == "cbt" || (n >= 265 && &hdr[257..262] == b"ustar") {
        let _ = file.seek(SeekFrom::Start(0));
        let mut file_count = 0usize;
        let mut total_uncompressed = 0u64;
        let mut pos = 0u64;
        let mut block = [0u8; 512];

        while pos + 512 <= file_size && file_count < 50_000 {
            if file.seek(SeekFrom::Start(pos)).is_err() || file.read_exact(&mut block).is_err() {
                break;
            }
            if block.iter().all(|&b| b == 0) {
                break;
            }
            let size_octal = String::from_utf8_lossy(&block[124..136]);
            let clean_octal = size_octal.trim_matches(|c: char| c == '\0' || c.is_whitespace());
            let entry_size = u64::from_str_radix(clean_octal, 8).unwrap_or(0);
            let typeflag = block[156];
            if typeflag == b'0' || typeflag == 0 {
                file_count += 1;
                total_uncompressed = total_uncompressed.saturating_add(entry_size);
            }
            let blocks = (entry_size + 511) / 512;
            pos += 512 + blocks * 512;
        }

        if file_count > 0 {
            return Some(ArchiveMetadata::from_counts(
                file_count,
                total_uncompressed.max(file_size),
                false,
            ));
        }
    }

    // 4. Microsoft Cabinet (.cab)
    if n >= 36 && &hdr[0..4] == b"MSCF" {
        let cab_files = u16::from_le_bytes([hdr[28], hdr[29]]) as usize;
        let uncomp = (file_size as f64 * 2.4) as u64;
        return Some(ArchiveMetadata::from_counts(
            cab_files.max(1),
            uncomp.max(file_size),
            false,
        ));
    }

    // 5. QCOW / QCOW2 (.qcow, .qcow2)
    if n >= 32 && &hdr[0..4] == b"QFI\xFB" {
        let virt_size = u64::from_be_bytes([
            hdr[24], hdr[25], hdr[26], hdr[27], hdr[28], hdr[29], hdr[30], hdr[31],
        ]);
        let uncomp = virt_size.max(file_size);
        return Some(ArchiveMetadata::from_counts(1, uncomp, false));
    }

    // 6. VirtualBox VDI (.vdi)
    if n >= 0x180 && &hdr[0x40..0x44] == b"\x7F\x10\xDA\xBE" {
        let virt_size = u64::from_le_bytes([
            hdr[0x178], hdr[0x179], hdr[0x17A], hdr[0x17B], hdr[0x17C], hdr[0x17D], hdr[0x17E],
            hdr[0x17F],
        ]);
        return Some(ArchiveMetadata::from_counts(
            1,
            virt_size.max(file_size),
            false,
        ));
    }

    // 7. ISO 9660 (.iso, .udf, .img)
    if matches!(ext.as_str(), "iso" | "udf" | "img") && file_size > 0x8060 {
        if file.seek(SeekFrom::Start(0x8000)).is_ok() {
            let mut pvd = [0u8; 156];
            if file.read_exact(&mut pvd).is_ok() && &pvd[1..6] == b"CD001" {
                let blocks = u32::from_le_bytes([pvd[80], pvd[81], pvd[82], pvd[83]]) as u64;
                let block_sz = u16::from_le_bytes([pvd[128], pvd[129]]) as u64;
                let vol_bytes = (blocks * block_sz.max(2048)).max(file_size);
                let est_files = ((vol_bytes / 65536) as usize).max(1);
                return Some(ArchiveMetadata::from_counts(est_files, vol_bytes, false));
            }
        }
    }

    // 8. 7z / RAR / XZ / BZ2 / ZST / VHD / VMDK / DMG / outros compactados e discos virtuais
    let ratio = match ext.as_str() {
        "7z" | "xz" | "txz" | "tar.xz" | "lzma" | "tlzma" | "tar.lzma" | "sql.xz" => 3.2,
        "zst" | "tzst" | "tar.zst" | "bz2" | "tbz" | "tbz2" | "tar.bz2" | "sql.bz2" => 2.9,
        "rar" | "cbr" | "ace" | "arj" | "lzh" | "lha" => 2.5,
        "iso" | "udf" | "img" | "vhd" | "vhdx" | "vmdk" | "vdi" | "wim" | "esd" | "ova" => 1.35,
        _ => 2.2,
    };

    let uncomp = ((file_size as f64) * ratio).round() as u64;
    let est_files = if matches!(
        ext.as_str(),
        "bz2" | "xz" | "lz" | "lzma" | "zst" | "lz4" | "br" | "z" | "sql.bz2" | "sql.xz"
    ) {
        1
    } else {
        ((uncomp / 32768) as usize).clamp(1, 50_000)
    };

    Some(ArchiveMetadata::from_counts(
        est_files,
        uncomp.max(file_size),
        false,
    ))
}
