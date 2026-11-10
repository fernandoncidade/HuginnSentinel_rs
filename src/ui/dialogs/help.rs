use egui::{Color32, RichText, ScrollArea};

use crate::locale::Localizer;

pub struct ShortcutEntry {
    pub sequence: &'static str,
    pub description: &'static str,
}

pub struct HelpDialog {
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
}

impl Default for HelpDialog {
    fn default() -> Self {
        Self {
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
        }
    }
}

impl HelpDialog {
    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer) {
        if !self.aberto {
            return;
        }

        let mut state = super::ModelessState {
            aberto: self.aberto,
            elevar_z_order: self.elevar_z_order,
            maximizado: self.maximizado,
            minimizado: self.minimizado,
        };


        super::show_modeless_dialog(
            ctx,
            "dialog_help",
            format!("❓ {}", loc.t("help")),
            [640.0, 420.0],
            [480.0, 320.0],
            &mut state,
            |ui, fechar| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(loc.t("keyboard_shortcuts"))
                            .strong()
                            .size(15.0),
                    );
                    ui.separator();

                    let available_h = (ui.available_height() - 36.0).max(180.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), available_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ScrollArea::vertical()
                                .id_salt("help_shortcuts_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    egui::Grid::new("shortcuts_grid")
                                        .striped(true)
                                        .spacing([24.0, 8.0])
                                        .min_col_width(140.0)
                                        .show(ui, |ui| {
                                            for entry in Self::atalhos_por_idioma(&loc.idioma_atual()) {
                                                let seq_color = if ui.visuals().dark_mode {
                                                    Color32::from_rgb(100, 181, 246)
                                                } else {
                                                    Color32::from_rgb(25, 118, 210)
                                                };
                                                ui.label(
                                                    RichText::new(entry.sequence)
                                                        .strong()
                                                        .color(seq_color),
                                                );
                                                ui.label(
                                                    RichText::new(entry.description)
                                                        .color(ui.visuals().text_color()),
                                                );
                                                ui.end_row();
                                            }
                                        });
                                });
                        },
                    );

                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(loc.t("ok")).clicked() {
                            *fechar = true;
                        }
                    });
                });
            },
        );

        self.aberto = state.aberto;
        self.maximizado = state.maximizado;
        self.minimizado = state.minimizado;
        self.elevar_z_order = state.elevar_z_order;
    }

    fn atalhos_por_idioma(lang: &str) -> Vec<ShortcutEntry> {
        let normalized = match lang.to_lowercase().as_str() {
            "en_us" | "en" => "en_US",
            "es_es" | "es" => "es_ES",
            "fr_fr" | "fr" => "fr_FR",
            "it_it" | "it" => "it_IT",
            "de_de" | "de" => "de_DE",
            _ => "pt_BR",
        };

        match normalized {
            "en_US" => vec![
                ShortcutEntry { sequence: "F1", description: "Opens the Help window with the list of available shortcuts." },
                ShortcutEntry { sequence: "Ctrl+M", description: "Opens the User Manual." },
                ShortcutEntry { sequence: "Ctrl+I", description: "Opens the About window." },
                ShortcutEntry { sequence: "Ctrl+Shift+O", description: "Opens the dialog to select the monitored directory." },
                ShortcutEntry { sequence: "Ctrl+R", description: "Starts or stops the directory analysis." },
                ShortcutEntry { sequence: "Ctrl+P", description: "Pauses or resumes monitoring or scanning." },
                ShortcutEntry { sequence: "Ctrl+Shift+S", description: "Opens the Save As dialog." },
                ShortcutEntry { sequence: "Ctrl+S", description: "Saves the current data." },
                ShortcutEntry { sequence: "Ctrl+G", description: "Opens the statistics window." },
                ShortcutEntry { sequence: "Ctrl+L", description: "Clears the loaded data." },
                ShortcutEntry { sequence: "Ctrl+Q", description: "Closes the application." },
                ShortcutEntry { sequence: "Ctrl+A", description: "Selects all columns in the column configuration menu." },
                ShortcutEntry { sequence: "Ctrl+Shift+A", description: "Selects all colored columns in the column coloring menu." },
                ShortcutEntry { sequence: "Ctrl+O", description: "Enables or disables automatic row sorting." },
            ],
            "es_ES" => vec![
                ShortcutEntry { sequence: "F1", description: "Abre la ventana Ayuda con la lista de atajos disponibles." },
                ShortcutEntry { sequence: "Ctrl+M", description: "Abre el Manual de Uso." },
                ShortcutEntry { sequence: "Ctrl+I", description: "Abre la ventana Acerca de." },
                ShortcutEntry { sequence: "Ctrl+Shift+O", description: "Abre el diálogo para seleccionar el directorio monitorizado." },
                ShortcutEntry { sequence: "Ctrl+R", description: "Inicia o detiene el análisis del directorio." },
                ShortcutEntry { sequence: "Ctrl+P", description: "Pausa o reanuda la monitorización o el escaneo." },
                ShortcutEntry { sequence: "Ctrl+Shift+S", description: "Abre el diálogo Guardar como." },
                ShortcutEntry { sequence: "Ctrl+S", description: "Guarda los datos actuales." },
                ShortcutEntry { sequence: "Ctrl+G", description: "Abre la ventana de estadísticas." },
                ShortcutEntry { sequence: "Ctrl+L", description: "Limpia los datos cargados." },
                ShortcutEntry { sequence: "Ctrl+Q", description: "Cierra la aplicación." },
                ShortcutEntry { sequence: "Ctrl+A", description: "Selecciona todas las columnas en el menú de configuración de columnas." },
                ShortcutEntry { sequence: "Ctrl+Shift+A", description: "Selecciona todas las columnas coloreadas en el menú de coloración de columnas." },
                ShortcutEntry { sequence: "Ctrl+O", description: "Activa o desactiva la ordenación automática de las filas." },
            ],
            "fr_FR" => vec![
                ShortcutEntry { sequence: "F1", description: "Ouvre la fenêtre d'aide avec la liste des raccourcis disponibles." },
                ShortcutEntry { sequence: "Ctrl+M", description: "Ouvre le manuel d'utilisation." },
                ShortcutEntry { sequence: "Ctrl+I", description: "Ouvre la fenêtre À propos." },
                ShortcutEntry { sequence: "Ctrl+Shift+O", description: "Ouvre la boîte de dialogue pour sélectionner le répertoire surveillé." },
                ShortcutEntry { sequence: "Ctrl+R", description: "Démarre ou arrête l'analyse du répertoire." },
                ShortcutEntry { sequence: "Ctrl+P", description: "Met en pause ou reprend la surveillance ou l'analyse." },
                ShortcutEntry { sequence: "Ctrl+Shift+S", description: "Ouvre la boîte de dialogue Enregistrer sous." },
                ShortcutEntry { sequence: "Ctrl+S", description: "Enregistre les données actuelles." },
                ShortcutEntry { sequence: "Ctrl+G", description: "Ouvre la fenêtre des statistiques." },
                ShortcutEntry { sequence: "Ctrl+L", description: "Efface les données chargées." },
                ShortcutEntry { sequence: "Ctrl+Q", description: "Ferme l'application." },
                ShortcutEntry { sequence: "Ctrl+A", description: "Sélectionne toutes les colonnes dans le menu de configuration des colonnes." },
                ShortcutEntry { sequence: "Ctrl+Shift+A", description: "Sélectionne toutes les colonnes colorées dans le menu de coloration des colonnes." },
                ShortcutEntry { sequence: "Ctrl+O", description: "Active ou désactive le tri automatique des lignes." },
            ],
            "it_IT" => vec![
                ShortcutEntry { sequence: "F1", description: "Apre la finestra Aiuto con l'elenco delle scorciatoie disponibili." },
                ShortcutEntry { sequence: "Ctrl+M", description: "Apre il manuale utente." },
                ShortcutEntry { sequence: "Ctrl+I", description: "Apre la finestra Informazioni." },
                ShortcutEntry { sequence: "Ctrl+Shift+O", description: "Apre la finestra per selezionare la cartella monitorata." },
                ShortcutEntry { sequence: "Ctrl+R", description: "Avvia o interrompe l'analisi della cartella." },
                ShortcutEntry { sequence: "Ctrl+P", description: "Mette in pausa o riprende il monitoraggio o la scansione." },
                ShortcutEntry { sequence: "Ctrl+Shift+S", description: "Apre la finestra Salva con nome." },
                ShortcutEntry { sequence: "Ctrl+S", description: "Salva i dati correnti." },
                ShortcutEntry { sequence: "Ctrl+G", description: "Apre la finestra delle statistiche." },
                ShortcutEntry { sequence: "Ctrl+L", description: "Cancella i dati caricati." },
                ShortcutEntry { sequence: "Ctrl+Q", description: "Chiude l'applicazione." },
                ShortcutEntry { sequence: "Ctrl+A", description: "Seleziona tutte le colonne nel menu di configurazione delle colonne." },
                ShortcutEntry { sequence: "Ctrl+Shift+A", description: "Seleziona tutte le colonne colorate nel menu di colorazione delle colonne." },
                ShortcutEntry { sequence: "Ctrl+O", description: "Attiva o disattiva l'ordinamento automatico delle righe." },
            ],
            "de_DE" => vec![
                ShortcutEntry { sequence: "F1", description: "Öffnet das Hilfefenster mit der Liste der verfügbaren Tastenkürzel." },
                ShortcutEntry { sequence: "Strg+M", description: "Öffnet das Benutzerhandbuch." },
                ShortcutEntry { sequence: "Strg+I", description: "Öffnet das Fenster Über." },
                ShortcutEntry { sequence: "Strg+Shift+O", description: "Öffnet den Dialog zum Auswählen des überwachten Verzeichnisses." },
                ShortcutEntry { sequence: "Strg+R", description: "Startet oder beendet die Verzeichnisanalyse." },
                ShortcutEntry { sequence: "Strg+P", description: "Pausiert oder setzt Überwachung bzw. Scan fort." },
                ShortcutEntry { sequence: "Strg+Shift+S", description: "Öffnet den Dialog Speichern unter." },
                ShortcutEntry { sequence: "Strg+S", description: "Speichert die aktuellen Daten." },
                ShortcutEntry { sequence: "Strg+G", description: "Öffnet das Statistikfenster." },
                ShortcutEntry { sequence: "Strg+L", description: "Löscht die geladenen Daten." },
                ShortcutEntry { sequence: "Strg+Q", description: "Beendet die Anwendung." },
                ShortcutEntry { sequence: "Strg+A", description: "Wählt alle Spalten im Menü zur Spaltenkonfiguration aus." },
                ShortcutEntry { sequence: "Strg+Shift+A", description: "Wählt alle farbigen Spalten im Menü zur Spaltenfärbung aus." },
                ShortcutEntry { sequence: "Strg+O", description: "Aktiviert oder deaktiviert die automatische Zeilensortierung." },
            ],
            _ => vec![
                ShortcutEntry { sequence: "F1", description: "Abre a janela Ajuda com a lista de atalhos disponíveis." },
                ShortcutEntry { sequence: "Ctrl+M", description: "Abre o Manual de Utilização." },
                ShortcutEntry { sequence: "Ctrl+I", description: "Abre a janela Sobre." },
                ShortcutEntry { sequence: "Ctrl+Shift+O", description: "Abre o diálogo para selecionar o diretório monitorado." },
                ShortcutEntry { sequence: "Ctrl+R", description: "Inicia ou interrompe a análise do diretório." },
                ShortcutEntry { sequence: "Ctrl+P", description: "Pausa ou retoma o monitoramento ou escaneamento." },
                ShortcutEntry { sequence: "Ctrl+Shift+S", description: "Abre o diálogo Salvar como." },
                ShortcutEntry { sequence: "Ctrl+S", description: "Salva os dados atuais." },
                ShortcutEntry { sequence: "Ctrl+G", description: "Abre a janela de estatísticas." },
                ShortcutEntry { sequence: "Ctrl+L", description: "Limpa os dados carregados." },
                ShortcutEntry { sequence: "Ctrl+Q", description: "Encerra a aplicação." },
                ShortcutEntry { sequence: "Ctrl+A", description: "Seleciona todas as colunas no menu de configuração de colunas." },
                ShortcutEntry { sequence: "Ctrl+Shift+A", description: "Seleciona todas as colunas coloridas no menu de coloração de colunas." },
                ShortcutEntry { sequence: "Ctrl+O", description: "Ativa ou desativa a ordenação automática das linhas." },
            ],
        }
    }
}
