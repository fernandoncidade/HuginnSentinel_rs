// Versão: 2026.11.11.0 | Data: 11 de novembro de 2026
// Version: 2026.11.11.0 | Date: November 11, 2026

#[derive(Debug, Clone)]
pub struct ManualDetails {
    pub summary: String,
    pub paragraphs: Vec<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ManualSection {
    pub id: String,
    pub title: String,
    pub paragraphs: Vec<String>,
    pub bullets: Vec<String>,
    pub details: Vec<ManualDetails>,
}

pub fn obter_secoes_manual(lang: &str) -> Vec<ManualSection> {
    match lang {
        "en_US" => doc_en_us(),
        "es_ES" => doc_es_es(),
        "fr_FR" => doc_fr_fr(),
        "it_IT" => doc_it_it(),
        "de_DE" => doc_de_de(),
        _ => doc_pt_br(),
    }
}

pub fn doc_pt_br() -> Vec<ManualSection> {
    vec![
        ManualSection {
            id: "novos-recursos".into(),
            title: "1. Novos Recursos e Arquitetura".into(),
            paragraphs: vec![
                "O Huginn Sentinel é uma plataforma desktop corporativa de alto desempenho para auditoria forense de integridade, monitoramento em tempo real do sistema de arquivos e detecção autônoma de ameaças no Windows.".into(),
                "A versão 2026.11.11.0 consolida a refatoração integral em Rust moderno (Edição 2021) com interface em egui 0.37 / eframe acelerada por GPU e persistência relacional SQLite3 em modo WAL.".into(),
                "Arquitetura Local-First / Sovereign Privacy: 100% das análises, cálculos de risco (0–100), harmonização de 58 colunas de metadados e bancos de dados ocorrem estritamente na máquina local, com zero telemetria, zero rede e sem IA externa.".into(),
            ],
            bullets: vec![
                "Menus e submenus persistentes por hover: a árvore de menus permanece aberta durante cliques e só fecha ao retirar o cursor da área do menu.".into(),
                "Planilha virtualizada em 4 quadrantes com fixação 2D arbitrária de quaisquer linhas e colunas.".into(),
                "Janelas Modeless nativas do Windows 11 com Z-Order dinâmico, controles de janela (🗕, 🗖/🗗, 🗙) e contador dinâmico traduzido no rodapé (1 Janela:, 2 Janelas:).".into(),
                "Sistema de Limpeza de Dados granular: Diálogo nativo com confirmações de segurança para purga seletiva de tabela da UI, banco de dados de eventos ou de segurança.".into(),
                "Bancos de dados organizados em pastas próprias (%LOCALAPPDATA%\\HuginnSentinel\\events e security) e inicialização centralizada na tela principal.".into(),
                "Utilitário de linha de comando autônomo (huginn-cli) para integração com scripts e esteiras CI/CD.".into(),
                "Central de Alertas com regras canônicas determinísticas (SEC-R01 a SEC-R06).".into(),
                "Exportação de evidências seladas em ZIP criptograficamente auditável com manifesto JSON.".into(),
            ],
            details: vec![
                ManualDetails {
                    summary: "Fluxo de Trabalho Recomendado".into(),
                    paragraphs: vec![
                        "1) Selecione o diretório raiz a ser monitorado com Ctrl+Shift+O ou pelo botão da barra de ferramentas.".into(),
                        "2) Execute a varredura inicial completa (Ctrl+R) para indexar a árvore de diretórios existente.".into(),
                        "3) Inicie o monitoramento em tempo real para capturar eventos de modificação, adição, remoção, renomeação e movimentação.".into(),
                    ],
                    bullets: vec![],
                },
            ],
        },
        ManualSection {
            id: "monitoramento".into(),
            title: "2. Monitoramento em Tempo Real".into(),
            paragraphs: vec![
                "O módulo de monitoramento emprega as APIs nativas do kernel do Windows (ReadDirectoryChangesW) desacopladas via canais assíncronos crossbeam para evitar perda de eventos e garantir fluidez da interface.".into(),
                "Eventos capturados: Arquivo Adicionado, Arquivo Modificado, Arquivo Excluído, Arquivo Renomeado e Arquivo Movido.".into(),
            ],
            bullets: vec![
                "Pausar e Retomar (Ctrl+P): Permite congelar a atualização da tabela temporariamente para inspeção tranquila das linhas.".into(),
                "Contadores em tempo real: O rodapé e o painel lateral atualizam os totais de eventos continuamente sem travar a interface.".into(),
                "Filtragem sem limites: Todos os eventos registrados são mantidos na tabela e no banco sem limitação artificial.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "escaneamento".into(),
            title: "3. Varredura e Escaneamento Recursivo".into(),
            paragraphs: vec![
                "A varredura recursiva (Ctrl+R) percorre todo o diretório selecionado em segundo plano através de threads assíncronas dedicadas.".into(),
                "A barra de progresso no rodapé exibe a porcentagem concluída, quantidade de itens processados e caminho do item atual.".into(),
            ],
            bullets: vec![
                "Cancelamento Limpo: Clicar no botão 'Parar' interrompe a varredura graciosamente sem travar nem fechar o aplicativo.".into(),
                "Multithreading Eficiente: O processamento de metadados, identificação por magic numbers e cálculo de hashes ocorre em pool assíncrono.".into(),
                "Fluxo Interativo de Encadeamento: Diálogos inteligentes perguntam se deseja monitorar após escanear ou escanear antes de monitorar, com opções configuráveis em Configurações > Escaneamento e Monitoramento.".into(),
                "Isolamento de Varredura: Durante o escaneamento inicial, o observador em tempo real é suspenso para registrar unicamente eventos do tipo Escaneado sem ruído concorrente.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "filtros-avancados".into(),
            title: "4. Filtros Avançados e Busca".into(),
            paragraphs: vec![
                "O sistema de filtros avançados permite isolar eventos por múltiplos critérios combinados.".into(),
                "Acesse a janela de Filtros Avançados pelo menu Configurações > Filtros > Filtros Avançados ou pelo botão no painel lateral esquerdo.".into(),
            ],
            bullets: vec![
                "Filtro de Operações: Ative ou desative individualmente Movido, Renomeado, Adicionado, Excluído, Modificado e Escaneado.".into(),
                "Filtro de Extensão: Digite extensões separadas por vírgula (ex: pdf, docx, exe) ou clique nos chips predefinidos.".into(),
                "Filtro por Data e Hora: Defina Data Inicial e Data Final com predefinições de 7 dias, 30 dias ou Hoje.".into(),
                "Flags de Exceção de Data: Permite ignorar o filtro de data individualmente para arquivos movidos, renomeados, adicionados, excluídos, modificados ou escaneados.".into(),
                "Filtro por Risco: Filtre por nível de ameaça (Crítico >= 85, Alto >= 60, Médio >= 30, Baixo < 30).".into(),
                "Total de Eventos Dinâmico: A contagem no painel lateral soma em tempo real exclusivamente os eventos das caixas de operações selecionadas.".into(),
                "Botão Limpar Filtros: Restaura todos os filtros para o estado padrão com um único clique.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "colunas".into(),
            title: "5. Planilha Virtualizada e Fixação 2D de Linhas e Colunas".into(),
            paragraphs: vec![
                "A tabela virtual oferece até 58 colunas detalhadas cobrindo metadados forenses, atributos NTFS, hashes criptográficos e dados de execução PE com harmonização determinística estrita.".into(),
                "O sistema inclui fixação bidimensional de linhas e colunas (Freeze Panes em 4 quadrantes), permitindo manter cabeçalhos e colunas de referência sempre visíveis durante a rolagem.".into(),
            ],
            bullets: vec![
                "Fixar Linhas (Configurações > 📌 Fixar Linhas): Fixe a 1ª linha de cabeçalhos, a linha selecionada, intervalos numéricos (ex: 1, 3, 5-8) ou marque linhas específicas no submenu.".into(),
                "Fixar Colunas (Configurações > 📌 Fixar Colunas): Fixe a coluna #, a coluna ativa, todas as visíveis ou selecione colunas no submenu com checkboxes.".into(),
                "Comportamento de Seleção (Configurações > Comportamento da Seleção): Alterne entre manter a seleção estática na posição física ou acompanhar os dados e novas linhas.".into(),
                "Personalizar Cor de Seleção: Escolha entre presets recomendados (com contraste automático para tema claro e escuro) ou ajuste livre com código HTML editável (#00ff00) e pré-visualização comparativa.".into(),
                "Seleção por Arraste Contínuo: Clique e segure com o botão esquerdo sobre células, cabeçalhos de colunas ou cabeçalhos de linhas e arraste para selecionar blocos contínuos com realce visual em tempo real.".into(),
                "Botão Selecionar Tudo: Clique no canto superior esquerdo para selecionar toda a planilha preservando perfeitamente as bordas e divisórias dos cabeçalhos.".into(),
                "Menus de Contexto (Botão Direito): Fixe ou desafixe linhas e colunas diretamente clicando com o botão direito nas células ou cabeçalhos.".into(),
                "Persistência Automática: Suas preferências de fixação são salvas automaticamente em painel_fixo_config.json.".into(),
                "Selecionar Todas (Ctrl+A): Torna todas as 58 colunas visíveis instantaneamente.".into(),
                "Alternar Ordenação de Linhas (Ctrl+O): Habilita ou desabilita a ordenação automática ao clicar no cabeçalho da tabela.".into(),
                "Inverter Ordenação: O botão '🔄 Inverter' na grade alterna instantaneamente a ordenação ascendente/descendente para a coluna selecionada.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "cores".into(),
            title: "6. Configuração e Personalização de Cores".into(),
            paragraphs: vec![
                "Cada tipo de operação possui cor de fundo e texto customizáveis para máxima legibilidade visual.".into(),
                "No menu Configurações > Configurar Cores, você pode definir esquemas de cores específicos para tema Claro e tema Escuro.".into(),
            ],
            bullets: vec![
                "Colorir Colunas Específicas: Permite escolher quais colunas recebem o realce colorido das operações.".into(),
                "Selecionar Todas as Cores (Ctrl+Shift+A): Aplica o realce colorido a todas as colunas visíveis.".into(),
                "Contraste Automático: O sistema calcula a luminância de fundo para garantir que o texto seja legível (preto ou branco).".into(),
                "Paleta Interativa com Código HTML Editável: Digite ou cole qualquer código de cor (#00ff00) com sincronização em tempo real, botão de fechamento ❌ e botões U8 e 📋 centralizados verticalmente.".into(),
                "Gerenciador de Colunas Simplificado: Janela em lista única com botões de seleção em lote para maior agilidade.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "seguranca".into(),
            title: "7. Segurança, Baseline e Quarentena".into(),
            paragraphs: vec![
                "O módulo de segurança atua de forma preventiva contra ameaças cibernéticas locais (ransomware, binários camuflados, scripts não autorizados e desvios de integridade).".into(),
            ],
            bullets: vec![
                "Baseline Criptográfico SHA-256: Registra hashes em SQLite WAL e alerta sobre discrepâncias e deriva de conformidade.".into(),
                "Quarentena Segura: Isola arquivos suspeitos em %LOCALAPPDATA%\\HuginnSentinel\\quarantine\\ com extensão neutra .quar e custódia em banco de dados.".into(),
                "Restauração Validação SHA-256: Permite restaurar um arquivo da quarentena somente se o hash conferir perfeitamente com o original gravado.".into(),
                "Regras Canônicas SEC-R01 a SEC-R06: Detecção determinística de ransomware, extensão dupla, executáveis não assinados e scripts perigosos.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "estatisticas".into(),
            title: "8. Gráficos Estatísticos (17 Gráficos Nativos)".into(),
            paragraphs: vec![
                "Pressione Ctrl+G ou acesse Opções > Estatísticas para abrir a central visual com 17 gráficos estatísticos interativos acelerados na GPU via egui::Painter.".into(),
            ],
            bullets: vec![
                "Distribuição por Tipo de Operação (Pizza e Barras — visual limpo sem linhas divisórias indesejadas).".into(),
                "Histograma de Tamanhos de Arquivo (KB / MB).".into(),
                "Linha Temporal e Mapa de Calor 7x24 de Atividade (com dias da semana e escalas traduzidos).".into(),
                "Controles Interativos em Diagramas Sankey: Filtros por caixas de seleção (checkboxes) para os tipos de operação e seletor de limite de itens plotados (10 a 100 e Todos) com rolagem fluida e fitas Bézier estáveis sem oscilações.".into(),
                "Top Extensões Mais Frequentes e Top Diretórios com Maior Atividade (com seletor de quantidade máxima).".into(),
                "Separação Estrita de Renomeado e Movido: Eventos de renomeação e movimentação são auditados e plotados separadamente na Linha do Tempo, Radar de Risco e Sankey.".into(),
                "Dispersão de Entropia Forense de Shannon (identificação de compactação ou cifragem ransomware) e Dotplots volumétricos.".into(),
                "Gráfico de Risco e Severidade de Ameaças, Séries Temporais (legendas centralizadas) e Boxplots Tukey.".into(),
                "Adaptação ao Tema Windows 11: Todos os gráficos alternam instantaneamente entre modo Claro e Escuro com paletas otimizadas de contraste e sem necessidade de reiniciar.".into(),
                "Exportação em Alta Resolução: Exporte qualquer gráfico individualmente ou em lote para PNG.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "exportacao".into(),
            title: "9. Exportação de Relatórios e Auditoria".into(),
            paragraphs: vec![
                "Exporte seus relatórios de auditoria com integridade completa em múltiplos formatos: CSV, JSON, HTML, TXT e XML, além de pacotes de evidência selados em ZIP.".into(),
            ],
            bullets: vec![
                "Salvar Como (Ctrl+Shift+S): Permite escolher o nome, pasta e formato de arquivo do relatório.".into(),
                "Salvar Rápido (Ctrl+S): Atualiza o último arquivo salvo com os novos eventos registrados.".into(),
                "Nomenclatura Padrão: Relatórios exportados utilizam o padrão oficial huginn_eventos_AAAAMMDD_HHMMSS.".into(),
                "Pacote ZIP Selado: Gera pacote forense com relatório, manifesto JSON e hashes SHA-256 de custódia.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "atalhos".into(),
            title: "10. Teclas de Atalho e Produtividade".into(),
            paragraphs: vec![
                "Acesse a janela de Ajuda completa pressionando F1 a qualquer momento.".into(),
            ],
            bullets: vec![
                "F1: Abre a janela de Ajuda com todas as teclas de atalho.".into(),
                "Ctrl+M: Abre o Manual do Usuário interativo.".into(),
                "Ctrl+I: Abre a janela Sobre o Huginn Sentinel.".into(),
                "Ctrl+Shift+O: Seleciona o diretório raiz para análise.".into(),
                "Ctrl+R: Inicia ou para a varredura recursiva.".into(),
                "Ctrl+P: Pausa ou retoma o monitoramento em tempo real.".into(),
                "Ctrl+S: Salva o relatório no formato atual.".into(),
                "Ctrl+Shift+S: Abre a caixa de diálogo Salvar Como...".into(),
                "Ctrl+G: Abre a Central de Gráficos e Estatísticas (17 gráficos).".into(),
                "Ctrl+L: Abre o Sistema de Limpeza de Dados (limpeza granular de tabela, banco de eventos e banco de segurança).".into(),
                "Ctrl+A: Seleciona e exibe todas as 58 colunas da tabela.".into(),
                "Ctrl+Shift+A: Seleciona todas as cores de colunas para realce.".into(),
                "Ctrl+O: Alterna a ordenação de linhas ao clicar no cabeçalho.".into(),
                "Ctrl+Q: Encerra o aplicativo com segurança.".into(),
            ],
            details: vec![],
        },
    ]
}

pub fn doc_en_us() -> Vec<ManualSection> {
    vec![
        ManualSection {
            id: "new-features".into(),
            title: "1. New Features & Architecture".into(),
            paragraphs: vec![
                "Huginn Sentinel is a high-performance Windows desktop platform for file integrity auditing, continuous real-time monitoring, and proactive local threat detection.".into(),
                "Version 2026.11.11.0 consolidates the complete rewrite in modern Rust (Edition 2021) with an egui 0.37 / eframe GPU immediate mode interface and SQLite3 WAL persistence.".into(),
                "Local-First Sovereign Privacy Architecture: 100% of telemetry, risk analytics (0–100), 58-column metadata harmonization, and databases reside exclusively on your local machine with zero network exposure and zero external AI.".into(),
            ],
            bullets: vec![
                "Persistent hover-open menu tree: menus and submenus remain open across clicks and automatically close only when the pointer leaves the menu area.".into(),
                "4-quadrant virtualized spreadsheet with arbitrary 2D row and column pinning (Freeze Panes).".into(),
                "Native Windows 11 modeless dialogs with dynamic Z-Order, standard controls (🗕, 🗖/🗗, 🗙), and live localized footer window counter (1 Window:, 2 Windows:).".into(),
                "Granular Data Cleanup System: Native modeless dialog with safety confirmations for selective purging of UI table, events database, or security database.".into(),
                "Isolated Database Storage: Databases stored in dedicated subdirectories (%LOCALAPPDATA%\\HuginnSentinel\\events and security) and ergonomic centered startup positioning.".into(),
                "Autonomous CLI utility (huginn-cli) for automation scripts and CI/CD pipelines.".into(),
                "Alerts center with canonical deterministic rules (SEC-R01 to SEC-R06).".into(),
                "Sealed cryptographic ZIP evidence export with integrity JSON manifest.".into(),
            ],
            details: vec![
                ManualDetails {
                    summary: "Recommended Workflow".into(),
                    paragraphs: vec![
                        "1) Select the root directory to monitor via Ctrl+Shift+O or toolbar button.".into(),
                        "2) Run an initial recursive scan (Ctrl+R) to catalog the existing directory tree.".into(),
                        "3) Start real-time monitoring to capture added, modified, deleted, renamed, and moved file events.".into(),
                    ],
                    bullets: vec![],
                },
            ],
        },
        ManualSection {
            id: "monitoring".into(),
            title: "2. Real-Time Monitoring".into(),
            paragraphs: vec![
                "The monitoring engine leverages native Windows kernel APIs (ReadDirectoryChangesW) decoupled via asynchronous crossbeam channels to prevent event drops and keep the UI responsive.".into(),
                "Captured event types: Added, Modified, Deleted, Renamed, and Moved files.".into(),
            ],
            bullets: vec![
                "Pause & Resume (Ctrl+P): Freezes table rendering temporarily for row inspection.".into(),
                "Real-time counters: Footer and sidebar counters update live without UI stuttering.".into(),
                "Unbounded retention: All recorded events are preserved without artificial truncations.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "scanning".into(),
            title: "3. Recursive Directory Scanning".into(),
            paragraphs: vec![
                "Recursive scanning (Ctrl+R) traverses the selected target directory using dedicated background threads.".into(),
                "The status footer progress bar displays completion percentage, processed item count, and active file path.".into(),
            ],
            bullets: vec![
                "Graceful Stop: Clicking 'Stop' cancels scanning gracefully without hanging or closing the app.".into(),
                "High-Throughput Multithreading: Metadata inspection, magic number detection, and hashing execute in an async pool.".into(),
                "Interactive Workflow Chaining: Prompts ask whether to start monitoring after scan or scan before monitor, configurable in Settings > Scanning and Monitoring.".into(),
                "Scan Isolation: During recursive scanning, real-time watcher events are paused so only 'Scanned' events are recorded.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "advanced-filters".into(),
            title: "4. Advanced Filters & Search".into(),
            paragraphs: vec![
                "Filter and isolate events through combined operations, extensions, risk ratings, and date ranges.".into(),
                "Open the Advanced Filters window via Settings > Filters > Advanced Filters or the left sidebar button.".into(),
            ],
            bullets: vec![
                "Operation Filters: Toggle Moved, Renamed, Added, Deleted, Modified, Scanned.".into(),
                "Extension Filter: Comma-separated list (e.g., pdf, docx, exe) or quick preset chips.".into(),
                "Date Filter: Set start and end dates with 7-day, 30-day, or Today presets.".into(),
                "Date Exception Flags: Ignore date filters individually for moved, renamed, added, deleted, modified, or scanned files.".into(),
                "Risk Rating Filter: Filter by severity (Critical >= 85, High >= 60, Medium >= 30, Low < 30).".into(),
                "Dynamic Total Events Counter: Sidebar total reflects the exact live sum of selected operation checkboxes.".into(),
                "Reset Button: Restores default filter parameters with one click.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "columns".into(),
            title: "5. Virtualized Spreadsheet & 2D Row/Column Pinning".into(),
            paragraphs: vec![
                "Virtual data table with up to 58 forensic columns covering NTFS attributes, security hashes, and PE headers with deterministic harmonization.".into(),
                "Features arbitrary 2D row and column pinning (4-quadrant freeze panes) to keep critical headers and identifier columns visible during scrolling.".into(),
            ],
            bullets: vec![
                "Pin Rows (Settings > 📌 Pin Rows): Freeze 1st row of column headers, selected row, numeric ranges (e.g., 1, 3, 5-8), or toggle specific rows.".into(),
                "Pin Columns (Settings > 📌 Pin Columns): Freeze row numbers column #, active column, all visible columns, or toggle columns in the submenu.".into(),
                "Selection Behavior (Settings > Selection Behavior): Choose between static cell/row positions or follow selection mode on sorting/updates.".into(),
                "Customize Cell Selection Color: Choose recommended presets (with adaptive contrast for Windows 11 Dark and Light themes) or custom RGB/HSV with editable HTML (#00ff00).".into(),
                "Continuous Drag Selection: Click and hold on cells, column headers, or row headers and drag to select contiguous blocks with instant real-time visual highlighting.".into(),
                "Select All Button: Click the top-left corner to select the entire grid while preserving header dividers and cell borders.".into(),
                "Context Menus (Right-Click): Pin and unpin rows or columns directly by right-clicking on cells or headers.".into(),
                "Persistent Preferences: Freezing layouts are saved automatically to painel_fixo_config.json.".into(),
                "Select All Columns (Ctrl+A): Instantly exposes all 58 columns.".into(),
                "Toggle Row Sorting (Ctrl+O): Enable or disable column header sorting.".into(),
                "Invert Sort Order: The '🔄 Invert' button on the grid toggles ascending/descending order for the selected column.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "colors".into(),
            title: "6. Color & Highlighting Configuration".into(),
            paragraphs: vec![
                "Customize operation background and text colors for light and dark themes.".into(),
                "Open Settings > Configure Colors to define custom color palettes.".into(),
            ],
            bullets: vec![
                "Colorize specific columns or all columns (Ctrl+Shift+A).".into(),
                "Automated contrast calculation ensures crisp black or white text readability.".into(),
                "Interactive Palette with Editable HTML: Type or paste any hex color (#00ff00) with live synchronization, '❌' close button, and centered U8 / 📋 actions.".into(),
                "Streamlined Column Manager: Unified single-column list with batch actions for rapid column visibility management.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "security".into(),
            title: "7. Security, Baseline & Quarantine".into(),
            paragraphs: vec![
                "Proactive protection against ransomware, trojans, script drops, unauthorized alterations, and compliance drift.".into(),
            ],
            bullets: vec![
                "Cryptographic SHA-256 Baseline: Records hashes in SQLite WAL and alerts on drift.".into(),
                "Safe Quarantine: Isolates malicious files safely to %LOCALAPPDATA%\\HuginnSentinel\\quarantine\\ with inert .quar extension.".into(),
                "Verified Restoration: Verifies SHA-256 integrity before restoring any file from quarantine.".into(),
                "Canonical Rules SEC-R01 to SEC-R06: Deterministic heuristic detection of ransomware, double extensions, and unverified executables.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "statistics".into(),
            title: "8. Statistical Analytics (17 Native Charts)".into(),
            paragraphs: vec![
                "Press Ctrl+G or select Options > Statistics to view 17 interactive statistical charts rendered natively via egui::Painter.".into(),
            ],
            bullets: vec![
                "Operation Distribution (Pie & Bar charts — clean view without center divider lines).".into(),
                "File Size Distribution Histograms (KB / MB).".into(),
                "7x24 Hourly and Daily Activity Heatmap (with localized weekdays and time units).".into(),
                "Sankey Interactive Controls: Checkbox-based operation filtering and destination item limiters (10 to 100 and All) with smooth scrollbars and deterministic Bézier ribbon stability.".into(),
                "Most Active File Extensions and Top Directory Mutations (with configurable item limiters).".into(),
                "Strict Renamed vs. Moved Distinction: Renamed and Moved events are tracked and reported separately in Timelines, Risk Radar, and Sankey diagrams.".into(),
                "Shannon Forensic Entropy Scatter (detecting compression or ransomware encryption) and volumetric Dotplots.".into(),
                "Threat Severity Breakdown, Time Series (centered legends), and Tukey Statistical Boxplots.".into(),
                "Windows 11 Light/Dark Theme Adaptation: All charts automatically adapt their color contrast, cards, grids, and axes to the active system theme in real time without restart.".into(),
                "High-DPI PNG Export: Export single charts or batch export all 17 charts with one click.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "export".into(),
            title: "9. Report & Evidence Export".into(),
            paragraphs: vec![
                "Export complete audit trails to CSV, JSON, HTML, TXT, or XML, plus cryptographically sealed ZIP evidence bundles.".into(),
            ],
            bullets: vec![
                "Save As (Ctrl+Shift+S) and Quick Save (Ctrl+S).".into(),
                "Standardized File Naming: Exported files automatically follow the structured huginn_eventos_YYYYMMDD_HHMMSS prefix.".into(),
                "Sealed ZIP Evidence: Generates an auditable ZIP package with report data, JSON manifest, and SHA-256 custody hashes.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "shortcuts".into(),
            title: "10. Keyboard Shortcuts & Productivity".into(),
            paragraphs: vec![
                "Press F1 at any time to open the complete shortcuts dialog.".into(),
            ],
            bullets: vec![
                "F1: Help | Ctrl+M: Manual | Ctrl+I: About | Ctrl+Shift+O: Select Directory.".into(),
                "Ctrl+R: Start/Stop Scan | Ctrl+P: Pause/Resume | Ctrl+S: Save | Ctrl+Shift+S: Save As.".into(),
                "Ctrl+G: Charts (17 native charts) | Ctrl+L: Data Cleanup System (granular purging of table, events database, or security database) | Ctrl+A: Select All Columns | Ctrl+O: Toggle Sorting | Ctrl+Q: Exit.".into(),
            ],
            details: vec![],
        },
    ]
}

pub fn doc_es_es() -> Vec<ManualSection> {
    vec![
        ManualSection {
            id: "nuevas-funciones".into(),
            title: "1. Nuevas Funciones y Arquitectura".into(),
            paragraphs: vec![
                "Huginn Sentinel es una plataforma de escritorio corporativa de alto rendimiento para auditoría forense de integridad, monitoreo en tiempo real del sistema de archivos de Windows y detección autónoma de amenazas.".into(),
                "La versión 2026.11.11.0 consolida la refactorización completa en Rust moderno (Edición 2021) con interfaz en egui 0.37 / eframe acelerada por GPU y almacenamiento relacional SQLite3 en modo WAL.".into(),
                "Arquitectura Local-First y Privacidad Soberana: el 100% de los análisis, puntuaciones de riesgo (0–100) y bases de datos permanecen exclusivamente en la máquina local sin telemetría ni IA externa.".into(),
            ],
            bullets: vec![
                "Árbol de menús persistentes por hover: los menús permanecen abiertos al hacer clic y se cierran solo al retirar el cursor del área del menú.".into(),
                "Tabla virtualizada en 4 cuadrantes con fijación bidimensional arbitraria de filas y columnas.".into(),
                "Ventanas modeless independientes estilo Windows 11 con Z-Order dinámico y contador dinámico traducido en el pie de página (1 Ventana:, 2 Ventanas:).".into(),
                "Sistema de Limpieza de Datos granular: Diálogo nativo con confirmaciones de seguridad para purga selectiva de tabla, base de datos de eventos o de seguridad.".into(),
                "Bases de datos en carpetas individuales (%LOCALAPPDATA%\\HuginnSentinel\\events y security) e inicio centrado en pantalla principal.".into(),
                "Utilidad CLI autónoma (huginn-cli) para scripts de automatización e integración continua.".into(),
                "Centro de Alertas con reglas canónicas deterministas (SEC-R01 a SEC-R06).".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "monitoreo".into(),
            title: "2. Monitoreo en Tiempo Real".into(),
            paragraphs: vec![
                "Monitoreo continuo de eventos de archivos (Añadido, Modificado, Eliminado, Renombrado y Movido) mediante APIs nativas de Windows (ReadDirectoryChangesW) con búferes asíncronos crossbeam.".into(),
            ],
            bullets: vec![
                "Pausa y Reanudación (Ctrl+P) para inspección tranquila de registros.".into(),
                "Contadores en tiempo real en la barra de estado y panel lateral.".into(),
                "Registro sin límites artificiales de eventos en memoria y base de datos.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "escaneo".into(),
            title: "3. Varredura y Escaneo Recursivo".into(),
            paragraphs: vec![
                "El escaneo recursivo (Ctrl+R) recorre el directorio seleccionado mediante hilos de trabajo asíncronos.".into(),
            ],
            bullets: vec![
                "Detención limpia: El botón 'Detener' pausa el escaneo sin cerrar la aplicación.".into(),
                "Extracción determinista de metadados, números mágicos y hashes SHA-256.".into(),
                "Encadenamiento Interactivo de Flujos: Diálogos que preguntan si desea iniciar el monitoreo tras escanear o escanear antes de monitorear, configurables en Configuración > Escaneo y Monitoreo.".into(),
                "Aislamiento del Escaneo: Durante el escaneo recursivo, los eventos del observador en tiempo real se pausan para registrar estrictamente solo eventos de tipo 'Escaneado'.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "filtros".into(),
            title: "4. Filtros Avanzados y Búsqueda".into(),
            paragraphs: vec![
                "Ventana de filtros avanzados con filtros por tipo de operación, extensión, severidad de riesgo y rangos de fechas con excepciones configurables.".into(),
            ],
            bullets: vec![
                "Filtros de operación: active o desactive Añadido, Modificado, Eliminado, Renombrado, Movido o Escaneado.".into(),
                "Banderas de excepción para omitir el filtro de fecha por operación.".into(),
                "Contador Dinámico de Total de Eventos: El total en el panel lateral refleja dinámicamente la suma exacta de las casillas de operación seleccionadas.".into(),
                "Botón Restablecer Filtros con un solo clic.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "columnas".into(),
            title: "5. Tabla Virtualizada y Fijación 2D de Filas y Columnas".into(),
            paragraphs: vec![
                "Tabla virtual de 58 columnas con metadados forenses y fijación bidimensional (Freeze Panes en 4 cuadrantes).".into(),
            ],
            bullets: vec![
                "Fijar Filas (Configuración > 📌 Fijar Filas): Fije la 1ª fila de cabeceras, la fila activa o rangos (ej: 1, 3, 5-8).".into(),
                "Fijar Columnas (Configuración > 📌 Fijar Columnas): Fije la columna #, la columna activa o todas las visibles.".into(),
                "Comportamiento de Selección (Configuración > Comportamiento de Selección): Elija entre mantener posiciones estáticas de celdas/filas o acompañar la selección durante ordenaciones/actualizaciones.".into(),
                "Personalizar Color de Selección de Celdas: Elija paletas recomendadas (con contraste adaptativo de texto para temas Claro y Oscuro de Windows 11) o defina colores personalizados RGB/HSV con código HTML (#00ff00) editable.".into(),
                "Selección por Arrastre Continuo: Haga clic y mantenga presionado sobre celdas o cabeceras para seleccionar bloques con resaltado visual en tiempo real.".into(),
                "Botón Seleccionar Todo: Selecciona toda la cuadrícula preservando divisores y bordes.".into(),
                "Menús contextuales: Clic derecho en celdas o cabeceras para fijar o desfijar.".into(),
                "Persistencia automática en painel_fixo_config.json.".into(),
                "Seleccionar Todas las Columnas (Ctrl+A) y Alternar Ordenación (Ctrl+O).".into(),
                "Invertir Ordenación: El botón '🔄 Invertir' en la cuadrícula invierte el orden ascendente/descendente de la columna seleccionada.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "colores".into(),
            title: "6. Personalización de Colores".into(),
            paragraphs: vec![
                "Configuración de colores de fondo y texto para cada operación en temas Claro y Oscuro.".into(),
            ],
            bullets: vec![
                "Colorear columnas específicas o todas las columnas (Ctrl+Shift+A).".into(),
                "Cálculo automático de contraste de texto.".into(),
                "Paleta Interactiva con HTML Editable: Ingrese o pegue cualquier código de color en formato HTML (#00ff00) con sincronización en tiempo real, botón de cierre '❌' y botones U8 y 📋 centrados verticalmente.".into(),
                "Administrador Simplificado de Columnas: Diseño unificado en lista única con selección en lote para gestión ágil de visibilidad de columnas.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "seguridad".into(),
            title: "7. Seguridad, Línea de Base y Cuarentena".into(),
            paragraphs: vec![
                "Protección activa contra ransomware, binarios no autorizados y desviaciones de integridad.".into(),
            ],
            bullets: vec![
                "Línea de base criptográfica SHA-256 en SQLite WAL.".into(),
                "Cuarentena segura en carpeta protegida con extensión .quar y custodia en base de datos.".into(),
                "Restauración validada estrictamente por hash SHA-256.".into(),
                "Reglas canónicas SEC-R01 a SEC-R06.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "estadisticas".into(),
            title: "8. 17 Gráficos Estadísticos Nativos".into(),
            paragraphs: vec![
                "Presione Ctrl+G o acceda a Opciones > Estadísticas para ver 17 gráficos estadísticos acelerados en GPU.".into(),
            ],
            bullets: vec![
                "Distribución de operaciones (gráfico de sectores limpio), histogramas de tamaño y mapa de calor 7x24.".into(),
                "Controles Interactivos en Diagramas Sankey: Filtro por casillas de verificación (checkboxes) para operaciones y selector de límite de elementos (10 a 100 y Todos) con barra de desplazamiento y cintas Bézier estables.".into(),
                "Separación Estricta de Renombrado y Movido: Reporte separado en Línea de Tiempo, Radar de Riesgo y Sankey.".into(),
                "Dispersión de entropía de Shannon, diagrama de puntos volumétrico y series temporales.".into(),
                "Adaptación a Temas de Windows 11: Ajuste dinámico automático entre tema Claro y Oscuro en tiempo de ejecución sin reiniciar.".into(),
                "Exportación de gráficos en alta resolución a PNG.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "exportacion".into(),
            title: "9. Exportación de Informes y Evidencias".into(),
            paragraphs: vec![
                "Exportación de auditorías en CSV, JSON, HTML, TXT, XML y paquetes ZIP sellados.".into(),
            ],
            bullets: vec![
                "Guardar Como (Ctrl+Shift+S) y Guardar Rápido (Ctrl+S).".into(),
                "Nomenclatura Oficial: Los informes exportados adoptan el patrón huginn_eventos_AAAAMMDD_HHMMSS.".into(),
                "Paquete de evidencia ZIP con manifiesto JSON y hashes SHA-256.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "atajos".into(),
            title: "10. Teclas de Acceso Rápido".into(),
            paragraphs: vec![
                "Presione F1 en cualquier momento para ver la lista completa de atajos de teclado.".into(),
            ],
            bullets: vec![
                "F1: Ayuda | Ctrl+M: Manual | Ctrl+I: Acerca de | Ctrl+Shift+O: Seleccionar Directorio.".into(),
                "Ctrl+R: Iniciar/Detener | Ctrl+P: Pausa/Reanudar | Ctrl+S: Guardar | Ctrl+Shift+S: Guardar Como.".into(),
                "Ctrl+G: Gráficos | Ctrl+L: Sistema de Limpieza de Datos (purga selectiva de tabla, base de datos o seguridad) | Ctrl+A: Todas las Columnas | Ctrl+O: Ordenar | Ctrl+Q: Salir.".into(),
            ],
            details: vec![],
        },
    ]
}

pub fn doc_fr_fr() -> Vec<ManualSection> {
    vec![
        ManualSection {
            id: "nouvelles-fonctions".into(),
            title: "1. Nouvelles Fonctionnalités et Architecture".into(),
            paragraphs: vec![
                "Huginn Sentinel est une plateforme de bureau haute performance pour l'audit d'intégrité, la surveillance en temps réel du système de fichiers Windows et la détection autonome des menaces.".into(),
                "La version 2026.11.11.0 consolide la réécriture intégrale en Rust moderne (Édition 2021) avec une interface GPU en egui 0.37 / eframe et stockage relationnel SQLite3 en mode WAL.".into(),
                "Architecture Sovereign Privacy Local-First : 100% des analyses, scores de risque (0–100) et bases de données restent strictement locaux, sans télémétrie ni IA externe.".into(),
            ],
            bullets: vec![
                "Arborescence de menus persistante au survol : les menus restent ouverts lors des clics et se ferment dès que le pointeur quitte la zone.".into(),
                "Tableau virtualisé à 4 quadrants avec gel bidimensionnel arbitraire des lignes et des colonnes.".into(),
                "Fenêtres modeless indépendantes style Windows 11 avec compteur dynamique traduit au pied de page (1 Fenêtre:, 2 Fenêtres:).".into(),
                "Système de Nettoyage de Données granulaire : Purge sélective des événements, base de sécurité ou tableau avec confirmations sécurisées.".into(),
                "Bases de données en dossiers dédiés (%LOCALAPPDATA%\\HuginnSentinel\\events et security) et démarrage centré sur l'écran principal.".into(),
                "Utilitaire CLI dédié (huginn-cli) pour l'automatisation et les pipelines CI/CD.".into(),
                "Centre d'alertes avec règles canoniques déterministes (SEC-R01 à SEC-R06).".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "surveillance".into(),
            title: "2. Surveillance en Temps Réel".into(),
            paragraphs: vec![
                "Surveillance continue via les API natives du noyau Windows (ReadDirectoryChangesW) et canaux asynchrones crossbeam.".into(),
            ],
            bullets: vec![
                "Pause et Reprise (Ctrl+P) pour inspecter sereinement les événements.".into(),
                "Compteurs d'événements en direct dans le panneau latéral et la barre d'état.".into(),
                "Enregistrement exhaustif des événements sans limitation arbitraire.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "balayage".into(),
            title: "3. Analyse et Balayage Récursif".into(),
            paragraphs: vec![
                "Le balayage récursif (Ctrl+R) parcourt le répertoire sélectionné via des threads en arrière-plan.".into(),
            ],
            bullets: vec![
                "Arrêt propre : Le bouton 'Arrêter' suspend l'analyse sans fermer l'application.".into(),
                "Extraction déterministe des métadonnées, nombres magiques et empreintes SHA-256.".into(),
                "Enchaînement Interactif des Flux : Dialogues demandant s'il faut démarrer la surveillance après l'analyse ou analyser avant de surveiller, configurables dans Paramètres > Analyse et Surveillance.".into(),
                "Isolation de l'Analyse : Pendant l'analyse récursive, les événements du surveillant en temps réel sont suspendus pour enregistrer strictement les événements de type 'Scanné'.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "filtres".into(),
            title: "4. Filtres Avancés et Recherche".into(),
            paragraphs: vec![
                "Dialogue de filtres avancés par opération, extension, niveau de risque et plages de dates avec exceptions configurables.".into(),
            ],
            bullets: vec![
                "Filtres d'opération : Ajouté, Modifié, Supprimé, Renommé, Déplacé et Scanné.".into(),
                "Options pour ignorer le filtre de date par opération.".into(),
                "Compteur Dynamique du Total d'Événements : Le total dans le panneau latéral reflète dynamiquement la somme exacte des cases d'opération cochées.".into(),
                "Réinitialisation des filtres en un seul clic.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "colonnes".into(),
            title: "5. Tableau Virtualisé et Gel 2D des Lignes/Colonnes".into(),
            paragraphs: vec![
                "Grille de 58 colonnes avec métadonnées médico-légales et gel bidimensionnel (Freeze Panes à 4 quadrants).".into(),
            ],
            bullets: vec![
                "Figer les Lignes (Paramètres > 📌 Figer les Lignes) : En-têtes, ligne active ou plages numériques (ex: 1, 3, 5-8).".into(),
                "Figer les Colonnes (Paramètres > 📌 Figer les Colonnes) : Colonne #, colonne active ou ensemble des colonnes visibles.".into(),
                "Comportement de Sélection (Paramètres > Comportement de Sélection) : Choisissez entre conserver des positions statiques de cellules/lignes ou suivre la sélection lors des tris/mises à jour.".into(),
                "Personnaliser la Couleur de Sélection des Cellules : Choisissez parmi les palettes recommandées (avec contraste adaptatif du texte pour les thèmes Clair et Sombre de Windows 11) ou définissez des couleurs RVB/TSV avec code HTML (#00ff00) modifiable.".into(),
                "Menus contextuels : Clic droit sur cellules ou en-têtes pour figer/libérer.".into(),
                "Persistance automatique dans painel_fixo_config.json.".into(),
                "Sélection Continue par Clic et Glisser : Sélectionnez des plages de cellules, colonnes ou lignes en maintenant le clic et en glissant avec surbrillance en temps réel.".into(),
                "Préservation des séparateurs d'en-tête intacte même lors de l'utilisation de Tout Sélectionner.".into(),
                "Sélectionner toutes les colonnes (Ctrl+A) et bascule de tri (Ctrl+O).".into(),
                "Inverser le Tri : Le bouton '🔄 Inverser' sur la grille inverse l'ordre croissant/décroissant de la colonne sélectionnée.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "couleurs".into(),
            title: "6. Personnalisation des Couleurs".into(),
            paragraphs: vec![
                "Palette de couleurs de fond et de texte personnalisable pour les thèmes Clair et Sombre.".into(),
            ],
            bullets: vec![
                "Colorer des colonnes spécifiques ou toutes les colonnes (Ctrl+Shift+A).".into(),
                "Contraste de lisibilité automatique noir ou blanc.".into(),
                "Palette Interactive avec HTML Modifiable : Saisissez ou collez n'importe quel code couleur HTML (#00ff00) avec synchronisation en temps réel, bouton de fermeture '❌' et boutons U8 et 📋 centrés verticalement.".into(),
                "Gestionnaire de Colonnes Épuré : Présentation unifiée en liste unique avec sélection par lot pour une gestion fluide de la visibilité des colonnes.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "securite".into(),
            title: "7. Sécurité, Ligne de Base et Quarantaine".into(),
            paragraphs: vec![
                "Protection proactive contre les ransomwares, exécutables non vérifiés et altérations non autorisées.".into(),
            ],
            bullets: vec![
                "Ligne de base cryptographique SHA-256 dans SQLite WAL.".into(),
                "Quarantaine sécurisée avec extension inerte .quar et traçabilité en base de données.".into(),
                "Restauration contrôlée et validée par empreinte SHA-256.".into(),
                "Règles canoniques SEC-R01 à SEC-R06.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "statistiques".into(),
            title: "8. 17 Graphiques Statistiques Natifs".into(),
            paragraphs: vec![
                "Appuyez sur Ctrl+G ou accédez à Options > Statistiques pour visualiser 17 graphiques interactifs accélérés par GPU.".into(),
            ],
            bullets: vec![
                "Distribution des opérations (graphique circulaire épuré), histogrammes de taille et carte thermique 7x24.".into(),
                "Contrôles Interactifs des Diagrammes Sankey : Filtrage par cases à cocher pour les opérations et sélecteur de limite d'éléments (10 à 100 et Tous) avec barre de défilement et rubans Bézier stables.".into(),
                "Séparation Stricte entre Renommé et Déplacé : Rapports distincts dans la Chronologie, le Radar de Risque et les flux Sankey.".into(),
                "Dispersion d'entropie de Shannon et diagrammes de flux.".into(),
                "Adaptation Dynamique aux Thèmes Windows 11 : Bascule automatique en temps réel entre thèmes Clair et Sombre sans redémarrage.".into(),
                "Exportation haute résolution vers des fichiers PNG.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "exportation".into(),
            title: "9. Exportation de Rapports et Éléments de Preuve".into(),
            paragraphs: vec![
                "Exportez vos rapports d'audit en CSV, JSON, HTML, TXT, XML et archives ZIP scellées.".into(),
            ],
            bullets: vec![
                "Enregistrer sous (Ctrl+Shift+S) et Enregistrement rapide (Ctrl+S).".into(),
                "Nomenclature Standardisée : Les rapports exportés adoptent le préfixe huginn_eventos_AAAAMMDD_HHMMSS.".into(),
                "Paquet de preuves ZIP avec manifeste JSON et condensats SHA-256.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "raccourcis".into(),
            title: "10. Raccourcis Clavier".into(),
            paragraphs: vec![
                "Appuyez sur F1 à tout moment pour afficher la liste complète des raccourcis.".into(),
            ],
            bullets: vec![
                "F1: Aide | Ctrl+M: Manuel | Ctrl+I: À propos | Ctrl+Shift+O: Choisir Dossier.".into(),
                "Ctrl+R: Démarrer/Arrêter | Ctrl+P: Pause/Reprise | Ctrl+S: Enregistrer | Ctrl+Shift+S: Enregistrer Sous.".into(),
                "Ctrl+G: Graphiques | Ctrl+L: Système de Nettoyage de Données (purge sélective du tableau, base de données ou sécurité) | Ctrl+A: Toutes les Colonnes | Ctrl+O: Tri | Ctrl+Q: Quitter.".into(),
            ],
            details: vec![],
        },
    ]
}

pub fn doc_it_it() -> Vec<ManualSection> {
    vec![
        ManualSection {
            id: "nuove-funzionalita".into(),
            title: "1. Nuove Funzionalità e Architettura".into(),
            paragraphs: vec![
                "Huginn Sentinel è una piattaforma desktop aziendale ad alte prestazioni per l'audit forense dell'integrità, il monitoraggio in tempo reale del file system di Windows e il rilevamento autonomo delle minacce.".into(),
                "La versione 2026.11.11.0 consolida la riscrittura completa in Rust moderno (Edizione 2021) con interfaccia grafica GPU in egui 0.37 / eframe e persistenza relazionale SQLite3 in modalità WAL.".into(),
                "Architettura Sovereign Privacy Local-First: il 100% delle analisi, punteggi di rischio (0–100) e database rimangono rigorosamente locali, senza telemetria e senza IA esterna.".into(),
            ],
            bullets: vec![
                "Albero dei menu persistente all'hover: i menu rimangono aperti durante i clic e si chiudono solo quando il puntatore esce dall'area.".into(),
                "Foglio di calcolo virtualizzato a 4 quadranti con blocco bidimensionale arbitrario di righe e colonne.".into(),
                "Finestre modeless indipendenti in stile Windows 11 con contatore dinamico tradotto nella barra di stato (1 Finestra:, 2 Finestre:).".into(),
                "Sistema di Pulizia Dati granulare: Cancellazione selettiva di eventi, database di sicurezza o tabella con conferme di sicurezza.".into(),
                "Database in cartelle dedicate (%LOCALAPPDATA%\\HuginnSentinel\\events e security) e avvio centrato sullo schermo principale.".into(),
                "Strumento da riga di comando autonomo (huginn-cli) per automazione e script.".into(),
                "Centro Avvisi con regole canoniche deterministiche (SEC-R01 a SEC-R06).".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "monitoraggio".into(),
            title: "2. Monitoraggio in Tempo Reale".into(),
            paragraphs: vec![
                "Cattura continua di file aggiunti, modificati, eliminati, rinominati e spostati tramite le API native del kernel di Windows (ReadDirectoryChangesW) e canali asincroni crossbeam.".into(),
            ],
            bullets: vec![
                "Pausa e Ripresa (Ctrl+P) per esaminare con calma gli eventi.".into(),
                "Contatori live nel pannello laterale e nella barra di stato.".into(),
                "Registrazione completa di tutti gli eventi senza tagli artificiali.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "scansione".into(),
            title: "3. Scansione Ricorsiva".into(),
            paragraphs: vec![
                "La scansione ricorsiva (Ctrl+R) esamina la directory selezionata tramite thread di lavoro dedicati in background.".into(),
            ],
            bullets: vec![
                "Arresto corretto: Il pulsante 'Ferma' interrompe la scansione senza chiudere l'applicazione.".into(),
                "Estrazione deterministica di metadati, numeri magici e hash SHA-256.".into(),
                "Concatenazione Interattiva dei Flussi: Finestre di dialogo che chiedono se avviare il monitoraggio dopo la scansione o scansionare prima di monitorare, configurabili in Impostazioni > Scansione e Monitoraggio.".into(),
                "Isolamento della Scansione: Durante la scansione ricorsiva, gli eventi del watcher in tempo reale vengono sospesi per registrare rigorosamente solo eventi di tipo 'Scansionato'.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "filtri".into(),
            title: "4. Filtri Avanzati e Ricerca".into(),
            paragraphs: vec![
                "Finestra filtri avanzati per tipo di operazione, estensione, severità del rischio e intervalli di date con opzioni di eccezione.".into(),
            ],
            bullets: vec![
                "Filtro operazioni: Aggiunto, Modificato, Eliminato, Rinominato, Spostato o Scansionato.".into(),
                "Opzioni per ignorare il filtro data per specifiche operazioni.".into(),
                "Contatore Dinamico del Totale Eventi: Il totale nel pannello laterale riflette dinamicamente la somma esatta delle caselle delle operazioni selezionate.".into(),
                "Pulsante Ripristina Filtri con un solo clic.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "colonne".into(),
            title: "5. Foglio Virtualizzato e Blocco 2D di Righe/Colonne".into(),
            paragraphs: vec![
                "Tabella virtuale con 58 colonne di metadati forensi e blocco bidimensionale (Freeze Panes a 4 quadranti).".into(),
            ],
            bullets: vec![
                "Blocca Righe (Impostazioni > 📌 Blocca Righe): Intestazioni, riga attiva o intervalli numerici (es: 1, 3, 5-8).".into(),
                "Blocca Colonne (Impostazioni > 📌 Blocca Colonne): Colonna #, colonna attiva o tutte le visibili.".into(),
                "Comportamento della Selezione (Impostazioni > Comportamento della Selezione): Scegli tra mantenere posizioni statiche di celle/righe o seguire la selezione durante ordinamenti/aggiornamenti.".into(),
                "Personalizza Colore Selezione Celle: Scegli tra le tavolozze consigliate (con contrasto adattivo del testo per i temi Chiaro e Scuro di Windows 11) o definisci colori RGB/HSV personalizzati con codice HTML (#00ff00) modificabile.".into(),
                "Menu contestuali: Clic destro su celle o intestazioni per bloccare/sbloccare.".into(),
                "Persistenza automatica nel file painel_fixo_config.json.".into(),
                "Selezione Continua con Clic e Trascina: Seleziona intervalli arbitrari di celle, colonne o righe tenendo premuto e trascinando con evidenziazione in tempo reale.".into(),
                "Preservazione delle linee di divisione delle intestazioni intatte anche con Seleziona Tutto.".into(),
                "Seleziona tutte le colonne (Ctrl+A) e attiva/disattiva ordinamento (Ctrl+O).".into(),
                "Inverti Ordinamento: Il pulsante '🔄 Inverti' sulla griglia inverte l'ordine crescente/decrescente della colonna selezionata.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "colori".into(),
            title: "6. Personalizzazione dei Colori".into(),
            paragraphs: vec![
                "Personalizzazione dei colori di sfondo e testo per ciascuna operazione nei temi Chiaro e Scuro.".into(),
            ],
            bullets: vec![
                "Colora colonne specifiche o tutte le colonne (Ctrl+Shift+A).".into(),
                "Contrasto automatico del testo in bianco o nero.".into(),
                "Tavolozza Interattiva con HTML Modificabile: Inserisci o incolla qualsiasi codice colore in formato HTML (#00ff00) con sincronizzazione in tempo reale, pulsante di chiusura '❌' e pulsanti U8 e 📋 centrati verticalmente.".into(),
                "Gestore Colonne Semplificato: Layout unificato in elenco singolo con azioni rapide per una gestione fluida della visibilità delle colonne.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "sicurezza".into(),
            title: "7. Sicurezza, Baseline e Quarantena".into(),
            paragraphs: vec![
                "Protezione proattiva contro ransomware, binari non autorizzati e deviazioni di conformità.".into(),
            ],
            bullets: vec![
                "Baseline crittografica SHA-256 memorizzata in SQLite WAL.".into(),
                "Quarantena protetta con estensione inerte .quar e tracciabilità nel database.".into(),
                "Ripristino convalidato rigorosamente tramite hash SHA-256.".into(),
                "Regole canoniche SEC-R01 a SEC-R06.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "statistiche".into(),
            title: "8. 17 Grafici Statistici Nativi".into(),
            paragraphs: vec![
                "Premi Ctrl+G o accedi a Opzioni > Statistiche per visualizzare 17 grafici interattivi accelerati su GPU.".into(),
            ],
            bullets: vec![
                "Distribuzione delle operazioni (grafico a torta pulito), istogrammi di dimensione e mappa termica 7x24.".into(),
                "Controlli Interattivi nei Diagrammi Sankey: Filtro con caselle di controllo (checkbox) per le operazioni e selettore limite elementi (da 10 a 100 e Tutti) con barra di scorrimento e nastri Bézier stabili.".into(),
                "Separazione Rigorosa tra Rinominato e Spostato: Report separati nella Cronologia, nel Radar del Rischio e nei flussi Sankey.".into(),
                "Dispersione dell'entropia di Shannon e diagrammi volumetrici.".into(),
                "Adattamento ai Temi di Windows 11: Passaggio dinamico automatico tra tema Chiaro e Scuro in tempo reale senza riavvio.".into(),
                "Esportazione dei grafici ad alta risoluzione in file PNG.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "esportazione".into(),
            title: "9. Esportazione di Report ed Evidenze".into(),
            paragraphs: vec![
                "Esporta i tuoi report di audit in CSV, JSON, HTML, TXT, XML e archivi ZIP sigillati.".into(),
            ],
            bullets: vec![
                "Salva con nome (Ctrl+Shift+S) e Salvataggio rapido (Ctrl+S).".into(),
                "Nomenclatura Standard: I report esportati utilizzano il prefisso huginn_eventos_AAAAMMDD_HHMMSS.".into(),
                "Pacchetto di prove ZIP con manifesto JSON e hash SHA-256 di custodia.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "scorciatoie".into(),
            title: "10. Scorciatoie da Tastiera".into(),
            paragraphs: vec![
                "Premi F1 in qualsiasi momento per consultare l'elenco completo delle scorciatoie.".into(),
            ],
            bullets: vec![
                "F1: Aiuto | Ctrl+M: Manuale | Ctrl+I: Informazioni | Ctrl+Shift+O: Scegli Cartella.".into(),
                "Ctrl+R: Avvia/Ferma | Ctrl+P: Pausa/Riprendi | Ctrl+S: Salva | Ctrl+Shift+S: Salva Come.".into(),
                "Ctrl+G: Grafici | Ctrl+L: Sistema di Pulizia Dati (cancellazione selettiva di tabella, database o sicurezza) | Ctrl+A: Tutte le Colonne | Ctrl+O: Ordina | Ctrl+Q: Esci.".into(),
            ],
            details: vec![],
        },
    ]
}

pub fn doc_de_de() -> Vec<ManualSection> {
    vec![
        ManualSection {
            id: "neue-funktionen".into(),
            title: "1. Neue Funktionen und Architektur".into(),
            paragraphs: vec![
                "Huginn Sentinel ist eine lokale Hochleistungs-Desktop-Plattform für Dateisicherheits- und forensische Integritätsüberwachung sowie autonome Bedrohungserkennung unter Windows.".into(),
                "Version 2026.11.11.0 konsolidiert die vollständige Neuentwicklung in modernem Rust (Edition 2021) mit GPU-beschleunigter egui 0.37 / eframe Oberfläche und SQLite3 WAL-Persistenz.".into(),
                "Local-First Sovereign Privacy Architektur: 100% der Analysen, Risikobewertungen (0–100) und Datenbanken verbleiben ausschließlich lokal ohne Telemetrie und ohne externe KI.".into(),
            ],
            bullets: vec![
                "Persistenter Hover-Menübaum: Menüs bleiben bei Mausklicks geöffnet und schließen sich erst, wenn der Zeiger den Menübereich verlässt.".into(),
                "Virtualisierte 4-Quadranten-Tabelle mit beliebiger 2D-Fixierung von Zeilen und Spalten (Freeze Panes).".into(),
                "Eigenständige Modeless-Fenster im Windows 11-Stil mit dynamischem Fensterzähler in der Statusleiste (1 Fenster:, 2 Fenster:).".into(),
                "Granulares Datenbereinigungssystem: Selektives Leeren von Ereignissen, Sicherheitsdatenbank oder Benutzeroberflächentabelle mit Sicherheitsbestätigung.".into(),
                "Isolierte Datenbankverzeichnisse (%LOCALAPPDATA%\\HuginnSentinel\\events und security) und zentrierter Start auf dem Hauptbildschirm.".into(),
                "Eigenständiges CLI-Dienstprogramm (huginn-cli) für Skripte und CI/CD-Pipelines.".into(),
                "Alarmzentrale mit kanonischen deterministischen Regeln (SEC-R01 bis SEC-R06).".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "ueberwachung".into(),
            title: "2. Echtzeit-Überwachung".into(),
            paragraphs: vec![
                "Kontinuierliche Erfassung hinzugefügter, geänderter, gelöschter, umbenannter und verschobener Dateien über native Windows Kernel-APIs (ReadDirectoryChangesW) und asynchrone crossbeam-Kanäle.".into(),
            ],
            bullets: vec![
                "Pause und Fortsetzen (Ctrl+P) zur ruhigen Inspektion von Ereigniszeilen.".into(),
                "Echtzeit-Zähler in der Seitenleiste und Fußzeile ohne Ruckeln der Oberfläche.".into(),
                "Vollständige Protokollierung aller Ereignisse ohne künstliche Begrenzung.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "suchlauf".into(),
            title: "3. Rekursiver Verzeichnissuchlauf".into(),
            paragraphs: vec![
                "Der rekursive Suchlauf (Ctrl+R) durchsucht das gewählte Verzeichnis über dedizierte Hintergrund-Threads.".into(),
            ],
            bullets: vec![
                "Sauberer Abbruch: Die Schaltfläche 'Stopp' hält den Suchlauf an, ohne die Anwendung zu schließen.".into(),
                "Deterministische Extraktion von Metadaten, Magic Numbers und SHA-256 Prüfsummen.".into(),
                "Interaktive Ablaufverkettung: Dialoge fragen, ob die Überwachung nach dem Suchlauf gestartet oder vor der Überwachung gesucht werden soll, konfigurierbar unter Einstellungen > Suchlauf und Überwachung.".into(),
                "Suchlauf-Isolierung: Während des rekursiven Suchlaufs werden Echtzeit-Beobachterereignisse pausiert, um strikt nur Ereignisse vom Typ 'Gescannt' zu erfassen.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "filter".into(),
            title: "4. Erweiterte Filter und Suche".into(),
            paragraphs: vec![
                "Erweitertes Filterfenster für Operationen, Dateiendungen, Risikostufen und Datumsbereiche mit Ausnahmeschaltern.".into(),
            ],
            bullets: vec![
                "Operationsfilter: Hinzugefügt, Geändert, Gelöscht, Umbenannt, Verschoben oder Gescannt.".into(),
                "Schalter zum Ignorieren des Datumsfilters pro Operation.".into(),
                "Dynamischer Gesamtereigniszähler: Die Gesamtzahl in der Seitenleiste spiegelt dynamisch die exakte Summe der ausgewählten Operations-Kontrollkästchen wider.".into(),
                "Schaltfläche zum Zurücksetzen aller Filter mit einem Klick.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "spalten".into(),
            title: "5. Virtualisierte Tabelle und 2D-Zeilen-/Spaltenfixierung".into(),
            paragraphs: vec![
                "58 forensische Spalten mit deterministischer Metadatenharmonisierung und 2D-Fixierung (4-Quadranten Freeze Panes).".into(),
            ],
            bullets: vec![
                "Zeilen fixieren (Einstellungen > 📌 Zeilen fixieren): Kopfzeile, aktive Zeile oder Zahlenbereiche (z. B. 1, 3, 5-8).".into(),
                "Spalten fixieren (Einstellungen > 📌 Spalten fixieren): Spalte #, aktive Spalte oder alle sichtbaren Spalten.".into(),
                "Auswahlverhalten (Einstellungen > Auswahlverhalten): Wählen Sie zwischen statischen Zell-/Zeilenpositionen oder Folgen der Auswahl bei Sortierungen/Aktualisierungen.".into(),
                "Zellauswahlfarbe anpassen: Wählen Sie empfohlene Paletten (mit adaptivem Textkontrast für helle und dunkle Windows 11-Designs) oder benutzerdefinierte RGB/HSV-Farben mit editierbarem HTML-Code (#00ff00).".into(),
                "Kontextmenüs: Rechtsklick auf Zellen oder Spaltenköpfe zum Fixieren/Lösen.".into(),
                "Automatische Speicherung der Einstellungen in painel_fixo_config.json.".into(),
                "Kontinuierliche Klick- und Ziehauswahl: Markieren Sie beliebige Zellbereiche, Spalten oder Zeilen durch Klicken und Ziehen mit visueller Echtzeit-Hervorhebung.".into(),
                "Erhalt der visuellen Trennlinien in Spalten- und Zeilenköpfen auch bei 'Alles auswählen'.".into(),
                "Alle Spalten auswählen (Ctrl+A) und Sortierung umschalten (Ctrl+O).".into(),
                "Sortierung umkehren: Die Schaltfläche '🔄 Umkehren' im Raster kehrt die aufsteigende/absteigende Reihenfolge der ausgewählten Spalte um.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "farben".into(),
            title: "6. Farbanpassung".into(),
            paragraphs: vec![
                "Anpassung der Hintergrund- und Textfarben für jede Operation in hellen und dunklen Themen.".into(),
            ],
            bullets: vec![
                "Färben bestimmter Spalten oder aller Spalten (Ctrl+Shift+A).".into(),
                "Automatische Kontrastberechnung für schwarze oder weiße Schrift.".into(),
                "Interaktive Farbpalette mit editierbarem HTML: Geben Sie einen beliebigen HTML-Farbcode (#00ff00) ein oder fügen Sie ihn ein – mit Echtzeitsynchronisierung, Schließen-Schaltfläche '❌' und vertikal zentrierten U8- und 📋-Aktionen.".into(),
                "Optimierter Spalten-Manager: Einheitliche einspaltige Liste mit Stapelauswahl für schnelle Verwaltung der Spaltensichtbarkeit.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "sicherheit".into(),
            title: "7. Sicherheit, Baseline und Quarantäne".into(),
            paragraphs: vec![
                "Präventiver Schutz vor Ransomware, unautorisierten Binärdateien und Integritätsabweichungen.".into(),
            ],
            bullets: vec![
                "Kryptografische SHA-256 Baseline in SQLite WAL.".into(),
                "Sichere Quarantäne mit inerter .quar-Erweiterung und Datenbank-Kette.".into(),
                "Strikte SHA-256-Validierung vor jeder Dateiwiederherstellung.".into(),
                "Kanonische Regeln SEC-R01 bis SEC-R06.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "statistiken".into(),
            title: "8. 17 Native Statistische Diagramme".into(),
            paragraphs: vec![
                "Drücken Sie Ctrl+G oder wählen Sie Optionen > Statistiken für 17 GPU-beschleunigte interaktive Diagramme.".into(),
            ],
            bullets: vec![
                "Operationsverteilung (sauberes Kreisdiagramm), Größen-Histogramme und 7x24-Aktivitäts-Heatmap.".into(),
                "Interaktive Steuerungen in Sankey-Diagrammen: Kontrollkästchen-Filter (Checkboxes) für Operationen und Elementbegrenzung (10 bis 100 und Alle) mit Bildlaufleiste und stabilen Bézier-Bändern.".into(),
                "Strikte Trennung von Umbenannt und Verschoben: Getrennte Berichterstattung in Zeitachse, Risiko-Radar und Sankey.".into(),
                "Shannon-Entropiestreuung und volumetrische Punktdiagramme.".into(),
                "Dynamische Windows 11-Designanpassung: Automatische Umschaltung zwischen hellem und dunklem Design zur Laufzeit ohne Neustart.".into(),
                "Hochauflösender PNG-Export einzeln oder im Stapel.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "export".into(),
            title: "9. Berichts- und Beweismittel-Export".into(),
            paragraphs: vec![
                "Exportieren Sie Audit-Berichte in CSV, JSON, HTML, TXT, XML sowie kryptografisch versiegelte ZIP-Beweispakete.".into(),
            ],
            bullets: vec![
                "Speichern unter (Ctrl+Shift+S) und Schnellspeichern (Ctrl+S).".into(),
                "Standardisierte Dateinamen: Exportierte Berichte verwenden das offizielle Präfix huginn_eventos_JJJJMMTT_HHMMSS.".into(),
                "Versiegeltes ZIP-Beweispaket mit JSON-Manifest und SHA-256 Prüfsummen.".into(),
            ],
            details: vec![],
        },
        ManualSection {
            id: "tastenkombinationen".into(),
            title: "10. Tastenkombinationen".into(),
            paragraphs: vec![
                "Drücken Sie jederzeit F1, um das vollständige Hilfefenster aufzurufen.".into(),
            ],
            bullets: vec![
                "F1: Hilfe | Ctrl+M: Handbuch | Ctrl+I: Über | Ctrl+Shift+O: Verzeichnis wählen.".into(),
                "Ctrl+R: Start/Stopp | Ctrl+P: Pause/Fortsetzen | Ctrl+S: Speichern | Ctrl+Shift+S: Speichern unter.".into(),
                "Ctrl+G: Diagramme | Ctrl+L: Datenbereinigungssystem (selektives Leeren von Tabelle, Ereignis- oder Sicherheitsdatenbank) | Ctrl+A: Alle Spalten | Ctrl+O: Sortieren | Ctrl+Q: Beenden.".into(),
            ],
            details: vec![],
        },
    ]
}
