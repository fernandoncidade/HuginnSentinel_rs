use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    pub dimensoes: String,
    pub autor: String,
}

pub fn extrair_imagem(caminho: &Path) -> Option<ImageMetadata> {
    extrair_imagem_com_ext(caminho, "")
}

pub fn extrair_imagem_com_ext(caminho: &Path, ext_override: &str) -> Option<ImageMetadata> {
    let mut meta = ImageMetadata::default();

    let ext = if !ext_override.is_empty() {
        ext_override.trim_start_matches('.').to_lowercase()
    } else {
        super::types::extensao_tipo_preferencial(caminho)
    };

    let mut buf = vec![0u8; 65536];
    let bytes_read = if let Ok(mut f) = super::extractor::abrir_arquivo_compartilhado(caminho) {
        f.read(&mut buf).unwrap_or(0)
    } else {
        0
    };
    buf.truncate(bytes_read);

    // Extrai autor de PNG / EXIF / SVG se presente no cabeçalho
    if bytes_read > 0 {
        meta.autor = extrair_autor_imagem(&buf, &ext);
    }

    // 1. Tenta via crate `image` (rápido, lê apenas cabeçalho)
    if let Ok(f) = super::extractor::abrir_arquivo_compartilhado(caminho) {
        let reader = image::ImageReader::new(BufReader::new(f));
        if let Ok(reader_guessed) = reader.with_guessed_format() {
            if let Ok((w, h)) = reader_guessed.into_dimensions() {
                if w > 0 && h > 0 {
                    meta.dimensoes = format!("{}x{}", w, h);
                    return Some(meta);
                }
            }
        }
    }

    // 2. Parsers binários especializados (cobrindo RAW, PSD, SVG, HEIC/AVIF, ICNS, EXR, HDR, TGA, PCX, JP2, DDS, QOI, TIFF/BigTIFF, XBM, XPM, FITS, SGI, RAS, EMF/WMF, etc.)
    if bytes_read >= 4 {
        if let Some((w, h)) = extrair_dimensoes_binarias(caminho, &buf, &ext) {
            if w > 0 && h > 0 {
                meta.dimensoes = format!("{}x{}", w, h);
                return Some(meta);
            }
        }
    }

    if !meta.autor.is_empty() {
        return Some(meta);
    }

    None
}

fn extrair_dimensoes_binarias(caminho: &Path, buf: &[u8], ext: &str) -> Option<(u32, u32)> {
    let n = buf.len();

    // PNG / APNG
    if n >= 24 && &buf[0..8] == b"\x89PNG\r\n\x1a\n" && &buf[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let h = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // GIF
    if n >= 10 && (&buf[0..6] == b"GIF87a" || &buf[0..6] == b"GIF89a") {
        let w = u16::from_le_bytes([buf[6], buf[7]]) as u32;
        let h = u16::from_le_bytes([buf[8], buf[9]]) as u32;
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // BMP / DIB
    if n >= 26 && &buf[0..2] == b"BM" {
        let dib_size = u32::from_le_bytes([buf[14], buf[15], buf[16], buf[17]]);
        if dib_size == 12 {
            let w = u16::from_le_bytes([buf[18], buf[19]]) as u32;
            let h = u16::from_le_bytes([buf[20], buf[21]]) as u32;
            return Some((w, h));
        } else if dib_size >= 40 {
            let w = i32::from_le_bytes([buf[18], buf[19], buf[20], buf[21]]).unsigned_abs();
            let h = i32::from_le_bytes([buf[22], buf[23], buf[24], buf[25]]).unsigned_abs();
            return Some((w, h));
        }
    }

    // WebP
    if n >= 30 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        let chunk = &buf[12..16];
        if chunk == b"VP8 " && n >= 30 {
            let w = (u16::from_le_bytes([buf[26], buf[27]]) & 0x3FFF) as u32;
            let h = (u16::from_le_bytes([buf[28], buf[29]]) & 0x3FFF) as u32;
            return Some((w, h));
        } else if chunk == b"VP8L" && n >= 25 {
            let bits = u32::from_le_bytes([buf[21], buf[22], buf[23], buf[24]]);
            let w = (bits & 0x3FFF) + 1;
            let h = ((bits >> 14) & 0x3FFF) + 1;
            return Some((w, h));
        } else if chunk == b"VP8X" && n >= 30 {
            let w = (buf[24] as u32 | ((buf[25] as u32) << 8) | ((buf[26] as u32) << 16)) + 1;
            let h = (buf[27] as u32 | ((buf[28] as u32) << 8) | ((buf[29] as u32) << 16)) + 1;
            return Some((w, h));
        }
    }

    // JPEG / JFIF
    if n >= 4 && buf[0] == 0xFF && buf[1] == 0xD8 {
        if let Some(dims) = parse_jpeg_dimensions(buf) {
            return Some(dims);
        }
    }

    // ICO / CUR
    if n >= 22 && buf[0] == 0 && buf[1] == 0 && (buf[2] == 1 || buf[2] == 2) {
        let count = u16::from_le_bytes([buf[4], buf[5]]) as usize;
        let mut max_w = 0u32;
        let mut max_h = 0u32;
        for i in 0..count.min(64) {
            let off = 6 + i * 16;
            if off + 16 > n {
                break;
            }
            let w = if buf[off] == 0 { 256 } else { buf[off] as u32 };
            let h = if buf[off + 1] == 0 {
                256
            } else {
                buf[off + 1] as u32
            };
            if w * h > max_w * max_h {
                max_w = w;
                max_h = h;
            }
        }
        if max_w > 0 && max_h > 0 {
            return Some((max_w, max_h));
        }
    }

    // ICNS
    if n >= 16 && &buf[0..4] == b"icns" {
        let mut pos = 8usize;
        let mut max_dim = 0u32;
        while pos + 8 <= n {
            let ostype = &buf[pos..pos + 4];
            let len = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]])
                as usize;
            let dim = match ostype {
                b"ic10" => 1024,
                b"ic09" | b"ic14" => 512,
                b"ic08" | b"ic13" => 256,
                b"ic07" => 128,
                b"ic12" | b"it32" => 64,
                b"ic11" | b"ih32" => 32,
                b"il32" | b"ic04" => 16,
                _ => 0,
            };
            if dim > max_dim {
                max_dim = dim;
            }
            if pos + 32 <= n && &buf[pos + 8..pos + 16] == b"\x89PNG\r\n\x1a\n" {
                let w = u32::from_be_bytes([
                    buf[pos + 24],
                    buf[pos + 25],
                    buf[pos + 26],
                    buf[pos + 27],
                ]);
                if w > max_dim {
                    max_dim = w;
                }
            }
            if len < 8 {
                break;
            }
            pos += len;
        }
        if max_dim > 0 {
            return Some((max_dim, max_dim));
        }
    }

    // PSD / PSB
    if n >= 22 && &buf[0..4] == b"8BPS" {
        let h = u32::from_be_bytes([buf[14], buf[15], buf[16], buf[17]]);
        let w = u32::from_be_bytes([buf[18], buf[19], buf[20], buf[21]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // TIFF / BigTIFF / Camera RAW (CR2, NEF, ARW, DNG, ORF, RW2, PEF, SRW, etc.)
    if n >= 8
        && ((&buf[0..2] == b"II"
            && (buf[2] == 42 || buf[2] == 43 || buf[2] == 0x55 || buf[2] == 0x52))
            || (&buf[0..2] == b"MM" && (buf[3] == 42 || buf[3] == 43 || buf[3] == 0x4F)))
    {
        if let Some(dims) = parse_tiff_dimensions(caminho, buf) {
            return Some(dims);
        }
    }

    // Fuji RAF
    if n >= 100 && &buf[0..8] == b"FUJIFILM" {
        let jpeg_offset = u32::from_be_bytes([buf[84], buf[85], buf[86], buf[87]]) as u64;
        if let Ok(mut f) = super::extractor::abrir_arquivo_compartilhado(caminho) {
            if f.seek(SeekFrom::Start(jpeg_offset)).is_ok() {
                let mut jbuf = vec![0u8; 32768];
                if let Ok(jn) = f.read(&mut jbuf) {
                    jbuf.truncate(jn);
                    if let Some(dims) = parse_jpeg_dimensions(&jbuf) {
                        return Some(dims);
                    }
                }
            }
        }
    }

    // Sigma X3F
    if n >= 40 && &buf[0..4] == b"FOVb" {
        let w = u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);
        let h = u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // Minolta MRW
    if n >= 32 && &buf[0..4] == b"\x00MRM" {
        let mut pos = 8usize;
        while pos + 8 <= n {
            let block = &buf[pos..pos + 4];
            let len = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]])
                as usize;
            if block == b"\x00PRD" && pos + 20 <= n {
                let h = u16::from_be_bytes([buf[pos + 16], buf[pos + 17]]) as u32;
                let w = u16::from_be_bytes([buf[pos + 18], buf[pos + 19]]) as u32;
                if w > 0 && h > 0 {
                    return Some((w, h));
                }
            }
            pos += 8 + len;
        }
    }

    // ISOBMFF (HEIC, HEIF, AVIF, CR3, JP2)
    if n >= 16 && (&buf[4..8] == b"ftyp" || &buf[4..8] == b"jP  ") {
        // Procura box `ispe` (HEIC/AVIF/CR3) ou `tkhd` (CR3) ou `ihdr` (JP2)
        for i in 0..n.saturating_sub(20) {
            if &buf[i..i + 4] == b"ispe" && i + 16 <= n {
                let w = u32::from_be_bytes([buf[i + 8], buf[i + 9], buf[i + 10], buf[i + 11]]);
                let h = u32::from_be_bytes([buf[i + 12], buf[i + 13], buf[i + 14], buf[i + 15]]);
                if w > 0 && h > 0 && w <= 65535 && h <= 65535 {
                    return Some((w, h));
                }
            } else if &buf[i..i + 4] == b"ihdr" && i + 12 <= n {
                let h = u32::from_be_bytes([buf[i + 4], buf[i + 5], buf[i + 6], buf[i + 7]]);
                let w = u32::from_be_bytes([buf[i + 8], buf[i + 9], buf[i + 10], buf[i + 11]]);
                if w > 0 && h > 0 && w <= 65535 && h <= 65535 {
                    return Some((w, h));
                }
            }
        }
    }

    // QOI
    if n >= 14 && &buf[0..4] == b"qoif" {
        let w = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let h = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // DDS
    if n >= 20 && &buf[0..4] == b"DDS " {
        let h = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let w = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // OpenEXR
    if n >= 32 && buf[0] == 0x76 && buf[1] == 0x2F && buf[2] == 0x31 && buf[3] == 0x01 {
        for key in [b"dataWindow\0".as_slice(), b"displayWindow\0".as_slice()] {
            if let Some(pos) = buf.windows(key.len()).position(|w| w == key) {
                let after = pos + key.len();
                // Pula "box2i\0" + 4 bytes size (16)
                if after + 6 + 4 + 16 <= n && &buf[after..after + 6] == b"box2i\0" {
                    let b_off = after + 10;
                    let xmin = i32::from_le_bytes([
                        buf[b_off],
                        buf[b_off + 1],
                        buf[b_off + 2],
                        buf[b_off + 3],
                    ]);
                    let ymin = i32::from_le_bytes([
                        buf[b_off + 4],
                        buf[b_off + 5],
                        buf[b_off + 6],
                        buf[b_off + 7],
                    ]);
                    let xmax = i32::from_le_bytes([
                        buf[b_off + 8],
                        buf[b_off + 9],
                        buf[b_off + 10],
                        buf[b_off + 11],
                    ]);
                    let ymax = i32::from_le_bytes([
                        buf[b_off + 12],
                        buf[b_off + 13],
                        buf[b_off + 14],
                        buf[b_off + 15],
                    ]);
                    if xmax >= xmin && ymax >= ymin {
                        return Some(((xmax - xmin + 1) as u32, (ymax - ymin + 1) as u32));
                    }
                }
            }
        }
    }

    // Radiance HDR (.hdr / .rgbe)
    if (ext == "hdr" || ext == "rgbe" || buf.starts_with(b"#?RADIANCE") || buf.starts_with(b"#?RGBE"))
        && n > 10
    {
        let text = String::from_utf8_lossy(&buf[..n.min(2048)]);
        for line in text.lines() {
            let trimmed = line.trim();
            if (trimmed.starts_with("-Y ") || trimmed.starts_with("+Y ")) && trimmed.contains("X ")
            {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(h), Ok(w)) = (parts[1].parse::<u32>(), parts[3].parse::<u32>()) {
                        if w > 0 && h > 0 {
                            return Some((w, h));
                        }
                    }
                }
            }
        }
    }

    // SVG
    if ext == "svg" || buf.windows(4).take(512).any(|w| w.eq_ignore_ascii_case(b"<svg")) {
        let text = String::from_utf8_lossy(buf);
        if let Some(dims) = parse_svg_dimensions(&text) {
            return Some(dims);
        }
    }

    // PCX
    if (ext == "pcx" || ext == "dcx") && n >= 12 && buf[0] == 0x0A {
        let xmin = u16::from_le_bytes([buf[4], buf[5]]) as i32;
        let ymin = u16::from_le_bytes([buf[6], buf[7]]) as i32;
        let xmax = u16::from_le_bytes([buf[8], buf[9]]) as i32;
        let ymax = u16::from_le_bytes([buf[10], buf[11]]) as i32;
        if xmax >= xmin && ymax >= ymin {
            let w = (xmax - xmin + 1) as u32;
            let h = (ymax - ymin + 1) as u32;
            if w > 0 && h > 0 && w <= 65535 && h <= 65535 {
                return Some((w, h));
            }
        }
    }

    // TGA
    if matches!(ext, "tga" | "icb" | "vda" | "vst") && n >= 18 {
        let img_type = buf[2];
        if matches!(img_type, 1 | 2 | 3 | 9 | 10 | 11) {
            let w = u16::from_le_bytes([buf[12], buf[13]]) as u32;
            let h = u16::from_le_bytes([buf[14], buf[15]]) as u32;
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        }
    }

    // SGI (.sgi, .rgb, .rgba, .bw)
    if n >= 10 && buf[0] == 0x01 && buf[1] == 0xDA {
        let w = u16::from_be_bytes([buf[6], buf[7]]) as u32;
        let h = u16::from_be_bytes([buf[8], buf[9]]) as u32;
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // Sun Raster (.ras, .sun)
    if n >= 12 && &buf[0..4] == b"\x59\xA6\x6A\x95" {
        let w = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let h = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // DPX
    if n >= 780 && (&buf[0..4] == b"SDPX" || &buf[0..4] == b"XPDS") {
        let be = &buf[0..4] == b"SDPX";
        let w = if be {
            u32::from_be_bytes([buf[772], buf[773], buf[774], buf[775]])
        } else {
            u32::from_le_bytes([buf[772], buf[773], buf[774], buf[775]])
        };
        let h = if be {
            u32::from_be_bytes([buf[776], buf[777], buf[778], buf[779]])
        } else {
            u32::from_le_bytes([buf[776], buf[777], buf[778], buf[779]])
        };
        if w > 0 && h > 0 && w <= 65535 && h <= 65535 {
            return Some((w, h));
        }
    }

    // PNM / PBM / PGM / PPM / PAM
    if n >= 7 && buf[0] == b'P' && matches!(buf[1], b'1'..=b'7') {
        let text = String::from_utf8_lossy(&buf[..n.min(1024)]);
        if buf[1] == b'7' {
            let mut w = 0u32;
            let mut h = 0u32;
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("WIDTH ") {
                    w = rest.trim().parse().unwrap_or(0);
                } else if let Some(rest) = line.strip_prefix("HEIGHT ") {
                    h = rest.trim().parse().unwrap_or(0);
                }
            }
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        } else {
            let mut nums = Vec::new();
            for line in text.lines() {
                let clean = line.split('#').next().unwrap_or("");
                for tok in clean.split_whitespace() {
                    if tok.starts_with('P') && nums.is_empty() {
                        continue;
                    }
                    if let Ok(v) = tok.parse::<u32>() {
                        nums.push(v);
                        if nums.len() >= 2 {
                            return Some((nums[0], nums[1]));
                        }
                    }
                }
            }
        }
    }

    // XBM / XPM
    if ext == "xbm" || ext == "xpm" {
        let text = String::from_utf8_lossy(&buf[..n.min(4096)]);
        if ext == "xbm" {
            let mut w = 0u32;
            let mut h = 0u32;
            for line in text.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "#define" {
                    if parts[1].ends_with("_width") {
                        w = parts[2].parse().unwrap_or(0);
                    } else if parts[1].ends_with("_height") {
                        h = parts[2].parse().unwrap_or(0);
                    }
                }
            }
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        } else {
            for line in text.lines() {
                let t = line.trim();
                if t.starts_with('"') {
                    let inner = t.trim_matches(|c| c == '"' || c == ',');
                    let parts: Vec<&str> = inner.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            if w > 0 && h > 0 {
                                return Some((w, h));
                            }
                        }
                    }
                }
            }
        }
    }

    // FITS
    if (ext == "fits" || ext == "fit" || ext == "fts") && buf.starts_with(b"SIMPLE") {
        let text = String::from_utf8_lossy(&buf[..n.min(2880)]);
        let mut w = 0u32;
        let mut h = 0u32;
        for chunk in text.as_bytes().chunks(80) {
            let card = String::from_utf8_lossy(chunk);
            if card.starts_with("NAXIS1") {
                if let Some(val) = card.split('=').nth(1) {
                    w = val
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .parse()
                        .unwrap_or(0);
                }
            } else if card.starts_with("NAXIS2") {
                if let Some(val) = card.split('=').nth(1) {
                    h = val
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .parse()
                        .unwrap_or(0);
                }
            }
        }
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    // EMF / WMF
    if ext == "emf" && n >= 44 && &buf[40..44] == b" EMF" {
        let left = i32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let top = i32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let right = i32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let bottom = i32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        if right > left && bottom > top {
            return Some(((right - left) as u32, (bottom - top) as u32));
        }
    }
    if ext == "wmf" && n >= 22 && &buf[0..4] == b"\xD7\xCD\xC6\x9A" {
        let left = i16::from_le_bytes([buf[6], buf[7]]) as i32;
        let top = i16::from_le_bytes([buf[8], buf[9]]) as i32;
        let right = i16::from_le_bytes([buf[10], buf[11]]) as i32;
        let bottom = i16::from_le_bytes([buf[12], buf[13]]) as i32;
        if right > left && bottom > top {
            return Some(((right - left) as u32, (bottom - top) as u32));
        }
    }

    None
}

fn parse_jpeg_dimensions(buf: &[u8]) -> Option<(u32, u32)> {
    let mut pos = 2usize;
    let n = buf.len();
    while pos + 4 <= n {
        if buf[pos] != 0xFF {
            pos += 1;
            continue;
        }
        while pos < n && buf[pos] == 0xFF {
            pos += 1;
        }
        if pos >= n {
            break;
        }
        let marker = buf[pos];
        pos += 1;
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            continue;
        }
        if pos + 2 > n {
            break;
        }
        let seg_len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
        if seg_len < 2 {
            break;
        }
        if (0xC0..=0xCF).contains(&marker)
            && marker != 0xC4
            && marker != 0xC8
            && marker != 0xCC
            && pos + 7 <= n
        {
            let h = u16::from_be_bytes([buf[pos + 3], buf[pos + 4]]) as u32;
            let w = u16::from_be_bytes([buf[pos + 5], buf[pos + 6]]) as u32;
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        }
        pos += seg_len;
    }
    None
}

fn parse_tiff_dimensions(caminho: &Path, buf: &[u8]) -> Option<(u32, u32)> {
    let le = &buf[0..2] == b"II";
    let read_u16 = |b: &[u8], off: usize| -> u16 {
        if off + 2 > b.len() {
            return 0;
        }
        if le {
            u16::from_le_bytes([b[off], b[off + 1]])
        } else {
            u16::from_be_bytes([b[off], b[off + 1]])
        }
    };
    let read_u32 = |b: &[u8], off: usize| -> u32 {
        if off + 4 > b.len() {
            return 0;
        }
        if le {
            u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
        } else {
            u32::from_be_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
        }
    };

    let version = read_u16(buf, 2);
    if version == 43 {
        // BigTIFF
        return None;
    }

    let first_ifd = read_u32(buf, 4) as usize;
    let mut best_w = 0u32;
    let mut best_h = 0u32;

    let mut queue = vec![first_ifd];
    let mut visited = Vec::new();

    while let Some(ifd_off) = queue.pop() {
        if ifd_off == 0 || ifd_off + 2 > buf.len() || visited.contains(&ifd_off) || visited.len() > 16
        {
            continue;
        }
        visited.push(ifd_off);

        let num_entries = read_u16(buf, ifd_off) as usize;
        let mut cur_w = 0u32;
        let mut cur_h = 0u32;

        for i in 0..num_entries.min(256) {
            let entry_off = ifd_off + 2 + i * 12;
            if entry_off + 12 > buf.len() {
                break;
            }
            let tag = read_u16(buf, entry_off);
            let field_type = read_u16(buf, entry_off + 2);
            let count = read_u32(buf, entry_off + 4) as usize;
            let val = if field_type == 3 {
                read_u16(buf, entry_off + 8) as u32
            } else {
                read_u32(buf, entry_off + 8)
            };

            match tag {
                0x0100 | 0xA002 => {
                    if val > cur_w {
                        cur_w = val;
                    }
                }
                0x0101 | 0xA003 => {
                    if val > cur_h {
                        cur_h = val;
                    }
                }
                0x8769 => {
                    // ExifOffset
                    queue.push(val as usize);
                }
                0x014A => {
                    // SubIFDs
                    if count == 1 {
                        queue.push(val as usize);
                    } else if (val as usize) + count * 4 <= buf.len() {
                        for k in 0..count.min(8) {
                            let sub_off = read_u32(buf, (val as usize) + k * 4) as usize;
                            queue.push(sub_off);
                        }
                    }
                }
                _ => {}
            }
        }

        if cur_w * cur_h > best_w * best_h {
            best_w = cur_w;
            best_h = cur_h;
        }

        let next_ifd_pos = ifd_off + 2 + num_entries * 12;
        if next_ifd_pos + 4 <= buf.len() {
            let next_ifd = read_u32(buf, next_ifd_pos) as usize;
            if next_ifd > 0 {
                queue.push(next_ifd);
            }
        }
    }

    if best_w > 0 && best_h > 0 {
        return Some((best_w, best_h));
    }

    // Fallback: se o IFD estiver no final do arquivo RAW, lê o bloco em first_ifd
    if first_ifd >= buf.len() {
        if let Ok(mut f) = super::extractor::abrir_arquivo_compartilhado(caminho) {
            if f.seek(SeekFrom::Start(first_ifd as u64)).is_ok() {
                let mut ibuf = [0u8; 4096];
                if let Ok(in_read) = f.read(&mut ibuf) {
                    if in_read >= 14 {
                        let num_entries = if le {
                            u16::from_le_bytes([ibuf[0], ibuf[1]])
                        } else {
                            u16::from_be_bytes([ibuf[0], ibuf[1]])
                        } as usize;
                        let mut cur_w = 0u32;
                        let mut cur_h = 0u32;
                        for i in 0..num_entries.min(256) {
                            let off = 2 + i * 12;
                            if off + 12 > in_read {
                                break;
                            }
                            let tag = if le {
                                u16::from_le_bytes([ibuf[off], ibuf[off + 1]])
                            } else {
                                u16::from_be_bytes([ibuf[off], ibuf[off + 1]])
                            };
                            let ftype = if le {
                                u16::from_le_bytes([ibuf[off + 2], ibuf[off + 3]])
                            } else {
                                u16::from_be_bytes([ibuf[off + 2], ibuf[off + 3]])
                            };
                            let val = if ftype == 3 {
                                if le {
                                    u16::from_le_bytes([ibuf[off + 8], ibuf[off + 9]]) as u32
                                } else {
                                    u16::from_be_bytes([ibuf[off + 8], ibuf[off + 9]]) as u32
                                }
                            } else if le {
                                u32::from_le_bytes([
                                    ibuf[off + 8],
                                    ibuf[off + 9],
                                    ibuf[off + 10],
                                    ibuf[off + 11],
                                ])
                            } else {
                                u32::from_be_bytes([
                                    ibuf[off + 8],
                                    ibuf[off + 9],
                                    ibuf[off + 10],
                                    ibuf[off + 11],
                                ])
                            };
                            if tag == 0x0100 {
                                cur_w = val;
                            } else if tag == 0x0101 {
                                cur_h = val;
                            }
                        }
                        if cur_w > 0 && cur_h > 0 {
                            return Some((cur_w, cur_h));
                        }
                    }
                }
            }
        }
    }

    None
}

fn parse_svg_dimensions(svg_text: &str) -> Option<(u32, u32)> {
    let lower = svg_text.to_lowercase();
    let svg_pos = lower.find("<svg")?;
    let tag_end = lower[svg_pos..].find('>')? + svg_pos;
    let svg_tag = &svg_text[svg_pos..=tag_end];

    let extract_attr = |attr: &str| -> Option<String> {
        for pat in [format!("{}=\"", attr), format!("{}='", attr)] {
            if let Some(idx) = svg_tag.find(&pat) {
                let start = idx + pat.len();
                let quote = pat.chars().last().unwrap();
                if let Some(end) = svg_tag[start..].find(quote) {
                    return Some(svg_tag[start..start + end].trim().to_string());
                }
            }
        }
        None
    };

    let parse_num = |s: &str| -> Option<u32> {
        let clean: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        clean.parse::<f64>().ok().map(|v| v.round() as u32)
    };

    let w_opt = extract_attr("width").and_then(|s| {
        if s.ends_with('%') {
            None
        } else {
            parse_num(&s)
        }
    });
    let h_opt = extract_attr("height").and_then(|s| {
        if s.ends_with('%') {
            None
        } else {
            parse_num(&s)
        }
    });

    if let (Some(w), Some(h)) = (w_opt, h_opt) {
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }

    if let Some(vb) = extract_attr("viewBox").or_else(|| extract_attr("viewbox")) {
        let nums: Vec<f64> = vb
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();
        if nums.len() >= 4 && nums[2] > 0.0 && nums[3] > 0.0 {
            return Some((nums[2].round() as u32, nums[3].round() as u32));
        }
    }

    None
}

fn extrair_autor_imagem(buf: &[u8], ext: &str) -> String {
    let n = buf.len();
    // PNG tEXt Author / Artist
    if n >= 24 && &buf[0..8] == b"\x89PNG\r\n\x1a\n" {
        let mut pos = 8usize;
        while pos + 8 <= n {
            let chunk_len =
                u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]) as usize;
            let ctype = &buf[pos + 4..pos + 8];
            if ctype == b"tEXt" && pos + 8 + chunk_len <= n {
                let data = &buf[pos + 8..pos + 8 + chunk_len];
                if let Some(sep) = data.iter().position(|&b| b == 0) {
                    let key = String::from_utf8_lossy(&data[..sep]);
                    if key.eq_ignore_ascii_case("Author") || key.eq_ignore_ascii_case("Artist") {
                        let val = String::from_utf8_lossy(&data[sep + 1..]).trim().to_string();
                        if !val.is_empty() {
                            return val;
                        }
                    }
                }
            }
            if ctype == b"IEND" || pos + 12 + chunk_len > n {
                break;
            }
            pos += 12 + chunk_len;
        }
    }

    // SVG <dc:creator>
    if ext == "svg" {
        let text = String::from_utf8_lossy(buf);
        if let Some(s) = text.find("<dc:creator>") {
            let start = s + "<dc:creator>".len();
            if let Some(e) = text[start..].find("</dc:creator>") {
                let inner = text[start..start + e].trim();
                if !inner.is_empty() && !inner.contains('<') {
                    return inner.to_string();
                }
            }
        }
    }

    // JPEG / TIFF EXIF tag 0x013B (Artist)
    let tiff_slice = if n >= 12 && buf[0] == 0xFF && buf[1] == 0xD8 {
        buf.windows(6)
            .position(|w| w == b"Exif\0\0")
            .and_then(|idx| buf.get(idx + 6..))
    } else if n >= 8 && (&buf[0..2] == b"II" || &buf[0..2] == b"MM") {
        Some(buf)
    } else {
        None
    };

    if let Some(tbuf) = tiff_slice {
        if tbuf.len() >= 8 && (&tbuf[0..2] == b"II" || &tbuf[0..2] == b"MM") {
            let le = &tbuf[0..2] == b"II";
            let ifd0 = if le {
                u32::from_le_bytes([tbuf[4], tbuf[5], tbuf[6], tbuf[7]])
            } else {
                u32::from_be_bytes([tbuf[4], tbuf[5], tbuf[6], tbuf[7]])
            } as usize;
            if ifd0 + 2 <= tbuf.len() {
                let entries = if le {
                    u16::from_le_bytes([tbuf[ifd0], tbuf[ifd0 + 1]])
                } else {
                    u16::from_be_bytes([tbuf[ifd0], tbuf[ifd0 + 1]])
                } as usize;
                for i in 0..entries.min(128) {
                    let off = ifd0 + 2 + i * 12;
                    if off + 12 > tbuf.len() {
                        break;
                    }
                    let tag = if le {
                        u16::from_le_bytes([tbuf[off], tbuf[off + 1]])
                    } else {
                        u16::from_be_bytes([tbuf[off], tbuf[off + 1]])
                    };
                    if tag == 0x013B {
                        let count = if le {
                            u32::from_le_bytes([
                                tbuf[off + 4],
                                tbuf[off + 5],
                                tbuf[off + 6],
                                tbuf[off + 7],
                            ])
                        } else {
                            u32::from_be_bytes([
                                tbuf[off + 4],
                                tbuf[off + 5],
                                tbuf[off + 6],
                                tbuf[off + 7],
                            ])
                        } as usize;
                        let val_off = if count <= 4 {
                            off + 8
                        } else if le {
                            u32::from_le_bytes([
                                tbuf[off + 8],
                                tbuf[off + 9],
                                tbuf[off + 10],
                                tbuf[off + 11],
                            ]) as usize
                        } else {
                            u32::from_be_bytes([
                                tbuf[off + 8],
                                tbuf[off + 9],
                                tbuf[off + 10],
                                tbuf[off + 11],
                            ]) as usize
                        };
                        if val_off + count <= tbuf.len() && count > 0 {
                            let s = String::from_utf8_lossy(&tbuf[val_off..val_off + count])
                                .trim_matches('\0')
                                .trim()
                                .to_string();
                            if !s.is_empty() {
                                return s;
                            }
                        }
                    }
                }
            }
        }
    }

    String::new()
}
