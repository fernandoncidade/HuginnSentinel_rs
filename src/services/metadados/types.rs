use std::io::Read;
use std::path::Path;

const COMPOUND_EXTENSIONS: &[&str] = &[
    "tar.gz",
    "tar.bz2",
    "tar.xz",
    "tar.zst",
    "tar.lz",
    "tar.lzma",
    "tar.z",
    "sql.gz",
    "sql.bz2",
    "sql.xz",
    "fb2.zip",
    "pb.gz",
    "paf.exe",
    "vbox-prev",
    "env.local",
    "env.development",
    "env.production",
    "env.test",
    "env.example",
];

/// Extrai a extensão preferencial do arquivo (em minúsculas, sem ponto inicial),
/// preservando extensões compostas e arquivos iniciados por ponto (dotfiles).
pub fn extensao_tipo_preferencial(caminho: &Path) -> String {
    let nome_arquivo = caminho
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .trim();

    if nome_arquivo.is_empty() {
        return String::new();
    }

    let lower = nome_arquivo.to_lowercase();

    if lower.ends_with(".lnk") {
        return "lnk".to_string();
    }
    if lower.ends_with(".db-shm") {
        return "db-shm".to_string();
    }
    if lower.ends_with(".db-wal") {
        return "db-wal".to_string();
    }
    if lower.ends_with(".db-journal") {
        return "db-journal".to_string();
    }

    for &composto in COMPOUND_EXTENSIONS {
        let sufixo = format!(".{}", composto);
        if lower.len() > sufixo.len() && lower.ends_with(&sufixo) {
            return composto.to_string();
        }
    }

    if let Some(ext) = caminho.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.trim().to_lowercase();
        if !ext_lower.is_empty() {
            return ext_lower;
        }
    }

    // Dotfiles como ".gitignore", ".env", ".bashrc"
    if lower.starts_with('.') && lower.len() > 1 && !lower[1..].contains('.') {
        return lower[1..].to_string();
    }

    String::new()
}

/// Retorna `(tipo, mime)` para um arquivo ou diretório.
/// Para diretórios, `tipo` é `"folder"` e `mime` é `"inode/directory"`.
/// Para arquivos com extensão, `tipo` é a extensão em letras minúsculas precedida por ponto (ex: `".txt"`, `".exe"`, `".tar.gz"`).
/// Para arquivos sem extensão, infere a extensão ou categoria a partir do nome ou assinatura mágica.
pub fn obter_tipo_e_mime(caminho: &Path, is_dir: bool) -> (String, String) {
    if is_dir {
        return ("folder".to_string(), "inode/directory".to_string());
    }

    let ext = extensao_tipo_preferencial(caminho);
    if !ext.is_empty() {
        let mime = obter_mime_por_extensao(&ext, caminho);
        return (format!(".{}", ext.trim_start_matches('.')), mime);
    }

    let (cat, mime) = identificar_sem_extensao(caminho);
    (cat.to_string(), mime.to_string())
}

fn identificar_sem_extensao(caminho: &Path) -> (&'static str, &'static str) {
    let nome = caminho
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let lower = nome.to_lowercase();

    match lower.as_str() {
        "makefile" | "gnumakefile" | "dockerfile" | "containerfile" | "jenkinsfile"
        | "vagrantfile" | "procfile" | "rakefile" | "gemfile" | "podfile" | "brewfile"
        | "justfile" | "cmakelists.txt" | "build" | "workspace" | "sconstruct" => {
            return ("code", "text/plain");
        }
        "readme" | "license" | "licence" | "copying" | "changelog" | "authors" | "news"
        | "todo" | "notice" | "patents" | "contributing" | "maintainers" => {
            return ("document", "text/plain");
        }
        "hosts" | "fstab" | "crypttab" | "mtab" | "sudoers" | "crontab" => {
            return ("config", "text/plain");
        }
        _ => {}
    }

    if let Ok(mut file) = super::extractor::abrir_arquivo_compartilhado(caminho) {
        let mut buf = [0u8; 512];
        if let Ok(n) = file.read(&mut buf) {
            if n >= 4 {
                if &buf[0..4] == b"%PDF" {
                    return (".pdf", "application/pdf");
                }
                if &buf[0..4] == b"\x89PNG" {
                    return (".png", "image/png");
                }
                if &buf[0..3] == b"\xFF\xD8\xFF" {
                    return (".jpg", "image/jpeg");
                }
                if &buf[0..4] == b"GIF8" {
                    return (".gif", "image/gif");
                }
                if &buf[0..4] == b"PK\x03\x04" {
                    return (".zip", "application/zip");
                }
                if &buf[0..2] == b"MZ" {
                    return (".exe", "application/vnd.microsoft.portable-executable");
                }
                if &buf[0..4] == b"\x7FELF" {
                    return (".elf", "application/x-executable");
                }
                if n >= 16 && &buf[0..15] == b"SQLite format 3" {
                    return (".sqlite", "application/vnd.sqlite3");
                }
                if &buf[0..2] == b"#!" {
                    return ("code", "text/x-script");
                }
            }
            if n > 0 {
                if amostra_parece_texto(&buf[..n]) {
                    return ("text", "text/plain");
                }
                return ("binary", "application/octet-stream");
            }
        }
    } else if std::fs::metadata(caminho).map(|m| m.len()).unwrap_or(0) > 0 {
        return ("binary", "application/octet-stream");
    }

    ("file", "application/octet-stream")
}

/// Tamanho da amostra (em bytes) usada para classificar conteúdo como texto ou binário.
pub const TAMANHO_AMOSTRA_TEXTO: usize = 512;

/// Classificador único de conteúdo texto x binário.
///
/// É usado tanto na detecção de tipo de arquivos sem extensão (`text`/`binary`) quanto na
/// extração de metadados de arquivos genéricos (`.tmp`, `.dat`, extensões desconhecidas),
/// garantindo que o mesmo conteúdo receba sempre as mesmas colunas de metadados.
pub fn amostra_parece_texto(buf: &[u8]) -> bool {
    if buf.is_empty() {
        return true;
    }
    // BOM UTF-16 (LE/BE) indica texto mesmo com bytes nulos intercalados
    if buf.len() >= 2 && ((buf[0] == 0xFF && buf[1] == 0xFE) || (buf[0] == 0xFE && buf[1] == 0xFF)) {
        return true;
    }
    let amostra = &buf[..buf.len().min(TAMANHO_AMOSTRA_TEXTO)];
    let non_text = amostra
        .iter()
        .filter(|&&b| b == 0 || (b < 0x09) || (b > 0x0D && b < 0x20 && b != 0x1B))
        .count();
    non_text * 10 < amostra.len()
}

pub fn obter_mime_por_extensao(ext: &str, caminho: &Path) -> String {
    let ext = ext.trim_start_matches('.');
    match ext {
        // Código Fonte
        "rs" => "text/x-rust",
        "cpp" | "cxx" | "cc" | "c++" | "ino" | "ipp" | "tpp" | "inl" => "text/x-c++src",
        "c" => "text/x-csrc",
        "h" | "hpp" | "hxx" | "hh" | "h++" => "text/x-chdr",
        "py" | "pyw" | "pyx" | "pxd" | "pxi" => "text/x-python",
        "js" | "mjs" | "cjs" | "jsx" => "application/javascript",
        "ts" | "tsx" | "mts" | "cts" => "application/typescript",
        "java" => "text/x-java-source",
        "kt" | "kts" => "text/x-kotlin",
        "scala" => "text/x-scala",
        "cs" | "csx" => "text/x-csharp",
        "fs" | "fsx" | "fsi" => "text/x-fsharp",
        "vb" | "vbs" => "text/x-vb",
        "go" => "text/x-go",
        "swift" => "text/x-swift",
        "zig" => "text/x-zig",
        "php" | "phtml" => "application/x-httpd-php",
        "rb" | "erb" | "rake" | "gemspec" => "text/x-ruby",
        "pl" | "pm" => "text/x-perl",
        "lua" => "text/x-lua",
        "r" | "rmd" | "qmd" => "text/x-r",
        "jl" => "text/x-julia",
        "dart" => "application/vnd.dart",
        "html" | "htm" | "xhtml" | "vue" | "svelte" | "astro" => "text/html",
        "css" | "scss" | "sass" | "less" | "styl" => "text/css",
        "sql" | "pls" | "plsql" | "psql" | "tsql" | "hql" | "cql" => "application/sql",
        "sh" | "bash" | "zsh" | "fish" | "ksh" | "csh" | "tcsh" => "application/x-sh",
        "bat" | "cmd" => "application/x-bat",
        "ps1" | "psm1" | "psd1" => "application/x-powershell",
        "asm" | "s" | "nasm" | "masm" => "text/x-asm",
        "ipynb" => "application/x-ipynb+json",
        "wasm" => "application/wasm",

        // Imagens
        "png" | "apng" => "image/png",
        "jpg" | "jpeg" | "jpe" | "jif" | "jfif" | "jfi" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" | "dib" | "rle" => "image/bmp",
        "webp" => "image/webp",
        "ico" | "cur" | "ani" => "image/x-icon",
        "icns" => "image/x-icns",
        "svg" | "svgz" => "image/svg+xml",
        "tiff" | "tif" | "geotiff" => "image/tiff",
        "heic" | "heif" | "heics" | "heifs" => "image/heic",
        "avif" | "avifs" => "image/avif",
        "jxl" => "image/jxl",
        "jp2" | "j2k" | "jpf" | "jpx" | "jpm" => "image/jp2",
        "psd" | "psb" | "pdd" => "image/vnd.adobe.photoshop",
        "tga" | "icb" | "vda" | "vst" => "image/x-tga",
        "pcx" | "dcx" => "image/x-pcx",
        "pbm" => "image/x-portable-bitmap",
        "pgm" => "image/x-portable-graymap",
        "ppm" => "image/x-portable-pixmap",
        "pnm" | "pam" => "image/x-portable-anymap",
        "xbm" => "image/x-xbitmap",
        "xpm" => "image/x-xpixmap",
        "exr" => "image/x-exr",
        "hdr" | "rgbe" | "xyze" => "image/vnd.radiance",
        "dds" => "image/vnd.ms-dds",
        "qoi" => "image/qoi",
        "emf" | "emz" => "image/emf",
        "wmf" | "wmz" => "image/wmf",
        "cr2" | "cr3" | "crw" | "nef" | "nrw" | "arw" | "dng" | "orf" | "rw2" | "pef" | "raf"
        | "srw" | "raw" => "image/x-dcraw",

        // Áudio
        "mp3" | "mp2" | "mp1" | "mpa" => "audio/mpeg",
        "wav" | "wave" | "bwf" | "rf64" => "audio/wav",
        "flac" | "fla" => "audio/flac",
        "ogg" | "oga" | "ogx" | "spx" => "audio/ogg",
        "opus" => "audio/opus",
        "aac" => "audio/aac",
        "m4a" | "m4b" | "m4p" | "m4r" | "alac" | "3ga" => "audio/mp4",
        "wma" => "audio/x-ms-wma",
        "aiff" | "aif" | "aifc" | "afc" => "audio/aiff",
        "ape" => "audio/x-ape",
        "wv" | "wvc" => "audio/x-wavpack",
        "ac3" | "eac3" | "ec3" => "audio/ac3",
        "dts" | "dtshd" => "audio/vnd.dts",
        "amr" | "awb" => "audio/amr",
        "au" | "snd" => "audio/basic",
        "mid" | "midi" | "kar" | "rmi" => "audio/midi",
        "mka" => "audio/x-matroska",
        "weba" => "audio/webm",
        "caf" => "audio/x-caf",

        // Vídeo
        "mp4" | "m4v" | "mp4v" | "mpg4" | "f4v" => "video/mp4",
        "mkv" | "mk3d" | "mks" => "video/x-matroska",
        "avi" | "divx" | "xvid" => "video/x-msvideo",
        "mov" | "qt" => "video/quicktime",
        "wmv" | "asf" => "video/x-ms-wmv",
        "webm" => "video/webm",
        "flv" => "video/x-flv",
        "mpg" | "mpeg" | "mpe" | "m1v" | "m2v" | "mpv" | "vob" => "video/mpeg",
        "m2ts" | "m2t" | "tp" | "trp" => "video/mp2t",
        "ogv" | "ogm" => "video/ogg",
        "3gp" | "3gpp" => "video/3gpp",
        "3g2" | "3gp2" | "3gpp2" => "video/3gpp2",
        "rm" | "rmvb" | "rv" => "application/vnd.rn-realmedia",

        // Documentos
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "docm" => "application/vnd.ms-word.document.macroEnabled.12",
        "dotx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.template",
        "dotm" => "application/vnd.ms-word.template.macroEnabled.12",
        "doc" | "dot" | "wbk" => "application/msword",
        "rtf" => "application/rtf",
        "odt" | "fodt" => "application/vnd.oasis.opendocument.text",
        "ott" => "application/vnd.oasis.opendocument.text-template",
        "pages" => "application/vnd.apple.pages",
        "abw" | "zabw" => "application/x-abiword",
        "epub" | "epub3" => "application/epub+zip",
        "mobi" | "prc" => "application/x-mobipocket-ebook",
        "azw" | "azw3" | "kf8" | "kfx" => "application/vnd.amazon.ebook",
        "fb2" | "fb2.zip" | "fbz" => "application/x-fictionbook+xml",
        "djvu" | "djv" => "image/vnd.djvu",
        "xps" | "oxps" => "application/vnd.ms-xpsdocument",
        "txt" | "text" | "nfo" | "diz" | "asc" | "rst" | "adoc" | "asciidoc" | "org" => {
            "text/plain"
        }
        "md" | "markdown" | "mdown" | "mkd" | "mkdn" => "text/markdown",
        "tex" | "ltx" | "sty" | "cls" | "bib" => "application/x-tex",

        // Planilhas
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xlsm" => "application/vnd.ms-excel.sheet.macroEnabled.12",
        "xltx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.template",
        "xltm" => "application/vnd.ms-excel.template.macroEnabled.12",
        "xlsb" => "application/vnd.ms-excel.sheet.binary.macroEnabled.12",
        "xls" | "xlt" | "xlm" | "xlw" => "application/vnd.ms-excel",
        "csv" | "bcsv" | "dsv" | "psv" | "ssv" => "text/csv",
        "tsv" | "tab" => "text/tab-separated-values",
        "ods" | "fods" => "application/vnd.oasis.opendocument.spreadsheet",
        "ots" => "application/vnd.oasis.opendocument.spreadsheet-template",
        "numbers" => "application/vnd.apple.numbers",
        "gnumeric" | "gnm" => "application/x-gnumeric",
        "slk" | "sylk" => "text/spreadsheet",
        "dif" => "application/x-dif",
        "parquet" => "application/vnd.apache.parquet",

        // Apresentações
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "pptm" => "application/vnd.ms-powerpoint.presentation.macroEnabled.12",
        "potx" => "application/vnd.openxmlformats-officedocument.presentationml.template",
        "ppsx" => "application/vnd.openxmlformats-officedocument.presentationml.slideshow",
        "ppt" | "pot" | "pps" | "ppa" => "application/vnd.ms-powerpoint",
        "odp" | "fodp" => "application/vnd.oasis.opendocument.presentation",
        "otp" => "application/vnd.oasis.opendocument.presentation-template",
        "key" | "keynote" => "application/vnd.apple.keynote",

        // Compactados e Imagens de Disco
        "zip" | "zipx" | "cbz" | "jar" | "war" | "ear" | "apk" | "aab" | "ipa" | "xpi" | "vsix"
        | "nupkg" | "whl" => "application/zip",
        "7z" | "cb7" => "application/x-7z-compressed",
        "tar" | "cbt" => "application/x-tar",
        "gz" | "tgz" | "tar.gz" | "taz" | "pb.gz" | "sql.gz" => "application/gzip",
        "bz2" | "tbz" | "tbz2" | "tar.bz2" | "sql.bz2" => "application/x-bzip2",
        "xz" | "txz" | "tar.xz" | "sql.xz" => "application/x-xz",
        "zst" | "tzst" | "tar.zst" => "application/zstd",
        "rar" | "cbr" | "r00" | "r01" => "application/vnd.rar",
        "iso" | "udf" | "img" => "application/x-iso9660-image",
        "vhd" | "vhdx" | "vmdk" | "vdi" | "qcow" | "qcow2" | "wim" | "esd" | "ova" => {
            "application/x-virtual-disk"
        }
        "cab" => "application/vnd.ms-cab-compressed",
        "deb" | "udeb" => "application/vnd.debian.binary-package",
        "rpm" => "application/x-rpm",
        "dmg" => "application/x-apple-diskimage",

        // Executáveis, Fontes e Sistema
        "exe" | "dll" | "scr" | "cpl" | "ocx" | "ax" | "ime" | "com" | "paf.exe" | "mui"
        | "mun" => "application/vnd.microsoft.portable-executable",
        "sys" | "drv" | "vxd" | "ko" => "application/x-sys",
        "msi" | "msp" | "mst" | "msu" => "application/x-msi",
        "elf" | "so" | "out" | "o" | "obj" | "a" | "lib" => "application/x-executable",
        "dylib" => "application/x-mach-binary",
        "ttf" | "ttc" => "font/ttf",
        "otf" | "otc" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "eot" => "application/vnd.ms-fontobject",
        "fon" | "fnt" => "application/x-font-dos",
        "lnk" => "application/x-ms-shortcut",

        // Bancos de Dados e GIS
        "db" | "sqlite" | "sqlite3" | "db3" | "s3db" | "sl3" | "db-shm" | "db-wal"
        | "db-journal" | "gpkg" | "mbtiles" | "spatialite" => "application/vnd.sqlite3",
        "mdb" | "accdb" | "accde" | "accdt" | "mde" => "application/x-msaccess",
        "dbf" => "application/x-dbf",
        "mdf" | "ldf" | "ndf" | "bak" => "application/octet-stream",
        "geojson" | "topojson" => "application/geo+json",
        "kml" => "application/vnd.google-earth.kml+xml",
        "kmz" => "application/vnd.google-earth.kmz",
        "gpx" => "application/gpx+xml",
        "shp" | "shx" => "application/x-esri-shape",

        // Configuração, Logs, Temp, Backup
        "json" | "jsonc" | "json5" | "jsonl" | "ndjson" | "hjson" | "har" => "application/json",
        "xml" | "xsd" | "dtd" | "xsl" | "xslt" | "plist" | "manifest" | "csproj" | "vcxproj"
        | "props" | "targets" | "qrc" | "ui" | "SVG" => "application/xml",
        "yaml" | "yml" => "application/x-yaml",
        "toml" | "tml" | "lock" => "application/toml",
        "ini" | "inf" | "cfg" | "conf" | "config" | "properties" | "prop" | "prefs" | "env"
        | "env.local" | "env.development" | "env.production" | "env.test" | "env.example"
        | "editorconfig" | "gitignore" | "gitattributes" | "gitmodules" | "dockerignore"
        | "npmrc" | "bashrc" | "zshrc" | "reg" => "text/plain",
        "log" | "logs" | "err" | "trace" | "trc" => "text/plain",
        "tmp" | "temp" | "swp" | "part" | "crdownload" | "vbox-prev" | "old" | "orig" | "bkp"
        | "backup" | "bin" | "dat" => "application/octet-stream",

        _ => {
            if !caminho.as_os_str().is_empty() && caminho.is_dir() {
                "inode/directory"
            } else {
                "application/octet-stream"
            }
        }
    }
    .to_string()
}

pub fn identificar_tipo_arquivo(caminho: &Path) -> (&'static str, &'static str) {
    if caminho.is_dir() {
        return ("folder", "inode/directory");
    }
    let ext = caminho
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "rs" => (".rs", "text/x-rust"),
        "cpp" | "cxx" | "cc" => (".cpp", "text/x-c++src"),
        "c" => (".c", "text/x-csrc"),
        "h" | "hpp" => (".h", "text/x-chdr"),
        "py" | "pyw" => (".py", "text/x-python"),
        "js" | "mjs" => (".js", "application/javascript"),
        "ts" | "tsx" => (".ts", "application/typescript"),
        "java" => (".java", "text/x-java-source"),
        "cs" => (".cs", "text/x-csharp"),
        "go" => (".go", "text/x-go"),
        "php" => (".php", "application/x-httpd-php"),
        "html" | "htm" => (".html", "text/html"),
        "css" => (".css", "text/css"),
        "sql" => (".sql", "application/sql"),
        "sh" | "bash" => (".sh", "application/x-sh"),
        "bat" | "cmd" => (".bat", "application/x-bat"),
        "ps1" | "psm1" => (".ps1", "application/x-powershell"),
        "png" => (".png", "image/png"),
        "jpg" | "jpeg" => (".jpg", "image/jpeg"),
        "gif" => (".gif", "image/gif"),
        "bmp" => (".bmp", "image/bmp"),
        "webp" => (".webp", "image/webp"),
        "ico" => (".ico", "image/x-icon"),
        "svg" => (".svg", "image/svg+xml"),
        "tiff" | "tif" => (".tiff", "image/tiff"),
        "mp3" => (".mp3", "audio/mpeg"),
        "wav" => (".wav", "audio/wav"),
        "flac" => (".flac", "audio/flac"),
        "ogg" => (".ogg", "audio/ogg"),
        "aac" => (".aac", "audio/aac"),
        "m4a" => (".m4a", "audio/mp4"),
        "mp4" => (".mp4", "video/mp4"),
        "mkv" => (".mkv", "video/x-matroska"),
        "avi" => (".avi", "video/x-msvideo"),
        "mov" => (".mov", "video/quicktime"),
        "wmv" => (".wmv", "video/x-ms-wmv"),
        "webm" => (".webm", "video/webm"),
        "pdf" => (".pdf", "application/pdf"),
        "docx" => (
            ".docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        "doc" => (".doc", "application/msword"),
        "rtf" => (".rtf", "application/rtf"),
        "odt" => (".odt", "application/vnd.oasis.opendocument.text"),
        "txt" => (".txt", "text/plain"),
        "md" => (".md", "text/markdown"),
        "xlsx" => (
            ".xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        "xls" => (".xls", "application/vnd.ms-excel"),
        "csv" => (".csv", "text/csv"),
        "ods" => (".ods", "application/vnd.oasis.opendocument.spreadsheet"),
        "pptx" => (
            ".pptx",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ),
        "ppt" => (".ppt", "application/vnd.ms-powerpoint"),
        "odp" => (".odp", "application/vnd.oasis.opendocument.presentation"),
        "zip" => (".zip", "application/zip"),
        "7z" => (".7z", "application/x-7z-compressed"),
        "tar" => (".tar", "application/x-tar"),
        "gz" => (".gz", "application/gzip"),
        "rar" => (".rar", "application/vnd.rar"),
        "exe" => (".exe", "application/vnd.microsoft.portable-executable"),
        "dll" => (".dll", "application/vnd.microsoft.portable-executable"),
        "sys" => (".sys", "application/x-sys"),
        "msi" => (".msi", "application/x-msi"),
        "bin" => (".bin", "application/octet-stream"),
        "db" | "sqlite" | "sqlite3" => (".sqlite", "application/vnd.sqlite3"),
        "mdb" | "accdb" => (".mdb", "application/x-msaccess"),
        "json" => (".json", "application/json"),
        "xml" => (".xml", "application/xml"),
        "yaml" | "yml" => (".yaml", "application/x-yaml"),
        "ini" => (".ini", "text/plain"),
        "log" => (".log", "text/plain"),
        "bak" | "old" => (".bak", "application/octet-stream"),
        "tmp" | "temp" => (".tmp", "application/octet-stream"),
        _ => identificar_sem_extensao(caminho),
    }
}

pub fn is_source_code_ext(ext: &str) -> bool {
    matches!(
        ext,
        "py" | "pyw"
            | "pyx"
            | "pxd"
            | "pxi"
            | "r"
            | "jl"
            | "ipynb"
            | "rmd"
            | "qmd"
            | "cpp"
            | "c"
            | "cxx"
            | "cc"
            | "c++"
            | "h"
            | "hpp"
            | "hxx"
            | "hh"
            | "h++"
            | "inl"
            | "ipp"
            | "tpp"
            | "ino"
            | "java"
            | "kt"
            | "kts"
            | "scala"
            | "groovy"
            | "gradle"
            | "clj"
            | "cljs"
            | "cljc"
            | "edn"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "ts"
            | "tsx"
            | "mts"
            | "cts"
            | "vue"
            | "svelte"
            | "astro"
            | "cs"
            | "csx"
            | "fs"
            | "fsx"
            | "fsi"
            | "vb"
            | "vbs"
            | "go"
            | "rs"
            | "swift"
            | "m"
            | "mm"
            | "zig"
            | "nim"
            | "v"
            | "d"
            | "odin"
            | "jai"
            | "php"
            | "phtml"
            | "rb"
            | "erb"
            | "rake"
            | "gemspec"
            | "pl"
            | "pm"
            | "t"
            | "pod"
            | "lua"
            | "tcl"
            | "awk"
            | "sed"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ksh"
            | "csh"
            | "tcsh"
            | "bat"
            | "cmd"
            | "ps1"
            | "psm1"
            | "psd1"
            | "dart"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "hs"
            | "lhs"
            | "ml"
            | "mli"
            | "ocaml"
            | "elm"
            | "purs"
            | "f"
            | "for"
            | "f90"
            | "f95"
            | "f03"
            | "f08"
            | "cob"
            | "cbl"
            | "ada"
            | "adb"
            | "ads"
            | "pas"
            | "pp"
            | "dpr"
            | "lpr"
            | "asm"
            | "s"
            | "nasm"
            | "masm"
            | "vh"
            | "sv"
            | "svh"
            | "vhdl"
            | "sql"
            | "pls"
            | "plsql"
            | "psql"
            | "tsql"
            | "hql"
            | "cql"
            | "html"
            | "htm"
            | "xhtml"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "styl"
            | "wat"
            | "proto"
            | "thrift"
            | "graphql"
            | "gql"
            | "sol"
            | "move"
            | "cairo"
            | "vy"
            | "cmake"
            | "make"
            | "mk"
            | "mak"
            | "meson"
            | "ninja"
            | "bazel"
            | "bzl"
            | "buck"
            | "gn"
            | "gni"
            | "qbs"
            | "pro"
            | "pri"
            | "glsl"
            | "vert"
            | "frag"
            | "geom"
            | "comp"
            | "tesc"
            | "tese"
            | "hlsl"
            | "fx"
            | "metal"
            | "wgsl"
            | "cu"
            | "cuh"
            | "cl"
            | "mat"
            | "mlx"
            | "nb"
            | "wl"
            | "wls"
            | "sas"
            | "do"
            | "ado"
            | "sps"
            | "ahk"
            | "au3"
            | "nsi"
            | "nsh"
            | "iss"
            | "wix"
            | "wxs"
            | "code"
    )
}

pub fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext,
        "jpg" | "jpeg"
            | "jpe"
            | "jif"
            | "jfif"
            | "jfi"
            | "png"
            | "apng"
            | "gif"
            | "bmp"
            | "dib"
            | "rle"
            | "tiff"
            | "tif"
            | "geotiff"
            | "webp"
            | "svg"
            | "svgz"
            | "ico"
            | "cur"
            | "ani"
            | "raw"
            | "arw"
            | "cr2"
            | "cr3"
            | "crw"
            | "nef"
            | "nrw"
            | "dng"
            | "orf"
            | "rw2"
            | "pef"
            | "ptx"
            | "srw"
            | "srf"
            | "sr2"
            | "raf"
            | "3fr"
            | "fff"
            | "iiq"
            | "kdc"
            | "dcr"
            | "mrw"
            | "erf"
            | "mef"
            | "mos"
            | "x3f"
            | "rwl"
            | "bay"
            | "psd"
            | "psb"
            | "pdd"
            | "ai"
            | "eps"
            | "epsf"
            | "epsi"
            | "cdr"
            | "cmx"
            | "indd"
            | "indt"
            | "idml"
            | "xd"
            | "fig"
            | "sketch"
            | "afdesign"
            | "afphoto"
            | "afpub"
            | "xcf"
            | "kra"
            | "ora"
            | "clip"
            | "lip"
            | "sai"
            | "sai2"
            | "mdp"
            | "pdn"
            | "procreate"
            | "heic"
            | "heif"
            | "heics"
            | "heifs"
            | "avif"
            | "avifs"
            | "jxl"
            | "jp2"
            | "j2k"
            | "jpf"
            | "jpx"
            | "jpm"
            | "mj2"
            | "wdp"
            | "jxr"
            | "hdp"
            | "bpg"
            | "flif"
            | "exr"
            | "hdr"
            | "rgbe"
            | "xyze"
            | "dds"
            | "ktx"
            | "ktx2"
            | "astc"
            | "pvr"
            | "vtf"
            | "tga"
            | "icb"
            | "vda"
            | "vst"
            | "pcx"
            | "dcx"
            | "pbm"
            | "pgm"
            | "ppm"
            | "pnm"
            | "pam"
            | "xbm"
            | "xpm"
            | "xwd"
            | "sgi"
            | "rgb"
            | "rgba"
            | "bw"
            | "int"
            | "inta"
            | "ras"
            | "sun"
            | "pct"
            | "pict"
            | "pic"
            | "mac"
            | "qti"
            | "qtif"
            | "icns"
            | "iff"
            | "ilbm"
            | "lbm"
            | "fpx"
            | "pcd"
            | "cin"
            | "dpx"
            | "fits"
            | "fit"
            | "fts"
            | "dcm"
            | "dicom"
            | "wbmp"
            | "wbm"
            | "emf"
            | "wmf"
            | "wmz"
            | "emz"
            | "cgm"
            | "odg"
            | "otg"
            | "sxd"
            | "std"
            | "vsd"
            | "vsdx"
            | "vdx"
            | "vsdm"
            | "vss"
            | "vssx"
            | "vstx"
            | "qoi"
    )
}

pub fn is_audio_ext(ext: &str) -> bool {
    matches!(
        ext,
        "mp3" | "mp2"
            | "mp1"
            | "mpa"
            | "wav"
            | "wave"
            | "bwf"
            | "rf64"
            | "flac"
            | "fla"
            | "ogg"
            | "oga"
            | "ogx"
            | "spx"
            | "opus"
            | "aac"
            | "m4a"
            | "m4b"
            | "m4p"
            | "m4r"
            | "alac"
            | "3ga"
            | "wma"
            | "aiff"
            | "aif"
            | "aifc"
            | "afc"
            | "ape"
            | "wv"
            | "wvc"
            | "tta"
            | "tak"
            | "ofr"
            | "ofs"
            | "shn"
            | "caf"
            | "dsf"
            | "dff"
            | "dsd"
            | "pcm"
            | "sam"
            | "au"
            | "snd"
            | "amr"
            | "awb"
            | "ac3"
            | "eac3"
            | "ec3"
            | "dts"
            | "dtshd"
            | "thd"
            | "truehd"
            | "mlp"
            | "mka"
            | "weba"
            | "ra"
            | "ram"
            | "mid"
            | "midi"
            | "kar"
            | "rmi"
            | "mod"
            | "s3m"
            | "xm"
            | "it"
            | "mptm"
            | "stm"
            | "669"
            | "far"
            | "mtm"
            | "ult"
            | "okt"
            | "med"
            | "nsf"
            | "spc"
            | "vgm"
            | "vgz"
            | "sid"
            | "gym"
            | "hes"
            | "ay"
            | "kss"
            | "gbs"
            | "psf"
            | "psf2"
            | "minipsf"
            | "usf"
            | "gsf"
            | "2sf"
            | "ncsf"
            | "snsf"
            | "adx"
            | "ahx"
            | "hvl"
            | "cda"
            | "m3u"
            | "m3u8"
            | "xspf"
            | "asx"
            | "wpl"
            | "cue"
            | "aup"
            | "aup3"
            | "sesx"
            | "als"
            | "flp"
            | "logicx"
            | "ptf"
            | "cpr"
            | "npr"
            | "rpp"
            | "reason"
            | "rns"
            | "mpc"
            | "mp+"
            | "mpp"
            | "voc"
            | "vox"
            | "gsm"
            | "dct"
            | "dss"
            | "msv"
            | "dvf"
            | "iklax"
            | "aa"
            | "aax"
    )
}

pub fn is_video_ext(ext: &str) -> bool {
    matches!(
        ext,
        "mp4" | "m4v"
            | "mp4v"
            | "mpg4"
            | "mkv"
            | "mk3d"
            | "mks"
            | "webm"
            | "avi"
            | "divx"
            | "xvid"
            | "mov"
            | "qt"
            | "wmv"
            | "asf"
            | "flv"
            | "f4v"
            | "f4p"
            | "f4a"
            | "f4b"
            | "swf"
            | "mpg"
            | "mpeg"
            | "mpe"
            | "m1v"
            | "m2v"
            | "mpv"
            | "mp2v"
            | "vob"
            | "ifo"
            | "bup"
            | "m2ts"
            | "m2t"
            | "tod"
            | "trp"
            | "tp"
            | "evo"
            | "ogv"
            | "ogm"
            | "3gp"
            | "3g2"
            | "3gp2"
            | "3gpp"
            | "3gpp2"
            | "rm"
            | "rmvb"
            | "rv"
            | "viv"
            | "vivo"
            | "dv"
            | "mxf"
            | "gxf"
            | "r3d"
            | "braw"
            | "ari"
            | "bik"
            | "bk2"
            | "smk"
            | "roq"
            | "uvf"
            | "mve"
            | "str"
            | "thp"
            | "usm"
            | "ivf"
            | "y4m"
            | "yuv"
            | "h264"
            | "264"
            | "avc"
            | "h265"
            | "265"
            | "hevc"
            | "av1"
            | "vp8"
            | "vp9"
            | "vc1"
            | "mjpeg"
            | "mjpg"
            | "wtv"
            | "dvr-ms"
            | "tivo"
            | "eyetv"
            | "nsv"
            | "amv"
            | "mtv"
            | "dmv"
            | "gifv"
            | "prproj"
            | "veg"
            | "vf"
            | "drp"
            | "fcpxml"
            | "fcp"
            | "fcpbundle"
            | "imovieproj"
            | "kdenlive"
            | "mlt"
            | "osp"
            | "wlmp"
            | "mswmm"
            | "camproj"
            | "tscproj"
            | "aep"
            | "aepx"
            | "aet"
            | "plproj"
            | "srt"
            | "sub"
            | "ass"
            | "ssa"
            | "vtt"
            | "sbv"
            | "smi"
            | "sami"
            | "lrc"
            | "stl"
            | "ttml"
            | "dfxp"
            | "scc"
    )
}

pub fn is_document_ext(ext: &str) -> bool {
    matches!(
        ext,
        "pdf" | "doc"
            | "dot"
            | "wbk"
            | "docx"
            | "docm"
            | "dotx"
            | "dotm"
            | "docb"
            | "txt"
            | "text"
            | "rtf"
            | "odt"
            | "ott"
            | "odm"
            | "oth"
            | "fodt"
            | "sxw"
            | "stw"
            | "sxg"
            | "pages"
            | "abw"
            | "zabw"
            | "awt"
            | "wpd"
            | "wp"
            | "wp4"
            | "wp5"
            | "wp6"
            | "wp7"
            | "wpt"
            | "wps"
            | "wri"
            | "lwp"
            | "mw"
            | "mcw"
            | "ami"
            | "xy"
            | "cwk"
            | "hwp"
            | "hwpx"
            | "jtd"
            | "jtt"
            | "ich"
            | "oas"
            | "oa2"
            | "oa3"
            | "tex"
            | "ltx"
            | "sty"
            | "cls"
            | "bib"
            | "bst"
            | "dtx"
            | "ins"
            | "lyx"
            | "md"
            | "markdown"
            | "mdown"
            | "mkdn"
            | "mkd"
            | "mdwn"
            | "mdtxt"
            | "mdtext"
            | "rst"
            | "rest"
            | "adoc"
            | "asciidoc"
            | "asc"
            | "org"
            | "wiki"
            | "mediawiki"
            | "creole"
            | "textile"
            | "man"
            | "1"
            | "2"
            | "3"
            | "4"
            | "5"
            | "6"
            | "7"
            | "8"
            | "me"
            | "ms"
            | "troff"
            | "groff"
            | "nroff"
            | "texi"
            | "texinfo"
            | "info"
            | "epub"
            | "epub3"
            | "mobi"
            | "prc"
            | "azw"
            | "azw1"
            | "azw3"
            | "azw4"
            | "kf8"
            | "kfx"
            | "tpz"
            | "fb2"
            | "fb2.zip"
            | "fbz"
            | "lit"
            | "lrf"
            | "lrx"
            | "pml"
            | "snb"
            | "tcr"
            | "tr2"
            | "tr3"
            | "cbz"
            | "cbr"
            | "cb7"
            | "cbt"
            | "cba"
            | "djvu"
            | "djv"
            | "xps"
            | "oxps"
            | "ps"
            | "chm"
            | "hlp"
            | "cnt"
            | "ibooks"
            | "opf"
            | "ncx"
            | "nfo"
            | "diz"
            | "document"
    )
}

pub fn is_spreadsheet_ext(ext: &str) -> bool {
    matches!(
        ext,
        "xls" | "xlt"
            | "xlm"
            | "xlw"
            | "xlsx"
            | "xlsm"
            | "xltx"
            | "xltm"
            | "xlsb"
            | "xlam"
            | "xla"
            | "xll"
            | "ods"
            | "ots"
            | "fods"
            | "sxc"
            | "stc"
            | "csv"
            | "tsv"
            | "tab"
            | "psv"
            | "dsv"
            | "ssv"
            | "bcsv"
            | "numbers"
            | "nmbtemplate"
            | "gnumeric"
            | "gnm"
            | "wks"
            | "wk1"
            | "wk2"
            | "wk3"
            | "wk4"
            | "123"
            | "wg1"
            | "wg2"
            | "fmt"
            | "fm3"
            | "all"
            | "wb1"
            | "wb2"
            | "wb3"
            | "qpw"
            | "wq1"
            | "wq2"
            | "slk"
            | "sylk"
            | "dif"
            | "prn"
            | "pxl"
            | "sdc"
            | "vor"
            | "aws"
            | "clf"
            | "ess"
            | "fcs"
            | "gsheet"
            | "imp"
            | "ncss"
            | "parquet"
            | "feather"
            | "arrow"
            | "ipc"
            | "orc"
    )
}

pub fn is_presentation_ext(ext: &str) -> bool {
    matches!(
        ext,
        "ppt" | "pot"
            | "pps"
            | "ppa"
            | "pptx"
            | "pptm"
            | "potx"
            | "potm"
            | "ppam"
            | "ppsx"
            | "ppsm"
            | "sldx"
            | "sldm"
            | "odp"
            | "otp"
            | "fodp"
            | "sxi"
            | "sti"
            | "key"
            | "keynote"
            | "kth"
            | "shw"
            | "sdd"
            | "sdp"
            | "uop"
            | "prz"
            | "pre"
            | "show"
            | "gslides"
    )
}

pub fn is_database_ext(ext: &str) -> bool {
    matches!(
        ext,
        "db" | "sqlite"
            | "sqlite3"
            | "db3"
            | "s3db"
            | "sl3"
            | "mdb"
            | "accdb"
            | "accde"
            | "accdt"
            | "accdr"
            | "mde"
            | "ade"
            | "adp"
            | "mdw"
            | "mdf"
            | "ldf"
            | "ndf"
            | "sdf"
            | "trn"
            | "dbf"
            | "dbt"
            | "fpt"
            | "cdx"
            | "idx"
            | "ndx"
            | "mdx"
            | "ntx"
            | "dbc"
            | "fdb"
            | "gdb"
            | "ib"
            | "ntf"
            | "ns2"
            | "ns3"
            | "ns4"
            | "fp3"
            | "fp4"
            | "fp5"
            | "fp7"
            | "fmp12"
            | "fmpsl"
            | "px"
            | "mb"
            | "val"
            | "xg0"
            | "yg0"
            | "wdb"
            | "nyf"
            | "myd"
            | "myi"
            | "frm"
            | "ibd"
            | "isl"
            | "csm"
            | "arz"
            | "arm"
            | "rrdi"
            | "ctl"
            | "rdo"
            | "dpdmp"
            | "rdb"
            | "rds"
            | "rda"
            | "rdata"
            | "avro"
            | "h5"
            | "hdf5"
            | "hdf"
            | "he5"
            | "nc"
            | "nc4"
            | "cdf"
            | "netcdf"
            | "zarr"
            | "lmdb"
            | "ldb"
            | "sst"
            | "leveldb"
            | "rocksdb"
            | "bolt"
            | "bbolt"
            | "gdbm"
            | "ndbm"
            | "dbm"
            | "sdbm"
            | "cdb"
            | "tdb"
            | "qdbm"
            | "kyoto"
            | "kch"
            | "kct"
            | "realm"
            | "isar"
            | "hive"
            | "objectbox"
            | "rrd"
            | "wsp"
            | "uca"
            | "dbs"
            | "udl"
            | "dsn"
            | "odc"
            | "dqy"
            | "oqy"
            | "rqy"
    )
}

pub fn is_gis_ext(ext: &str) -> bool {
    matches!(
        ext,
        "shp" | "shx"
            | "prj"
            | "sbn"
            | "sbx"
            | "fbn"
            | "fbx"
            | "ain"
            | "aih"
            | "ixs"
            | "mxs"
            | "atx"
            | "cpg"
            | "qix"
            | "kml"
            | "kmz"
            | "gpx"
            | "geojson"
            | "topojson"
            | "gml"
            | "gpkg"
            | "spatialite"
            | "osm"
            | "pbf"
            | "mbtiles"
            | "pmtiles"
            | "ecw"
            | "dem"
            | "dt0"
            | "dt1"
            | "dt2"
            | "hgt"
            | "las"
            | "laz"
            | "e57"
            | "ply"
            | "xyz"
            | "pts"
            | "mif"
            | "wor"
            | "gst"
            | "mxd"
            | "mxt"
            | "mpk"
            | "aprx"
            | "ppkx"
            | "pagx"
            | "lyrx"
            | "lyr"
            | "stylx"
            | "sde"
            | "qgs"
            | "qgz"
            | "qlr"
            | "qml"
            | "qpj"
            | "vct"
            | "vdc"
            | "rdc"
            | "smp"
            | "avl"
            | "apr"
            | "odb"
            | "e00"
            | "grib"
            | "grb"
            | "grb2"
            | "bufr"
            | "bsq"
            | "bil"
            | "bip"
            | "stx"
            | "clr"
            | "ers"
            | "alg"
            | "lan"
            | "gis"
            | "000"
            | "s57"
            | "kap"
            | "bsb"
            | "enos"
            | "dgn"
            | "dwg"
            | "dxf"
    )
}

pub fn is_executable_ext(ext: &str) -> bool {
    matches!(
        ext,
        "exe" | "dll"
            | "so"
            | "dylib"
            | "bin"
            | "msi"
            | "msp"
            | "mst"
            | "msu"
            | "appx"
            | "msix"
            | "appxbundle"
            | "msixbundle"
            | "appinstaller"
            | "apk"
            | "aab"
            | "dex"
            | "odex"
            | "vdex"
            | "art"
            | "ipa"
            | "deb"
            | "udeb"
            | "rpm"
            | "pkg"
            | "mpkg"
            | "dmg"
            | "app"
            | "appimage"
            | "flatpak"
            | "snap"
            | "run"
            | "bundle"
            | "out"
            | "elf"
            | "axf"
            | "prx"
            | "puff"
            | "ko"
            | "o"
            | "obj"
            | "a"
            | "lib"
            | "exp"
            | "pdb"
            | "ilk"
            | "sym"
            | "dbg"
            | "dsym"
            | "com"
            | "scr"
            | "cpl"
            | "ocx"
            | "ax"
            | "ime"
            | "drv"
            | "sys"
            | "vxd"
            | "386"
            | "fon"
            | "jar"
            | "war"
            | "ear"
            | "sar"
            | "par"
            | "class"
            | "jmod"
            | "jad"
            | "gadget"
            | "wsf"
            | "wsh"
            | "hta"
            | "vbe"
            | "jse"
            | "pif"
            | "pyc"
            | "pyo"
            | "pyd"
            | "whl"
            | "egg"
            | "phar"
            | "rbc"
            | "beam"
            | "luac"
            | "wasm"
            | "nexe"
            | "pexe"
            | "xex"
            | "xbe"
            | "self"
            | "fself"
            | "dol"
            | "rel"
            | "rso"
            | "sel"
            | "nro"
            | "nso"
            | "nca"
            | "nsp"
            | "xci"
            | "nds"
            | "3ds"
            | "cia"
            | "cxi"
            | "cfa"
            | "vpk"
            | "pbp"
            | "cso"
            | "wad"
            | "wbfs"
            | "rvz"
            | "gcz"
            | "xap"
            | "air"
            | "crx"
            | "xpi"
            | "vsix"
            | "nupkg"
            | "snupkg"
            | "gem"
            | "paf.exe"
    )
}

pub fn is_font_ext(ext: &str) -> bool {
    matches!(
        ext,
        "ttf" | "otf"
            | "woff"
            | "woff2"
            | "eot"
            | "fon"
            | "fnt"
            | "pfb"
            | "pfa"
            | "pfm"
            | "afm"
            | "ttc"
            | "otc"
            | "dfont"
            | "suit"
            | "bdf"
            | "pcf"
            | "psfu"
            | "snf"
            | "sfd"
            | "ufo"
            | "glyphs"
            | "vfb"
            | "vfj"
            | "txf"
            | "bmf"
    )
}

pub fn is_system_ext(ext: &str) -> bool {
    matches!(
        ext,
        "sys" | "drv"
            | "vxd"
            | "386"
            | "ko"
            | "kext"
            | "cat"
            | "pnf"
            | "hiv"
            | "pol"
            | "adm"
            | "admx"
            | "adml"
            | "cpl"
            | "msc"
            | "mmc"
            | "ocx"
            | "ax"
            | "tlb"
            | "olb"
            | "mui"
            | "mun"
            | "nls"
            | "icm"
            | "icc"
            | "dmp"
            | "hdmp"
            | "mdmp"
            | "evtx"
            | "evt"
            | "etl"
            | "blf"
            | "regtrans-ms"
            | "pf"
            | "db-shm"
            | "db-wal"
            | "db-journal"
            | "manifest"
            | "mum"
            | "cdf-ms"
            | "p7x"
            | "efi"
            | "rom"
            | "bios"
            | "fd"
            | "fv"
            | "desklink"
            | "mapimail"
            | "mydocs"
            | "zfsendtotarget"
            | "search-ms"
            | "library-ms"
            | "settingcontent-ms"
            | "appref-ms"
            | "scf"
            | "grp"
            | "pif"
            | "prf"
            | "uce"
            | "ime"
            | "lex"
            | "dic"
            | "aff"
            | "spl"
            | "shd"
            | "bud"
            | "gpd"
            | "ppd"
            | "inf_loc"
    )
}

pub fn is_compressed_ext(ext: &str) -> bool {
    matches!(
        ext,
        "zip" | "zipx"
            | "rar"
            | "7z"
            | "tar"
            | "gz"
            | "tgz"
            | "tar.gz"
            | "taz"
            | "bz2"
            | "tbz"
            | "tbz2"
            | "tar.bz2"
            | "tz2"
            | "xz"
            | "txz"
            | "tar.xz"
            | "lz"
            | "tlz"
            | "tar.lz"
            | "lzma"
            | "tlzma"
            | "tar.lzma"
            | "lzo"
            | "tzo"
            | "zst"
            | "tzst"
            | "tar.zst"
            | "z"
            | "tar.z"
            | "lz4"
            | "br"
            | "sz"
            | "cab"
            | "iso"
            | "udf"
            | "img"
            | "dmg"
            | "sparseimage"
            | "sparsebundle"
            | "toast"
            | "daa"
            | "uif"
            | "nrg"
            | "mds"
            | "ccd"
            | "ecm"
            | "b5t"
            | "b6t"
            | "bwt"
            | "cdi"
            | "pdi"
            | "isz"
            | "cso"
            | "dax"
            | "jso"
            | "wbi"
            | "wim"
            | "swm"
            | "esd"
            | "vhd"
            | "vhdx"
            | "vmdk"
            | "vdi"
            | "qcow"
            | "qcow2"
            | "qed"
            | "hdd"
            | "hds"
            | "ova"
            | "ovf"
            | "box"
            | "ace"
            | "arj"
            | "arc"
            | "ark"
            | "lzh"
            | "lha"
            | "zoo"
            | "pak"
            | "pk3"
            | "pk4"
            | "vpk"
            | "gcf"
            | "ncf"
            | "wad"
            | "bsp"
            | "mpq"
            | "bsa"
            | "ba2"
            | "big"
            | "erf"
            | "rim"
            | "hak"
            | "jar"
            | "war"
            | "ear"
            | "sar"
            | "apk"
            | "aab"
            | "ipa"
            | "xapk"
            | "apks"
            | "apkm"
            | "obb"
            | "deb"
            | "udeb"
            | "rpm"
            | "srpm"
            | "pkg"
            | "mpkg"
            | "msi"
            | "msp"
            | "msu"
            | "appx"
            | "msix"
            | "cpio"
            | "shar"
            | "ar"
            | "mar"
            | "sfs"
            | "squashfs"
            | "cramfs"
            | "romfs"
            | "ext2"
            | "ext3"
            | "ext4"
            | "xfs"
            | "btrfs"
            | "ntfs"
            | "fat"
            | "exfat"
            | "hfs"
            | "apfs"
            | "sit"
            | "sitx"
            | "sea"
            | "hqx"
            | "uu"
            | "uue"
            | "xxe"
            | "b64"
            | "mim"
            | "mme"
            | "r00"
            | "r01"
            | "z01"
            | "001"
            | "rev"
            | "par2"
            | "pb.gz"
            | "sql.gz"
            | "sql.bz2"
            | "sql.xz"
            | "fb2.zip"
    )
}

pub fn is_virtual_disk_ext(ext: &str) -> bool {
    matches!(
        ext,
        "vhd" | "vhdx"
            | "avhd"
            | "avhdx"
            | "vmdk"
            | "vdi"
            | "hdd"
            | "hds"
            | "qcow"
            | "qcow2"
            | "qed"
            | "ova"
            | "ovf"
            | "wim"
            | "swm"
            | "esd"
            | "iso"
            | "udf"
            | "img"
            | "dmg"
            | "sparseimage"
            | "sparsebundle"
            | "toast"
            | "nrg"
            | "mds"
            | "ccd"
            | "isz"
            | "daa"
            | "uif"
            | "cso"
            | "wbfs"
            | "rvz"
            | "gcz"
            | "chd"
    )
}

pub fn is_backup_ext(ext: &str) -> bool {
    matches!(
        ext,
        "bak" | "bkp"
            | "backup"
            | "old"
            | "orig"
            | "original"
            | "save"
            | "saved"
            | "~"
            | "swp"
            | "swo"
            | "swn"
            | "prv"
            | "prev"
            | "vbox-prev"
            | "abk"
            | "ac$"
            | "sv$"
            | "wbk"
            | "xlk"
            | "asd"
            | "gho"
            | "ghs"
            | "tib"
            | "tibx"
            | "v2i"
            | "iv2i"
            | "pqi"
            | "fbf"
            | "fkc"
            | "bkf"
            | "adi"
            | "mrimg"
            | "mrbak"
            | "sn1"
            | "sn2"
            | "sna"
            | "rbf"
            | "rbs"
            | "nbi"
            | "nba"
            | "nbd"
            | "nbf"
            | "nbu"
            | "qic"
            | "sqb"
            | "dmp"
            | "dump"
            | "sql.gz"
            | "sql.bz2"
            | "sql.xz"
            | "tar.gz"
            | "tar.bz2"
            | "tar.xz"
            | "tar.zst"
            | "tgz"
            | "tbz2"
            | "txz"
            | "psc"
            | "nbk"
            | "fbk"
            | "gbk"
            | "mbk"
            | "sbk"
            | "tbk"
            | "vbk"
            | "vrb"
            | "vib"
            | "vbm"
            | "bck"
            | "bup"
            | "001"
            | "002"
            | "~1"
            | "~2"
    )
}

pub fn is_log_ext(ext: &str) -> bool {
    matches!(
        ext,
        "log" | "logs"
            | "err"
            | "stdout"
            | "stderr"
            | "trace"
            | "trc"
            | "xel"
            | "wer"
            | "syslog"
            | "messages"
            | "audit"
            | "journal"
            | "asl"
            | "crash"
            | "ips"
            | "diag"
            | "spin"
            | "hang"
            | "panic"
            | "anr"
            | "tombstone"
            | "bugreport"
            | "lo_"
            | "har"
            | "saz"
            | "pcap"
            | "pcapng"
            | "cap"
            | "nettrace"
            | "npcap"
            | "snoop"
            | "bfr"
            | "pkt"
            | "nmon"
            | "blg"
            | "perf"
            | "callgrind"
            | "cachegrind"
            | "massif"
            | "heap"
            | "hprof"
            | "jfr"
            | "gc"
            | "ulog"
            | "plog"
            | "ilog"
            | "clog"
            | "alog"
            | "wlog"
            | "dlog"
            | "slog"
            | "rlog"
    )
}

pub fn is_config_ext(ext: &str) -> bool {
    matches!(
        ext,
        "ini" | "inf"
            | "cfg"
            | "conf"
            | "config"
            | "cf"
            | "cnf"
            | "properties"
            | "prop"
            | "prefs"
            | "preference"
            | "settings"
            | "set"
            | "opt"
            | "options"
            | "env"
            | "env.local"
            | "env.development"
            | "env.production"
            | "env.test"
            | "env.example"
            | "json"
            | "jsonc"
            | "json5"
            | "jsonl"
            | "ndjson"
            | "hjson"
            | "yaml"
            | "yml"
            | "toml"
            | "tml"
            | "xml"
            | "xsd"
            | "dtd"
            | "xsl"
            | "xslt"
            | "plist"
            | "reg"
            | "rc"
            | "bashrc"
            | "bash_profile"
            | "profile"
            | "zshrc"
            | "zprofile"
            | "vimrc"
            | "gvimrc"
            | "nanorc"
            | "inputrc"
            | "gitconfig"
            | "gitignore"
            | "gitattributes"
            | "gitmodules"
            | "editorconfig"
            | "prettierrc"
            | "eslintrc"
            | "babelrc"
            | "npmrc"
            | "yarnrc"
            | "nvmrc"
            | "dockerignore"
            | "flake8"
            | "pylintrc"
            | "condarc"
            | "service"
            | "socket"
            | "timer"
            | "mount"
            | "automount"
            | "swap"
            | "target"
            | "path"
            | "slice"
            | "scope"
            | "device"
            | "desktop"
            | "directory"
            | "menu"
            | "kbd"
            | "kmap"
            | "rules"
            | "policy"
            | "repo"
            | "list"
            | "sources"
            | "fstab"
            | "crypttab"
            | "mtab"
            | "hosts"
            | "resolv"
            | "nsswitch"
            | "sudoers"
            | "crontab"
            | "logrotate"
            | "sysctl"
            | "modprobe"
            | "limits"
            | "exports"
            | "smb"
            | "httpd"
            | "nginx"
            | "apache"
            | "htaccess"
            | "htpasswd"
            | "robots"
            | "sitemap"
            | "webmanifest"
            | "webapp"
            | "appcache"
            | "lock"
            | "lockb"
            | "dhall"
            | "hcl"
            | "tf"
            | "tfvars"
            | "tfstate"
            | "nomad"
            | "consul"
            | "vault"
            | "packer"
            | "vagrant"
            | "k8s"
            | "helm"
            | "chart"
            | "values"
            | "kustomization"
            | "tiltfile"
            | "brewfile"
            | "podfile"
            | "cartfile"
            | "gemfile"
            | "pipfile"
            | "poetry"
            | "cargo"
            | "csproj"
            | "vcxproj"
            | "sln"
            | "props"
            | "targets"
            | "qrc"
            | "ui"
    )
}

pub fn is_temp_ext(ext: &str) -> bool {
    matches!(
        ext,
        "tmp" | "temp"
            | "~"
            | "swp"
            | "swo"
            | "swn"
            | "part"
            | "partial"
            | "crdownload"
            | "download"
            | "opdownload"
            | "!ut"
            | "bc!"
            | "!qb"
            | "dlm"
            | "dtapart"
            | "filepart"
            | "$$$"
            | "$a"
            | "~$"
            | "lck"
            | "laccdb"
            | "shm"
            | "wal"
            | "db-shm"
            | "db-wal"
            | "db-journal"
            | "pid"
            | "gid"
            | "syd"
            | "chk"
            | "ftg"
            | "msf"
            | "cache"
            | "caches"
            | "chc"
    )
}
