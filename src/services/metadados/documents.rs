use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    pub autor: String,
    pub protegido: bool,
    pub paginas: String,
    pub paginas_estimadas: String,
    pub linhas: String,
    pub palavras: String,
    pub linhas_codigo: String,
    pub total_linhas: String,
    pub slides: String,
    pub slides_estimados: String,
    pub planilhas: String,
    pub colunas: String,
    pub registros: String,
    pub tabelas: String,
}

/// Extrai metadados de arquivos de código-fonte (paridade com gmet_02_ExtrairMetadadosCodigoFonte.cpp).
pub fn extrair_codigo_fonte(caminho: &Path, file_size: u64) -> Option<DocumentMetadata> {
    let text = ler_texto_flexivel(caminho, file_size.min(8 * 1024 * 1024) as usize)?;
    let mut total_lines = 0usize;
    let mut code_lines = 0usize;
    let mut total_words = 0usize;
    let mut in_block_comment = false;

    for line in text.lines() {
        total_lines += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        total_words += trimmed.split_whitespace().count();

        if in_block_comment {
            if let Some(pos) = trimmed.find("*/") {
                in_block_comment = false;
                let rest = trimmed[pos + 2..].trim();
                if !rest.is_empty() && !rest.starts_with("//") {
                    code_lines += 1;
                }
            } else if trimmed.contains("-->") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                in_block_comment = true;
            }
            continue;
        }
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("--")
            || trimmed.starts_with(';')
            || trimmed.starts_with('%')
            || trimmed.eq_ignore_ascii_case("rem")
            || trimmed.to_lowercase().starts_with("rem ")
        {
            continue;
        }

        code_lines += 1;
    }

    if total_lines == 0 && file_size > 0 {
        total_lines = 1;
        code_lines = 1;
    }

    let pages = ((total_lines as f64) / 50.0).ceil() as usize;
    let pages_str = pages.max(1).to_string();

    Some(DocumentMetadata {
        paginas: pages_str.clone(),
        paginas_estimadas: pages_str,
        linhas: total_lines.to_string(),
        total_linhas: total_lines.to_string(),
        linhas_codigo: code_lines.to_string(),
        palavras: total_words.to_string(),
        ..Default::default()
    })
}

/// Compatibilidade com chamadas legadas de `extrair_documento_texto`.
pub fn extrair_documento_texto(caminho: &Path, is_code: bool) -> Option<DocumentMetadata> {
    let sz = std::fs::metadata(caminho).map(|m| m.len()).unwrap_or(0);
    if is_code {
        extrair_codigo_fonte(caminho, sz)
    } else {
        extrair_documento(caminho, "txt", sz)
    }
}

/// Extrai metadados de documentos (PDF, DOCX, ODT, RTF, EPUB, DOC, TXT, MD, etc.)
/// (paridade com gmet_06_ExtrairMetadadosDocumento.cpp e gmet_16_ExtrairMetadadosOle.cpp).
pub fn extrair_documento(caminho: &Path, ext: &str, file_size: u64) -> Option<DocumentMetadata> {
    match ext {
        "pdf" => extrair_pdf(caminho, file_size),
        "docx" | "docm" | "dotx" | "dotm" | "odt" | "ott" | "odm" | "oth" | "epub" | "epub3"
        | "xps" | "oxps" | "pages" | "abw" | "zabw" | "fb2.zip" | "fbz" | "cbz" | "hwpx"
        | "ibooks" => extrair_documento_zip(caminho, ext, file_size),
        "rtf" => extrair_rtf(caminho, file_size),
        "doc" | "dot" | "wbk" | "wps" | "wpd" | "wri" | "lwp" | "chm" | "hlp" | "djvu" | "djv"
        | "mobi" | "prc" | "azw" | "azw3" | "kf8" | "kfx" | "lit" => {
            extrair_documento_binario(caminho, file_size)
        }
        _ => {
            if let Some(text) = ler_texto_flexivel(caminho, file_size.min(8 * 1024 * 1024) as usize)
            {
                let mut lines = 0usize;
                let mut words = 0usize;
                for line in text.lines() {
                    lines += 1;
                    words += line.split_whitespace().count();
                }
                if lines == 0 && file_size > 0 {
                    lines = 1;
                }
                let pages = ((lines as f64) / 45.0).ceil() as usize;
                let pages_str = pages.max(1).to_string();
                Some(DocumentMetadata {
                    paginas: pages_str.clone(),
                    paginas_estimadas: pages_str,
                    linhas: lines.to_string(),
                    total_linhas: lines.to_string(),
                    palavras: words.to_string(),
                    ..Default::default()
                })
            } else {
                extrair_documento_binario(caminho, file_size)
            }
        }
    }
}

fn extrair_pdf(caminho: &Path, file_size: u64) -> Option<DocumentMetadata> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let read_len = file_size.min(4 * 1024 * 1024) as usize;
    let mut buf = vec![0u8; read_len];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);

    // Se o arquivo for maior que 4 MB, também lê os últimos 256 KB (onde ficam xref/trailer/Pages)
    if file_size > (read_len as u64) {
        let tail_len = 262144.min((file_size - read_len as u64) as usize);
        if file
            .seek(SeekFrom::End(-(tail_len as i64)))
            .is_ok()
        {
            let mut tail = vec![0u8; tail_len];
            if let Ok(tn) = file.read(&mut tail) {
                buf.extend_from_slice(&tail[..tn]);
            }
        }
    }

    let text = String::from_utf8_lossy(&buf);
    let protegido = text.contains("/Encrypt");

    // Conta objetos `/Type /Page` (sem contar `/Type /Pages`)
    let mut page_obj_count = 0usize;
    let mut search_pos = 0usize;
    while let Some(idx) = text[search_pos..].find("/Type") {
        let abs = search_pos + idx + 5;
        let rest = text[abs..].trim_start();
        if let Some(after_page) = rest.strip_prefix("/Page") {
            let next_ch = after_page.chars().next().unwrap_or(' ');
            if !next_ch.is_ascii_alphabetic() {
                page_obj_count += 1;
            }
        }
        search_pos = abs;
    }

    // Também verifica `/Count <N>` dentro de dicionários `/Pages`
    let mut max_count_attr = 0usize;
    let mut cpos = 0usize;
    while let Some(idx) = text[cpos..].find("/Count") {
        let abs = cpos + idx + 6;
        let rest = text[abs..].trim_start();
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(v) = num_str.parse::<usize>() {
            if v > max_count_attr && v < 100_000 {
                max_count_attr = v;
            }
        }
        cpos = abs;
    }

    let pages = if page_obj_count > 0 {
        page_obj_count.max(max_count_attr)
    } else if max_count_attr > 0 {
        max_count_attr
    } else {
        ((file_size as f64) / 30_000.0).ceil() as usize
    }
    .max(1);

    // Extrai /Author (...) se disponível
    let mut autor = String::new();
    if let Some(aidx) = text.find("/Author") {
        let rest = text[aidx + 7..].trim_start();
        if let Some(inner) = rest.strip_prefix('(') {
            if let Some(end) = inner.find(')') {
                let candidate = inner[..end].trim();
                if !candidate.is_empty() && candidate.chars().all(|c| !c.is_control()) {
                    autor = candidate.to_string();
                }
            }
        }
    }

    let est_words = pages * 280;
    let est_lines = pages * 42;

    Some(DocumentMetadata {
        autor,
        protegido,
        paginas: pages.to_string(),
        paginas_estimadas: pages.to_string(),
        linhas: est_lines.to_string(),
        total_linhas: est_lines.to_string(),
        palavras: est_words.to_string(),
        ..Default::default()
    })
}

fn extrair_documento_zip(
    caminho: &Path,
    ext: &str,
    file_size: u64,
) -> Option<DocumentMetadata> {
    let file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut zip = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return extrair_documento_binario(caminho, file_size),
    };

    let mut autor = String::new();
    let mut pages = 0usize;
    let mut words = 0usize;
    let mut lines = 0usize;

    // 1. docProps/core.xml (OOXML)
    if let Some(xml) = read_zip_entry_string(&mut zip, "docProps/core.xml") {
        if let Some(c) = extract_xml_tag_value(&xml, "dc:creator")
            .or_else(|| extract_xml_tag_value(&xml, "cp:lastModifiedBy"))
        {
            autor = c;
        }
    }

    // 2. docProps/app.xml (OOXML)
    if let Some(xml) = read_zip_entry_string(&mut zip, "docProps/app.xml") {
        if let Some(p) = extract_xml_tag_value(&xml, "Pages").and_then(|s| s.parse::<usize>().ok())
        {
            pages = p;
        }
        if let Some(w) = extract_xml_tag_value(&xml, "Words").and_then(|s| s.parse::<usize>().ok())
        {
            words = w;
        }
        if let Some(l) = extract_xml_tag_value(&xml, "Lines").and_then(|s| s.parse::<usize>().ok())
        {
            lines = l;
        }
    }

    // 3. word/document.xml (DOCX)
    if (words == 0 || pages == 0 || lines == 0) && zip.by_name("word/document.xml").is_ok() {
        if let Some(xml) = read_zip_entry_string(&mut zip, "word/document.xml") {
            let plain = strip_xml_tags(&xml);
            let w_count = plain.split_whitespace().count();
            let p_breaks = xml.matches("<w:lastRenderedPageBreak").count()
                + xml.matches("w:type=\"page\"").count();
            let para_count = xml.matches("<w:p ").count() + xml.matches("<w:p>").count();
            if words == 0 {
                words = w_count;
            }
            if lines == 0 {
                lines = para_count.max((words / 10).max(1));
            }
            if pages == 0 {
                pages = (p_breaks + 1).max(((words as f64) / 300.0).ceil() as usize);
            }
        }
    }

    // 4. meta.xml / content.xml (ODF: .odt, .ott)
    if let Some(xml) = read_zip_entry_string(&mut zip, "meta.xml") {
        if autor.is_empty() {
            if let Some(c) = extract_xml_tag_value(&xml, "dc:creator")
                .or_else(|| extract_xml_tag_value(&xml, "meta:initial-creator"))
            {
                autor = c;
            }
        }
        if let Some(p) = extract_xml_attr(&xml, "meta:page-count").and_then(|s| s.parse().ok()) {
            pages = p;
        }
        if let Some(w) = extract_xml_attr(&xml, "meta:word-count").and_then(|s| s.parse().ok()) {
            words = w;
        }
        if let Some(l) =
            extract_xml_attr(&xml, "meta:paragraph-count").and_then(|s| s.parse().ok())
        {
            lines = l;
        }
    }

    if (words == 0 || lines == 0) && zip.by_name("content.xml").is_ok() {
        if let Some(xml) = read_zip_entry_string(&mut zip, "content.xml") {
            let plain = strip_xml_tags(&xml);
            if words == 0 {
                words = plain.split_whitespace().count();
            }
            if lines == 0 {
                lines = (xml.matches("<text:p").count()).max((words / 10).max(1));
            }
            if pages == 0 {
                pages = ((words as f64) / 300.0).ceil() as usize;
            }
        }
    }

    // 5. EPUB / XPS / CBZ / outros containers ZIP
    if pages == 0 {
        let mut html_chapters = 0usize;
        let mut xps_pages = 0usize;
        let mut image_pages = 0usize;
        for i in 0..zip.len() {
            if let Ok(entry) = zip.by_index(i) {
                let name = entry.name().unwrap_or_default().to_lowercase();
                if name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm") {
                    html_chapters += 1;
                } else if name.ends_with(".fpage") {
                    xps_pages += 1;
                } else if name.ends_with(".jpg")
                    || name.ends_with(".jpeg")
                    || name.ends_with(".png")
                    || name.ends_with(".webp")
                {
                    image_pages += 1;
                }
            }
        }
        if xps_pages > 0 {
            pages = xps_pages;
        } else if html_chapters > 0 {
            pages = (html_chapters * 4).max(1);
        } else if ext == "cbz" && image_pages > 0 {
            pages = image_pages;
        }
    }

    if pages == 0 {
        pages = ((file_size as f64) / 16_384.0).ceil() as usize;
    }
    pages = pages.max(1);
    if words == 0 {
        words = pages * 250;
    }
    if lines == 0 {
        lines = (words / 10).max(pages * 30);
    }

    let pages_str = pages.to_string();
    Some(DocumentMetadata {
        autor,
        protegido: false,
        paginas: pages_str.clone(),
        paginas_estimadas: pages_str,
        linhas: lines.to_string(),
        total_linhas: lines.to_string(),
        palavras: words.to_string(),
        ..Default::default()
    })
}

fn extrair_rtf(caminho: &Path, file_size: u64) -> Option<DocumentMetadata> {
    let raw = ler_texto_flexivel(caminho, file_size.min(4 * 1024 * 1024) as usize)?;
    let mut autor = String::new();
    if let Some(aidx) = raw.find("\\author") {
        let rest = &raw[aidx + 7..];
        if let Some(end) = rest.find('}') {
            let cand = rest[..end].trim();
            if !cand.is_empty() {
                autor = cand.to_string();
            }
        }
    }

    let para_breaks = raw.matches("\\par").count() + raw.matches("\\line").count();
    let page_breaks = raw.matches("\\page").count();

    // Remove comandos RTF para contar palavras reais
    let mut clean = String::with_capacity(raw.len() / 2);
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_alphanumeric() || nc == '-' {
                    chars.next();
                } else {
                    if nc == ' ' {
                        chars.next();
                    }
                    break;
                }
            }
            clean.push(' ');
        } else if c != '{' && c != '}' {
            clean.push(c);
        }
    }

    let words = clean.split_whitespace().count();
    let lines = para_breaks.max(clean.lines().count()).max(1);
    let pages = (page_breaks + 1).max(((lines as f64) / 45.0).ceil() as usize);
    let pages_str = pages.max(1).to_string();

    Some(DocumentMetadata {
        autor,
        protegido: false,
        paginas: pages_str.clone(),
        paginas_estimadas: pages_str,
        linhas: lines.to_string(),
        total_linhas: lines.to_string(),
        palavras: words.to_string(),
        ..Default::default()
    })
}

fn extrair_documento_binario(caminho: &Path, file_size: u64) -> Option<DocumentMetadata> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut buf = vec![0u8; file_size.min(524288) as usize];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);

    // Extrai runs de texto legíveis para contar palavras reais
    let mut words = 0usize;
    let mut cur_word_len = 0usize;
    for &b in &buf {
        if b.is_ascii_alphabetic() {
            cur_word_len += 1;
        } else {
            if cur_word_len >= 3 {
                words += 1;
            }
            cur_word_len = 0;
        }
    }
    if words == 0 {
        words = ((file_size as f64) / 18.0).ceil() as usize;
    }
    let lines = (words / 10).max(1);
    let pages = ((lines as f64) / 45.0).ceil() as usize;
    let pages_str = pages.max(1).to_string();

    Some(DocumentMetadata {
        paginas: pages_str.clone(),
        paginas_estimadas: pages_str,
        linhas: lines.to_string(),
        total_linhas: lines.to_string(),
        palavras: words.to_string(),
        ..Default::default()
    })
}

/// Extrai metadados de planilhas (.csv, .tsv, .xlsx, .xlsm, .xlsb, .ods, .xls, etc.)
/// (paridade com gmet_07_ExtrairMetadadosPlanilha.cpp).
pub fn extrair_planilha(caminho: &Path, ext: &str, file_size: u64) -> Option<DocumentMetadata> {
    match ext {
        "csv" | "tsv" | "tab" | "psv" | "dsv" | "ssv" | "bcsv" | "prn" => {
            extrair_csv(caminho)
        }
        "slk" | "sylk" | "dif" => extrair_slk_dif(caminho, ext, file_size),
        "xlsx" | "xlsm" | "xltx" | "xltm" | "xlsb" | "xlam" | "ods" | "ots" | "sxc" | "stc"
        | "numbers" | "gnumeric" | "gnm" => extrair_planilha_zip(caminho, ext, file_size),
        "xls" | "xlt" | "xlm" | "xlw" => extrair_xls_biff(caminho, file_size),
        _ => {
            // Tenta CSV/texto ou estimativa binária (ex: .parquet, .feather, .wks, .123)
            if let Some(meta) = extrair_csv(caminho) {
                if meta.colunas != "0" && meta.colunas != "1" {
                    return Some(meta);
                }
            }
            let est_records = ((file_size as f64) / 128.0).ceil() as usize;
            let rec_str = est_records.max(1).to_string();
            Some(DocumentMetadata {
                planilhas: "1".to_string(),
                tabelas: "1".to_string(),
                colunas: "8".to_string(),
                registros: rec_str.clone(),
                linhas: rec_str.clone(),
                total_linhas: rec_str,
                paginas_estimadas: "1".to_string(),
                ..Default::default()
            })
        }
    }
}

pub fn extrair_csv(caminho: &Path) -> Option<DocumentMetadata> {
    let file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut reader = BufReader::with_capacity(65536, file);
    let mut total_lines: usize = 0;
    let mut first_line = String::new();
    let mut line_buf = String::with_capacity(512);

    while total_lines < 1_000_000 {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => break,
            Ok(_) => {
                if first_line.is_empty() && !line_buf.trim().is_empty() {
                    first_line = line_buf.trim().to_string();
                }
                total_lines += 1;
            }
            Err(_) => break,
        }
    }

    let col_count = if !first_line.is_empty() {
        let commas = first_line.matches(',').count();
        let semis = first_line.matches(';').count();
        let tabs = first_line.matches('\t').count();
        let pipes = first_line.matches('|').count();
        let max_sep = commas.max(semis).max(tabs).max(pipes);
        max_sep + 1
    } else {
        1
    };

    let records = if total_lines > 1 {
        total_lines - 1
    } else {
        total_lines
    };
    let pages = ((total_lines as f64) / 50.0).ceil().max(1.0) as usize;

    Some(DocumentMetadata {
        paginas: pages.to_string(),
        paginas_estimadas: pages.to_string(),
        linhas: total_lines.to_string(),
        total_linhas: total_lines.to_string(),
        planilhas: "1".to_string(),
        colunas: col_count.to_string(),
        registros: records.to_string(),
        tabelas: "1".to_string(),
        ..Default::default()
    })
}

fn extrair_slk_dif(caminho: &Path, _ext: &str, file_size: u64) -> Option<DocumentMetadata> {
    let text = ler_texto_flexivel(caminho, file_size.min(2 * 1024 * 1024) as usize)?;
    let mut max_row = 0usize;
    let mut max_col = 0usize;
    let mut lines = 0usize;

    for line in text.lines() {
        lines += 1;
        if line.starts_with("B;Y") || line.starts_with("C;Y") {
            for part in line.split(';') {
                if let Some(y) = part.strip_prefix('Y') {
                    if let Ok(r) = y.parse::<usize>() {
                        max_row = max_row.max(r);
                    }
                } else if let Some(x) = part.strip_prefix('X') {
                    if let Ok(c) = x.parse::<usize>() {
                        max_col = max_col.max(c);
                    }
                }
            }
        }
    }
    if max_row == 0 {
        max_row = lines.max(1);
    }
    if max_col == 0 {
        max_col = 4;
    }

    Some(DocumentMetadata {
        planilhas: "1".to_string(),
        tabelas: "1".to_string(),
        colunas: max_col.to_string(),
        registros: max_row.to_string(),
        linhas: lines.to_string(),
        total_linhas: lines.to_string(),
        paginas_estimadas: "1".to_string(),
        ..Default::default()
    })
}

fn extrair_planilha_zip(
    caminho: &Path,
    _ext: &str,
    file_size: u64,
) -> Option<DocumentMetadata> {
    let file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut zip = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return extrair_xls_biff(caminho, file_size),
    };

    let mut autor = String::new();
    if let Some(xml) = read_zip_entry_string(&mut zip, "docProps/core.xml") {
        if let Some(c) = extract_xml_tag_value(&xml, "dc:creator") {
            autor = c;
        }
    }

    let mut sheet_names = Vec::new();
    let mut table_count = 0usize;
    let mut xlsb_sheets = 0usize;

    for i in 0..zip.len() {
        if let Ok(entry) = zip.by_index(i) {
            let name = entry.name().unwrap_or_default().to_string();
            if name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml") {
                sheet_names.push(name);
            } else if name.starts_with("xl/worksheets/sheet") && name.ends_with(".bin") {
                xlsb_sheets += 1;
            } else if name.starts_with("xl/tables/table") && name.ends_with(".xml") {
                table_count += 1;
            }
        }
    }

    let mut total_rows = 0usize;
    let mut max_cols = 0usize;
    let mut sheets = sheet_names.len() + xlsb_sheets;

    for sname in &sheet_names {
        if let Some(xml) = read_zip_entry_string(&mut zip, sname) {
            let (r, c) = parse_xlsx_sheet_metrics(&xml);
            total_rows += r;
            max_cols = max_cols.max(c);
        }
    }

    // ODF (.ods / .ots)
    if sheets == 0 && zip.by_name("content.xml").is_ok() {
        if let Some(xml) = read_zip_entry_string(&mut zip, "content.xml") {
            sheets = xml.matches("<table:table ").count().max(1);
            total_rows = xml.matches("<table:table-row").count();
            let cells = xml.matches("<table:table-cell").count();
            if total_rows > 0 {
                max_cols = (cells / total_rows).max(1);
            }
        }
        if autor.is_empty() {
            if let Some(mxml) = read_zip_entry_string(&mut zip, "meta.xml") {
                if let Some(c) = extract_xml_tag_value(&mxml, "dc:creator") {
                    autor = c;
                }
            }
        }
    }

    if sheets == 0 {
        sheets = 1;
    }
    if table_count == 0 {
        table_count = sheets;
    }
    if total_rows == 0 {
        total_rows = ((file_size as f64) / 256.0).ceil().max(1.0) as usize;
    }
    if max_cols == 0 {
        max_cols = 6;
    }
    let records = if total_rows > 1 {
        total_rows - 1
    } else {
        total_rows
    };

    Some(DocumentMetadata {
        autor,
        planilhas: sheets.to_string(),
        tabelas: table_count.to_string(),
        colunas: max_cols.to_string(),
        registros: records.to_string(),
        linhas: total_rows.to_string(),
        total_linhas: total_rows.to_string(),
        paginas_estimadas: sheets.to_string(),
        ..Default::default()
    })
}

fn parse_xlsx_sheet_metrics(xml: &str) -> (usize, usize) {
    // 1. Tenta `<dimension ref="A1:F120"/>`
    if let Some(dpos) = xml.find("<dimension") {
        let rest = &xml[dpos..];
        if let Some(end) = rest.find('>') {
            let tag = &rest[..end];
            if let Some(ref_val) = extract_xml_attr(tag, "ref") {
                let bound = ref_val.split(':').last().unwrap_or(&ref_val);
                let (c, r) = split_excel_cell_ref(bound);
                if r > 0 && c > 0 {
                    return (r, c);
                }
            }
        }
    }

    // 2. Conta `<row` e maior coluna em `<c r="C5"`
    let rows = xml.matches("<row ").count() + xml.matches("<row>").count();
    let mut max_col = 0usize;
    let mut search = 0usize;
    while let Some(cpos) = xml[search..].find("<c ") {
        let abs = search + cpos;
        let rest = &xml[abs..];
        if let Some(end) = rest.find('>') {
            if let Some(r_attr) = extract_xml_attr(&rest[..end], "r") {
                let (c, _) = split_excel_cell_ref(&r_attr);
                max_col = max_col.max(c);
            }
            search = abs + end + 1;
        } else {
            break;
        }
    }

    (rows, max_col.max(if rows > 0 { 1 } else { 0 }))
}

fn split_excel_cell_ref(cell: &str) -> (usize, usize) {
    let mut col = 0usize;
    let mut row_str = String::new();
    for ch in cell.chars() {
        if ch.is_ascii_alphabetic() {
            col = col * 26 + ((ch.to_ascii_uppercase() as u8 - b'A') as usize + 1);
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        }
    }
    let row = row_str.parse::<usize>().unwrap_or(0);
    (col, row)
}

fn extrair_xls_biff(caminho: &Path, file_size: u64) -> Option<DocumentMetadata> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut buf = vec![0u8; file_size.min(524288) as usize];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);

    // Conta registros BIFF8 BoundSheet8 (opcode 0x0085) e DIMENSIONS (opcode 0x0200)
    let mut sheets = 0usize;
    let mut max_rows = 0usize;
    let mut max_cols = 0usize;

    let mut i = 0usize;
    while i + 4 <= n {
        let rec_id = u16::from_le_bytes([buf[i], buf[i + 1]]);
        let rec_len = u16::from_le_bytes([buf[i + 2], buf[i + 3]]) as usize;
        if rec_id == 0x0085 && (6..=256).contains(&rec_len) {
            sheets += 1;
        } else if rec_id == 0x0200 && rec_len == 14 && i + 18 <= n {
            let last_row =
                u32::from_le_bytes([buf[i + 8], buf[i + 9], buf[i + 10], buf[i + 11]]) as usize;
            let last_col = u16::from_le_bytes([buf[i + 14], buf[i + 15]]) as usize;
            if last_row <= 1_048_576 && last_col <= 16384 {
                max_rows = max_rows.max(last_row);
                max_cols = max_cols.max(last_col);
            }
        }
        i += 1;
    }

    if sheets == 0 {
        sheets = 1;
    }
    if max_rows == 0 {
        max_rows = ((file_size as f64) / 128.0).ceil().max(1.0) as usize;
    }
    if max_cols == 0 {
        max_cols = 8;
    }

    Some(DocumentMetadata {
        planilhas: sheets.to_string(),
        tabelas: sheets.to_string(),
        colunas: max_cols.to_string(),
        registros: max_rows.to_string(),
        linhas: max_rows.to_string(),
        total_linhas: max_rows.to_string(),
        paginas_estimadas: sheets.to_string(),
        ..Default::default()
    })
}

/// Extrai metadados de apresentações (.pptx, .pptm, .odp, .ppt, .key, etc.)
/// (paridade com gmet_08_ExtrairMetadadosApresentacao.cpp).
pub fn extrair_apresentacao(
    caminho: &Path,
    _ext: &str,
    file_size: u64,
) -> Option<DocumentMetadata> {
    if let Ok(file) = super::extractor::abrir_arquivo_compartilhado(caminho) {
        if let Ok(mut zip) = zip::ZipArchive::new(file) {
            let mut autor = String::new();
            let mut slides = 0usize;
            let mut words = 0usize;

            if let Some(xml) = read_zip_entry_string(&mut zip, "docProps/core.xml") {
                if let Some(c) = extract_xml_tag_value(&xml, "dc:creator") {
                    autor = c;
                }
            }
            if let Some(xml) = read_zip_entry_string(&mut zip, "docProps/app.xml") {
                if let Some(s) =
                    extract_xml_tag_value(&xml, "Slides").and_then(|v| v.parse::<usize>().ok())
                {
                    slides = s;
                }
                if let Some(w) =
                    extract_xml_tag_value(&xml, "Words").and_then(|v| v.parse::<usize>().ok())
                {
                    words = w;
                }
            }

            if slides == 0 {
                for i in 0..zip.len() {
                    if let Ok(entry) = zip.by_index(i) {
                        let name = entry.name().unwrap_or_default();
                        if (name.starts_with("ppt/slides/slide") && name.ends_with(".xml"))
                            || (name.starts_with("Index/Slide") && name.ends_with(".iwa"))
                        {
                            slides += 1;
                        }
                    }
                }
            }

            if slides == 0 && zip.by_name("content.xml").is_ok() {
                if let Some(xml) = read_zip_entry_string(&mut zip, "content.xml") {
                    slides = xml.matches("<draw:page ").count();
                }
                if autor.is_empty() {
                    if let Some(mxml) = read_zip_entry_string(&mut zip, "meta.xml") {
                        if let Some(c) = extract_xml_tag_value(&mxml, "dc:creator") {
                            autor = c;
                        }
                    }
                }
            }

            slides = slides.max(1);
            let s_str = slides.to_string();
            return Some(DocumentMetadata {
                autor,
                slides: s_str.clone(),
                slides_estimados: s_str.clone(),
                paginas: s_str.clone(),
                paginas_estimadas: s_str,
                palavras: if words > 0 {
                    words.to_string()
                } else {
                    (slides * 45).to_string()
                },
                ..Default::default()
            });
        }
    }

    // Legacy .ppt (OLE) ou apresentação binária
    let est_slides = ((file_size as f64) / 32_768.0).ceil().max(1.0) as usize;
    let s_str = est_slides.to_string();
    Some(DocumentMetadata {
        slides: s_str.clone(),
        slides_estimados: s_str.clone(),
        paginas: s_str.clone(),
        paginas_estimadas: s_str,
        palavras: (est_slides * 45).to_string(),
        ..Default::default()
    })
}

/// Extrai metadados de bancos de dados e arquivos GIS (.sqlite, .db, .dbf, .mdb, .geojson, .kml, etc.)
/// (paridade com gmet_09_ExtrairMetadadosBancoDados.cpp).
pub fn extrair_banco_dados(
    caminho: &Path,
    ext: &str,
    file_size: u64,
) -> Option<DocumentMetadata> {
    let mut header = [0u8; 32];
    let n = super::extractor::abrir_arquivo_compartilhado(caminho)
        .and_then(|mut f| f.read(&mut header))
        .unwrap_or(0);

    // 1. SQLite 3 (qualquer extensão com header SQLite format 3)
    if (n >= 16 && &header[0..15] == b"SQLite format 3")
        || matches!(
            ext,
            "sqlite" | "sqlite3" | "db3" | "s3db" | "sl3" | "gpkg" | "mbtiles" | "spatialite"
        )
    {
        if let Some(meta) = extrair_sqlite(caminho) {
            return Some(meta);
        }
    }

    // 2. dBase / Shapefile DBF (.dbf)
    if ext == "dbf" && n >= 32 {
        let num_records = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let hdr_bytes = u16::from_le_bytes([header[8], header[9]]) as usize;
        let num_cols = if hdr_bytes > 32 {
            ((hdr_bytes - 32) / 32).max(1)
        } else {
            1
        };
        let rec_str = num_records.to_string();
        return Some(DocumentMetadata {
            tabelas: "1".to_string(),
            colunas: num_cols.to_string(),
            registros: rec_str.clone(),
            linhas: rec_str.clone(),
            total_linhas: rec_str,
            ..Default::default()
        });
    }

    // 3. GIS baseados em texto (.geojson, .topojson, .kml, .gpx, .gml, .osm)
    if matches!(ext, "geojson" | "topojson" | "kml" | "gpx" | "gml" | "osm") {
        if let Some(text) = ler_texto_flexivel(caminho, file_size.min(4 * 1024 * 1024) as usize) {
            let lines = text.lines().count().max(1);
            let records = match ext {
                "geojson" | "topojson" => text.matches("\"Feature\"").count().max(1),
                "kml" => text.matches("<Placemark").count().max(1),
                "gpx" => (text.matches("<wpt").count() + text.matches("<trkpt").count()).max(1),
                "osm" => (text.matches("<node").count() + text.matches("<way").count()).max(1),
                _ => (lines / 10).max(1),
            };
            return Some(DocumentMetadata {
                tabelas: "1".to_string(),
                colunas: "4".to_string(),
                registros: records.to_string(),
                linhas: lines.to_string(),
                total_linhas: lines.to_string(),
                ..Default::default()
            });
        }
    }

    // 4. Outros formatos de banco de dados / GIS (.mdb, .accdb, .mdf, .shp, .parquet, .h5, etc.)
    let est_tables = if file_size > 1024 * 1024 { 4 } else { 1 };
    let est_records = ((file_size as f64) / 256.0).ceil().max(1.0) as usize;
    let rec_str = est_records.to_string();
    Some(DocumentMetadata {
        tabelas: est_tables.to_string(),
        colunas: "6".to_string(),
        registros: rec_str.clone(),
        linhas: rec_str.clone(),
        total_linhas: rec_str,
        ..Default::default()
    })
}

fn extrair_sqlite(caminho: &Path) -> Option<DocumentMetadata> {
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = rusqlite::Connection::open_with_flags(caminho, flags).ok()?;
    let _ = conn.busy_timeout(std::time::Duration::from_millis(50));

    let mut tables = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' LIMIT 64",
            )
            .ok()?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0)).ok()?;
        for r in rows {
            tables.push(r.ok()?);
        }
    }

    if tables.is_empty() {
        return Some(DocumentMetadata {
            tabelas: "0".to_string(),
            colunas: "0".to_string(),
            registros: "0".to_string(),
            linhas: "0".to_string(),
            total_linhas: "0".to_string(),
            ..Default::default()
        });
    }

    let mut max_cols = 0usize;
    let mut total_records = 0u64;

    for tbl in tables.iter().take(24) {
        let safe_tbl = tbl.replace('"', "\"\"");
        let pragma_sql = format!("PRAGMA table_info(\"{}\")", safe_tbl);
        if let Ok(mut stmt) = conn.prepare(&pragma_sql) {
            if let Ok(rows) = stmt.query_map([], |_r| Ok(())) {
                let c = rows.count();
                max_cols = max_cols.max(c);
            }
        }
        let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", safe_tbl);
        if let Ok(cnt) = conn.query_row(&count_sql, [], |r| r.get::<_, i64>(0)) {
            if cnt > 0 {
                total_records += cnt as u64;
            }
        }
    }

    let rec_str = total_records.to_string();
    Some(DocumentMetadata {
        tabelas: tables.len().to_string(),
        colunas: max_cols.to_string(),
        registros: rec_str.clone(),
        linhas: rec_str.clone(),
        total_linhas: rec_str,
        ..Default::default()
    })
}

/// Lê arquivo texto suportando UTF-8, UTF-16LE, UTF-16BE e fallback Latin-1/Windows-1252.
pub fn ler_texto_flexivel(caminho: &Path, max_bytes: usize) -> Option<String> {
    let mut file = super::extractor::abrir_arquivo_compartilhado(caminho).ok()?;
    let mut buf = vec![0u8; max_bytes.min(8 * 1024 * 1024)];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);

    if n == 0 {
        return Some(String::new());
    }

    // UTF-16LE BOM
    if n >= 2 && buf[0] == 0xFF && buf[1] == 0xFE {
        let u16s: Vec<u16> = buf[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16s));
    }
    // UTF-16BE BOM
    if n >= 2 && buf[0] == 0xFE && buf[1] == 0xFF {
        let u16s: Vec<u16> = buf[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16s));
    }

    // Verifica se parece texto ou binário com a mesma heurística determinística de `types.rs`
    let sample_len = n.min(super::types::TAMANHO_AMOSTRA_TEXTO);
    if !super::types::amostra_parece_texto(&buf[..sample_len]) {
        return None;
    }

    if let Ok(s) = std::str::from_utf8(&buf) {
        return Some(s.to_string());
    }

    Some(String::from_utf8_lossy(&buf).into_owned())
}

fn read_zip_entry_string(
    zip: &mut zip::ZipArchive<File>,
    name: &str,
) -> Option<String> {
    let mut entry = zip.by_name(name).ok()?;
    let mut s = String::new();
    entry.read_to_string(&mut s).ok()?;
    Some(s)
}

fn extract_xml_tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start_tag = xml.find(&open)?;
    let after_open = xml[start_tag..].find('>')? + start_tag + 1;
    let end_tag = xml[after_open..].find(&close)? + after_open;
    let val = xml[after_open..end_tag].trim();
    if val.is_empty() || val.contains('<') {
        None
    } else {
        Some(val.to_string())
    }
}

fn extract_xml_attr(xml: &str, attr: &str) -> Option<String> {
    let pat = format!("{}=\"", attr);
    let idx = xml.find(&pat)?;
    let start = idx + pat.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].trim().to_string())
}

fn strip_xml_tags(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len() / 2);
    let mut in_tag = false;
    for ch in xml.chars() {
        if ch == '<' {
            in_tag = true;
            out.push(' ');
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(ch);
        }
    }
    out
}
