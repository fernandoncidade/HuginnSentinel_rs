use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use chrono::{Datelike, Local, NaiveDate};
use regex::Regex;

pub const PT_MONTHS: [&str; 12] = [
    "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
    "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
];
pub const EN_MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];
pub const ES_MONTHS: [&str; 12] = [
    "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
    "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre",
];
pub const FR_MONTHS: [&str; 12] = [
    "Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
    "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre",
];
pub const IT_MONTHS: [&str; 12] = [
    "Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno",
    "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre",
];
pub const DE_MONTHS: [&str; 12] = [
    "Januar", "Februar", "März", "April", "Mai", "Juni",
    "Juli", "August", "September", "Oktober", "November", "Dezember",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangInput {
    pub version: String,
    pub day: u32,
    pub month_index: u32, // 0..11
    pub year: i32,
}

impl Default for LangInput {
    fn default() -> Self {
        let today = Local::now().date_naive();
        let day = today.day();
        let month_index = today.month() - 1;
        let year = today.year();
        let version = format_version_from_date(year, month_index + 1, day, "");
        Self {
            version,
            day,
            month_index,
            year,
        }
    }
}

impl LangInput {
    pub fn new(ver: &str, d: u32, m: u32, y: i32) -> Self {
        Self {
            version: ver.to_string(),
            day: d,
            month_index: m,
            year: y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub key: String,
    pub rel_path: String,
    pub default_group: String,
    pub lang: String,
    pub is_updatable: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileCheckStatus {
    pub key: String,
    pub rel_path: String,
    pub group: String,
    pub lang: String,
    pub absolute_path: PathBuf,
    pub exists: bool,
    pub detected_version: Option<String>,
    pub detected_date: Option<String>,
    pub found_patterns: Vec<String>,
    pub missing_patterns: Vec<String>,
    pub is_updatable: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UpdateResult {
    pub key: String,
    pub rel_path: String,
    pub group: String,
    pub ok: bool,
    pub msg: String,
}

pub fn project_root_dir() -> PathBuf {
    // 1. Check parent directories of current executable
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        for _ in 0..8 {
            if let Some(dir) = cur {
                if let Some(name) = dir.file_name() {
                    if name == "version" || name == "mocks" {
                        cur = dir.parent();
                        continue;
                    }
                }
                if dir.join("Cargo.toml").exists() && (dir.join("src").exists() || dir.join("source").exists()) {
                    return dir.to_path_buf();
                }
                cur = dir.parent();
            } else {
                break;
            }
        }
    }

    // 2. Check current working directory and ancestors
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = Some(cwd.as_path());
        for _ in 0..8 {
            if let Some(dir) = cur {
                if let Some(name) = dir.file_name() {
                    if name == "version" || name == "mocks" {
                        cur = dir.parent();
                        continue;
                    }
                }
                if dir.join("Cargo.toml").exists() && (dir.join("src").exists() || dir.join("source").exists()) {
                    return dir.to_path_buf();
                }
                cur = dir.parent();
            } else {
                break;
            }
        }
    }

    // Default fallback
    PathBuf::from(".")
}

pub fn plain_version(version: &str) -> String {
    let mut v = version.trim();
    if v.starts_with('v') || v.starts_with('V') {
        v = &v[1..];
    }
    v.to_string()
}

pub fn readme_version(version: &str) -> String {
    let plain = plain_version(version);
    format!("v{}", plain)
}

pub fn cargo_version(version: &str) -> String {
    let plain = plain_version(version);
    let parts: Vec<&str> = plain.split('.').collect();
    if parts.len() >= 3 {
        format!("{}.{}.{}", parts[0], parts[1], parts[2])
    } else {
        plain
    }
}

pub fn format_version_from_date(year: i32, month: u32, day: u32, existing_version: &str) -> String {
    let mut revision = "0".to_string();
    if !existing_version.is_empty() {
        let plain = plain_version(existing_version);
        let parts: Vec<&str> = plain.split('.').collect();
        if parts.len() >= 4 {
            if let Ok(rev) = parts[3].parse::<u32>() {
                revision = rev.to_string();
            }
        }
    }
    format!("{}.{}.{}.{}", year, month, day, revision)
}

pub fn parse_month_name_to_index(month_name: &str) -> Option<u32> {
    let mut norm = month_name.trim().to_lowercase();

    if let Ok(num) = norm.parse::<u32>() {
        if (1..=12).contains(&num) {
            return Some(num - 1);
        }
    }

    let replacements = [
        ("á", "a"), ("ã", "a"), ("à", "a"), ("ä", "a"),
        ("é", "e"), ("ê", "e"), ("è", "e"), ("ë", "e"),
        ("í", "i"), ("ì", "i"), ("ï", "i"),
        ("ó", "o"), ("õ", "o"), ("ô", "o"), ("ö", "o"),
        ("ú", "u"), ("ù", "u"), ("ü", "u"),
        ("ç", "c"), ("ñ", "n"), ("ß", "ss"),
    ];
    for (from, to) in replacements {
        norm = norm.replace(from, to);
    }

    // 0: Janeiro (pt), January (en), Enero (es), Janvier (fr), Gennaio (it), Januar (de)
    if norm.starts_with("jan") || norm.starts_with("ene") || norm.starts_with("gen") { return Some(0); }
    // 1: Fevereiro (pt), February (en), Febrero (es), Février (fr), Febbraio (it), Februar (de)
    if norm.starts_with("fev") || norm.starts_with("feb") { return Some(1); }
    // 2: Março (pt), March (en), Marzo (es/it), Mars (fr), März (de)
    if norm.starts_with("mar") { return Some(2); }
    // 3: Abril (pt/es), April (en/de), Avril (fr), Aprile (it)
    if norm.starts_with("abr") || norm.starts_with("apr") || norm.starts_with("avr") || norm.starts_with("av") { return Some(3); }
    // 4: Maio (pt), May (en), Mayo (es), Mai (fr/de), Maggio (it)
    if norm.starts_with("mai") || norm.starts_with("may") || norm.starts_with("mag") { return Some(4); }
    // 5: Junho (pt), June (en), Junio (es), Juin (fr), Giugno (it), Juni (de)
    if norm.starts_with("jun") || norm.starts_with("giu") || norm.starts_with("juin") { return Some(5); }
    // 6: Julho (pt), July (en), Julio (es), Juillet (fr), Luglio (it), Juli (de)
    if norm.starts_with("jul") || norm.starts_with("lug") || norm.starts_with("juil") { return Some(6); }
    // 7: Agosto (pt/es/it), August (en/de), Août (fr)
    if norm.starts_with("ago") || norm.starts_with("aug") || norm.starts_with("aou") || norm.starts_with("aot") { return Some(7); }
    // 8: Setembro (pt), September (en/de), Septiembre (es), Septembre (fr), Settembre (it)
    if norm.starts_with("set") || norm.starts_with("sep") { return Some(8); }
    // 9: Outubro (pt), October (en), Octubre (es), Octobre (fr), Ottobre (it), Oktober (de)
    if norm.starts_with("out") || norm.starts_with("oct") || norm.starts_with("okt") || norm.starts_with("ott") { return Some(9); }
    // 10: Novembro (pt), November (en/de), Noviembre (es), Novembre (fr/it)
    if norm.starts_with("nov") { return Some(10); }
    // 11: Dezembro (pt), December (en), Diciembre (es), Décembre (fr), Dicembre (it), Dezember (de)
    if norm.starts_with("dez") || norm.starts_with("dec") || norm.starts_with("dic") { return Some(11); }

    None
}

static RE_VERSION_TO_DATE: OnceLock<Regex> = OnceLock::new();
static RE_DATE_TEXTUAL: OnceLock<Regex> = OnceLock::new();
static RE_DATE_MONTH_FIRST: OnceLock<Regex> = OnceLock::new();
static RE_DATE_ISO: OnceLock<Regex> = OnceLock::new();
static RE_DATE_DMY: OnceLock<Regex> = OnceLock::new();
static RE_VERSION_CALVER: OnceLock<Regex> = OnceLock::new();
static RE_VERSION_LABEL: OnceLock<Regex> = OnceLock::new();
static RE_VERSION_V_PREFIX: OnceLock<Regex> = OnceLock::new();

pub fn parse_version_to_date(version_str: &str) -> Option<(u32, u32, i32, Option<u32>)> {
    let re = RE_VERSION_TO_DATE.get_or_init(|| {
        Regex::new(r"(?i)^v?(\d{4})\.(\d{1,2})\.(\d{1,2})(?:\.(\d+))?$").unwrap()
    });
    let caps = re.captures(version_str.trim())?;
    let y = caps[1].parse::<i32>().ok()?;
    let mo = caps[2].parse::<u32>().ok()?;
    let d = caps[3].parse::<u32>().ok()?;
    if (1..=12).contains(&mo) && (1..=31).contains(&d) && (1900..=3000).contains(&y) {
        if NaiveDate::from_ymd_opt(y, mo, d).is_some() {
            let rev = caps.get(4).and_then(|m| m.as_str().parse::<u32>().ok());
            return Some((d, mo - 1, y, rev));
        }
    }
    None
}

pub fn parse_date_string(raw_text: &str) -> Option<(u32, u32, i32)> {
    let mut text = raw_text.trim();
    if text.starts_with('`') && text.ends_with('`') && text.len() >= 2 {
        text = text[1..text.len() - 1].trim();
    }

    // 1. Textual: e.g. "28 de setembro de 2026", "28 September 2026", "28. September 2026", "30. Oktober 2026", "30 octobre 2026"
    let re_textual = RE_DATE_TEXTUAL.get_or_init(|| {
        Regex::new(r"(?i)\b(\d{1,2})(?:st|nd|rd|th|\.)?\s+(?:de\s+|del\s+|d'|du\s+)?([A-Za-zÀ-ÿ]+)(?:\s+(?:de|del|d'|du))?\s+(20[2-9]\d)\b").unwrap()
    });
    if let Some(m) = re_textual.captures(text) {
        if let (Ok(d), Ok(y)) = (m[1].parse::<u32>(), m[3].parse::<i32>()) {
            if let Some(m_idx) = parse_month_name_to_index(&m[2]) {
                if (1..=31).contains(&d) && (1900..=3000).contains(&y) {
                    return Some((d, m_idx, y));
                }
            }
        }
    }

    // 2. Textual month first: e.g. "September 28, 2026", "October 30, 2026", "October 30 2026"
    let re_month_first = RE_DATE_MONTH_FIRST.get_or_init(|| {
        Regex::new(r"(?i)\b([A-Za-zÀ-ÿ]+)\s+(\d{1,2})(?:st|nd|rd|th)?(?:,?\s+)(20[2-9]\d)\b").unwrap()
    });
    if let Some(m) = re_month_first.captures(text) {
        if let Some(m_idx) = parse_month_name_to_index(&m[1]) {
            if let (Ok(d), Ok(y)) = (m[2].parse::<u32>(), m[3].parse::<i32>()) {
                if (1..=31).contains(&d) && (1900..=3000).contains(&y) {
                    return Some((d, m_idx, y));
                }
            }
        }
    }

    // 3. Numeric ISO: e.g. "2026-09-28", "2026/09/28"
    let re_iso = RE_DATE_ISO.get_or_init(|| {
        Regex::new(r"\b(20[2-9]\d)[-/](\d{1,2})[-/](\d{1,2})\b").unwrap()
    });
    if let Some(m) = re_iso.captures(text) {
        if let (Ok(y), Ok(mo), Ok(d)) = (m[1].parse::<i32>(), m[2].parse::<u32>(), m[3].parse::<u32>()) {
            if (1..=12).contains(&mo) && (1..=31).contains(&d) && (1900..=3000).contains(&y) {
                return Some((d, mo - 1, y));
            }
        }
    }

    // 4. Numeric DMY or MDY: e.g. "28/09/2026", "10/21/2026", "28.09.2026"
    let re_dmy = RE_DATE_DMY.get_or_init(|| {
        Regex::new(r"\b(\d{1,2})[-/.](\d{1,2})[-/.](20[2-9]\d)\b").unwrap()
    });
    if let Some(m) = re_dmy.captures(text) {
        if let (Ok(n1), Ok(n2), Ok(y)) = (m[1].parse::<u32>(), m[2].parse::<u32>(), m[3].parse::<i32>()) {
            if (1..=31).contains(&n1) && (1..=12).contains(&n2) && (1900..=3000).contains(&y) {
                // DMY format (e.g. 21/10/2026)
                return Some((n1, n2 - 1, y));
            } else if (1..=12).contains(&n1) && (1..=31).contains(&n2) && (1900..=3000).contains(&y) {
                // MDY format (e.g. 10/21/2026)
                return Some((n2, n1 - 1, y));
            }
        }
    }

    None
}

pub fn parse_version_string(raw_text: &str) -> Option<String> {
    // 1. CalVer pattern: e.g. 2026.10.30.0, 2026.10.30, v2026.10.30.0, v2026.10.30 (year 2020..=2099)
    let re_calver = RE_VERSION_CALVER.get_or_init(|| {
        Regex::new(r"(?i)\b((?:v|V)?20[2-9]\d\.(?:0?[1-9]|1[0-2])(?:\.(?:0?[1-9]|[12]\d|3[01]))?(?:\.\d+)?)\b").unwrap()
    });
    if let Some(caps) = re_calver.captures(raw_text) {
        return Some(caps[1].to_string());
    }

    // 2. Explicit version label: e.g. version = "1.0", Versão: 2.1, app_version: "3.0"
    let re_label = RE_VERSION_LABEL.get_or_init(|| {
        Regex::new(r#"(?i)\b(?:vers[aã]o|version|versione|versionen|app_version)\b\s*[:=\-]?\s*["'`]?\s*((?:v|V)?[0-9]+(?:\.[0-9]+){1,3})\b"#).unwrap()
    });
    if let Some(caps) = re_label.captures(raw_text) {
        return Some(caps[1].to_string());
    }

    // 3. Explicit 'v' prefix: e.g. v3.24, v1.0.0, v2026.10.30.0
    let re_v = RE_VERSION_V_PREFIX.get_or_init(|| {
        Regex::new(r"\b([vV][0-9]+(?:\.[0-9]+){1,3})\b").unwrap()
    });
    if let Some(caps) = re_v.captures(raw_text) {
        return Some(caps[1].to_string());
    }

    None
}

pub fn is_release_notes_file(rel_path: &str) -> bool {
    let norm = rel_path.replace('\\', "/").to_lowercase();
    norm.ends_with("release notes_de_de.txt")
        || norm.ends_with("release notes_en_us.txt")
        || norm.ends_with("release notes_es_es.txt")
        || norm.ends_with("release notes_fr_fr.txt")
        || norm.ends_with("release notes_it_it.txt")
        || norm.ends_with("release notes_pt_br.txt")
}

#[allow(dead_code)]
pub fn is_updatable_file(rel_path: &str) -> bool {
    if is_release_notes_file(rel_path) {
        return false;
    }
    true
}

pub fn resolve_file_path(base_dir: &Path, rel_path: &str) -> PathBuf {
    let direct = base_dir.join(rel_path);
    if direct.exists() {
        return direct;
    }

    let norm = rel_path.replace('\\', "/");
    // Alternate 1: "source/assets/" <-> "src/assets/"
    if norm.starts_with("source/assets/") {
        let alt = base_dir.join("src/assets").join(&norm["source/assets/".len()..]);
        if alt.exists() {
            return alt;
        }
    } else if norm.starts_with("src/assets/") {
        let alt = base_dir.join("source/assets").join(&norm["src/assets/".len()..]);
        if alt.exists() {
            return alt;
        }
    }

    // Alternate 2: "docs/" <-> "src/assets/docs/"
    if norm.starts_with("docs/") {
        let alt = base_dir.join("src/assets/docs").join(&norm["docs/".len()..]);
        if alt.exists() {
            return alt;
        }
    } else if norm.starts_with("src/assets/docs/") {
        let alt = base_dir.join("docs").join(&norm["src/assets/docs/".len()..]);
        if alt.exists() {
            return alt;
        }
    }

    // Alternate 3: "source/" <-> "src/"
    if norm.starts_with("source/") {
        let alt = base_dir.join("src").join(&norm["source/".len()..]);
        if alt.exists() {
            return alt;
        }
    } else if norm.starts_with("src/") {
        let alt = base_dir.join("source").join(&norm["src/".len()..]);
        if alt.exists() {
            return alt;
        }
    }

    direct
}

pub fn find_release_doc_rel_path(base_dir: &Path) -> String {
    let candidate_dirs = [
        base_dir.join("docs"),
        base_dir.join("src/assets/docs"),
    ];

    for (idx, dir) in candidate_dirs.iter().enumerate() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("release-v") && name.ends_with(".md") {
                    if idx == 0 {
                        return format!("docs/{}", name);
                    } else {
                        return format!("src/assets/docs/{}", name);
                    }
                }
            }
        }
    }

    if base_dir.join("docs").exists() {
        "docs/release-v2026.11.3.0.md".to_string()
    } else {
        "src/assets/docs/release-v2026.11.3.0.md".to_string()
    }
}

pub fn get_all_file_entries(base_dir: &Path) -> Vec<FileEntry> {
    let rel_doc_path = find_release_doc_rel_path(base_dir);
    let is_rust_layout = base_dir.join("src").exists();
    let asset_prefix = if is_rust_layout { "src/assets" } else { "source/assets" };

    let mut entries = vec![
        // PRIVACY_POLICY
        FileEntry { key: "privacy_pt".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_pt_BR.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "privacy_en".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_en_US.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "en".into(), is_updatable: true },
        FileEntry { key: "privacy_es".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_es_ES.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "es".into(), is_updatable: true },
        FileEntry { key: "privacy_fr".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_fr_FR.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "fr".into(), is_updatable: true },
        FileEntry { key: "privacy_it".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_it_IT.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "it".into(), is_updatable: true },
        FileEntry { key: "privacy_de".into(), rel_path: format!("{}/PRIVACY_POLICY/Privacy_Policy_de_DE.txt", asset_prefix), default_group: "PRIVACY_POLICY".into(), lang: "de".into(), is_updatable: true },

        // EULA
        FileEntry { key: "eula_pt".into(), rel_path: format!("{}/EULA/EULA_pt_BR - Huginn.txt", asset_prefix), default_group: "EULA".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "eula_en".into(), rel_path: format!("{}/EULA/EULA_en_US - Huginn.txt", asset_prefix), default_group: "EULA".into(), lang: "en".into(), is_updatable: true },
        FileEntry { key: "eula_es".into(), rel_path: format!("{}/EULA/EULA_es_ES - Huginn.txt", asset_prefix), default_group: "EULA".into(), lang: "es".into(), is_updatable: true },
        FileEntry { key: "eula_fr".into(), rel_path: format!("{}/EULA/EULA_fr_FR - Huginn.txt", asset_prefix), default_group: "EULA".into(), lang: "fr".into(), is_updatable: true },
        FileEntry { key: "eula_it".into(), rel_path: format!("{}/EULA/EULA_it_IT - Huginn.txt", asset_prefix), default_group: "EULA".into(), lang: "it".into(), is_updatable: true },
        FileEntry { key: "eula_de".into(), rel_path: format!("{}/EULA/EULA_de_DE - Huginn.txt", asset_prefix), default_group: "EULA".into(), lang: "de".into(), is_updatable: true },

        // CLC
        FileEntry { key: "clc_pt".into(), rel_path: format!("{}/CLC/CLC_pt_BR - Huginn.txt", asset_prefix), default_group: "CLC".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "clc_en".into(), rel_path: format!("{}/CLC/CLC_en_US - Huginn.txt", asset_prefix), default_group: "CLC".into(), lang: "en".into(), is_updatable: true },
        FileEntry { key: "clc_es".into(), rel_path: format!("{}/CLC/CLC_es_ES - Huginn.txt", asset_prefix), default_group: "CLC".into(), lang: "es".into(), is_updatable: true },
        FileEntry { key: "clc_fr".into(), rel_path: format!("{}/CLC/CLC_fr_FR - Huginn.txt", asset_prefix), default_group: "CLC".into(), lang: "fr".into(), is_updatable: true },
        FileEntry { key: "clc_it".into(), rel_path: format!("{}/CLC/CLC_it_IT - Huginn.txt", asset_prefix), default_group: "CLC".into(), lang: "it".into(), is_updatable: true },
        FileEntry { key: "clc_de".into(), rel_path: format!("{}/CLC/CLC_de_DE - Huginn.txt", asset_prefix), default_group: "CLC".into(), lang: "de".into(), is_updatable: true },

        // ABOUT
        FileEntry { key: "about_pt".into(), rel_path: format!("{}/ABOUT/ABOUT_pt_BR.txt", asset_prefix), default_group: "ABOUT".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "about_en".into(), rel_path: format!("{}/ABOUT/ABOUT_en_US.txt", asset_prefix), default_group: "ABOUT".into(), lang: "en".into(), is_updatable: true },
        FileEntry { key: "about_es".into(), rel_path: format!("{}/ABOUT/ABOUT_es_ES.txt", asset_prefix), default_group: "ABOUT".into(), lang: "es".into(), is_updatable: true },
        FileEntry { key: "about_fr".into(), rel_path: format!("{}/ABOUT/ABOUT_fr_FR.txt", asset_prefix), default_group: "ABOUT".into(), lang: "fr".into(), is_updatable: true },
        FileEntry { key: "about_it".into(), rel_path: format!("{}/ABOUT/ABOUT_it_IT.txt", asset_prefix), default_group: "ABOUT".into(), lang: "it".into(), is_updatable: true },
        FileEntry { key: "about_de".into(), rel_path: format!("{}/ABOUT/ABOUT_de_DE.txt", asset_prefix), default_group: "ABOUT".into(), lang: "de".into(), is_updatable: true },

        // NOTICES
        FileEntry { key: "notice_pt".into(), rel_path: format!("{}/NOTICES/NOTICE_pt_BR.txt", asset_prefix), default_group: "NOTICES".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "notice_en".into(), rel_path: format!("{}/NOTICES/NOTICE_en_US.txt", asset_prefix), default_group: "NOTICES".into(), lang: "en".into(), is_updatable: true },
        FileEntry { key: "notice_es".into(), rel_path: format!("{}/NOTICES/NOTICE_es_ES.txt", asset_prefix), default_group: "NOTICES".into(), lang: "es".into(), is_updatable: true },
        FileEntry { key: "notice_fr".into(), rel_path: format!("{}/NOTICES/NOTICE_fr_FR.txt", asset_prefix), default_group: "NOTICES".into(), lang: "fr".into(), is_updatable: true },
        FileEntry { key: "notice_it".into(), rel_path: format!("{}/NOTICES/NOTICE_it_IT.txt", asset_prefix), default_group: "NOTICES".into(), lang: "it".into(), is_updatable: true },
        FileEntry { key: "notice_de".into(), rel_path: format!("{}/NOTICES/NOTICE_de_DE.txt", asset_prefix), default_group: "NOTICES".into(), lang: "de".into(), is_updatable: true },

        // DOCS
        FileEntry { key: "doc_architecture".into(), rel_path: "docs/architecture.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_cli_reference".into(), rel_path: "docs/cli-reference.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_compliance".into(), rel_path: "docs/compliance.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_licensing".into(), rel_path: "docs/licensing.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_release".into(), rel_path: rel_doc_path, default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_security_model".into(), rel_path: "docs/security-model.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_suggested_issues".into(), rel_path: "docs/suggested-issues.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_user_guide".into(), rel_path: "docs/user-guide.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "doc_migration_report".into(), rel_path: "docs/MIGRATION_REPORT.md".into(), default_group: "DOCS".into(), lang: "all".into(), is_updatable: true },

        // RAIZ
        FileEntry { key: "readme".into(), rel_path: "README.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "security_md".into(), rel_path: "SECURITY.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "code_of_conduct_md".into(), rel_path: "CODE_OF_CONDUCT.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "contributing_md".into(), rel_path: "CONTRIBUTING.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true },
    ];

    if base_dir.join("MANUAL.md").exists() {
        entries.push(FileEntry { key: "manual".into(), rel_path: "MANUAL.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true });
    }
    if base_dir.join("OBSERVACAO.md").exists() {
        entries.push(FileEntry { key: "observacao".into(), rel_path: "OBSERVACAO.md".into(), default_group: "RAIZ".into(), lang: "all".into(), is_updatable: true });
    }

    entries.extend(vec![
        // SRC_RUST (Source code)
        FileEntry { key: "about_dialog_rs".into(), rel_path: "src/ui/dialogs/about.rs".into(), default_group: "SRC_RUST".into(), lang: "pt".into(), is_updatable: true },
        FileEntry { key: "main_cli_rs".into(), rel_path: "src/bin/main_cli.rs".into(), default_group: "SRC_RUST".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "locale_manual_rs".into(), rel_path: "src/locale/manual.rs".into(), default_group: "SRC_RUST".into(), lang: "all".into(), is_updatable: true },

        // MANIFESTS
        FileEntry { key: "cargo_toml".into(), rel_path: "Cargo.toml".into(), default_group: "MANIFESTS".into(), lang: "all".into(), is_updatable: true },
        FileEntry { key: "cargo_lock".into(), rel_path: "Cargo.lock".into(), default_group: "MANIFESTS".into(), lang: "all".into(), is_updatable: true },
    ]);

    // REPO: AppxManifest.xml (MSIX / Windows Store Package Manifest na aba "📦 REPO")
    entries.push(FileEntry {
        key: "appx_manifest".into(),
        rel_path: "repo/AppxManifest.xml".into(),
        default_group: "REPO".into(),
        lang: "all".into(),
        is_updatable: true,
    });

    // REPO (Inno Setup Installers - scan dynamically all .iss files in repo/)
    let repo_dir = base_dir.join("repo");
    let mut iss_entries = Vec::new();

    if repo_dir.is_dir() {
        if let Ok(dir_entries) = fs::read_dir(&repo_dir) {
            let mut paths: Vec<_> = dir_entries.filter_map(|e| e.ok().map(|de| de.path())).collect();
            paths.sort();
            for path in paths {
                if path.is_file() && path.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("iss")).unwrap_or(false) {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                        let key = if stem == "huginn" || stem == "panoptes_patrol" {
                            "installer_default".to_string()
                        } else if let Some(suffix) = stem.strip_prefix("huginn_").or_else(|| stem.strip_prefix("panoptes_patrol_")) {
                            format!("installer_{}", suffix)
                        } else {
                            format!("installer_{}", stem)
                        };

                        iss_entries.push(FileEntry {
                            key,
                            rel_path: format!("repo/{}", file_name),
                            default_group: "REPO".into(),
                            lang: "pt".into(),
                            is_updatable: true,
                        });
                    }
                }
            }
        }
    }

    if iss_entries.is_empty() {
        // Fallback para testes unitários ou ambientes onde repo/ ainda não existe
        iss_entries = vec![
            FileEntry { key: "installer_default".into(), rel_path: "repo/Huginn.iss".into(), default_group: "REPO".into(), lang: "pt".into(), is_updatable: true },
            FileEntry { key: "installer_mingw".into(), rel_path: "repo/Huginn_mingw.iss".into(), default_group: "REPO".into(), lang: "pt".into(), is_updatable: true },
            FileEntry { key: "installer_msvc".into(), rel_path: "repo/Huginn_msvc.iss".into(), default_group: "REPO".into(), lang: "pt".into(), is_updatable: true },
            FileEntry { key: "installer_msvc_mingw".into(), rel_path: "repo/Huginn_msvc_mingw.iss".into(), default_group: "REPO".into(), lang: "pt".into(), is_updatable: true },
        ];
    }

    entries.extend(iss_entries);

    entries
}

pub fn build_patterns() -> HashMap<String, Vec<&'static str>> {
    let mut map = HashMap::new();
    map.insert("privacy_pt".into(), vec![r"(?m)^(Versão:?\s*.*)$", r"(?m)^(Última atualização:?\s*.*)$"]);
    map.insert("privacy_en".into(), vec![r"(?m)^(Version:?\s*.*)$", r"(?m)^(Last updated:?\s*.*)$"]);
    map.insert("privacy_es".into(), vec![r"(?m)^(Versión:?\s*.*)$", r"(?m)^(Última actualización:?\s*.*)$"]);
    map.insert("privacy_fr".into(), vec![r"(?m)^(Version:?\s*.*)$", r"(?m)^(Dernière mise à jour\s*:?\s*.*)$"]);
    map.insert("privacy_it".into(), vec![r"(?m)^(Versione:?\s*.*)$", r"(?m)^(Ultimo aggiornamento:?\s*.*)$"]);
    map.insert("privacy_de".into(), vec![r"(?m)^(Version:?\s*.*)$", r"(?m)^(Letzte Aktualisierung:?\s*.*)$"]);

    map.insert("eula_pt".into(), vec![r"(?m)^(Versão.*)$"]);
    map.insert("eula_en".into(), vec![r"(?m)^(Version.*)$"]);
    map.insert("eula_es".into(), vec![r"(?m)^(Versión.*)$"]);
    map.insert("eula_fr".into(), vec![r"(?m)^(Version.*)$"]);
    map.insert("eula_it".into(), vec![r"(?m)^(Versione.*)$"]);
    map.insert("eula_de".into(), vec![r"(?m)^(Version.*)$"]);

    map.insert("clc_pt".into(), vec![r"(?m)^(Versão.*)$"]);
    map.insert("clc_en".into(), vec![r"(?m)^(Version.*)$"]);
    map.insert("clc_es".into(), vec![r"(?m)^(Versión.*)$"]);
    map.insert("clc_fr".into(), vec![r"(?m)^(Version.*)$"]);
    map.insert("clc_it".into(), vec![r"(?m)^(Versione.*)$"]);
    map.insert("clc_de".into(), vec![r"(?m)^(Version.*)$"]);

    map.insert("about_pt".into(), vec![r"(?m)^(Versão:?\s*.*)$"]);
    map.insert("about_en".into(), vec![r"(?m)^(Version:?\s*.*)$"]);
    map.insert("about_es".into(), vec![r"(?m)^(Versión:?\s*.*)$"]);
    map.insert("about_fr".into(), vec![r"(?m)^(Version:?\s*.*)$"]);
    map.insert("about_it".into(), vec![r"(?m)^(Versione:?\s*.*)$"]);
    map.insert("about_de".into(), vec![r"(?m)^(Version:?\s*.*)$"]);

    map.insert("notice_pt".into(), vec![r"(?i)\(versão\s+[0-9]+(?:\.[0-9]+){1,3}\)"]);
    map.insert("notice_en".into(), vec![r"(?i)\(version\s+[0-9]+(?:\.[0-9]+){1,3}\)"]);
    map.insert("notice_es".into(), vec![r"(?i)\(versión\s+[0-9]+(?:\.[0-9]+){1,3}\)"]);
    map.insert("notice_fr".into(), vec![r"(?i)\(version\s+[0-9]+(?:\.[0-9]+){1,3}\)"]);
    map.insert("notice_it".into(), vec![r"(?i)\(versione\s+[0-9]+(?:\.[0-9]+){1,3}\)"]);
    map.insert("notice_de".into(), vec![r"(?i)\(Version\s+[0-9]+(?:\.[0-9]+){1,3}\)"]);

    let doc_patterns = vec![
        r"(?m)^(\*\*Vers(?:ã|a)o:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`.*)$",
        r"(?m)^(\*\*Data:\*\*\s*`[^`\r\n]+`.*)$",
        r"(?m)^(\*\*Version:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`.*)$",
        r"(?m)^(\*\*Date:\*\*\s*`[^`\r\n]+`.*)$",
    ];
    map.insert("doc_architecture".into(), doc_patterns.clone());
    map.insert("doc_cli_reference".into(), doc_patterns.clone());
    map.insert("doc_compliance".into(), doc_patterns.clone());
    map.insert("doc_licensing".into(), doc_patterns.clone());
    map.insert("doc_security_model".into(), doc_patterns.clone());
    map.insert("doc_user_guide".into(), doc_patterns);
    map.insert("doc_migration_report".into(), vec![
        r"(?i)\(versão\s+`?v?[0-9]+(?:\.[0-9]+){1,3}`?\)",
    ]);
    map.insert("doc_suggested_issues".into(), vec![
        r"(?i)\((?:ex:|e\.g\.,)\s*`?v?[0-9]+(?:\.[0-9]+){1,3}`?\)",
    ]);

    map.insert("doc_release".into(), vec![
        r"(?m)^(# Notas de Lançamento / Release Notes — v[0-9]+(?:\.[0-9]+){1,3}.*)$",
        r"(?m)^(\*\*Tag de Vers(?:ã|a)o:\*\*\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
        r"(?m)^(\*\*Data de Publica(?:çã|ca)o:\*\*\s*.*)$",
        r"(?m)^(\*\*Version Tag:\*\*\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
        r"(?m)^(\*\*Publication Date:\*\*\s*.*)$",
    ]);

    map.insert("readme".into(), vec![
        r"(Release-(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}-blue\.svg)",
        r#"(?im)^\s*-\s*\*\*Vers[aã]o:\*\*\s*[`"]?(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}[`"]?\s*\(\s*Data\s*:\s*[^)]+\)"#,
        r#"(?im)^\s*-\s*\*\*Version:\*\*\s*[`"]?(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}[`"]?\s*\(\s*Date\s*:\s*[^)]+\)"#,
        r#"(?im)^\s*-\s*\*\*Versi[oó]n:\*\*\s*[`"]?(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}[`"]?\s*\(\s*Fecha\s*:\s*[^)]+\)"#,
        r#"(?im)^\s*-\s*\*\*Version\s*:\*\*\s*[`"]?(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}[`"]?\s*\(\s*Date\s*:\s*[^)]+\)"#,
        r#"(?im)^\s*-\s*\*\*Versione:\*\*\s*[`"]?(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}[`"]?\s*\(\s*Data\s*:\s*[^)]+\)"#,
        r#"(?im)^\s*-\s*\*\*Version:\*\*\s*[`"]?(?:v|V)?[0-9]+(?:\.[0-9]+){1,3}[`"]?\s*\(\s*Datum\s*:\s*[^)]+\)"#,
        r#"(?im)^\s*>\s*\*\*(?:Observa[cç][aã]o|Note|Nota|Remarque|Hinweis)\s*:\*\*.*?\*\*(?:`v?|v?`?)[0-9]+(?:\.[0-9]+){1,3}(?:`|\*\*)?"#,
    ]);

    map.insert("security_md".into(), vec![
        r"(\|\s*`v?[0-9]+\.[0-9]+\.x`\s*\|\s*:white_check_mark:\s*\|)",
        r"(\|\s*`< v?[0-9]+\.[0-9]+`\s*\|\s*:x:\s*\|)",
        r"(?m)^(\*\*Vers(?:ã|a)o:\*\*\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
    ]);

    map.insert("code_of_conduct_md".into(), vec![
        r"(?m)^(\*\*Vers(?:ã|a)o:\*\*\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
    ]);

    map.insert("contributing_md".into(), vec![
        r"(?m)^(\*\*Vers(?:ã|a)o:\*\*\s*`[0-9]+(?:\.[0-9]+){1,3}`.*)$",
    ]);

    map.insert("manual".into(), vec![
        r"(?m)^(\*\*Vers(?:ã|a)o:\*\*\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
        r"(?m)^(Versão documentada\s*:\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
    ]);

    map.insert("observacao".into(), vec![
        r"(?m)^(Versão documentada\s*:\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
        r"(?m)^(Documented version\s*:\s*`v?[0-9]+(?:\.[0-9]+){1,3}`.*)$",
    ]);

    map.insert("about_dialog_rs".into(), vec![
        r#"ui\.label\("[0-9]+(?:\.[0-9]+){1,3}"\);"#,
        r#"AboutTab::Sobre => "Huginn Sentinel [0-9]+(?:\.[0-9]+){1,3}"#,
    ]);

    map.insert("main_cli_rs".into(), vec![
        r#"Version [0-9]+(?:\.[0-9]+){1,3} \| Rust Security"#,
        r#"Huginn Sentinel CLI v[0-9]+(?:\.[0-9]+){1,3}"#,
    ]);

    map.insert("locale_manual_rs".into(), vec![
        r#"(?m)^\s*//\s*Vers(?:ã|a)o:\s*[0-9]+(?:\.[0-9]+){1,3}\s*\|\s*Data:\s*.*$"#,
        r#"A versão [0-9]+(?:\.[0-9]+){1,3}"#,
        r#"Version [0-9]+(?:\.[0-9]+){1,3}"#,
    ]);

    map.insert("cargo_toml".into(), vec![
        r#"(?m)^version\s*=\s*"[0-9]+(?:\.[0-9]+){1,2}""#,
    ]);

    map.insert("cargo_lock".into(), vec![
        r#"name = "huginn_rs""#,
    ]);

    map.insert("installer_default".into(), vec![
        r#"(?m)^(#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
    ]);
    map.insert("installer_mingw".into(), vec![
        r#"(?m)^(#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
        r#"(?m)^(OutputBaseFilename=.*_v(?:\{#MyAppVersion\}|[0-9]+(?:\.[0-9]+){1,3}).*)$"#,
    ]);
    map.insert("installer_msvc".into(), vec![
        r#"(?m)^(#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
        r#"(?m)^(OutputBaseFilename=.*_v(?:\{#MyAppVersion\}|[0-9]+(?:\.[0-9]+){1,3}).*)$"#,
    ]);
    map.insert("installer_msvc_mingw".into(), vec![
        r#"(?m)^(#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
        r#"(?m)^(OutputBaseFilename=.*_v(?:\{#MyAppVersion\}|[0-9]+(?:\.[0-9]+){1,3}).*)$"#,
    ]);

    map.insert("appx_manifest".into(), vec![
        r#"(?m)<Identity\b[^>]*?\bVersion\s*=\s*"[0-9]+(?:\.[0-9]+){1,3}""#,
    ]);

    map
}

pub fn replace_line_by_regex(text: &str, pattern: &str, replacement_full_line: &str) -> String {
    if let Ok(re) = Regex::new(pattern) {
        re.replace_all(text, replacement_full_line).into_owned()
    } else {
        text.to_string()
    }
}

pub fn replace_regex_capture(text: &str, pattern: &str, capture_index: usize, replacement: &str) -> String {
    if let Ok(re) = Regex::new(pattern) {
        let mut result = String::new();
        let mut last_end = 0;
        for caps in re.captures_iter(text) {
            if let Some(m) = caps.get(capture_index) {
                result.push_str(&text[last_end..m.start()]);
                result.push_str(replacement);
                last_end = m.end();
            }
        }
        result.push_str(&text[last_end..]);
        result
    } else {
        text.to_string()
    }
}

pub fn read_file_robust(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    // Try UTF-8
    if let Ok(mut s) = String::from_utf8(bytes.clone()) {
        if s.starts_with('\u{feff}') {
            s.remove(0);
        }
        return Ok(s);
    }
    // Fallback: Latin-1 lossy conversion
    let s: String = bytes.into_iter().map(|b| b as char).collect();
    Ok(s)
}

pub fn write_file_utf8(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content.as_bytes())
}

pub fn check_all_files(base_dir: &Path) -> Vec<FileCheckStatus> {
    let mut statuses = Vec::new();
    let entries = get_all_file_entries(base_dir);
    let patterns_map = build_patterns();

    for entry in &entries {
        if is_release_notes_file(&entry.rel_path) {
            continue;
        }

        let abs_path = resolve_file_path(base_dir, &entry.rel_path);
        let exists = abs_path.exists();
        let mut detected_version = None;
        let mut detected_date = None;
        let mut found_patterns = Vec::new();
        let mut missing_patterns = Vec::new();

        if exists {
            if let Ok(raw_txt) = read_file_robust(&abs_path) {
                let clean_txt = raw_txt.replace('\r', "");

                // 1. Verify configured patterns
                let default_iss_patterns = [
                    r#"(?m)^(\s*#define\s+MyAppVersion\s+"[0-9]+(?:\.[0-9]+){1,3}".*)$"#,
                ];
                let configured_pats = patterns_map.get(&entry.key);
                let pats: &[&str] = if let Some(p) = configured_pats {
                    p.as_slice()
                } else if entry.rel_path.to_lowercase().ends_with(".iss") || entry.key.starts_with("installer_") {
                    &default_iss_patterns
                } else {
                    &[]
                };

                for &pat in pats {
                    if let Ok(re) = Regex::new(pat) {
                        if let Some(caps) = re.captures(&clean_txt) {
                            let captured = caps.get(0).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                            found_patterns.push(captured);
                        } else {
                            missing_patterns.push(pat.to_string());
                        }
                    }
                }

                // 2. Detect version
                if entry.key == "readme" {
                    let (ver_opt, date_opt) = extract_readme_section_version_and_date(&clean_txt, "ptbr");
                    if ver_opt.is_some() {
                        detected_version = ver_opt;
                    }
                    if let Some((d, mo, y)) = date_opt {
                        detected_date = Some(format!("{}/{}/{}", d, mo + 1, y));
                    }
                } else if entry.key == "cargo_lock" {
                    if let Ok(re_lock) = Regex::new(r#"(?m)\[\[package\]\]\s*\r?\nname\s*=\s*"huginn_rs"\s*\r?\nversion\s*=\s*"([^"]+)""#) {
                        if let Some(caps) = re_lock.captures(&clean_txt) {
                            detected_version = Some(caps[1].to_string());
                        }
                    }
                } else if entry.key == "cargo_toml" {
                    if let Ok(re_pkg) = Regex::new(r#"(?m)\[package\][\s\S]*?^version\s*=\s*"([^"]+)""#) {
                        if let Some(caps) = re_pkg.captures(&clean_txt) {
                            detected_version = Some(caps[1].to_string());
                        }
                    }
                } else if entry.key == "appx_manifest" {
                    if let Ok(re_appx) = Regex::new(r#"(?m)<Identity\b[^>]*?\bVersion\s*=\s*"([^"]+)""#) {
                        if let Some(caps) = re_appx.captures(&clean_txt) {
                            detected_version = Some(caps[1].to_string());
                        }
                    }
                }

                if detected_version.is_none() {
                    for f in &found_patterns {
                        if let Some(v) = parse_version_string(f) {
                            detected_version = Some(v);
                            break;
                        }
                    }
                }

                if detected_version.is_none() {
                    for line in clean_txt.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with('#') && !trimmed.to_lowercase().starts_with("#define") {
                            continue;
                        }
                        if line.contains("MyAppVersion")
                            || line.contains("OutputBaseFilename")
                            || line.contains("Versão")
                            || line.contains("Version")
                            || line.contains("Versión")
                            || line.contains("Versione")
                            || line.contains("version = ")
                        {
                            if let Some(v) = parse_version_string(line) {
                                detected_version = Some(v);
                                break;
                            }
                        }
                    }
                }

                // 3. Detect date
                for f in &found_patterns {
                    if let Some((d, m, y)) = parse_date_string(f) {
                        detected_date = Some(format!("{}/{}/{}", d, m + 1, y));
                        break;
                    }
                }

                if detected_date.is_none() {
                    for line in clean_txt.lines() {
                        if line.trim().starts_with('#') { continue; }
                        let lower = line.to_lowercase();
                        if lower.contains("atualiza")
                            || lower.contains("actualiza")
                            || lower.contains("updated")
                            || lower.contains("mise à jour")
                            || lower.contains("aggiorna")
                            || lower.contains("aktualisier")
                            || lower.contains("data")
                            || lower.contains("date")
                            || lower.contains("fecha")
                            || lower.contains("datum")
                        {
                            if let Some((d, m, y)) = parse_date_string(line) {
                                detected_date = Some(format!("{}/{}/{}", d, m + 1, y));
                                break;
                            }
                        }
                    }
                }

                // Fallback date from version if version is YYYY.M.D.rev
                if detected_date.is_none() {
                    if let Some(ref ver) = detected_version {
                        if let Some((vd, vm, vy, _)) = parse_version_to_date(ver) {
                            detected_date = Some(format!("{}/{}/{}", vd, vm + 1, vy));
                        }
                    }
                }
            }
        }

        statuses.push(FileCheckStatus {
            key: entry.key.clone(),
            rel_path: entry.rel_path.clone(),
            group: entry.default_group.clone(),
            lang: entry.lang.clone(),
            absolute_path: abs_path,
            exists,
            detected_version,
            detected_date,
            found_patterns,
            missing_patterns,
            is_updatable: entry.is_updatable,
        });
    }

    statuses
}

pub fn update_standard_doc_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let normalized_pt_ver = plain_version(&pt.version);
    let normalized_en_ver = plain_version(&en.version);
    let pt_date = format!("{} de {} de {}", pt.day, PT_MONTHS[pt.month_index as usize].to_lowercase(), pt.year);
    let en_date = format!("{} {}, {}", EN_MONTHS[en.month_index as usize], en.day, en.year);

    let mut updated = text.to_string();
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Vers(?:ã|a)o:\*\*\s*`)([^`]+)(`)", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Data:\*\*\s*`)([^`]+)(`)", 2, &pt_date);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Version:\*\*\s*`)([^`]+)(`)", 2, &normalized_en_ver);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Date:\*\*\s*`)([^`]+)(`)", 2, &en_date);
    updated
}

pub fn update_release_doc_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let normalized_pt_ver = plain_version(&pt.version);
    let normalized_en_ver = plain_version(&en.version);
    let pt_date = format!("{} de {} de {}", pt.day, PT_MONTHS[pt.month_index as usize].to_lowercase(), pt.year);
    let en_date = format!("{} {}, {}", EN_MONTHS[en.month_index as usize], en.day, en.year);

    let mut updated = text.to_string();
    updated = replace_regex_capture(&updated, r"(# Notas de Lançamento / Release Notes — v)([\d\.]+)", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(# Huginn — Release v)([\d\.]+)", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(\*\*Huginn v)([\d\.]+)(\*\*)", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Tag de Vers(?:ã|a)o:\*\*\s*`v?)([^`]+)(`)", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Data de Publica(?:çã|ca)o:\*\*\s*)([^\r\n]+)", 2, &pt_date);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Version Tag:\*\*\s*`v?)([^`]+)(`)", 2, &normalized_en_ver);
    updated = replace_regex_capture(&updated, r"(?i)(\*\*Publication Date:\*\*\s*)([^\r\n]+)", 2, &en_date);
    updated = replace_regex_capture(&updated, r"(Huginn-v)([\d\.]+)(-x64-(?:Setup\.exe|Portable\.zip))", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(?i)(## 2\. Destaques da Vers(?:ã|a)o v)([\d\.]+)", 2, &normalized_pt_ver);
    updated = replace_regex_capture(&updated, r"(?i)(## 2\. Highlights in Release v)([\d\.]+)", 2, &normalized_en_ver);
    updated
}

pub fn update_suggested_issues_versions(text: &str, pt: &LangInput) -> String {
    let normalized_ver = plain_version(&pt.version);
    replace_regex_capture(text, r"((?:\(ex:|\(e\.g\.,)\s*`v?)([\d\.]+)(`\))", 2, &normalized_ver)
}

pub fn update_migration_report_version(text: &str, pt: &LangInput) -> String {
    let normalized_pt_ver = plain_version(&pt.version);
    replace_regex_capture(
        text,
        r"(?i)(\(vers[ãa]o\s+`?v?)([0-9]+(?:\.[0-9]+){1,3})(`?\))",
        2,
        &normalized_pt_ver,
    )
}

pub fn format_updated_version(existing_str: &str, target_version: &str) -> String {
    let clean = existing_str.trim().trim_matches('`').trim();
    let has_v = clean.starts_with('v') || clean.starts_with('V');
    let is_short = !clean.is_empty() && {
        let plain_clean = plain_version(clean);
        plain_clean.split('.').count() == 2
    };

    let plain_target = plain_version(target_version);
    let final_plain = if is_short {
        let parts: Vec<&str> = plain_target.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            plain_target
        }
    } else {
        plain_target
    };

    if has_v {
        format!("v{}", final_plain)
    } else {
        final_plain
    }
}

pub fn format_updated_date(existing_str: &str, day: u32, month_idx: u32, year: i32, lang: &str) -> String {
    let clean = existing_str.trim().trim_matches('`').trim();
    if clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() == 3 {
            let p0 = parts[0].parse::<u32>().unwrap_or(0);
            let p1 = parts[1].parse::<u32>().unwrap_or(0);
            let pad0 = parts[0].len() == 2;
            let pad1 = parts[1].len() == 2;
            if (p0 <= 12 && p1 > 12) || (lang == "en" && p0 <= 12) {
                if pad0 || pad1 {
                    return format!("{:02}/{:02}/{}", month_idx + 1, day, year);
                } else {
                    return format!("{}/{}/{}", month_idx + 1, day, year);
                }
            } else {
                if pad0 || pad1 {
                    return format!("{:02}/{:02}/{}", day, month_idx + 1, year);
                } else {
                    return format!("{}/{}/{}", day, month_idx + 1, year);
                }
            }
        }
    }
    if clean.contains('-') && clean.len() == 10 {
        return format!("{:04}-{:02}-{:02}", year, month_idx + 1, day);
    }
    match lang {
        "en" => format!("{} {}, {}", EN_MONTHS[month_idx as usize], day, year),
        "es" => format!("{} de {} de {}", day, ES_MONTHS[month_idx as usize].to_lowercase(), year),
        "fr" => format!("{} {} {}", day, FR_MONTHS[month_idx as usize].to_lowercase(), year),
        "it" => format!("{} {} {}", day, IT_MONTHS[month_idx as usize].to_lowercase(), year),
        "de" => format!("{}. {} {}", day, DE_MONTHS[month_idx as usize], year),
        _ => format!("{} de {} de {}", day, PT_MONTHS[month_idx as usize].to_lowercase(), year),
    }
}

pub fn update_markdown_version_and_date(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let mut updated = text.to_string();

    // 1. PT Version: **Versão:** `...` or Versão: `...`
    if let Ok(re_pt_ver) = Regex::new(r#"(?im)(^\s*(?:>\s*)?\*?\*?Vers(?:ã|a)o\s*:\*?\*?\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_pt_ver.captures_iter(&updated) {
            if let Some(matched_v) = caps.get(2) {
                res.push_str(&updated[last..matched_v.start()]);
                let rep = format_updated_version(matched_v.as_str(), &pt.version);
                res.push_str(&rep);
                last = matched_v.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    // 2. EN Version: **Version:** `...` or Version: `...`
    if let Ok(re_en_ver) = Regex::new(r#"(?im)(^\s*(?:>\s*)?\*?\*?Version\s*:\*?\*?\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_en_ver.captures_iter(&updated) {
            if let Some(matched_v) = caps.get(2) {
                res.push_str(&updated[last..matched_v.start()]);
                let rep = format_updated_version(matched_v.as_str(), &en.version);
                res.push_str(&rep);
                last = matched_v.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    // 3. PT Date: **Data:** `...` or Data: `...`
    if let Ok(re_pt_date) = Regex::new(r#"(?im)(^\s*(?:>\s*)?\*?\*?Data\s*:\*?\*?\s*[`"]?)([^`"\r\n]+)([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_pt_date.captures_iter(&updated) {
            if let Some(matched_d) = caps.get(2) {
                res.push_str(&updated[last..matched_d.start()]);
                let rep = format_updated_date(matched_d.as_str(), pt.day, pt.month_index, pt.year, "pt");
                res.push_str(&rep);
                last = matched_d.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    // 4. EN Date: **Date:** `...` or Date: `...`
    if let Ok(re_en_date) = Regex::new(r#"(?im)(^\s*(?:>\s*)?\*?\*?Date\s*:\*?\*?\s*[`"]?)([^`"\r\n]+)([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_en_date.captures_iter(&updated) {
            if let Some(matched_d) = caps.get(2) {
                res.push_str(&updated[last..matched_d.start()]);
                let rep = format_updated_date(matched_d.as_str(), en.day, en.month_index, en.year, "en");
                res.push_str(&rep);
                last = matched_d.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    updated
}

pub fn update_security_md_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    let (year, month_idx) = if let Some((_, m, y, _)) = parse_version_to_date(&pt.version) {
        (y, m)
    } else {
        (pt.year, pt.month_index)
    };
    let ym = format!("{}.{}", year, month_idx + 1);

    let mut updated = replace_regex_capture(text, r"(\|\s*`v?)(\d+\.\d+)(\.x`\s*\|\s*:white_check_mark:\s*\|)", 2, &ym);
    updated = replace_regex_capture(&updated, r"(\|\s*`< v?)(\d+\.\d+)(`\s*\|\s*:x:\s*\|)", 2, &ym);
    updated = update_markdown_version_and_date(&updated, pt, en);
    updated
}

pub fn update_code_of_conduct_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    update_markdown_version_and_date(text, pt, en)
}

pub fn update_contributing_versions(text: &str, pt: &LangInput, en: &LangInput) -> String {
    update_markdown_version_and_date(text, pt, en)
}

pub fn update_readme_section(text: &str, section_id: &str, version: &str, date_str: &str) -> String {
    let start_marker = format!("<h2 id=\"{}\">", section_id);
    let start_index = match text.find(&start_marker) {
        Some(idx) => idx,
        None => return text.to_string(),
    };

    let search_rest = &text[start_index + start_marker.len()..];
    let end_index = match search_rest.find("<h2 id=\"") {
        Some(idx) => start_index + start_marker.len() + idx,
        None => text.len(),
    };

    let mut section = text[start_index..end_index].to_string();
    let target_plain_ver = plain_version(version);
    let target_v_ver = readme_version(version);

    // 1. Callout note:
    // With backticks: **`...`**
    section = replace_regex_capture(
        &section,
        r"(?m)(^\s*(?:>\s*)?\*\*(?:Observação|Observacao|Note|Nota|Remarque|Hinweis)\s*:\*\*.*?\*\*`)(v?[0-9]+(?:\.[0-9]+){1,3})(`\*\*.*$)",
        2,
        &target_plain_ver,
    );
    // Without backticks: **v...**
    section = replace_regex_capture(
        &section,
        r"(?m)(^\s*(?:>\s*)?\*\*(?:Observação|Observacao|Note|Nota|Remarque|Hinweis)\s*:\*\*.*?\*\*)(v?[0-9]+(?:\.[0-9]+){1,3})(\*\*.*$)",
        2,
        &target_v_ver,
    );

    // 2. Compound Version + Date line:
    // e.g. - **Versão:** `2026.10.30.0` (Data: 30 de outubro de 2026)<br>
    // or   - **Version :** `2026.10.30.0` (Date : 30 octobre 2026)<br>
    if let Ok(re_compound) = Regex::new(r#"(?im)(^\s*-\s*\*\*(?:Versão|Versao|Version|Versión|Version|Versione)\s*:\s*\*\*\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?\s*\((?:Data|Date|Fecha|Datum)\s*:\s*)([^)]+)(\)\s*(?:<br>)?\s*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_compound.captures_iter(&section) {
            let full_match = caps.get(0).unwrap();
            res.push_str(&section[last..full_match.start()]);
            let p1 = &caps[1];
            let p3 = &caps[3];
            let p5 = &caps[5];
            res.push_str(p1);
            res.push_str(&target_plain_ver);
            res.push_str(p3);
            res.push_str(date_str);
            res.push_str(p5);
            last = full_match.end();
        }
        res.push_str(&section[last..]);
        section = res;
    }

    // 3. Standalone Version line with bold label:
    section = replace_regex_capture(
        &section,
        r#"(?m)(^\s*(?:-\s*)?\*\*(?:Versão|Versao|Version|Versión|Version|Versione)\s*:\s*\*\*\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?(?:<br>)?\r?$)"#,
        2,
        &target_plain_ver,
    );

    // 4. Standalone Version line with simple label:
    section = replace_regex_capture(
        &section,
        r#"(?m)(^\s*(?:Versão|Versao|Version|Versión|Version|Versione)\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?(?:<br>)?\r?$)"#,
        2,
        &target_v_ver,
    );

    // 5. Standalone Date lines:
    section = replace_regex_capture(
        &section,
        r#"(?m)(^\s*(?:-\s*)?\*?\*?(?:Data(?:\s+técnica(?:\s+desta\s+revisão)?)?|Data(?:\s+tecnica(?:\s+desta\s+revisao)?)?|Technical\s+revision\s+date|Date(?:\s+de\s+révision\s+technique)?|Date(?:\s+de\s+revision\s+technique)?|Fecha(?:\s+de\s+revisión\s+técnica)?|Fecha(?:\s+de\s+revision\s+tecnica)?|Data\s+di\s+revisione\s+tecnica|Technisches\s+Revisionsdatum|Datum)\s*:\s*\*?\*?\s*[`"]?)([^`"\r\n<]+)([`"]?(?:<br>)?\r?$)"#,
        2,
        date_str,
    );

    let mut updated = text[..start_index].to_string();
    updated.push_str(&section);
    updated.push_str(&text[end_index..]);
    updated
}

pub fn update_readme_versions(text: &str, pt: &LangInput, en: &LangInput, es: &LangInput, fr: &LangInput, it: &LangInput, de: &LangInput) -> String {
    let mut updated = text.to_string();

    let target_plain_ver = plain_version(&pt.version);
    // 1. Badge: Release-v2026.10.21.0-blue.svg
    updated = replace_regex_capture(&updated, r"(Release-v?)([\d\.]+)(-blue\.svg)", 2, &target_plain_ver);

    // 2. Release doc links: release-v2026.10.21.0.md
    updated = replace_regex_capture(&updated, r"(((?:docs/|src/assets/docs/)?release-v)([\d\.]+)(\.md))", 2, &target_plain_ver);

    // 3. Release Notes text links: [Release Notes v...] and [Notas de Lançamento v...]
    updated = replace_regex_capture(&updated, r"(\[\s*(?:Release Notes|Notas de Lançamento)\s+v?)([\d\.]+)(\])", 2, &target_plain_ver);

    let pt_date = format!("{} de {} de {}", pt.day, PT_MONTHS[pt.month_index as usize].to_lowercase(), pt.year);
    let en_date = format!("{} {}, {}", EN_MONTHS[en.month_index as usize], en.day, en.year);
    let es_date = format!("{} de {} de {}", es.day, ES_MONTHS[es.month_index as usize].to_lowercase(), es.year);
    let fr_date = format!("{} {} {}", fr.day, FR_MONTHS[fr.month_index as usize].to_lowercase(), fr.year);
    let it_date = format!("{} {} {}", it.day, IT_MONTHS[it.month_index as usize].to_lowercase(), it.year);
    let de_date = format!("{}. {} {}", de.day, DE_MONTHS[de.month_index as usize], de.year);

    // 4. Localized sections (Atualiza versão e data nos seis idiomas suportados)
    updated = update_readme_section(&updated, "ptbr", &pt.version, &pt_date);
    updated = update_readme_section(&updated, "enus", &en.version, &en_date);
    updated = update_readme_section(&updated, "eses", &es.version, &es_date);
    updated = update_readme_section(&updated, "frfr", &fr.version, &fr_date);
    updated = update_readme_section(&updated, "itit", &it.version, &it_date);
    updated = update_readme_section(&updated, "dede", &de.version, &de_date);

    updated
}

pub fn extract_readme_section_version_and_date(text: &str, section_id: &str) -> (Option<String>, Option<(u32, u32, i32)>) {
    let start_marker = format!("<h2 id=\"{}\">", section_id);
    let start_index = match text.find(&start_marker) {
        Some(idx) => idx,
        None => return (None, None),
    };

    let search_rest = &text[start_index + start_marker.len()..];
    let end_index = match search_rest.find("<h2 id=\"") {
        Some(idx) => start_index + start_marker.len() + idx,
        None => text.len(),
    };

    let section = &text[start_index..end_index];
    let mut detected_version = None;
    let mut detected_date = None;

    // 1. Compound Version + Date line:
    // e.g. - **Versão:** `2026.10.30.0` (Data: 30 de outubro de 2026)<br>
    // or   - **Version :** `2026.10.30.0` (Date : 30 octobre 2026)<br>
    // or   - **Version:** `2026.10.30.0` (Datum: 30. Oktober 2026)<br>
    if let Ok(re_compound) = Regex::new(r#"(?im)^\s*-\s*\*\*(?:Versão|Versao|Version|Versión|Versione)\s*:\s*\*\*\s*[`"]?((?:v|V)?[0-9]+(?:\.[0-9]+){1,3})[`"]?\s*\(\s*(?:Data|Date|Fecha|Datum)\s*:\s*([^)]+)\)"#) {
        if let Some(caps) = re_compound.captures(section) {
            detected_version = caps.get(1).map(|m| m.as_str().to_string());
            if let Some(d_str) = caps.get(2) {
                detected_date = parse_date_string(d_str.as_str());
            }
        }
    }

    // 2. Callout note:
    // > **Observação:** ... **`2026.10.30.0`**
    if detected_version.is_none() {
        if let Ok(re_callout) = Regex::new(r#"(?im)^\s*>\s*\*\*(?:Observação|Observacao|Note|Nota|Remarque|Hinweis)\s*:\*\*.*?\*\*(?:`v?|v?`?)([0-9]+(?:\.[0-9]+){1,3})(?:`|\*\*)?"#) {
            if let Some(caps) = re_callout.captures(section) {
                detected_version = caps.get(1).map(|m| m.as_str().to_string());
            }
        }
    }

    // 3. Fallbacks within section
    if detected_version.is_none() {
        detected_version = parse_version_string(section);
    }
    if detected_date.is_none() {
        detected_date = parse_date_string(section);
    }

    (detected_version, detected_date)
}

pub fn update_manual_section(text: &str, section_id: &str, version: &str, date_str: &str) -> String {
    let start_marker = format!("<h2 id=\"{}\">", section_id);
    let start_index = match text.find(&start_marker) {
        Some(idx) => idx,
        None => return text.to_string(),
    };

    let search_rest = &text[start_index + start_marker.len()..];
    let end_index = match search_rest.find("<h2 id=\"") {
        Some(idx) => start_index + start_marker.len() + idx,
        None => text.len(),
    };

    let mut section = text[start_index..end_index].to_string();
    let normalized_ver = plain_version(version);

    section = replace_regex_capture(
        &section,
        r"(?m)(^(?:Versão documentada|Documented version|Versión documentada|Version documentée|Versione documentata|Dokumentierte Version)\s*:\s*`?)(v?[0-9]+(?:\.[0-9]+){1,3})(`?.*$)",
        2,
        &normalized_ver,
    );

    section = replace_regex_capture(
        &section,
        r"(?m)(^(?:Data|Date|Fecha|Datum)\s*:\s*`?)([^`\r\n]+)(`?.*$)",
        2,
        date_str,
    );

    let mut updated = text[..start_index].to_string();
    updated.push_str(&section);
    updated.push_str(&text[end_index..]);
    updated
}

pub fn update_manual_versions(text: &str, pt: &LangInput, en: &LangInput, es: &LangInput, fr: &LangInput, it: &LangInput, de: &LangInput) -> String {
    let mut updated = update_markdown_version_and_date(text, pt, en);

    // Update "Versão documentada:" and "Documented version:"
    if let Ok(re_pt_doc) = Regex::new(r#"(?im)(^\s*Vers(?:ã|a)o documentada\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_pt_doc.captures_iter(&updated) {
            if let Some(matched_v) = caps.get(2) {
                res.push_str(&updated[last..matched_v.start()]);
                let rep = format_updated_version(matched_v.as_str(), &pt.version);
                res.push_str(&rep);
                last = matched_v.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    if let Ok(re_en_doc) = Regex::new(r#"(?im)(^\s*Documented version\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_en_doc.captures_iter(&updated) {
            if let Some(matched_v) = caps.get(2) {
                res.push_str(&updated[last..matched_v.start()]);
                let rep = format_updated_version(matched_v.as_str(), &en.version);
                res.push_str(&rep);
                last = matched_v.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    let pt_date = format!("{} de {} de {}", pt.day, PT_MONTHS[pt.month_index as usize].to_lowercase(), pt.year);
    let en_date = format!("{} {}, {}", EN_MONTHS[en.month_index as usize], en.day, en.year);
    let es_date = format!("{} de {} de {}", es.day, ES_MONTHS[es.month_index as usize].to_lowercase(), es.year);
    let fr_date = format!("{} {} {}", fr.day, FR_MONTHS[fr.month_index as usize].to_lowercase(), fr.year);
    let it_date = format!("{} {} {}", it.day, IT_MONTHS[it.month_index as usize].to_lowercase(), it.year);
    let de_date = format!("{}. {} {}", de.day, DE_MONTHS[de.month_index as usize], de.year);

    updated = update_manual_section(&updated, "ptbr", &pt.version, &pt_date);
    updated = update_manual_section(&updated, "enus", &en.version, &en_date);
    updated = update_manual_section(&updated, "eses", &es.version, &es_date);
    updated = update_manual_section(&updated, "frfr", &fr.version, &fr_date);
    updated = update_manual_section(&updated, "itit", &it.version, &it_date);
    updated = update_manual_section(&updated, "dede", &de.version, &de_date);
    updated
}

pub fn update_observacao_versions(text: &str, pt: &LangInput, en: &LangInput, es: &LangInput, fr: &LangInput, it: &LangInput, de: &LangInput) -> String {
    let mut updated = text.to_string();

    // 1. "Versão documentada:" and "Documented version:"
    if let Ok(re_pt) = Regex::new(r#"(?im)(^\s*Vers(?:ã|a)o documentada\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_pt.captures_iter(&updated) {
            if let Some(matched_v) = caps.get(2) {
                res.push_str(&updated[last..matched_v.start()]);
                let rep = format_updated_version(matched_v.as_str(), &pt.version);
                res.push_str(&rep);
                last = matched_v.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    if let Ok(re_en) = Regex::new(r#"(?im)(^\s*Documented version\s*:\s*[`"]?)(v?[0-9]+(?:\.[0-9]+){1,3})([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_en.captures_iter(&updated) {
            if let Some(matched_v) = caps.get(2) {
                res.push_str(&updated[last..matched_v.start()]);
                let rep = format_updated_version(matched_v.as_str(), &en.version);
                res.push_str(&rep);
                last = matched_v.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    // 2. "Data do snapshot técnico:" and "Technical snapshot date:"
    if let Ok(re_pt_snap) = Regex::new(r#"(?im)(^\s*Data(?:\s+do\s+snapshot\s+t[ée]cnico)?\s*:\s*[`"]?)([^`"\r\n]+)([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_pt_snap.captures_iter(&updated) {
            if let Some(matched_d) = caps.get(2) {
                res.push_str(&updated[last..matched_d.start()]);
                let rep = format_updated_date(matched_d.as_str(), pt.day, pt.month_index, pt.year, "pt");
                res.push_str(&rep);
                last = matched_d.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    if let Ok(re_en_snap) = Regex::new(r#"(?im)(^\s*Technical\s+snapshot\s+date\s*:\s*[`"]?)([^`"\r\n]+)([`"]?.*$)"#) {
        let mut res = String::new();
        let mut last = 0;
        for caps in re_en_snap.captures_iter(&updated) {
            if let Some(matched_d) = caps.get(2) {
                res.push_str(&updated[last..matched_d.start()]);
                let rep = format_updated_date(matched_d.as_str(), en.day, en.month_index, en.year, "en");
                res.push_str(&rep);
                last = matched_d.end();
            }
        }
        res.push_str(&updated[last..]);
        updated = res;
    }

    // 3. Section headings with CalVer
    let pt_plain = plain_version(&pt.version);
    let en_plain = plain_version(&en.version);
    let es_plain = plain_version(&es.version);
    let fr_plain = plain_version(&fr.version);
    let it_plain = plain_version(&it.version);
    let de_plain = plain_version(&de.version);

    let pt_readme_ver = readme_version(&pt.version);
    updated = replace_line_by_regex(&updated, r"(?m)^-\s*`?v?[0-9]+(?:\.[0-9]+){1,3}`?\s*$", &format!("- `{}`", pt_readme_ver));

    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Linha T[ée]cnica e Destaques da Vers(?:ã|a)o\s+)([\d\.]+)", 2, &pt_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Technical Highlights of Version\s+)([\d\.]+)", 2, &en_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Atualiza(?:çã|ca)o [Ff]uncional [Oo]bservada na vers(?:ã|a)o\s+)([\d\.]+)", 2, &pt_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Functional [Uu]pdate [Oo]bserved in [Vv]ersion\s+)([\d\.]+)", 2, &en_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Actualizaci(?:ó|o)n funcional observada en la versi(?:ó|o)n\s+)([\d\.]+)", 2, &es_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Mise à jour fonctionnelle observée dans la version\s+)([\d\.]+)", 2, &fr_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Aggiornamento funzionale osservato nella versione\s+)([\d\.]+)", 2, &it_plain);
    updated = replace_regex_capture(&updated, r"(?i)(##\s*(?:[0-9]+\.\s*)?Funktionelles Update beobachtet in Version\s+)([\d\.]+)", 2, &de_plain);

    updated
}

pub fn update_notices_version(text: &str, version: &str, lang: &str) -> String {
    let v = plain_version(version);
    match lang {
        "pt" => replace_regex_capture(text, r"(?i)(\(versão\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
        "en" => replace_regex_capture(text, r"(?i)(\(version\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
        "es" => replace_regex_capture(text, r"(?i)(\(versión\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
        "fr" => replace_regex_capture(text, r"(?i)(\(version\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
        "it" => replace_regex_capture(text, r"(?i)(\(versione\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
        "de" => replace_regex_capture(text, r"(?i)(\(Version\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
        _ => replace_regex_capture(text, r"(?i)(\(vers(?:ão|ion|ión|ione)\s+)([0-9]+(?:\.[0-9]+){1,3})(\))", 2, &v),
    }
}

pub fn update_about_dialog_rs(text: &str, version: &str) -> String {
    let v = plain_version(version);
    let mut updated = replace_regex_capture(text, r#"(ui\.label\(")([0-9]+(?:\.[0-9]+){1,3})("\);)"#, 2, &v);
    updated = replace_regex_capture(&updated, r#"(AboutTab::Sobre => "Huginn Sentinel )([0-9]+(?:\.[0-9]+){1,3})("\\n)"#, 2, &v);
    updated = replace_regex_capture(&updated, r#"(AboutTab::Release => "Versão )([0-9]+(?:\.[0-9]+){1,3})(:\\n)"#, 2, &v);
    updated
}

pub fn update_main_cli_rs(text: &str, version: &str) -> String {
    let v = plain_version(version);
    let mut updated = replace_regex_capture(text, r#"(\s*Version )([0-9]+(?:\.[0-9]+){1,3})(\s*\|\s*Rust Security)"#, 2, &v);
    updated = replace_regex_capture(&updated, r#"(Huginn Sentinel CLI v?)([0-9]+(?:\.[0-9]+){1,3})(\s*\(Rust Edition\))"#, 2, &v);
    updated
}

pub fn update_locale_manual_rs(
    text: &str,
    pt: &LangInput,
    en: &LangInput,
    es: &LangInput,
    fr: &LangInput,
    it: &LangInput,
    de: &LangInput,
) -> String {
    let mut updated = text.to_string();

    let pt_date = format!("{} de {} de {}", pt.day, PT_MONTHS[pt.month_index as usize].to_lowercase(), pt.year);
    let en_date = format!("{} {}, {}", EN_MONTHS[en.month_index as usize], en.day, en.year);

    // 1. Atualizar ou inserir comentários de cabeçalho com Versão e Data
    if updated.contains("// Versão:") || updated.contains("// Version:") || updated.contains("// Versao:") {
        updated = replace_line_by_regex(
            &updated,
            r#"(?m)^\s*//\s*Vers(?:ã|a)o:.*$"#,
            &format!("// Versão: {} | Data: {}", plain_version(&pt.version), pt_date),
        );
        updated = replace_line_by_regex(
            &updated,
            r#"(?m)^\s*//\s*Version:.*$"#,
            &format!("// Version: {} | Date: {}", plain_version(&en.version), en_date),
        );
    } else {
        updated = format!(
            "// Versão: {} | Data: {}\n// Version: {} | Date: {}\n\n{}",
            plain_version(&pt.version),
            pt_date,
            plain_version(&en.version),
            en_date,
            updated
        );
    }

    // 2. Atualizar versão nos corpos dos 6 idiomas (compatível com consolida, formaliza ou outros verbos)
    // PT: "A versão 2026.10.30.0 consolida..."
    updated = replace_regex_capture(
        &updated,
        r#"(A versão\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+(?:consolida|formaliza|[a-zA-ZÀ-ÿ]+))"#,
        2,
        &plain_version(&pt.version),
    );
    // EN: "Version 2026.10.30.0 consolidates..."
    updated = replace_regex_capture(
        &updated,
        r#"(Version\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+(?:consolidates|marks|[a-zA-Z]+))"#,
        2,
        &plain_version(&en.version),
    );
    // ES: "La versión 2026.10.30.0 consolida..."
    updated = replace_regex_capture(
        &updated,
        r#"(La versi[óo]n\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+(?:consolida|marca|[a-zA-ZÀ-ÿ]+))"#,
        2,
        &plain_version(&es.version),
    );
    // FR: "La version 2026.10.29.0 consolide..."
    updated = replace_regex_capture(
        &updated,
        r#"(La version\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+(?:consolide|marque|[a-zA-ZÀ-ÿ]+))"#,
        2,
        &plain_version(&fr.version),
    );
    // IT: "La versione 2026.10.29.0 consolida..."
    updated = replace_regex_capture(
        &updated,
        r#"(La versione\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+(?:consolida|segna|[a-zA-ZÀ-ÿ]+))"#,
        2,
        &plain_version(&it.version),
    );
    // DE: "Version 2026.10.29.0 konsolidiert..." ou "Die Version 2026.10.29.0..."
    updated = replace_regex_capture(
        &updated,
        r#"((?:Die\s+)?Version\s+)([0-9]+(?:\.[0-9]+){1,3})(\s+(?:konsolidiert|markiert|[a-zA-ZäöüÄÖÜß]+))"#,
        2,
        &plain_version(&de.version),
    );

    updated
}

pub fn update_cargo_toml(text: &str, version: &str) -> String {
    let cv = cargo_version(version);
    // Replace package version
    replace_regex_capture(text, r#"(?m)(^\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")"#, 2, &cv)
}

pub fn update_cargo_lock(text: &str, version: &str) -> String {
    let cv = cargo_version(version);
    replace_regex_capture(text, r#"(?m)(\[\[package\]\]\s*\r?\nname\s*=\s*"huginn_rs"\s*\r?\nversion\s*=\s*")([^"]+)(")"#, 2, &cv)
}

pub fn update_installer_version(text: &str, version: &str) -> String {
    let normalized = plain_version(version);
    let mut updated = replace_regex_capture(
        text,
        r#"(?m)(^\s*#define\s+MyAppVersion\s+")([0-9]+(?:\.[0-9]+){1,3})(")"#,
        2,
        &normalized,
    );
    updated = replace_regex_capture(
        &updated,
        r#"(?m)(^\s*OutputBaseFilename=.*_v)([0-9]+(?:\.[0-9]+){1,3})(\r?$)"#,
        2,
        &normalized,
    );
    updated
}

pub fn appx_manifest_version(version: &str) -> String {
    let plain = plain_version(version);
    let parts: Vec<&str> = plain.split('.').collect();
    match parts.len() {
        1 => format!("{}.0.0.0", parts[0]),
        2 => format!("{}.{}.0.0", parts[0], parts[1]),
        3 => format!("{}.{}.{}.0", parts[0], parts[1], parts[2]),
        _ => format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]),
    }
}

pub fn update_appx_manifest_version(text: &str, version: &str) -> String {
    let normalized = appx_manifest_version(version);
    replace_regex_capture(
        text,
        r#"(?m)(<Identity\b[^>]*?\bVersion\s*=\s*")([^"]+)(")"#,
        2,
        &normalized,
    )
}

pub fn apply_updates(
    pt: &LangInput,
    en: &LangInput,
    es: &LangInput,
    fr: &LangInput,
    it: &LangInput,
    de: &LangInput,
    base_dir: &Path,
) -> Vec<UpdateResult> {
    let mut results = Vec::new();
    let entries = get_all_file_entries(base_dir);

    let pt_date = format!("{} de {} de {}", pt.day, PT_MONTHS[pt.month_index as usize].to_lowercase(), pt.year);
    let en_date = format!("{} {}, {}", EN_MONTHS[en.month_index as usize], en.day, en.year);
    let es_date = format!("{} de {} de {}", es.day, ES_MONTHS[es.month_index as usize].to_lowercase(), es.year);
    let fr_date = format!("{} {} {}", fr.day, FR_MONTHS[fr.month_index as usize].to_lowercase(), fr.year);
    let it_date = format!("{} {} {}", it.day, IT_MONTHS[it.month_index as usize].to_lowercase(), it.year);
    let de_date = format!("{}. {} {}", de.day, DE_MONTHS[de.month_index as usize], de.year);

    for entry in &entries {
        if !entry.is_updatable || is_release_notes_file(&entry.rel_path) {
            continue;
        }

        let mut abs_path = resolve_file_path(base_dir, &entry.rel_path);
        let group = entry.default_group.clone();

        let original = match read_file_robust(&abs_path) {
            Ok(content) => content,
            Err(_) => {
                results.push(UpdateResult {
                    key: entry.key.clone(),
                    rel_path: entry.rel_path.clone(),
                    group,
                    ok: false,
                    msg: "Arquivo não encontrado ou erro de leitura".to_string(),
                });
                continue;
            }
        };

        let mut new_txt = original.clone();
        let key = entry.key.as_str();

        match key {
            "privacy_pt" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versão:?.*$", &format!("Versão: {}", plain_version(&pt.version)));
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Última atualização:?.*$", &format!("Última atualização: {}", pt_date));
            }
            "privacy_en" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version:?.*$", &format!("Version: {}", plain_version(&en.version)));
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Last updated:?.*$", &format!("Last updated: {}", en_date));
            }
            "privacy_es" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versión:?.*$", &format!("Versión: {}", plain_version(&es.version)));
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Última actualización:?.*$", &format!("Última actualización: {}", es_date));
            }
            "privacy_fr" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version:?.*$", &format!("Version: {}", plain_version(&fr.version)));
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Dernière mise à jour\s*:?.*$", &format!("Dernière mise à jour: {}", fr_date));
            }
            "privacy_it" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versione:?.*$", &format!("Versione: {}", plain_version(&it.version)));
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Ultimo aggiornamento:?.*$", &format!("Ultimo aggiornamento: {}", it_date));
            }
            "privacy_de" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version:?.*$", &format!("Version: {}", plain_version(&de.version)));
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Letzte Aktualisierung:?.*$", &format!("Letzte Aktualisierung: {}", de_date));
            }
            "eula_pt" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versão\s*.*$", &format!("Versão {}, {}", plain_version(&pt.version), pt_date));
            }
            "eula_en" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version\s*.*$", &format!("Version {}, {}", plain_version(&en.version), en_date));
            }
            "eula_es" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versión\s*.*$", &format!("Versión {}, {}", plain_version(&es.version), es_date));
            }
            "eula_fr" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version\s*.*$", &format!("Version {}, {}", plain_version(&fr.version), fr_date));
            }
            "eula_it" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versione\s*.*$", &format!("Versione {}, {}", plain_version(&it.version), it_date));
            }
            "eula_de" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version\s*.*$", &format!("Version {}, {}", plain_version(&de.version), de_date));
            }
            "clc_pt" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versão\s*.*$", &format!("Versão {}, {}", plain_version(&pt.version), pt_date));
            }
            "clc_en" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version\s*.*$", &format!("Version {}, {}", plain_version(&en.version), en_date));
            }
            "clc_es" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versión\s*.*$", &format!("Versión {}, {}", plain_version(&es.version), es_date));
            }
            "clc_fr" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version\s*.*$", &format!("Version {}, {}", plain_version(&fr.version), fr_date));
            }
            "clc_it" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versione\s*.*$", &format!("Versione {}, {}", plain_version(&it.version), it_date));
            }
            "clc_de" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version\s*.*$", &format!("Version {}, {}", plain_version(&de.version), de_date));
            }
            "about_pt" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versão:?.*$", &format!("Versão: {}", plain_version(&pt.version)));
            }
            "about_en" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version:?.*$", &format!("Version: {}", plain_version(&en.version)));
            }
            "about_es" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versión:?.*$", &format!("Versión: {}", plain_version(&es.version)));
            }
            "about_fr" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version:?.*$", &format!("Version: {}", plain_version(&fr.version)));
            }
            "about_it" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Versione:?.*$", &format!("Versione: {}", plain_version(&it.version)));
            }
            "about_de" => {
                new_txt = replace_line_by_regex(&new_txt, r"(?m)^Version:?.*$", &format!("Version: {}", plain_version(&de.version)));
            }
            "notice_pt" | "notice_en" | "notice_es" | "notice_fr" | "notice_it" | "notice_de" => {
                let lang = entry.lang.as_str();
                let ver = match lang {
                    "pt" => &pt.version,
                    "en" => &en.version,
                    "es" => &es.version,
                    "fr" => &fr.version,
                    "it" => &it.version,
                    "de" => &de.version,
                    _ => &pt.version,
                };
                new_txt = update_notices_version(&new_txt, ver, lang);
            }
            "doc_architecture" | "doc_cli_reference" | "doc_compliance"
            | "doc_licensing" | "doc_security_model" | "doc_user_guide" => {
                new_txt = update_standard_doc_versions(&new_txt, pt, en);
            }
            "doc_release" => {
                new_txt = update_release_doc_versions(&new_txt, pt, en);
                let target_ver = plain_version(&pt.version);
                let new_rel_path = if entry.rel_path.starts_with("src/assets/docs/") && !base_dir.join("docs").exists() {
                    format!("src/assets/docs/release-v{}.md", target_ver)
                } else {
                    format!("docs/release-v{}.md", target_ver)
                };

                if entry.rel_path != new_rel_path && abs_path.exists() {
                    let new_abs = resolve_file_path(base_dir, &new_rel_path);
                    if new_abs != abs_path {
                        let _ = fs::remove_file(&new_abs);
                        if fs::rename(&abs_path, &new_abs).is_ok() {
                            abs_path = new_abs;
                        }
                    }
                }
            }
            "doc_suggested_issues" => {
                new_txt = update_suggested_issues_versions(&new_txt, pt);
            }
            "doc_migration_report" => {
                new_txt = update_migration_report_version(&new_txt, pt);
            }
            "readme" => {
                new_txt = update_readme_versions(&new_txt, pt, en, es, fr, it, de);
            }
            "security_md" => {
                new_txt = update_security_md_versions(&new_txt, pt, en);
            }
            "code_of_conduct_md" => {
                new_txt = update_code_of_conduct_versions(&new_txt, pt, en);
            }
            "contributing_md" => {
                new_txt = update_contributing_versions(&new_txt, pt, en);
            }
            "manual" => {
                new_txt = update_manual_versions(&new_txt, pt, en, es, fr, it, de);
            }
            "observacao" => {
                new_txt = update_observacao_versions(&new_txt, pt, en, es, fr, it, de);
            }
            "about_dialog_rs" => {
                new_txt = update_about_dialog_rs(&new_txt, &pt.version);
            }
            "main_cli_rs" => {
                new_txt = update_main_cli_rs(&new_txt, &pt.version);
            }
            "locale_manual_rs" => {
                new_txt = update_locale_manual_rs(&new_txt, pt, en, es, fr, it, de);
            }
            "cargo_toml" => {
                new_txt = update_cargo_toml(&new_txt, &pt.version);
            }
            "cargo_lock" => {
                new_txt = update_cargo_lock(&new_txt, &pt.version);
            }
            "appx_manifest" => {
                new_txt = update_appx_manifest_version(&new_txt, &pt.version);
            }
            k if k.starts_with("installer_") || entry.rel_path.to_lowercase().ends_with(".iss") => {
                new_txt = update_installer_version(&new_txt, &pt.version);
            }
            _ => {}
        }

        if new_txt != original {
            match write_file_utf8(&abs_path, &new_txt) {
                Ok(_) => {
                    results.push(UpdateResult {
                        key: entry.key.clone(),
                        rel_path: entry.rel_path.clone(),
                        group,
                        ok: true,
                        msg: "Atualizado com sucesso".to_string(),
                    });
                }
                Err(e) => {
                    results.push(UpdateResult {
                        key: entry.key.clone(),
                        rel_path: entry.rel_path.clone(),
                        group,
                        ok: false,
                        msg: format!("Erro ao gravar: {}", e),
                    });
                }
            }
        } else {
            results.push(UpdateResult {
                key: entry.key.clone(),
                rel_path: entry.rel_path.clone(),
                group,
                ok: true,
                msg: "Sem alterações necessárias (já atualizado)".to_string(),
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_locale_manual_rs_version_and_date() {
        let sample = r#"// Versão: 2026.10.29.0 | Data: 29 de outubro de 2026
// Version: 2026.10.29.0 | Date: October 29, 2026

#[derive(Debug, Clone)]
pub struct ManualDetails {
}

pub fn doc_pt_br() -> Vec<ManualSection> {
    vec![ManualSection {
        paragraphs: vec!["A versão 2026.10.29.0 consolida a refatoração integral em Rust moderno...".into()],
    }]
}
pub fn doc_en_us() -> Vec<ManualSection> {
    vec![ManualSection {
        paragraphs: vec!["Version 2026.10.29.0 consolidates the complete rewrite in modern Rust...".into()],
    }]
}
pub fn doc_es_es() -> Vec<ManualSection> {
    vec![ManualSection {
        paragraphs: vec!["La versión 2026.10.29.0 consolida la refactorización completa...".into()],
    }]
}
pub fn doc_fr_fr() -> Vec<ManualSection> {
    vec![ManualSection {
        paragraphs: vec!["La version 2026.10.29.0 consolide la réécriture intégrale...".into()],
    }]
}
pub fn doc_it_it() -> Vec<ManualSection> {
    vec![ManualSection {
        paragraphs: vec!["La versione 2026.10.29.0 consolida la riscrittura completa...".into()],
    }]
}
pub fn doc_de_de() -> Vec<ManualSection> {
    vec![ManualSection {
        paragraphs: vec!["Version 2026.10.29.0 konsolidiert die vollständige Neuentwicklung...".into()],
    }]
}
"#;

        let pt = LangInput { version: "2026.11.15.0".into(), day: 15, month_index: 10, year: 2026 }; // 10 = Novembro
        let en = LangInput { version: "2026.11.15.0".into(), day: 15, month_index: 10, year: 2026 };
        let es = LangInput { version: "2026.11.15.0".into(), day: 15, month_index: 10, year: 2026 };
        let fr = LangInput { version: "2026.11.15.0".into(), day: 15, month_index: 10, year: 2026 };
        let it = LangInput { version: "2026.11.15.0".into(), day: 15, month_index: 10, year: 2026 };
        let de = LangInput { version: "2026.11.15.0".into(), day: 15, month_index: 10, year: 2026 };

        let updated = update_locale_manual_rs(sample, &pt, &en, &es, &fr, &it, &de);

        assert!(updated.contains("// Versão: 2026.11.15.0 | Data: 15 de novembro de 2026"), "PT header mismatch: {}", updated);
        assert!(updated.contains("// Version: 2026.11.15.0 | Date: November 15, 2026"), "EN header mismatch: {}", updated);
        assert!(updated.contains("A versão 2026.11.15.0 consolida"), "PT body mismatch");
        assert!(updated.contains("Version 2026.11.15.0 consolidates"), "EN body mismatch");
        assert!(updated.contains("La versión 2026.11.15.0 consolida"), "ES body mismatch");
        assert!(updated.contains("La version 2026.11.15.0 consolide"), "FR body mismatch");
        assert!(updated.contains("La versione 2026.11.15.0 consolida"), "IT body mismatch");
        assert!(updated.contains("Version 2026.11.15.0 konsolidiert"), "DE body mismatch");
    }

    #[test]
    fn test_locale_manual_rs_patterns_match() {
        let patterns = build_patterns();
        let pats = patterns.get("locale_manual_rs").expect("pattern exists");
        let content = r#"// Versão: 2026.10.30.0 | Data: 30 de outubro de 2026
// Version: 2026.10.30.0 | Date: October 30, 2026

pub fn doc_pt_br() {
    "A versão 2026.10.30.0 consolida a refatoração...";
}
pub fn doc_en_us() {
    "Version 2026.10.30.0 consolidates the rewrite...";
}
"#;
        for &p in pats {
            let re = Regex::new(p).unwrap();
            assert!(re.is_match(content), "Pattern failed to match: {}", p);
        }
    }

    #[test]
    fn test_update_readme_versions_and_dates_all_languages() {
        let sample = r#"
<h2 id="ptbr">🇧🇷 Português (BR)</h2>
> **Observação:** Este repositório refere-se à versão estável **`2026.10.30.0`** do Projeto Huginn Sentinel.
- **Versão:** `2026.10.30.0` (Data: 30 de outubro de 2026)<br>

<h2 id="enus">🇺🇸 English (US)</h2>
> **Note:** This repository refers to stable version **`2026.10.30.0`** of the Huginn Sentinel Project.
- **Version:** `2026.10.30.0` (Date: October 30, 2026)<br>

<h2 id="eses">🇪🇸 Español (ES)</h2>
> **Nota:** Este repositorio se refiere a la versión estable **`2026.10.30.0`** del Proyecto Huginn Sentinel.
- **Versión:** `2026.10.30.0` (Fecha: 30 de octubre de 2026)<br>

<h2 id="frfr">🇫🇷 Français (FR)</h2>
> **Remarque :** Ce dépôt fait référence à la version stable **`2026.10.30.0`** du Projet Huginn Sentinel.
- **Version :** `2026.10.30.0` (Date : 30 octobre 2026)<br>

<h2 id="itit">🇮🇹 Italiano (IT)</h2>
> **Nota:** Questo repository fa riferimento alla versione stabile **`2026.10.30.0`** del Progetto Huginn Sentinel.
- **Versione:** `2026.10.30.0` (Data: 30 ottobre 2026)<br>

<h2 id="dede">🇩🇪 Deutsch (DE)</h2>
> **Hinweis:** Dieses Repository bezieht sich auf die stabile Version **`2026.10.30.0`** des Projekts Huginn Sentinel.
- **Version:** `2026.10.30.0` (Datum: 30. Oktober 2026)<br>
"#;

        let pt = LangInput { version: "2026.11.05.0".into(), day: 5, month_index: 10, year: 2026 }; // Novembro
        let en = LangInput { version: "2026.11.05.0".into(), day: 5, month_index: 10, year: 2026 };
        let es = LangInput { version: "2026.11.05.0".into(), day: 5, month_index: 10, year: 2026 };
        let fr = LangInput { version: "2026.11.05.0".into(), day: 5, month_index: 10, year: 2026 };
        let it = LangInput { version: "2026.11.05.0".into(), day: 5, month_index: 10, year: 2026 };
        let de = LangInput { version: "2026.11.05.0".into(), day: 5, month_index: 10, year: 2026 };

        let updated = update_readme_versions(sample, &pt, &en, &es, &fr, &it, &de);

        // Check PT
        assert!(updated.contains("**`2026.11.05.0`** do Projeto Huginn Sentinel"), "PT callout mismatch");
        assert!(updated.contains("- **Versão:** `2026.11.05.0` (Data: 5 de novembro de 2026)<br>"), "PT spec mismatch");

        // Check EN
        assert!(updated.contains("**`2026.11.05.0`** of the Huginn Sentinel Project"), "EN callout mismatch");
        assert!(updated.contains("- **Version:** `2026.11.05.0` (Date: November 5, 2026)<br>"), "EN spec mismatch");

        // Check ES
        assert!(updated.contains("- **Versión:** `2026.11.05.0` (Fecha: 5 de noviembre de 2026)<br>"), "ES spec mismatch");

        // Check FR
        assert!(updated.contains("- **Version :** `2026.11.05.0` (Date : 5 novembre 2026)<br>"), "FR spec mismatch");

        // Check IT
        assert!(updated.contains("- **Versione:** `2026.11.05.0` (Data: 5 novembre 2026)<br>"), "IT spec mismatch");

        // Check DE
        assert!(updated.contains("- **Version:** `2026.11.05.0` (Datum: 5. November 2026)<br>"), "DE spec mismatch");
    }

    #[test]
    fn test_parse_version_formats() {
        assert_eq!(parse_version_string("2026.10.30.0"), Some("2026.10.30.0".into()));
        assert_eq!(parse_version_string("v2026.10.30.0"), Some("v2026.10.30.0".into()));
        assert_eq!(plain_version("v2026.10.30.0"), "2026.10.30.0");
        assert_eq!(plain_version("2026.10.30.0"), "2026.10.30.0");
        assert_eq!(readme_version("2026.10.30.0"), "v2026.10.30.0");
        assert_eq!(readme_version("v2026.10.30.0"), "v2026.10.30.0");
        assert_eq!(parse_version_to_date("2026.10.30.0"), Some((30, 9, 2026, Some(0))));
        assert_eq!(parse_version_to_date("v2026.10.30.0"), Some((30, 9, 2026, Some(0))));
    }

    #[test]
    fn test_parse_dates_all_languages() {
        // PT
        assert_eq!(parse_date_string("30 de outubro de 2026"), Some((30, 9, 2026)));
        // EN
        assert_eq!(parse_date_string("October 30, 2026"), Some((30, 9, 2026)));
        assert_eq!(parse_date_string("October 30 2026"), Some((30, 9, 2026)));
        assert_eq!(parse_date_string("30th October 2026"), Some((30, 9, 2026)));
        // ES
        assert_eq!(parse_date_string("30 de octubre de 2026"), Some((30, 9, 2026)));
        // FR
        assert_eq!(parse_date_string("30 octobre 2026"), Some((30, 9, 2026)));
        // IT
        assert_eq!(parse_date_string("30 ottobre 2026"), Some((30, 9, 2026)));
        // DE
        assert_eq!(parse_date_string("30. Oktober 2026"), Some((30, 9, 2026)));
    }

    #[test]
    fn test_readme_patterns_and_extraction() {
        let root = project_root_dir();
        let readme_path = root.join("README.md");
        assert!(readme_path.exists(), "README.md exists at root");
        let content = fs::read_to_string(&readme_path).expect("read README.md");

        // 1. All patterns in build_patterns()["readme"] must match
        let patterns = build_patterns();
        let readme_pats = patterns.get("readme").expect("readme patterns exist");
        for &pat in readme_pats {
            let re = Regex::new(pat).expect("valid regex");
            assert!(re.is_match(&content), "README.md failed to match pattern: {}", pat);
        }

        // 2. All 6 language sections must resolve to valid version and date, and be consistent
        let langs = [
            ("ptbr", "PT"),
            ("enus", "EN"),
            ("eses", "ES"),
            ("frfr", "FR"),
            ("itit", "IT"),
            ("dede", "DE"),
        ];
        let mut expected_ver = None;
        let mut expected_date = None;
        for (sec_id, name) in langs {
            let (ver, date) = extract_readme_section_version_and_date(&content, sec_id);
            assert!(ver.is_some(), "Section {} ({}) version missing", sec_id, name);
            assert!(date.is_some(), "Section {} ({}) date missing", sec_id, name);
            if expected_ver.is_none() {
                expected_ver = ver.clone();
                expected_date = date;
            } else {
                assert_eq!(ver, expected_ver, "Section {} ({}) version mismatch with PT", sec_id, name);
                assert_eq!(date, expected_date, "Section {} ({}) date mismatch with PT", sec_id, name);
            }
        }
    }

    #[test]
    fn test_update_installer_version() {
        let sample = r#"
#define MyAppName "Huginn Sentinel"
#define MyAppVersion "2026.10.30.0"
OutputBaseFilename=Huginn_v2026.10.30.0
OutputBaseFilenameMacro=Huginn_v{#MyAppVersion}
"#;
        let updated = update_installer_version(sample, "2026.11.01.0");
        assert!(updated.contains(r#"#define MyAppVersion "2026.11.01.0""#));
        assert!(updated.contains("OutputBaseFilename=Huginn_v2026.11.01.0"));
        assert!(updated.contains("OutputBaseFilenameMacro=Huginn_v{#MyAppVersion}"));
    }

    #[test]
    fn test_installer_files_discovery_and_patterns() {
        let root = project_root_dir();
        let entries = get_all_file_entries(&root);
        let iss_entries: Vec<_> = entries.iter().filter(|e| e.rel_path.ends_with(".iss")).collect();

        // At least 4 .iss files must be discovered from repo/
        assert!(iss_entries.len() >= 4, "Expected at least 4 .iss files, found {}", iss_entries.len());
        assert!(iss_entries.iter().any(|e| e.rel_path == "repo/Huginn.iss"));
        assert!(iss_entries.iter().any(|e| e.rel_path == "repo/Huginn_mingw.iss"));
        assert!(iss_entries.iter().any(|e| e.rel_path == "repo/Huginn_msvc.iss"));
        assert!(iss_entries.iter().any(|e| e.rel_path == "repo/Huginn_msvc_mingw.iss"));

        let statuses = check_all_files(&root);
        for entry in &iss_entries {
            let st = statuses.iter().find(|s| s.rel_path == entry.rel_path).expect("status found");
            assert!(st.exists, "Installer file {} exists", st.rel_path);
            assert!(st.missing_patterns.is_empty(), "Installer file {} has no missing patterns: {:?}", st.rel_path, st.missing_patterns);
            assert!(st.detected_version.is_some(), "Installer file {} has detected version", st.rel_path);
        }
    }

    #[test]
    fn test_docs_files_discovery_and_patterns() {
        let root = project_root_dir();
        let entries = get_all_file_entries(&root);
        let doc_entries: Vec<_> = entries.iter().filter(|e| e.default_group == "DOCS").collect();

        assert_eq!(doc_entries.len(), 9, "Expected 9 DOCS entries, found {}", doc_entries.len());
        for entry in &doc_entries {
            assert!(
                entry.rel_path.starts_with("docs/"),
                "Expected {} to start with 'docs/', but got {}",
                entry.key,
                entry.rel_path
            );
        }

        let statuses = check_all_files(&root);
        for entry in &doc_entries {
            let st = statuses.iter().find(|s| s.rel_path == entry.rel_path).expect("status found");
            assert!(st.exists, "Doc file {} does not exist at {}", entry.key, st.rel_path);
            assert!(
                st.missing_patterns.is_empty(),
                "Doc file {} at {} has missing patterns: {:?}",
                entry.key,
                st.rel_path,
                st.missing_patterns
            );
            assert!(
                st.detected_version.is_some(),
                "Doc file {} at {} has no detected version",
                entry.key,
                st.rel_path
            );
        }
    }

    #[test]
    fn test_update_standard_and_release_doc_versions() {
        let sample_standard = r#"
- **Versão:** `2026.10.30.0`
- **Data:** `30 de outubro de 2026`
- **Version:** `2026.10.30.0`
- **Date:** `October 30, 2026`
"#;
        let pt = LangInput { version: "2026.11.03.0".into(), day: 3, month_index: 10, year: 2026 };
        let en = LangInput { version: "2026.11.03.0".into(), day: 3, month_index: 10, year: 2026 };

        let updated_std = update_standard_doc_versions(sample_standard, &pt, &en);
        assert!(updated_std.contains("- **Versão:** `2026.11.03.0`"));
        assert!(updated_std.contains("- **Data:** `3 de novembro de 2026`"));
        assert!(updated_std.contains("- **Version:** `2026.11.03.0`"));
        assert!(updated_std.contains("- **Date:** `November 3, 2026`"));

        let sample_release = r#"
# Notas de Lançamento / Release Notes — v2026.10.30.0
- **Tag de Versão:** `v2026.10.30.0`
- **Data de Publicação:** 30 de outubro de 2026
- **Version Tag:** `v2026.10.30.0`
- **Publication Date:** October 30, 2026
"#;
        let updated_rel = update_release_doc_versions(sample_release, &pt, &en);
        assert!(updated_rel.contains("# Notas de Lançamento / Release Notes — v2026.11.03.0"));
        assert!(updated_rel.contains("- **Tag de Versão:** `v2026.11.03.0`"));
        assert!(updated_rel.contains("- **Data de Publicação:** 3 de novembro de 2026"));
        assert!(updated_rel.contains("- **Version Tag:** `v2026.11.03.0`"));
        assert!(updated_rel.contains("- **Publication Date:** November 3, 2026"));
    }

    #[test]
    fn test_appx_manifest_version_and_repo_tab() {
        let root = project_root_dir();
        let entries = get_all_file_entries(&root);
        let appx_entry = entries.iter().find(|e| e.key == "appx_manifest").expect("appx_manifest entry exists");
        assert_eq!(appx_entry.default_group, "REPO", "appx_manifest must be in REPO group");
        assert_eq!(appx_entry.rel_path, "repo/AppxManifest.xml");

        let statuses = check_all_files(&root);
        let st = statuses.iter().find(|s| s.rel_path == "repo/AppxManifest.xml").expect("appx status found");
        assert!(st.exists, "repo/AppxManifest.xml must exist");
        assert_eq!(st.group, "REPO", "status group must be REPO");
        assert!(st.missing_patterns.is_empty(), "missing patterns: {:?}", st.missing_patterns);
        assert_eq!(st.detected_version.as_deref(), Some("2026.11.9.0"));

        let sample = r#"<Identity Name="fernandoncidade.Huginn" Publisher="CN=TEST" Version="2026.11.9.0" ProcessorArchitecture="x64" />"#;
        let updated = update_appx_manifest_version(sample, "2026.11.10.1");
        assert!(updated.contains(r#"Version="2026.11.10.1""#));

        let updated_quad = update_appx_manifest_version(sample, "2026.11.10");
        assert!(updated_quad.contains(r#"Version="2026.11.10.0""#));
    }
}
