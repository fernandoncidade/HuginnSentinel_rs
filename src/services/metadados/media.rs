use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::types::{extensao_tipo_preferencial, is_audio_ext, is_video_ext};

#[derive(Debug, Clone, Default)]
pub struct MediaMetadata {
    pub duracao: String,
    pub taxa_bits: String,
    pub dimensoes: String,
    pub autor: String,
}

pub fn formatar_duracao(segundos: u64) -> String {
    let horas = segundos / 3600;
    let minutos = (segundos % 3600) / 60;
    let segs = segundos % 60;
    format!("{:02}:{:02}:{:02}", horas, minutos, segs)
}

pub fn extrair_midia(caminho: &Path, file_size: u64) -> Option<MediaMetadata> {
    extrair_midia_com_ext(caminho, "", file_size)
}

pub fn extrair_midia_com_ext(
    caminho: &Path,
    ext_override: &str,
    file_size: u64,
) -> Option<MediaMetadata> {
    if file_size == 0 {
        return None;
    }
    let ext = if !ext_override.is_empty() {
        ext_override.trim_start_matches('.').to_lowercase()
    } else {
        extensao_tipo_preferencial(caminho)
    };

    if is_video_ext(&ext) {
        return extrair_video(caminho, &ext, file_size);
    }
    if is_audio_ext(&ext) {
        return extrair_audio(caminho, &ext, file_size);
    }

    None
}

pub fn extrair_audio(caminho: &Path, ext: &str, file_size: u64) -> Option<MediaMetadata> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut buf = vec![0u8; 65536.min(file_size as usize)];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);

    // 1. WAV / BWF / RF64
    if n >= 36
        && (&buf[0..4] == b"RIFF" || &buf[0..4] == b"RF64")
        && &buf[8..12] == b"WAVE"
    {
        if let Some(meta) = parse_wav(&buf, file_size) {
            return Some(meta);
        }
    }

    // 2. FLAC
    if n >= 42 && (buf.starts_with(b"fLaC") || ext == "flac" || ext == "fla") {
        if let Some(meta) = parse_flac(&buf, file_size) {
            return Some(meta);
        }
    }

    // 3. OGG / OGA / Opus / Speex
    if n >= 36 && (&buf[0..4] == b"OggS" || matches!(ext, "ogg" | "oga" | "opus" | "spx")) {
        if let Some(meta) = parse_ogg_audio(&mut file, &buf, file_size) {
            return Some(meta);
        }
    }

    // 4. MP4 Audio (M4A, M4B, M4P, M4R, ALAC, 3GA)
    if matches!(ext, "m4a" | "m4b" | "m4p" | "m4r" | "alac" | "3ga")
        || (n >= 12 && &buf[4..8] == b"ftyp")
    {
        if let Some(mut meta) = parse_isobmff(&mut file, file_size) {
            meta.dimensoes.clear();
            if !meta.duracao.is_empty() || !meta.taxa_bits.is_empty() {
                return Some(meta);
            }
        }
    }

    // 5. AIFF / AIF / AIFC
    if n >= 30 && &buf[0..4] == b"FORM" && (&buf[8..12] == b"AIFF" || &buf[8..12] == b"AIFC") {
        if let Some(meta) = parse_aiff(&buf, file_size) {
            return Some(meta);
        }
    }

    // 6. WMA / ASF
    if n >= 30
        && buf[0..16]
            == [
                0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62,
                0xCE, 0x6C,
            ]
    {
        if let Some(mut meta) = parse_asf(&buf, file_size) {
            meta.dimensoes.clear();
            return Some(meta);
        }
    }

    // 7. Matroska Audio (.mka / .weba)
    if n >= 4 && &buf[0..4] == b"\x1A\x45\xDF\xA3" {
        if let Some(mut meta) = parse_ebml(&buf, file_size) {
            meta.dimensoes.clear();
            return Some(meta);
        }
    }

    // 8. Monkey's Audio (.ape)
    if n >= 76 && &buf[0..4] == b"MAC " {
        let version = u16::from_le_bytes([buf[4], buf[5]]);
        let (total_frames, blocks_per_frame, final_frame_blocks, sample_rate) = if version >= 3980 {
            let bpf = u32::from_le_bytes([buf[56], buf[57], buf[58], buf[59]]) as u64;
            let ffb = u32::from_le_bytes([buf[60], buf[61], buf[62], buf[63]]) as u64;
            let tf = u32::from_le_bytes([buf[64], buf[65], buf[66], buf[67]]) as u64;
            let sr = u32::from_le_bytes([buf[72], buf[73], buf[74], buf[75]]) as u64;
            (tf, bpf, ffb, sr)
        } else {
            (0, 0, 0, 0)
        };
        if sample_rate > 0 && total_frames > 0 {
            let samples = (total_frames - 1) * blocks_per_frame + final_frame_blocks;
            let secs = (samples / sample_rate).max(1);
            let kbps = ((file_size * 8) / (secs * 1000)).max(1);
            return Some(MediaMetadata {
                duracao: formatar_duracao(secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: String::new(),
                autor: String::new(),
            });
        }
    }

    // 9. WavPack (.wv)
    if n >= 32 && &buf[0..4] == b"wvpk" {
        let total_samples = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as u64;
        let flags = u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]);
        let sr_idx = ((flags >> 23) & 0x0F) as usize;
        const WV_SR: [u64; 16] = [
            6000, 8000, 9600, 11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000, 64000, 88200,
            96000, 192000, 44100,
        ];
        let sr = WV_SR[sr_idx];
        if total_samples > 0 && total_samples != 0xFFFF_FFFF && sr > 0 {
            let secs = (total_samples / sr).max(1);
            let kbps = ((file_size * 8) / (secs * 1000)).max(1);
            return Some(MediaMetadata {
                duracao: formatar_duracao(secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: String::new(),
                autor: String::new(),
            });
        }
    }

    // 10. True Audio (.tta)
    if n >= 22 && &buf[0..4] == b"TTA1" {
        let sr = u32::from_le_bytes([buf[10], buf[11], buf[12], buf[13]]) as u64;
        let samples = u32::from_le_bytes([buf[14], buf[15], buf[16], buf[17]]) as u64;
        if sr > 0 && samples > 0 {
            let secs = (samples / sr).max(1);
            let kbps = ((file_size * 8) / (secs * 1000)).max(1);
            return Some(MediaMetadata {
                duracao: formatar_duracao(secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: String::new(),
                autor: String::new(),
            });
        }
    }

    // 11. AU / SND (.au, .snd)
    if n >= 24 && &buf[0..4] == b".snd" {
        let data_offset = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as u64;
        let encoding = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let sample_rate = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]) as u64;
        let channels = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]) as u64;
        let bytes_per_sample = match encoding {
            1 | 2 | 27 => 1,
            3 => 2,
            4 => 3,
            5 | 6 => 4,
            7 => 8,
            _ => 2,
        };
        let byte_rate = sample_rate * channels.max(1) * bytes_per_sample;
        if byte_rate > 0 {
            let audio_bytes = file_size.saturating_sub(data_offset);
            let secs = (audio_bytes / byte_rate).max(1);
            let kbps = ((byte_rate * 8) / 1000).max(1);
            return Some(MediaMetadata {
                duracao: formatar_duracao(secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: String::new(),
                autor: String::new(),
            });
        }
    }

    // 12. DSF (.dsf)
    if n >= 52 && &buf[0..4] == b"DSD " && &buf[28..32] == b"fmt " {
        let sr = u32::from_le_bytes([buf[56.min(n - 4)], buf[57.min(n - 3)], buf[58.min(n - 2)], buf[59.min(n - 1)]]) as u64;
        if n >= 68 && sr > 0 {
            let samples = u64::from_le_bytes([
                buf[60], buf[61], buf[62], buf[63], buf[64], buf[65], buf[66], buf[67],
            ]);
            let secs = (samples / sr).max(1);
            let kbps = ((file_size * 8) / (secs * 1000)).max(1);
            return Some(MediaMetadata {
                duracao: formatar_duracao(secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: String::new(),
                autor: String::new(),
            });
        }
    }

    // 13. MP3 / MP2 / MP1 / AAC
    if matches!(ext, "mp3" | "mp2" | "mp1" | "mpa" | "aac") || buf.starts_with(b"ID3") {
        if let Some(meta) = parse_mp3(&mut file, &buf, file_size) {
            return Some(meta);
        }
    }

    // 14. Fallback robusto baseado em taxa de bits típica por formato de áudio (paridade com gmet_04)
    let fallback_kbps: u64 = match ext {
        "wav" | "wave" | "bwf" | "rf64" | "aiff" | "aif" | "aifc" | "pcm" => 1411,
        "flac" | "fla" | "ape" | "wv" | "tta" | "tak" | "shn" | "alac" => 850,
        "dsf" | "dff" | "dsd" => 5644,
        "ac3" | "eac3" | "ec3" => 384,
        "dts" | "dtshd" | "thd" | "truehd" => 1509,
        "mp3" | "mp2" | "mp1" | "mpa" | "m4a" | "m4b" | "m4p" | "m4r" | "aac" | "wma"
        | "ogg" | "oga" | "mpc" | "weba" | "mka" => 192,
        "opus" | "spx" => 96,
        "amr" | "awb" | "gsm" | "vox" => 13,
        "mid" | "midi" | "kar" | "rmi" => 32,
        _ => 128,
    };
    let dur_secs = ((file_size * 8) / (fallback_kbps * 1000)).max(1);
    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", fallback_kbps),
        dimensoes: String::new(),
        autor: String::new(),
    })
}

pub fn extrair_video(caminho: &Path, ext: &str, file_size: u64) -> Option<MediaMetadata> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut buf = vec![0u8; 65536.min(file_size as usize)];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);

    // 1. MP4 / MOV / M4V / 3GP / 3G2 / F4V / QT (ISOBMFF)
    if matches!(
        ext,
        "mp4" | "m4v" | "mp4v" | "mpg4" | "mov" | "qt" | "3gp" | "3g2" | "3gpp" | "3gpp2" | "f4v"
    ) || (n >= 12 && (&buf[4..8] == b"ftyp" || &buf[4..8] == b"moov" || &buf[4..8] == b"mdat" || &buf[4..8] == b"wide"))
    {
        if let Some(meta) = parse_isobmff(&mut file, file_size) {
            if !meta.duracao.is_empty() || !meta.dimensoes.is_empty() {
                return Some(meta);
            }
        }
    }

    // 2. MKV / WebM (EBML)
    if n >= 4 && (&buf[0..4] == b"\x1A\x45\xDF\xA3" || matches!(ext, "mkv" | "webm" | "mk3d" | "mks"))
    {
        if let Some(meta) = parse_ebml(&buf, file_size) {
            if !meta.duracao.is_empty() || !meta.dimensoes.is_empty() {
                return Some(meta);
            }
        }
    }

    // 3. AVI / DivX
    if n >= 64 && &buf[0..4] == b"RIFF" && (&buf[8..12] == b"AVI " || &buf[8..12] == b"AVIX") {
        if let Some(meta) = parse_avi(&buf, file_size) {
            return Some(meta);
        }
    }

    // 4. ASF / WMV
    if n >= 30
        && buf[0..16]
            == [
                0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62,
                0xCE, 0x6C,
            ]
    {
        if let Some(meta) = parse_asf(&buf, file_size) {
            return Some(meta);
        }
    }

    // 5. FLV
    if n >= 13 && &buf[0..3] == b"FLV" {
        if let Some(meta) = parse_flv(&mut file, &buf, file_size) {
            return Some(meta);
        }
    }

    // 6. OGV / OGM
    if n >= 42 && (&buf[0..4] == b"OggS" || matches!(ext, "ogv" | "ogm")) {
        if let Some(meta) = parse_ogv(&mut file, &buf, file_size) {
            return Some(meta);
        }
    }

    // 7. MPEG-PS (.mpg, .mpeg, .vob, .m2v, .mpv, .m1v, .mpe)
    if matches!(ext, "mpg" | "mpeg" | "vob" | "m2v" | "mpv" | "m1v" | "mpe" | "mp2v")
        || (n >= 4 && (&buf[0..4] == b"\x00\x00\x01\xBA" || &buf[0..4] == b"\x00\x00\x01\xB3"))
    {
        if let Some(meta) = parse_mpeg_ps(&buf, file_size) {
            return Some(meta);
        }
    }

    // 8. RealMedia (.rm, .rmvb, .rv)
    if n >= 50 && &buf[0..4] == b".RMF" {
        if let Some(meta) = parse_realmedia(&buf, file_size) {
            return Some(meta);
        }
    }

    // 9. Y4M (.y4m)
    if n >= 16 && buf.starts_with(b"YUV4MPEG2") {
        let header_line = String::from_utf8_lossy(&buf[..n.min(256)]);
        let first = header_line.lines().next().unwrap_or("");
        let mut w = 0u32;
        let mut h = 0u32;
        for tok in first.split_whitespace() {
            if let Some(val) = tok.strip_prefix('W') {
                w = val.parse().unwrap_or(0);
            } else if let Some(val) = tok.strip_prefix('H') {
                h = val.parse().unwrap_or(0);
            }
        }
        let secs = (file_size / (1024 * 512)).max(1);
        let kbps = ((file_size * 8) / (secs * 1000)).max(1);
        return Some(MediaMetadata {
            duracao: formatar_duracao(secs),
            taxa_bits: format!("{} kbps", kbps),
            dimensoes: if w > 0 && h > 0 {
                format!("{}x{}", w, h)
            } else {
                String::new()
            },
            autor: String::new(),
        });
    }

    // Fallback para formatos de vídeo (TS/M2TS/RAW streams/etc.)
    let fallback_kbps: u64 = match ext {
        "ts" | "mts" | "m2ts" | "m2t" | "tp" | "trp" | "vob" | "evo" => 8000,
        "mp4" | "mkv" | "mov" | "webm" | "avi" | "wmv" | "mpg" | "mpeg" => 2500,
        _ => 1500,
    };
    let dur_secs = ((file_size * 8) / (fallback_kbps * 1000)).max(1);
    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", fallback_kbps),
        dimensoes: String::new(),
        autor: String::new(),
    })
}

fn parse_wav(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let mut pos = 12usize;
    let mut byte_rate = 0u32;
    let mut data_size = 0u64;
    let mut autor = String::new();

    while pos + 8 <= buf.len() {
        let id = &buf[pos..pos + 4];
        let sz = u32::from_le_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]])
            as usize;
        if id == b"fmt " && pos + 20 <= buf.len() {
            byte_rate =
                u32::from_le_bytes([buf[pos + 16], buf[pos + 17], buf[pos + 18], buf[pos + 19]]);
        } else if id == b"data" {
            data_size = if sz == 0xFFFF_FFFF || sz == 0 {
                file_size.saturating_sub((pos + 8) as u64)
            } else {
                sz as u64
            };
        } else if id == b"LIST" && pos + 12 <= buf.len() && &buf[pos + 8..pos + 12] == b"INFO" {
            let mut lpos = pos + 12;
            let lend = (pos + 8 + sz).min(buf.len());
            while lpos + 8 <= lend {
                let sub_id = &buf[lpos..lpos + 4];
                let sub_sz = u32::from_le_bytes([
                    buf[lpos + 4],
                    buf[lpos + 5],
                    buf[lpos + 6],
                    buf[lpos + 7],
                ]) as usize;
                if sub_id == b"IART" && lpos + 8 + sub_sz <= lend {
                    let s = String::from_utf8_lossy(&buf[lpos + 8..lpos + 8 + sub_sz])
                        .trim_matches('\0')
                        .trim()
                        .to_string();
                    if !s.is_empty() {
                        autor = s;
                    }
                }
                lpos += 8 + sub_sz + (sub_sz & 1);
            }
        }
        if sz == 0 {
            break;
        }
        pos += 8 + sz + (sz & 1);
    }

    if byte_rate == 0 && buf.len() >= 32 {
        byte_rate = u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);
    }
    if byte_rate == 0 {
        return None;
    }
    if data_size == 0 {
        data_size = file_size.saturating_sub(44);
    }

    let dur_secs = (data_size / (byte_rate as u64)).max(1);
    let kbps = ((byte_rate as u64 * 8) / 1000).max(1);

    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", kbps),
        dimensoes: String::new(),
        autor,
    })
}

fn parse_flac(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let mut pos = 0usize;
    if buf.len() >= 10 && &buf[0..3] == b"ID3" {
        let tag_sz = (((buf[6] & 0x7F) as usize) << 21)
            | (((buf[7] & 0x7F) as usize) << 14)
            | (((buf[8] & 0x7F) as usize) << 7)
            | ((buf[9] & 0x7F) as usize);
        pos = 10 + tag_sz;
    }
    if pos + 4 > buf.len() || &buf[pos..pos + 4] != b"fLaC" {
        return None;
    }
    pos += 4;

    let mut sample_rate = 0u64;
    let mut total_samples = 0u64;
    let mut autor = String::new();

    while pos + 4 <= buf.len() {
        let header = buf[pos];
        let is_last = (header & 0x80) != 0;
        let block_type = header & 0x7F;
        let block_len = ((buf[pos + 1] as usize) << 16)
            | ((buf[pos + 2] as usize) << 8)
            | (buf[pos + 3] as usize);
        pos += 4;

        if pos + block_len > buf.len() {
            break;
        }

        if block_type == 0 && block_len >= 18 {
            // STREAMINFO
            let b = &buf[pos..pos + block_len];
            sample_rate =
                ((b[10] as u64) << 12) | ((b[11] as u64) << 4) | ((b[12] as u64) >> 4);
            total_samples = (((b[13] & 0x0F) as u64) << 32)
                | ((b[14] as u64) << 24)
                | ((b[15] as u64) << 16)
                | ((b[16] as u64) << 8)
                | (b[17] as u64);
        } else if block_type == 4 {
            // VORBIS_COMMENT
            if let Some(a) = extract_vorbis_artist(&buf[pos..pos + block_len]) {
                autor = a;
            }
        }

        pos += block_len;
        if is_last {
            break;
        }
    }

    if sample_rate > 0 && total_samples > 0 {
        let dur_secs = (total_samples / sample_rate).max(1);
        let kbps = ((file_size * 8) / (dur_secs * 1000)).max(1);
        return Some(MediaMetadata {
            duracao: formatar_duracao(dur_secs),
            taxa_bits: format!("{} kbps", kbps),
            dimensoes: String::new(),
            autor,
        });
    }

    None
}

fn extract_vorbis_artist(comment_block: &[u8]) -> Option<String> {
    if comment_block.len() < 8 {
        return None;
    }
    let vendor_len = u32::from_le_bytes([
        comment_block[0],
        comment_block[1],
        comment_block[2],
        comment_block[3],
    ]) as usize;
    let mut pos = 4 + vendor_len;
    if pos + 4 > comment_block.len() {
        return None;
    }
    let count = u32::from_le_bytes([
        comment_block[pos],
        comment_block[pos + 1],
        comment_block[pos + 2],
        comment_block[pos + 3],
    ]) as usize;
    pos += 4;

    for _ in 0..count.min(64) {
        if pos + 4 > comment_block.len() {
            break;
        }
        let clen = u32::from_le_bytes([
            comment_block[pos],
            comment_block[pos + 1],
            comment_block[pos + 2],
            comment_block[pos + 3],
        ]) as usize;
        pos += 4;
        if pos + clen > comment_block.len() {
            break;
        }
        let s = String::from_utf8_lossy(&comment_block[pos..pos + clen]);
        if let Some((k, v)) = s.split_once('=') {
            if k.eq_ignore_ascii_case("ARTIST") || k.eq_ignore_ascii_case("AUTHOR") {
                let trimmed = v.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        pos += clen;
    }
    None
}

fn parse_ogg_audio(file: &mut File, buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let mut sample_rate = 0u64;
    let mut nominal_bitrate = 0u64;
    let mut autor = String::new();

    for i in 0..buf.len().saturating_sub(28) {
        if &buf[i..i + 7] == b"\x01vorbis" && i + 28 <= buf.len() {
            sample_rate =
                u32::from_le_bytes([buf[i + 12], buf[i + 13], buf[i + 14], buf[i + 15]]) as u64;
            let br =
                i32::from_le_bytes([buf[i + 20], buf[i + 21], buf[i + 22], buf[i + 23]]);
            if br > 0 {
                nominal_bitrate = (br as u64) / 1000;
            }
        } else if &buf[i..i + 7] == b"\x03vorbis" {
            if let Some(a) = extract_vorbis_artist(&buf[i + 7..]) {
                autor = a;
            }
        } else if &buf[i..i + 8] == b"OpusHead" && i + 16 <= buf.len() {
            sample_rate = 48000;
        } else if &buf[i..i + 8] == b"OpusTags" {
            if let Some(a) = extract_vorbis_artist(&buf[i + 8..]) {
                autor = a;
            }
        } else if &buf[i..i + 8] == b"Speex   " && i + 52 <= buf.len() {
            sample_rate =
                u32::from_le_bytes([buf[i + 36], buf[i + 37], buf[i + 38], buf[i + 39]]) as u64;
            let br =
                i32::from_le_bytes([buf[i + 48], buf[i + 49], buf[i + 50], buf[i + 51]]);
            if br > 0 {
                nominal_bitrate = (br as u64) / 1000;
            }
        }
    }

    // Procura último granule_position no final do arquivo
    let mut max_granule = 0u64;
    let tail_size = 65536.min(file_size as usize);
    if file
        .seek(SeekFrom::Start(file_size.saturating_sub(tail_size as u64)))
        .is_ok()
    {
        let mut tail = vec![0u8; tail_size];
        if let Ok(tn) = file.read(&mut tail) {
            for i in 0..tn.saturating_sub(14) {
                if &tail[i..i + 4] == b"OggS" {
                    let g = u64::from_le_bytes([
                        tail[i + 6],
                        tail[i + 7],
                        tail[i + 8],
                        tail[i + 9],
                        tail[i + 10],
                        tail[i + 11],
                        tail[i + 12],
                        tail[i + 13],
                    ]);
                    if g != u64::MAX && g > max_granule {
                        max_granule = g;
                    }
                }
            }
        }
    }

    let dur_secs = if sample_rate > 0 && max_granule > 0 {
        (max_granule / sample_rate).max(1)
    } else if nominal_bitrate > 0 {
        ((file_size * 8) / (nominal_bitrate * 1000)).max(1)
    } else {
        0
    };

    if dur_secs == 0 && nominal_bitrate == 0 {
        return None;
    }

    let kbps = if dur_secs > 0 {
        ((file_size * 8) / (dur_secs * 1000)).max(1)
    } else {
        nominal_bitrate
    };

    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs.max(1)),
        taxa_bits: format!("{} kbps", kbps),
        dimensoes: String::new(),
        autor,
    })
}

fn parse_aiff(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let mut pos = 12usize;
    while pos + 8 <= buf.len() {
        let id = &buf[pos..pos + 4];
        let sz = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]])
            as usize;
        if id == b"COMM" && pos + 26 <= buf.len() {
            let num_frames =
                u32::from_be_bytes([buf[pos + 10], buf[pos + 11], buf[pos + 12], buf[pos + 13]])
                    as u64;
            let exp = (((buf[pos + 16] & 0x7F) as i32) << 8) | (buf[pos + 17] as i32);
            let mant = u64::from_be_bytes([
                buf[pos + 18],
                buf[pos + 19],
                buf[pos + 20],
                buf[pos + 21],
                buf[pos + 22],
                buf[pos + 23],
                buf[pos + 24],
                buf[pos + 25],
            ]);
            let sr = if exp == 0 && mant == 0 {
                0.0
            } else {
                (mant as f64) * 2.0f64.powi(exp - 16383 - 63)
            };
            if sr > 0.0 && num_frames > 0 {
                let secs = ((num_frames as f64 / sr).round() as u64).max(1);
                let kbps = ((file_size * 8) / (secs * 1000)).max(1);
                return Some(MediaMetadata {
                    duracao: formatar_duracao(secs),
                    taxa_bits: format!("{} kbps", kbps),
                    dimensoes: String::new(),
                    autor: String::new(),
                });
            }
        }
        if sz == 0 {
            break;
        }
        pos += 8 + sz + (sz & 1);
    }
    None
}

fn parse_mp3(file: &mut File, buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let n = buf.len();
    if n < 4 {
        return None;
    }

    let mut data_start = 0usize;
    let mut autor = String::new();

    // ID3v2
    if n >= 10 && &buf[0..3] == b"ID3" {
        let ver = buf[3];
        let tag_sz = (((buf[6] & 0x7F) as usize) << 21)
            | (((buf[7] & 0x7F) as usize) << 14)
            | (((buf[8] & 0x7F) as usize) << 7)
            | ((buf[9] & 0x7F) as usize);
        data_start = 10 + tag_sz;
        autor = extract_id3v2_artist(&buf[10..(10 + tag_sz).min(n)], ver);
    }

    // ID3v1 fallback
    if autor.is_empty() && file_size >= 128 {
        if file.seek(SeekFrom::End(-128)).is_ok() {
            let mut tag_buf = [0u8; 128];
            if file.read_exact(&mut tag_buf).is_ok() && &tag_buf[0..3] == b"TAG" {
                let s = String::from_utf8_lossy(&tag_buf[33..63])
                    .trim_matches('\0')
                    .trim()
                    .to_string();
                if !s.is_empty() {
                    autor = s;
                }
            }
        }
    }

    // Se o tag ID3v2 for maior que o buffer inicial, faz seek para data_start
    let mut frame_buf = Vec::new();
    let search_slice: &[u8] = if data_start + 512 <= n {
        &buf[data_start..n]
    } else if (data_start as u64) < file_size {
        if file.seek(SeekFrom::Start(data_start as u64)).is_ok() {
            frame_buf.resize(16384.min((file_size - data_start as u64) as usize), 0);
            let fn_read = file.read(&mut frame_buf).unwrap_or(0);
            frame_buf.truncate(fn_read);
        }
        &frame_buf
    } else {
        buf
    };

    const BITRATES_V1_L1: [u32; 16] = [
        0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0,
    ];
    const BITRATES_V1_L2: [u32; 16] = [
        0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0,
    ];
    const BITRATES_V1_L3: [u32; 16] = [
        0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
    ];
    const BITRATES_V2_L1: [u32; 16] = [
        0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0,
    ];
    const BITRATES_V2_L23: [u32; 16] = [
        0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
    ];
    const SAMPLE_RATES_V1: [u32; 4] = [44100, 48000, 32000, 0];
    const SAMPLE_RATES_V2: [u32; 4] = [22050, 24000, 16000, 0];
    const SAMPLE_RATES_V25: [u32; 4] = [11025, 12000, 8000, 0];

    for i in 0..search_slice.len().saturating_sub(4) {
        if search_slice[i] == 0xFF && (search_slice[i + 1] & 0xE0) == 0xE0 {
            let ver_bits = (search_slice[i + 1] >> 3) & 0x03;
            let layer_bits = (search_slice[i + 1] >> 1) & 0x03;
            let br_idx = ((search_slice[i + 2] >> 4) & 0x0F) as usize;
            let sr_idx = ((search_slice[i + 2] >> 2) & 0x03) as usize;

            if ver_bits == 1 || layer_bits == 0 || br_idx == 0 || br_idx == 15 || sr_idx == 3 {
                continue;
            }

            let kbps = match (ver_bits, layer_bits) {
                (3, 3) => BITRATES_V1_L1[br_idx],
                (3, 2) => BITRATES_V1_L2[br_idx],
                (3, 1) => BITRATES_V1_L3[br_idx],
                (_, 3) => BITRATES_V2_L1[br_idx],
                _ => BITRATES_V2_L23[br_idx],
            };
            let sr = match ver_bits {
                3 => SAMPLE_RATES_V1[sr_idx],
                2 => SAMPLE_RATES_V2[sr_idx],
                _ => SAMPLE_RATES_V25[sr_idx],
            };

            if kbps == 0 || sr == 0 {
                continue;
            }

            let samples_per_frame: u64 = match (ver_bits, layer_bits) {
                (_, 3) => 384,
                (3, _) => 1152,
                (_, 2) => 1152,
                _ => 576,
            };

            // Verifica cabeçalho VBR Xing / Info / VBRI
            let mut vbr_frames = 0u64;
            let look_end = (i + 200).min(search_slice.len());
            let frame_window = &search_slice[i..look_end];
            for k in 0..frame_window.len().saturating_sub(12) {
                if (&frame_window[k..k + 4] == b"Xing" || &frame_window[k..k + 4] == b"Info")
                    && k + 12 <= frame_window.len()
                {
                    let flags = u32::from_be_bytes([
                        frame_window[k + 4],
                        frame_window[k + 5],
                        frame_window[k + 6],
                        frame_window[k + 7],
                    ]);
                    if (flags & 1) != 0 {
                        vbr_frames = u32::from_be_bytes([
                            frame_window[k + 8],
                            frame_window[k + 9],
                            frame_window[k + 10],
                            frame_window[k + 11],
                        ]) as u64;
                    }
                    break;
                } else if &frame_window[k..k + 4] == b"VBRI" && k + 18 <= frame_window.len() {
                    vbr_frames = u32::from_be_bytes([
                        frame_window[k + 14],
                        frame_window[k + 15],
                        frame_window[k + 16],
                        frame_window[k + 17],
                    ]) as u64;
                    break;
                }
            }

            let audio_bytes = file_size.saturating_sub(data_start as u64);
            let (dur_secs, final_kbps) = if vbr_frames > 0 {
                let d = ((vbr_frames * samples_per_frame) / (sr as u64)).max(1);
                let br = ((audio_bytes * 8) / (d * 1000)).max(1);
                (d, br)
            } else {
                let d = ((audio_bytes * 8) / (kbps as u64 * 1000)).max(1);
                (d, kbps as u64)
            };

            return Some(MediaMetadata {
                duracao: formatar_duracao(dur_secs),
                taxa_bits: format!("{} kbps", final_kbps),
                dimensoes: String::new(),
                autor,
            });
        }
    }

    None
}

fn extract_id3v2_artist(tag_data: &[u8], version: u8) -> String {
    let mut pos = 0usize;
    let n = tag_data.len();

    while pos + 10 <= n {
        if tag_data[pos] == 0 {
            break;
        }
        if version == 2 {
            if pos + 6 > n {
                break;
            }
            let fid = &tag_data[pos..pos + 3];
            let fsize = ((tag_data[pos + 3] as usize) << 16)
                | ((tag_data[pos + 4] as usize) << 8)
                | (tag_data[pos + 5] as usize);
            pos += 6;
            if fsize == 0 || pos + fsize > n {
                break;
            }
            if fid == b"TP1" || fid == b"TP2" {
                let s = decode_id3_text(&tag_data[pos..pos + fsize]);
                if !s.is_empty() {
                    return s;
                }
            }
            pos += fsize;
        } else {
            let fid = &tag_data[pos..pos + 4];
            let fsize = if version == 4 {
                (((tag_data[pos + 4] & 0x7F) as usize) << 21)
                    | (((tag_data[pos + 5] & 0x7F) as usize) << 14)
                    | (((tag_data[pos + 6] & 0x7F) as usize) << 7)
                    | ((tag_data[pos + 7] & 0x7F) as usize)
            } else {
                u32::from_be_bytes([
                    tag_data[pos + 4],
                    tag_data[pos + 5],
                    tag_data[pos + 6],
                    tag_data[pos + 7],
                ]) as usize
            };
            pos += 10;
            if fsize == 0 || pos + fsize > n {
                break;
            }
            if fid == b"TPE1" || fid == b"TPE2" {
                let s = decode_id3_text(&tag_data[pos..pos + fsize]);
                if !s.is_empty() {
                    return s;
                }
            }
            pos += fsize;
        }
    }
    String::new()
}

fn decode_id3_text(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let enc = data[0];
    let payload = &data[1..];
    match enc {
        1 | 2 => {
            // UTF-16
            let mut u16s = Vec::new();
            let le = payload.len() >= 2 && payload[0] == 0xFF && payload[1] == 0xFE;
            let start = if payload.len() >= 2
                && ((payload[0] == 0xFF && payload[1] == 0xFE)
                    || (payload[0] == 0xFE && payload[1] == 0xFF))
            {
                2
            } else {
                0
            };
            for chunk in payload[start..].chunks_exact(2) {
                let code = if le || enc == 1 {
                    u16::from_le_bytes([chunk[0], chunk[1]])
                } else {
                    u16::from_be_bytes([chunk[0], chunk[1]])
                };
                if code == 0 {
                    break;
                }
                u16s.push(code);
            }
            String::from_utf16_lossy(&u16s).trim().to_string()
        }
        _ => String::from_utf8_lossy(payload)
            .trim_matches('\0')
            .trim()
            .to_string(),
    }
}

fn parse_isobmff(file: &mut File, file_size: u64) -> Option<MediaMetadata> {
    let _ = file.seek(SeekFrom::Start(0));
    let mut pos = 0u64;
    let mut moov_buf = Vec::new();

    while pos + 8 <= file_size {
        if file.seek(SeekFrom::Start(pos)).is_err() {
            break;
        }
        let mut hdr = [0u8; 16];
        let hr = file.read(&mut hdr).unwrap_or(0);
        if hr < 8 {
            break;
        }
        let sz32 = u32::from_be_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as u64;
        let btype = &hdr[4..8];
        let (box_size, header_len) = if sz32 == 1 && hr >= 16 {
            let sz64 = u64::from_be_bytes([
                hdr[8], hdr[9], hdr[10], hdr[11], hdr[12], hdr[13], hdr[14], hdr[15],
            ]);
            (sz64, 16u64)
        } else if sz32 == 0 {
            (file_size - pos, 8u64)
        } else {
            (sz32, 8u64)
        };

        if box_size < header_len {
            break;
        }

        if btype == b"moov" {
            let payload_len = (box_size - header_len).min(4 * 1024 * 1024) as usize;
            if file.seek(SeekFrom::Start(pos + header_len)).is_ok() {
                moov_buf.resize(payload_len, 0);
                let mr = file.read(&mut moov_buf).unwrap_or(0);
                moov_buf.truncate(mr);
            }
            break;
        }

        pos += box_size;
    }

    if moov_buf.is_empty() {
        return None;
    }

    let mut duration_secs = 0u64;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut autor = String::new();

    parse_moov_boxes(
        &moov_buf,
        &mut duration_secs,
        &mut width,
        &mut height,
        &mut autor,
    );

    if duration_secs == 0 && width == 0 && autor.is_empty() {
        return None;
    }

    let duracao = if duration_secs > 0 {
        formatar_duracao(duration_secs)
    } else {
        String::new()
    };
    let taxa_bits = if duration_secs > 0 {
        let kbps = ((file_size * 8) / (duration_secs * 1000)).max(1);
        format!("{} kbps", kbps)
    } else {
        String::new()
    };
    let dimensoes = if width > 0 && height > 0 {
        format!("{}x{}", width, height)
    } else {
        String::new()
    };

    Some(MediaMetadata {
        duracao,
        taxa_bits,
        dimensoes,
        autor,
    })
}

fn parse_moov_boxes(
    buf: &[u8],
    duration_secs: &mut u64,
    width: &mut u32,
    height: &mut u32,
    autor: &mut String,
) {
    let mut pos = 0usize;
    let n = buf.len();

    while pos + 8 <= n {
        let sz = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]) as usize;
        let btype = &buf[pos + 4..pos + 8];
        let box_size = if sz == 0 { n - pos } else { sz };
        if box_size < 8 || pos + box_size > n {
            break;
        }
        let payload = &buf[pos + 8..pos + box_size];

        match btype {
            b"mvhd" => {
                if payload.len() >= 20 {
                    let ver = payload[0];
                    let (ts, dur) = if ver == 0 && payload.len() >= 20 {
                        let t = u32::from_be_bytes([
                            payload[12],
                            payload[13],
                            payload[14],
                            payload[15],
                        ]) as u64;
                        let d = u32::from_be_bytes([
                            payload[16],
                            payload[17],
                            payload[18],
                            payload[19],
                        ]) as u64;
                        (t, d)
                    } else if ver == 1 && payload.len() >= 32 {
                        let t = u32::from_be_bytes([
                            payload[20],
                            payload[21],
                            payload[22],
                            payload[23],
                        ]) as u64;
                        let d = u64::from_be_bytes([
                            payload[24],
                            payload[25],
                            payload[26],
                            payload[27],
                            payload[28],
                            payload[29],
                            payload[30],
                            payload[31],
                        ]);
                        (t, d)
                    } else {
                        (0, 0)
                    };
                    if ts > 0 && dur > 0 {
                        *duration_secs = (dur / ts).max(1);
                    }
                }
            }
            b"tkhd" => {
                let ver = payload.first().copied().unwrap_or(0);
                let (w_off, h_off) = if ver == 0 { (76, 80) } else { (88, 92) };
                if h_off + 2 <= payload.len() {
                    let w = u16::from_be_bytes([payload[w_off], payload[w_off + 1]]) as u32;
                    let h = u16::from_be_bytes([payload[h_off], payload[h_off + 1]]) as u32;
                    if w * h > (*width) * (*height) {
                        *width = w;
                        *height = h;
                    }
                }
            }
            b"stsd" => {
                if payload.len() >= 44
                    && !matches!(
                        &payload[12..16],
                        b"mp4a"
                            | b"alac"
                            | b"ac-3"
                            | b"ec-3"
                            | b"Opus"
                            | b"fLaC"
                            | b"samr"
                            | b"sawb"
                            | b"twos"
                            | b"sowt"
                            | b"lpcm"
                    )
                {
                    let w = u16::from_be_bytes([payload[40], payload[41]]) as u32;
                    let h = u16::from_be_bytes([payload[42], payload[43]]) as u32;
                    if w > 0 && h > 0 && w <= 16384 && h <= 16384 && (*width == 0 || *height == 0) {
                        *width = w;
                        *height = h;
                    }
                }
            }
            b"\xA9ART" => {
                // Apple iTunes Artist atom
                if payload.len() >= 16 && &payload[4..8] == b"data" {
                    let s = String::from_utf8_lossy(&payload[16..])
                        .trim_matches('\0')
                        .trim()
                        .to_string();
                    if !s.is_empty() {
                        *autor = s;
                    }
                }
            }
            b"trak" | b"mdia" | b"minf" | b"stbl" | b"udta" | b"ilst" => {
                parse_moov_boxes(payload, duration_secs, width, height, autor);
            }
            b"meta" => {
                if payload.len() > 4 {
                    parse_moov_boxes(&payload[4..], duration_secs, width, height, autor);
                }
            }
            _ => {}
        }

        pos += box_size;
    }
}

fn parse_ebml(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let n = buf.len();
    let mut timecode_scale: u64 = 1_000_000;
    let mut duration_raw: f64 = 0.0;
    let mut width: u32 = 0;
    let mut height: u32 = 0;

    for i in 0..n.saturating_sub(12) {
        // TimecodeScale (0x2AD7B1)
        if buf[i] == 0x2A && buf[i + 1] == 0xD7 && buf[i + 2] == 0xB1 {
            let len = (buf[i + 3] & 0x7F) as usize;
            if len >= 1 && len <= 8 && i + 4 + len <= n {
                let mut val = 0u64;
                for &b in &buf[i + 4..i + 4 + len] {
                    val = (val << 8) | (b as u64);
                }
                if val > 0 {
                    timecode_scale = val;
                }
            }
        }
        // Duration (0x4489)
        if buf[i] == 0x44 && buf[i + 1] == 0x89 {
            let len = (buf[i + 2] & 0x7F) as usize;
            if len == 4 && i + 7 <= n {
                let v = f32::from_be_bytes([buf[i + 3], buf[i + 4], buf[i + 5], buf[i + 6]]) as f64;
                if v > 0.0 && v < 1e12 {
                    duration_raw = v;
                }
            } else if len == 8 && i + 11 <= n {
                let v = f64::from_be_bytes([
                    buf[i + 3],
                    buf[i + 4],
                    buf[i + 5],
                    buf[i + 6],
                    buf[i + 7],
                    buf[i + 8],
                    buf[i + 9],
                    buf[i + 10],
                ]);
                if v > 0.0 && v < 1e12 {
                    duration_raw = v;
                }
            }
        }
        // Video Track Settings (0xE0) -> PixelWidth (0xB0) & PixelHeight (0xBA)
        if buf[i] == 0xB0 && (buf[i + 1] == 0x81 || buf[i + 1] == 0x82) && width == 0 {
            let len = (buf[i + 1] & 0x7F) as usize;
            if i + 2 + len <= n {
                let mut w = 0u32;
                for &b in &buf[i + 2..i + 2 + len] {
                    w = (w << 8) | (b as u32);
                }
                // Verifica se 0xBA (PixelHeight) aparece logo em seguida
                let after = i + 2 + len;
                for k in after..(after + 16).min(n.saturating_sub(4)) {
                    if buf[k] == 0xBA && (buf[k + 1] == 0x81 || buf[k + 1] == 0x82) {
                        let hlen = (buf[k + 1] & 0x7F) as usize;
                        if k + 2 + hlen <= n {
                            let mut h = 0u32;
                            for &b in &buf[k + 2..k + 2 + hlen] {
                                h = (h << 8) | (b as u32);
                            }
                            if w >= 16 && w <= 16384 && h >= 16 && h <= 16384 {
                                width = w;
                                height = h;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let dur_secs = if duration_raw > 0.0 {
        (((duration_raw * (timecode_scale as f64)) / 1_000_000_000.0).round() as u64).max(1)
    } else {
        ((file_size * 8) / (2500 * 1000)).max(1)
    };
    let kbps = ((file_size * 8) / (dur_secs * 1000)).max(1);

    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", kbps),
        dimensoes: if width > 0 && height > 0 {
            format!("{}x{}", width, height)
        } else {
            String::new()
        },
        autor: String::new(),
    })
}

fn parse_avi(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let n = buf.len();
    for i in 12..n.saturating_sub(48) {
        if &buf[i..i + 4] == b"avih" {
            let usec_per_frame =
                u32::from_le_bytes([buf[i + 8], buf[i + 9], buf[i + 10], buf[i + 11]]) as u64;
            let total_frames =
                u32::from_le_bytes([buf[i + 24], buf[i + 25], buf[i + 26], buf[i + 27]]) as u64;
            let width = u32::from_le_bytes([buf[i + 40], buf[i + 41], buf[i + 42], buf[i + 43]]);
            let height = u32::from_le_bytes([buf[i + 44], buf[i + 45], buf[i + 46], buf[i + 47]]);

            let dur_secs = if usec_per_frame > 0 && total_frames > 0 {
                ((usec_per_frame * total_frames) / 1_000_000).max(1)
            } else {
                ((file_size * 8) / (2000 * 1000)).max(1)
            };
            let kbps = ((file_size * 8) / (dur_secs * 1000)).max(1);

            return Some(MediaMetadata {
                duracao: formatar_duracao(dur_secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: if width > 0 && height > 0 {
                    format!("{}x{}", width, height)
                } else {
                    String::new()
                },
                autor: String::new(),
            });
        }
    }
    None
}

fn parse_asf(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    const FILE_PROP_GUID: [u8; 16] = [
        0xA1, 0xDC, 0xAB, 0x8C, 0x47, 0xA9, 0xCF, 0x11, 0x8E, 0xE4, 0x00, 0xC0, 0x0C, 0x20, 0x53,
        0x65,
    ];
    const VIDEO_MEDIA_GUID: [u8; 16] = [
        0xC0, 0xEF, 0x19, 0xBC, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44,
        0x2B,
    ];

    let n = buf.len();
    let mut dur_secs = 0u64;
    let mut max_bitrate = 0u64;
    let mut width = 0u32;
    let mut height = 0u32;

    if let Some(pos) = buf.windows(16).position(|w| w == FILE_PROP_GUID) {
        if pos + 104 <= n {
            let play_dur = u64::from_le_bytes([
                buf[pos + 64],
                buf[pos + 65],
                buf[pos + 66],
                buf[pos + 67],
                buf[pos + 68],
                buf[pos + 69],
                buf[pos + 70],
                buf[pos + 71],
            ]);
            let preroll = u64::from_le_bytes([
                buf[pos + 80],
                buf[pos + 81],
                buf[pos + 82],
                buf[pos + 83],
                buf[pos + 84],
                buf[pos + 85],
                buf[pos + 86],
                buf[pos + 87],
            ]);
            max_bitrate = u32::from_le_bytes([
                buf[pos + 96],
                buf[pos + 97],
                buf[pos + 98],
                buf[pos + 99],
            ]) as u64;
            let net_ms = (play_dur / 10_000).saturating_sub(preroll);
            if net_ms > 0 {
                dur_secs = (net_ms / 1000).max(1);
            }
        }
    }

    if let Some(vpos) = buf.windows(16).position(|w| w == VIDEO_MEDIA_GUID) {
        if vpos + 86 <= n {
            width = u32::from_le_bytes([
                buf[vpos + 78],
                buf[vpos + 79],
                buf[vpos + 80],
                buf[vpos + 81],
            ]);
            height = u32::from_le_bytes([
                buf[vpos + 82],
                buf[vpos + 83],
                buf[vpos + 84],
                buf[vpos + 85],
            ]);
        }
    }

    if dur_secs == 0 && max_bitrate > 0 {
        dur_secs = ((file_size * 8) / max_bitrate).max(1);
    }
    if dur_secs == 0 {
        dur_secs = ((file_size * 8) / (1000 * 1000)).max(1);
    }
    let kbps = if max_bitrate > 0 {
        (max_bitrate / 1000).max(1)
    } else {
        ((file_size * 8) / (dur_secs * 1000)).max(1)
    };

    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", kbps),
        dimensoes: if width > 0 && height > 0 {
            format!("{}x{}", width, height)
        } else {
            String::new()
        },
        autor: String::new(),
    })
}

fn parse_flv(file: &mut File, buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let extract_amf_num = |key: &[u8]| -> Option<f64> {
        let pos = buf.windows(key.len()).position(|w| w == key)?;
        let val_pos = pos + key.len();
        if val_pos + 9 <= buf.len() && buf[val_pos] == 0x00 {
            let v = f64::from_be_bytes([
                buf[val_pos + 1],
                buf[val_pos + 2],
                buf[val_pos + 3],
                buf[val_pos + 4],
                buf[val_pos + 5],
                buf[val_pos + 6],
                buf[val_pos + 7],
                buf[val_pos + 8],
            ]);
            if v.is_finite() && v > 0.0 {
                return Some(v);
            }
        }
        None
    };

    let mut dur_secs = extract_amf_num(b"duration")
        .map(|d| d.round() as u64)
        .unwrap_or(0);
    let width = extract_amf_num(b"width")
        .map(|w| w.round() as u32)
        .unwrap_or(0);
    let height = extract_amf_num(b"height")
        .map(|h| h.round() as u32)
        .unwrap_or(0);

    // Fallback: lê o último tag size no fim do FLV para obter o timestamp do último tag
    if dur_secs == 0 && file_size > 24 {
        if file.seek(SeekFrom::End(-4)).is_ok() {
            let mut lb = [0u8; 4];
            if file.read_exact(&mut lb).is_ok() {
                let prev_tag_size = u32::from_be_bytes(lb) as u64;
                if prev_tag_size >= 11 && prev_tag_size + 4 <= file_size {
                    if file
                        .seek(SeekFrom::End(-((prev_tag_size + 4) as i64)))
                        .is_ok()
                    {
                        let mut thdr = [0u8; 8];
                        if file.read_exact(&mut thdr).is_ok() {
                            let ts_ms = ((thdr[4] as u32) << 16)
                                | ((thdr[5] as u32) << 8)
                                | (thdr[6] as u32)
                                | ((thdr[7] as u32) << 24);
                            if ts_ms > 0 {
                                dur_secs = (ts_ms as u64 / 1000).max(1);
                            }
                        }
                    }
                }
            }
        }
    }

    if dur_secs == 0 {
        dur_secs = ((file_size * 8) / (1500 * 1000)).max(1);
    }
    let kbps = ((file_size * 8) / (dur_secs * 1000)).max(1);

    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", kbps),
        dimensoes: if width > 0 && height > 0 {
            format!("{}x{}", width, height)
        } else {
            String::new()
        },
        autor: String::new(),
    })
}

fn parse_ogv(file: &mut File, buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let mut width = 0u32;
    let mut height = 0u32;
    let mut fps = 0.0f64;

    for i in 0..buf.len().saturating_sub(42) {
        if &buf[i..i + 7] == b"\x80theora" {
            let w_mb = ((buf[i + 14] as u32) << 16)
                | ((buf[i + 15] as u32) << 8)
                | (buf[i + 16] as u32);
            let h_mb = ((buf[i + 17] as u32) << 16)
                | ((buf[i + 18] as u32) << 8)
                | (buf[i + 19] as u32);
            if w_mb > 0 && h_mb > 0 {
                width = w_mb;
                height = h_mb;
            }
            let frn = u32::from_be_bytes([buf[i + 22], buf[i + 23], buf[i + 24], buf[i + 25]]);
            let frd = u32::from_be_bytes([buf[i + 26], buf[i + 27], buf[i + 28], buf[i + 29]]);
            if frn > 0 && frd > 0 {
                fps = (frn as f64) / (frd as f64);
            }
            break;
        }
    }

    let mut base = parse_ogg_audio(file, buf, file_size).unwrap_or_default();
    if width > 0 && height > 0 {
        base.dimensoes = format!("{}x{}", width, height);
    }
    if base.duracao.is_empty() {
        let dur_secs = if fps > 0.0 {
            ((file_size * 8) / (1500 * 1000)).max(1)
        } else {
            ((file_size * 8) / (1500 * 1000)).max(1)
        };
        base.duracao = formatar_duracao(dur_secs);
        base.taxa_bits = format!("{} kbps", ((file_size * 8) / (dur_secs * 1000)).max(1));
    }
    Some(base)
}

fn parse_mpeg_ps(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    let mut width = 0u32;
    let mut height = 0u32;
    let mut br_kbps = 0u64;

    for i in 0..buf.len().saturating_sub(12) {
        if buf[i] == 0x00 && buf[i + 1] == 0x00 && buf[i + 2] == 0x01 && buf[i + 3] == 0xB3 {
            let w = ((buf[i + 4] as u32) << 4) | ((buf[i + 5] as u32) >> 4);
            let h = (((buf[i + 5] & 0x0F) as u32) << 8) | (buf[i + 6] as u32);
            let br_raw = ((buf[i + 8] as u32) << 10)
                | ((buf[i + 9] as u32) << 2)
                | ((buf[i + 10] as u32) >> 6);
            if w > 0 && h > 0 {
                width = w;
                height = h;
            }
            if br_raw > 0 && br_raw != 0x3FFFF {
                br_kbps = ((br_raw as u64) * 400) / 1000;
            }
            break;
        }
    }

    let kbps = if br_kbps > 0 { br_kbps } else { 4000 };
    let dur_secs = ((file_size * 8) / (kbps * 1000)).max(1);

    Some(MediaMetadata {
        duracao: formatar_duracao(dur_secs),
        taxa_bits: format!("{} kbps", kbps),
        dimensoes: if width > 0 && height > 0 {
            format!("{}x{}", width, height)
        } else {
            String::new()
        },
        autor: String::new(),
    })
}

fn parse_realmedia(buf: &[u8], file_size: u64) -> Option<MediaMetadata> {
    for i in 0..buf.len().saturating_sub(40) {
        if &buf[i..i + 4] == b"PROP" {
            let avg_br =
                u32::from_be_bytes([buf[i + 14], buf[i + 15], buf[i + 16], buf[i + 17]]) as u64;
            let dur_ms =
                u32::from_be_bytes([buf[i + 26], buf[i + 27], buf[i + 28], buf[i + 29]]) as u64;
            let dur_secs = if dur_ms > 0 {
                (dur_ms / 1000).max(1)
            } else {
                ((file_size * 8) / avg_br.max(500_000)).max(1)
            };
            let kbps = if avg_br > 0 {
                (avg_br / 1000).max(1)
            } else {
                ((file_size * 8) / (dur_secs * 1000)).max(1)
            };
            return Some(MediaMetadata {
                duracao: formatar_duracao(dur_secs),
                taxa_bits: format!("{} kbps", kbps),
                dimensoes: String::new(),
                autor: String::new(),
            });
        }
    }
    None
}
