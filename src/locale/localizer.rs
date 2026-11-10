use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

pub const IDIOMAS_SUPORTADOS: &[(&str, &str)] = &[
    ("pt_BR", "Português (Brasil) | Brazilian Portuguese"),
    ("en_US", "English (US) | Inglês (EUA)"),
    ("es_ES", "Español | Spanish"),
    ("fr_FR", "Français | French"),
    ("it_IT", "Italiano | Italian"),
    ("de_DE", "Deutsch | German"),
];

// Dicionários nativos compilados e embutidos diretamente no binário Rust
// Elimina 100% de dependência de arquivos .qm e .ts do Qt6/C++ em tempo de execução
const JSON_PT_BR: &str = include_str!("../language/pt_BR.json");
const JSON_EN_US: &str = include_str!("../language/en_US.json");
const JSON_ES_ES: &str = include_str!("../language/es_ES.json");
const JSON_FR_FR: &str = include_str!("../language/fr_FR.json");
const JSON_IT_IT: &str = include_str!("../language/it_IT.json");
const JSON_DE_DE: &str = include_str!("../language/de_DE.json");

pub struct Localizer {
    idioma_atual: RwLock<String>,
    traducoes: RwLock<HashMap<(String, String), String>>,
}

impl Default for Localizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Localizer {
    pub fn new() -> Self {
        let loc = Self {
            idioma_atual: RwLock::new(String::from("pt_BR")),
            traducoes: RwLock::new(HashMap::new()),
        };

        // 1. Carrega os dicionários nativos embutidos (zero overhead de I/O em runtime)
        loc.carregar_dicionarios_embutidos();

        // 2. Permite carregar eventuais customizações JSON locais se presentes no disco
        loc.carregar_arquivos_json_disco();

        // 3. Carrega preferência persistente se salva pelo usuário
        let preferido = loc.carregar_preferencia_idioma();
        *loc.idioma_atual.write() = preferido;

        loc
    }

    fn carregar_dicionarios_embutidos(&self) {
        let mut map = self.traducoes.write();

        let pares = [
            ("pt_BR", JSON_PT_BR),
            ("en_US", JSON_EN_US),
            ("es_ES", JSON_ES_ES),
            ("fr_FR", JSON_FR_FR),
            ("it_IT", JSON_IT_IT),
            ("de_DE", JSON_DE_DE),
        ];

        for (lang, json_str) in pares {
            if let Ok(dicionario) = serde_json::from_str::<HashMap<String, String>>(json_str) {
                for (k, v) in dicionario {
                    map.insert((lang.to_string(), k), v);
                }
            }
        }
    }

    fn carregar_arquivos_json_disco(&self) {
        let locais = [
            "src/language",
            "language",
            "assets/language",
            "src/assets/language",
            "locale",
        ];
        let base_dir = crate::utils::paths::obter_diretorio_base();
        let mut map = self.traducoes.write();

        for dir_cand in locais {
            let mut cand_paths = vec![
                base_dir.join(dir_cand),
                PathBuf::from(dir_cand),
            ];
            if let Some(parent) = base_dir.parent() {
                cand_paths.push(parent.join(dir_cand));
                if let Some(grandparent) = parent.parent() {
                    cand_paths.push(grandparent.join(dir_cand));
                }
            }
            for dir in &cand_paths {
                if dir.exists() {
                    for (code, _) in IDIOMAS_SUPORTADOS {
                        let json_file = dir.join(format!("{}.json", code));
                        if json_file.exists() {
                            if let Ok(content) = std::fs::read_to_string(&json_file) {
                                if let Ok(dicionario) =
                                    serde_json::from_str::<HashMap<String, String>>(&content)
                                {
                                    for (k, v) in dicionario {
                                        map.insert((code.to_string(), k), v);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn idioma_atual(&self) -> String {
        self.idioma_atual.read().clone()
    }

    pub fn set_idioma(&self, idioma: &str) {
        if IDIOMAS_SUPORTADOS.iter().any(|(code, _)| *code == idioma) {
            *self.idioma_atual.write() = idioma.to_string();
        }
    }

    pub fn salvar_preferencia_idioma(&self, idioma: &str) {
        let base = crate::utils::obter_caminho_persistente();
        let val = serde_json::json!({ "idioma": idioma });
        let s = val.to_string();
        let _ = std::fs::write(base.join("idioma_config.json"), &s);
        let _ = std::fs::write(base.join("language_config.json"), &s);
    }

    pub fn carregar_preferencia_idioma(&self) -> String {
        let base = crate::utils::obter_caminho_persistente();
        let arquivos = ["idioma_config.json", "language_config.json"];

        for arq in &arquivos {
            let p = base.join(arq);
            if let Ok(bytes) = std::fs::read(p) {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if let Some(id) = json.get("idioma").and_then(|v| v.as_str()) {
                        if IDIOMAS_SUPORTADOS.iter().any(|(code, _)| *code == id) {
                            return id.to_string();
                        }
                    }
                }
            }
        }

        // Primeira execução ou ausência de seleção manual: detecta o idioma do sistema operacional Windows 11.
        // Caso o idioma do SO não seja um dos suportados, reporta a GUI em Inglês (en_US).
        // Não grava arquivo em disco prematuramente; a persistência ocorre apenas quando o usuário alterar via menu.
        Self::detectar_idioma_sistema()
    }

    /// Detecta o idioma nativo do sistema operacional Windows / POSIX
    pub fn detectar_idioma_sistema() -> String {
        #[cfg(target_os = "windows")]
        {
            extern "system" {
                fn GetUserDefaultUILanguage() -> u16;
                fn GetSystemDefaultUILanguage() -> u16;
                fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
            }

            // 1. Prioridade máxima: Idioma da interface de usuário (Windows 11 UI Display Language)
            let ui_lang = unsafe { GetUserDefaultUILanguage() };
            let primary = ui_lang & 0x03ff;
            match primary {
                0x16 => return "pt_BR".to_string(), // LANG_PORTUGUESE
                0x09 => return "en_US".to_string(), // LANG_ENGLISH
                0x0a => return "es_ES".to_string(), // LANG_SPANISH
                0x0c => return "fr_FR".to_string(), // LANG_FRENCH
                0x10 => return "it_IT".to_string(), // LANG_ITALIAN
                0x07 => return "de_DE".to_string(), // LANG_GERMAN
                _ => {}
            }

            // 2. Idioma de localidade configurado pelo usuário (Locale Name BCP-47)
            let mut buffer = [0u16; 85];
            let len = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
            if len > 1 {
                let name = String::from_utf16_lossy(&buffer[..(len as usize - 1)]).to_lowercase();
                if name.starts_with("pt") {
                    return "pt_BR".to_string();
                } else if name.starts_with("en") {
                    return "en_US".to_string();
                } else if name.starts_with("es") {
                    return "es_ES".to_string();
                } else if name.starts_with("fr") {
                    return "fr_FR".to_string();
                } else if name.starts_with("it") {
                    return "it_IT".to_string();
                } else if name.starts_with("de") {
                    return "de_DE".to_string();
                }
            }

            // 3. Fallback: Idioma padrão do sistema (System Default UI Language)
            let sys_lang = unsafe { GetSystemDefaultUILanguage() };
            let sys_primary = sys_lang & 0x03ff;
            match sys_primary {
                0x16 => return "pt_BR".to_string(),
                0x09 => return "en_US".to_string(),
                0x0a => return "es_ES".to_string(),
                0x0c => return "fr_FR".to_string(),
                0x10 => return "it_IT".to_string(),
                0x07 => return "de_DE".to_string(),
                _ => {}
            }
        }

        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let v = val.to_lowercase();
                if v.starts_with("pt") {
                    return "pt_BR".to_string();
                } else if v.starts_with("en") {
                    return "en_US".to_string();
                } else if v.starts_with("es") {
                    return "es_ES".to_string();
                } else if v.starts_with("fr") {
                    return "fr_FR".to_string();
                } else if v.starts_with("it") {
                    return "it_IT".to_string();
                } else if v.starts_with("de") {
                    return "de_DE".to_string();
                }
            }
        }

        // Se o sistema operacional utilizar idioma não suportado pelas traduções disponíveis,
        // o padrão estipulado é o idioma Inglês (en_US)
        "en_US".to_string()
    }

    /// Obtém a tradução para a chave especificada no idioma ativo com cascata de fallback
    pub fn get_text(&self, key: &str) -> String {
        let lang = self.idioma_atual();
        let map = self.traducoes.read();

        // 1. Idioma ativo
        if let Some(txt) = map.get(&(lang.clone(), key.to_string())) {
            if !txt.is_empty() {
                return txt.clone();
            }
        }
        // 2. Fallback pt_BR
        if let Some(txt) = map.get(&("pt_BR".to_string(), key.to_string())) {
            if !txt.is_empty() {
                return txt.clone();
            }
        }
        // 3. Fallback en_US
        if let Some(txt) = map.get(&("en_US".to_string(), key.to_string())) {
            if !txt.is_empty() {
                return txt.clone();
            }
        }

        key.to_string()
    }

    /// Alias conveniente e idiomático para `get_text`
    #[inline]
    pub fn t(&self, key: &str) -> String {
        self.get_text(key)
    }

    /// Tradução com substituição de argumentos posicionais (%1, %2, etc. ou {0}, {1}, etc.)
    pub fn get_text_args(&self, key: &str, args: &[&str]) -> String {
        let mut s = self.get_text(key);
        for (idx, arg) in args.iter().enumerate() {
            let placeholder_pct = format!("%{}", idx + 1);
            s = s.replace(&placeholder_pct, arg);
            let placeholder_idx = format!("{{{}}}", idx);
            s = s.replace(&placeholder_idx, arg);
            let placeholder_one = format!("{{{}}}", idx + 1);
            s = s.replace(&placeholder_one, arg);
        }
        s
    }

    /// Alias conveniente e idiomático para `get_text_args`
    #[inline]
    pub fn t_args(&self, key: &str, args: &[&str]) -> String {
        self.get_text_args(key, args)
    }

    /// Traduz dinamicamente os tipos de operação do monitor e auditoria
    pub fn traduzir_tipo_operacao(&self, tipo: &str) -> String {
        let clean = tipo.trim().to_lowercase();
        let key = if clean.contains("adic")
            || clean.contains("add")
            || clean.contains("cri")
            || clean.contains("añad")
            || clean.contains("anad")
            || clean.contains("ajout")
            || clean.contains("aggiunt")
            || clean.contains("hinzu")
        {
            "op_added"
        } else if clean.contains("mod")
            || clean.contains("edit")
            || clean.contains("chang")
            || clean.contains("geänd")
            || clean.contains("geand")
        {
            "op_modified"
        } else if clean.contains("excl")
            || clean.contains("del")
            || clean.contains("remov")
            || clean.contains("elim")
            || clean.contains("suppr")
            || clean.contains("lösch")
            || clean.contains("losch")
        {
            "op_deleted"
        } else if clean.contains("renom")
            || clean.contains("rename")
            || clean.contains("rinom")
            || clean.contains("umben")
        {
            "op_renamed"
        } else if clean.contains("mov")
            || clean.contains("move")
            || clean.contains("dépl")
            || clean.contains("depl")
            || clean.contains("spost")
            || clean.contains("verschob")
        {
            "op_moved"
        } else if clean.contains("escan") || clean.contains("scan") {
            "op_scanned"
        } else {
            return self.t(tipo);
        };
        self.t(key)
    }

    /// Traduz dinamicamente os níveis de risco para o idioma ativo
    pub fn traduzir_risco(&self, risco: &str) -> String {
        let clean = risco.trim().to_lowercase();
        if clean.contains("crít") || clean.contains("crit") || clean.contains("kri") {
            self.t("risk_critical")
        } else if clean.contains("alt")
            || clean.contains("high")
            || clean.contains("haut")
            || clean.contains("hoch")
        {
            self.t("risk_high")
        } else if clean.contains("méd")
            || clean.contains("med")
            || clean.contains("moyen")
            || clean.contains("mittel")
        {
            self.t("risk_medium")
        } else if clean.contains("baix")
            || clean.contains("low")
            || clean.contains("baj")
            || clean.contains("bas")
            || clean.contains("basso")
            || clean.contains("niedrig")
        {
            self.t("risk_low")
        } else if clean.contains("info") {
            self.t("risk_info")
        } else if clean.contains("tod")
            || clean.contains("all")
            || clean.contains("tous")
            || clean.contains("tutti")
            || clean.contains("alle")
        {
            self.t("all")
        } else {
            self.t(risco)
        }
    }

    /// Traduz dinamicamente tipos de arquivo e preserva extensões monitoradas (ex: "txt", "dll", "json", "tar.gz")
    pub fn traduzir_tipo_arquivo(&self, tipo: &str) -> String {
        let clean = tipo.trim();
        if clean.is_empty() {
            return String::new();
        }

        let lower = clean.to_lowercase();

        // Pastas / Diretórios
        if lower == "folder"
            || lower == "pasta"
            || lower == "directory"
            || lower == "pasta de arquivos"
        {
            return self.t("folder");
        }

        // Categorias genéricas de arquivos sem extensão ou rótulos legados de categoria
        let lang = self.idioma_atual();
        match lower.as_str() {
            "unknown" | "desconhecido" => return self.t("unknown"),
            "file" | "arquivo" => return self.t("file"),
            "code" | "código fonte" | "codigo fonte" => {
                return match lang.as_str() {
                    "en_US" => "Source Code".to_string(),
                    "es_ES" => "Código Fuente".to_string(),
                    "fr_FR" => "Code Source".to_string(),
                    "it_IT" => "Codice Sorgente".to_string(),
                    "de_DE" => "Quellcode".to_string(),
                    _ => "Código Fonte".to_string(),
                };
            }
            "document" | "documento" => {
                return match lang.as_str() {
                    "en_US" => "Document".to_string(),
                    "es_ES" => "Documento".to_string(),
                    "fr_FR" => "Document".to_string(),
                    "it_IT" => "Documento".to_string(),
                    "de_DE" => "Dokument".to_string(),
                    _ => "Documento".to_string(),
                };
            }
            "image" | "imagem" => {
                return match lang.as_str() {
                    "en_US" => "Image".to_string(),
                    "es_ES" => "Imagen".to_string(),
                    "fr_FR" => "Image".to_string(),
                    "it_IT" => "Immagine".to_string(),
                    "de_DE" => "Bild".to_string(),
                    _ => "Imagem".to_string(),
                };
            }
            "audio" | "áudio" => {
                return match lang.as_str() {
                    "en_US" | "es_ES" | "fr_FR" | "it_IT" | "de_DE" => "Audio".to_string(),
                    _ => "Áudio".to_string(),
                };
            }
            "video" | "vídeo" => {
                return match lang.as_str() {
                    "en_US" | "it_IT" | "de_DE" => "Video".to_string(),
                    "fr_FR" => "Vidéo".to_string(),
                    _ => "Vídeo".to_string(),
                };
            }
            "spreadsheet" | "planilha" => {
                return match lang.as_str() {
                    "en_US" => "Spreadsheet".to_string(),
                    "es_ES" => "Hoja de Cálculo".to_string(),
                    "fr_FR" => "Feuille de Calcul".to_string(),
                    "it_IT" => "Foglio di Calcolo".to_string(),
                    "de_DE" => "Tabelle".to_string(),
                    _ => "Planilha".to_string(),
                };
            }
            "presentation" | "apresentação" | "apresentacao" => {
                return match lang.as_str() {
                    "en_US" => "Presentation".to_string(),
                    "es_ES" => "Presentación".to_string(),
                    "fr_FR" => "Présentation".to_string(),
                    "it_IT" => "Presentazione".to_string(),
                    "de_DE" => "Präsentation".to_string(),
                    _ => "Apresentação".to_string(),
                };
            }
            "database" | "banco de dados" => {
                return match lang.as_str() {
                    "en_US" => "Database".to_string(),
                    "es_ES" => "Base de Datos".to_string(),
                    "fr_FR" => "Base de Données".to_string(),
                    "it_IT" => "Database".to_string(),
                    "de_DE" => "Datenbank".to_string(),
                    _ => "Banco de Dados".to_string(),
                };
            }
            "executable" | "executável" | "executavel" => {
                return match lang.as_str() {
                    "en_US" => "Executable".to_string(),
                    "es_ES" => "Ejecutable".to_string(),
                    "fr_FR" => "Exécutable".to_string(),
                    "it_IT" => "Eseguibile".to_string(),
                    "de_DE" => "Ausführbare Datei".to_string(),
                    _ => "Executável".to_string(),
                };
            }
            "executável windows" | "executavel windows" => {
                return match lang.as_str() {
                    "en_US" => "Windows Executable".to_string(),
                    "es_ES" => "Ejecutable Windows".to_string(),
                    "fr_FR" => "Exécutable Windows".to_string(),
                    "it_IT" => "Eseguibile Windows".to_string(),
                    "de_DE" => "Windows-Programm".to_string(),
                    _ => "Executável Windows".to_string(),
                };
            }
            "arquivo de texto" => {
                return match lang.as_str() {
                    "en_US" => "Text File".to_string(),
                    "es_ES" => "Archivo de Texto".to_string(),
                    "fr_FR" => "Fichier Texte".to_string(),
                    "it_IT" => "File di Testo".to_string(),
                    "de_DE" => "Textdatei".to_string(),
                    _ => "Arquivo de Texto".to_string(),
                };
            }
            "compressed" | "compactado" => {
                return match lang.as_str() {
                    "en_US" => "Archive".to_string(),
                    "es_ES" => "Comprimido".to_string(),
                    "fr_FR" => "Archive".to_string(),
                    "it_IT" => "Archivio".to_string(),
                    "de_DE" => "Archiv".to_string(),
                    _ => "Compactado".to_string(),
                };
            }
            "backup" => return "Backup".to_string(),
            "config" | "configuração" | "configuracao" => {
                return match lang.as_str() {
                    "en_US" | "fr_FR" => "Configuration".to_string(),
                    "es_ES" => "Configuración".to_string(),
                    "it_IT" => "Configurazione".to_string(),
                    "de_DE" => "Konfiguration".to_string(),
                    _ => "Configuração".to_string(),
                };
            }
            "binary" | "binário" | "binario" => {
                return match lang.as_str() {
                    "en_US" => "Binary".to_string(),
                    "es_ES" | "it_IT" => "Binario".to_string(),
                    "fr_FR" => "Binaire".to_string(),
                    "de_DE" => "Binärdatei".to_string(),
                    _ => "Binário".to_string(),
                };
            }
            "text" | "texto" => {
                return match lang.as_str() {
                    "en_US" | "de_DE" => "Text".to_string(),
                    "fr_FR" => "Texte".to_string(),
                    "it_IT" => "Testo".to_string(),
                    _ => "Texto".to_string(),
                };
            }
            _ => {}
        }

        // Se for uma extensão de arquivo (sem espaços, ex: ".rs", "rs", ".dll", "dll", ".tar.gz", "db-wal"),
        // retorna a extensão em minúsculas precedida por ponto (ex: ".exe", ".txt", ".rs") para a coluna "type"!
        if !lower.contains(' ') {
            if lower.starts_with('.') {
                return lower;
            } else {
                return format!(".{}", lower);
            }
        }

        // Compatibilidade com descrições legadas salvas no banco de dados
        let tent_direta = self.t(clean);
        if tent_direta != clean {
            return tent_direta;
        }

        clean.to_string()
    }

    /// Traduz dinamicamente atributos de arquivos (ex: "file, archive", "folder, readonly")
    pub fn traduzir_atributos(&self, attrs: &str) -> String {
        let clean = attrs.trim();
        if clean.is_empty() {
            return String::new();
        }

        let partes: Vec<&str> = clean.split(',').collect();
        let mut traduzidas = Vec::with_capacity(partes.len());

        for parte in partes {
            let item = parte.trim();
            let item_lower = item.to_lowercase();
            let traduzido = match item_lower.as_str() {
                "folder" | "directory" | "diretório" | "diretorio" | "pasta" => self.t("folder"),
                "file" | "arquivo" => self.t("file"),
                "readonly" | "somente leitura" | "read-only" => self.t("readonly"),
                "hidden" | "oculto" => self.t("hidden"),
                "system" | "sistema" => self.t("system"),
                "archive" | "arquivamento" | "arquivo normal" => self.t("archive"),
                "encrypted" | "criptografado" => self.t("encrypted"),
                "compressed" | "compactado" | "comprimido" => self.t("compressed"),
                _ => {
                    if item_lower.contains("leitura") || item_lower.contains("read") {
                        self.t("readonly")
                    } else if item_lower.contains("ocult") || item_lower.contains("hidden") {
                        self.t("hidden")
                    } else if item_lower.contains("sistem") || item_lower.contains("system") {
                        self.t("system")
                    } else if item_lower.contains("diret") || item_lower.contains("folder") {
                        self.t("folder")
                    } else if item_lower.contains("archiv") {
                        self.t("archive")
                    } else if item_lower.contains("cript") || item_lower.contains("encrypt") {
                        self.t("encrypted")
                    } else if item_lower.contains("comprim") || item_lower.contains("compress") {
                        self.t("compressed")
                    } else {
                        self.t(item)
                    }
                }
            };
            traduzidas.push(traduzido);
        }

        traduzidas.join(", ")
    }

    /// Traduz valores booleanos como Sim / Não
    pub fn traduzir_booleano(&self, val: &str) -> String {
        let clean = val.trim().to_lowercase();
        if clean == "sim"
            || clean == "yes"
            || clean == "true"
            || clean == "1"
            || clean == "s"
            || clean == "y"
        {
            self.t("yes")
        } else if clean == "não"
            || clean == "nao"
            || clean == "no"
            || clean == "false"
            || clean == "0"
            || clean == "n"
        {
            self.t("no")
        } else {
            val.to_string()
        }
    }

    /// Traduz status de baseline criptográfico
    pub fn traduzir_baseline_status(&self, status: &str) -> String {
        let clean = status.trim().to_lowercase();
        if clean.is_empty() {
            return String::new();
        }
        let lang = self.idioma_atual();
        match clean.as_str() {
            "unchanged" | "integral" => self.t("security_baseline_unchanged"),
            "intacto" | "intact" => match lang.as_str() {
                "en_US" => "Intact".to_string(),
                "es_ES" => "Intacto".to_string(),
                "fr_FR" => "Intact".to_string(),
                "it_IT" => "Intatto".to_string(),
                "de_DE" => "Intakt".to_string(),
                _ => "Intacto".to_string(),
            },
            "changed" | "modificado" | "modified" | "alterado" => {
                self.t("security_baseline_changed")
            }
            "new" | "novo" => self.t("security_baseline_new"),
            "missing" | "excluído" | "excluido" | "ausente" => {
                self.t("security_baseline_missing")
            }
            "directory" | "diretório" | "diretorio" | "folder" => {
                self.t("security_baseline_directory")
            }
            "unknown" | "desconhecido" => self.t("security_baseline_unknown"),
            _ => self.t(status.trim()),
        }
    }

    /// Traduz status de quarentena
    pub fn traduzir_quarantine_status(&self, status: &str) -> String {
        let clean = status.trim().to_lowercase();
        if clean.is_empty() {
            return String::new();
        }
        let lang = self.idioma_atual();
        match clean.as_str() {
            "not_applicable" | "não aplicável" | "nao aplicavel" => {
                self.t("security_quarantine_not_applicable")
            }
            "queued" | "fila" => self.t("security_quarantine_queued"),
            "captured" | "capturado" => self.t("security_quarantine_captured"),
            "isolado" | "isolated" => match lang.as_str() {
                "en_US" => "Isolated".to_string(),
                "es_ES" => "Aislado".to_string(),
                "fr_FR" => "Isolé".to_string(),
                "it_IT" => "Isolato".to_string(),
                "de_DE" => "Isoliert".to_string(),
                _ => "Isolado".to_string(),
            },
            "failed" | "falha" => self.t("security_quarantine_failed"),
            "restored" | "restaurado" => match lang.as_str() {
                "en_US" => "Restored".to_string(),
                "es_ES" => "Restaurado".to_string(),
                "fr_FR" => "Restauré".to_string(),
                "it_IT" => "Ripristinato".to_string(),
                "de_DE" => "Wiederhergestellt".to_string(),
                _ => "Restaurado".to_string(),
            },
            _ => self.t(status.trim()),
        }
    }

    /// Traduz rótulo de sessão de segurança
    pub fn traduzir_session_label(&self, label: &str) -> String {
        let clean = label.trim();
        if clean.is_empty() {
            return String::new();
        }
        match clean {
            "sessao_monitoramento" => self.t("sessao_monitoramento"),
            "build_refatoracao" => self.t("build_refatoracao"),
            "instalacao_ou_extracao" => self.t("instalacao_ou_extracao"),
            "limpeza_automatica" => self.t("limpeza_automatica"),
            _ => self.t(clean),
        }
    }

    /// Traduz resumo de diff textual (ex: "security_diff_lines_changed|10|2" ou "sem_alteracao_textual")
    pub fn traduzir_diff_resumo(&self, diff: &str) -> String {
        let clean = diff.trim();
        if clean.is_empty() {
            return String::new();
        }
        if clean == "sem_alteracao_textual" {
            return self.t("sem_alteracao_textual");
        }
        if let Some(rest) = clean.strip_prefix("security_diff_lines_changed|") {
            let parts: Vec<&str> = rest.split('|').collect();
            let added = parts.first().copied().unwrap_or("0");
            let removed = parts.get(1).copied().unwrap_or("0");
            let tpl = self.t("security_diff_lines_changed");
            return tpl.replace("%1", added).replace("%2", removed);
        }
        self.t(clean)
    }

    /// Traduz processo ator forense
    pub fn traduzir_actor_process(&self, proc_name: &str) -> String {
        let clean = proc_name.trim();
        if clean.is_empty() || clean == "unknown_process" {
            return self.t("unknown_process");
        }
        clean.to_string()
    }

    /// Traduz alertas e regras de segurança dinamicamente
    pub fn traduzir_alerta(&self, alerta: &str) -> String {
        let clean = alerta.trim();
        if clean.is_empty() {
            return String::new();
        }

        let separador = if clean.contains(';') { ';' } else { '|' };
        let partes: Vec<&str> = clean.split(separador).collect();
        let mut traduzidos = Vec::with_capacity(partes.len());

        for parte in partes {
            let item = parte.trim();
            let lower = item.to_lowercase();
            let trad = match item {
                "rule_ransomware" | "rule_ransomware_burst" | "SEC-R01" => {
                    self.t("rule_ransomware")
                }
                "rule_double_ext" | "rule_suspicious_rename_to_executable" | "SEC-R02" => {
                    self.t("rule_double_ext")
                }
                "rule_binary_temp" | "rule_new_unsigned_executable" | "SEC-R03" => {
                    self.t("rule_binary_temp")
                }
                "rule_script_temp" | "rule_high_risk_script" | "SEC-R04" => {
                    self.t("rule_script_temp")
                }
                "rule_hidden_executable" | "Executável com Atributo Oculto" | "SEC-R05" => {
                    self.t("rule_hidden_executable")
                }
                "rule_baseline_divergence" | "rule_critical_config_change" | "SEC-R06" => {
                    self.t("rule_baseline_divergence")
                }
                "security_baseline_unchanged" => self.t("security_baseline_unchanged"),
                "security_baseline_changed" => self.t("security_baseline_changed"),
                "security_baseline_new" => self.t("security_baseline_new"),
                "security_baseline_missing" => self.t("security_baseline_missing"),
                "security_baseline_directory" => self.t("security_baseline_directory"),
                "security_baseline_unavailable" => self.t("security_baseline_unavailable"),
                _ => {
                    if lower.contains("ransomware") {
                        self.t("rule_ransomware")
                    } else if lower.contains("dupla") || lower.contains("double") {
                        self.t("rule_double_ext")
                    } else if lower.contains("binário em diretório temporário")
                        || lower.contains("binary in temp")
                        || lower.contains("não assinado")
                        || lower.contains("unsigned")
                    {
                        self.t("rule_binary_temp")
                    } else if lower.contains("script") {
                        self.t("rule_script_temp")
                    } else if lower.contains("oculto") || lower.contains("hidden") {
                        self.t("rule_hidden_executable")
                    } else if lower.contains("baseline") || lower.contains("divergência") {
                        self.t("rule_baseline_divergence")
                    } else {
                        let res = self.t(item);
                        if res == item {
                            let sem_prefixo = item.trim_start_matches("rule_").trim_start_matches("security_");
                            let res2 = self.t(sem_prefixo);
                            if res2 != sem_prefixo {
                                res2
                            } else {
                                item.to_string()
                            }
                        } else {
                            res
                        }
                    }
                }
            };
            traduzidos.push(trad);
        }

        traduzidos.join(if separador == ';' { "; " } else { " | " })
    }

    /// Retorna o nome amigável do idioma
    pub fn traduzir_nome_idioma(&self, code: &str) -> String {
        match code {
            "pt_BR" => "Português (Brasil)".to_string(),
            "en_US" => "English (US)".to_string(),
            "es_ES" => "Español".to_string(),
            "fr_FR" => "Français".to_string(),
            "it_IT" => "Italiano".to_string(),
            "de_DE" => "Deutsch".to_string(),
            _ => code.to_string(),
        }
    }
}
