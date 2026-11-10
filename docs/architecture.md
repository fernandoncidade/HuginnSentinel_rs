# Arquitetura do Sistema / System Architecture — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher a arquitetura em português</summary><br>

# Huginn Sentinel — Arquitetura de Sistema e Especificações Técnicas

**Versão:** `2026.11.9.0`  
**Data:** `9 de novembro de 2026`  
**Padrão:** Rust (Edition 2021) | `egui 0.37.0` / `eframe 0.37.0` | Win32 API (`windows-sys 0.61.2`: `ReadDirectoryChangesW`, `WinTrust`, `crypt32`) | SQLite3 (`rusqlite 0.41.0` `bundled`, modo WAL)  
**Plataforma Alvo:** Windows 11 / Windows 10 / Windows Server 2022+ (x64)  
**Binários (`builds/huginnsentinel/release/`):** `huginnsentinel.exe` (`src/main.rs`) | `huginnsentinel-cli.exe` (`src/bin/main_cli.rs`)  
**Licença:** GNU Affero General Public License v3.0 (`GNU AGPLv3`)  
**Autor:** Fernando Nillsson Cidade  

---

## 1. Visão Geral e Filosofia Arquitetural

O **Huginn Sentinel** é uma plataforma desktop nativa para Windows desenvolvida integralmente em **Rust** (Edition `2021`) com interface gráfica em modo imediato acelerada por hardware em **`egui 0.37.0` / `eframe 0.37.0`**, projetada para fornecer visibilidade absoluta, histórico cronológico estruturado, inteligência determinística de arquivos sem IA e auditoria forense de segurança local de altíssimo desempenho.

A arquitetura é fundamentada no princípio da **Privacidade Soberana (Local-First / Sovereign Privacy)**: todos os eventos do sistema de arquivos, as **58 colunas de metadados enriquecidos e harmonizados**, os bancos de dados SQLite3 locais em modo WAL, as configurações de painéis congelados e os pacotes forenses de custódia são processados e armazenados estritamente na máquina local do usuário (`%LOCALAPPDATA%\HuginnSentinel Sentinel`), sem conexões de rede externas, sem telemetria, sem nuvem e **sem dependência de modelos de linguagem (LLMs) ou IA externa**. Toda a análise, correlação semântica, harmonização de metadados e pontuação de risco (0–100) utilizam algoritmos determinísticos puros.

Estruturalmente, o sistema opera como uma **pipeline local dividida em 7 domínios cooperantes**:

```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          PIPELINE DE DADOS - HUGINN SENTINEL                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
                                             │
 1. DOMÍNIO DE MONITORAMENTO (notify 8.2.1 / ReadDirectoryChangesW / DiretorioScanner)
                                             ↓
 2. DOMÍNIO DE EVENTOS E CORRELAÇÃO (crossbeam-channel 0.5.17 / BaseEvento Move & Rename)
                                             ↓
 3. DOMÍNIO DE METADADOS (58 Colunas em extractor.rs + harmonizacao.rs, Magic Numbers, SHA-256)
                                             ↓
 4. DOMÍNIO DE PERSISTÊNCIA (rusqlite 0.41.0 bundled em modo WAL / DbWriter em lotes de 2048)
                                             ↓
 5. DOMÍNIO DE ANALYTICS & ESTATÍSTICAS (17 Gráficos Nativos em TipoGrafico / Render Raster)
                                             ↓
 6. DOMÍNIO DE SEGURANÇA E AUDITORIA (Baseline SHA-256, MotorRegras, ScoreRisco, Quarentena)
                                             ↓
 7. DOMÍNIO DE APRESENTAÇÃO (egui/eframe: Menus Hover-Open, Planilha 4 Quadrantes, 16 Janelas Modeless, CLI)
```

### 1.1. Diagrama Geral das Camadas do Sistema

```text
+-----------------------------------------------------------------------------------------+
|                                 CAMADA DE APRESENTAÇÃO                                  |
|                    (src/ui/ — egui 0.37.0 / eframe 0.37.0 + CLI Nativo)                 |
|                                                                                         |
|  - Árvore de Menus Persistente por Hover (MenuConfig IgnoreClicks + MenuHoverState)     |
|  - Planilha 2D Sincronizada em 4 Quadrantes (TL, TR, BL, BR) com Virtualização O(vis)   |
|  - Até 16 Janelas Filhas Modeless Independentes com Contador Dinâmico e Pílulas Z-Order |
|  - Utilitário Autônomo de Console Headless (`huginnsentinel-cli.exe`)                |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                           SUBSISTEMA GRÁFICO & ESTATÍSTICAS                             |
|                       (src/data/ — 17 Gráficos Nativos TipoGrafico)                     |
|                                                                                         |
|    generator.rs (Agregação Determinística) | render/raster.rs & render/export.rs        |
|    - Pizza, Barras, Timeline, Treemap      | - Heatmap, Sankey, Pareto, Boxplot, Radar  |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                    NÚCLEO DE SERVIÇOS                                   |
|                                     (src/services/)                                     |
|                                                                                         |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  Observador & Scanner       |  |  Correlação Semântica (BaseEvento)               |  |
|  |  - notify 8.2.1 (Win32)     |  |  - Renomeado vs. Movido entre pastas distintas   |  |
|  |  - crossbeam-channel 0.5.17 |  |  - Recuperação histórica de atributos pós-delete |  |
|  |  - Pool std::thread (2..8)  |  |  - GerenciadorColunas & GerenciadorTabela        |  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  Motor de 58 Metadados      |  |  Persistência Assíncrona SQLite3                 |  |
|  |  - metadados/extractor.rs   |  |  - rusqlite 0.41.0 (bundled) em modo WAL         |  |
|  |  - metadados/harmonizacao.rs|  |  - DbWriter transacional (lotes de 2048 / 50 ms) |  |
|  |  - sha2 / sha1 / md-5 puros |  |  - %LOCALAPPDATA%\HuginnSentinel Sentinel               |  |
|  +-----------------------------+  +--------------------------------------------------+  |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                  SUBSISTEMA DE SEGURANÇA                                |
|                                     (src/security/)                                     |
|                                                                                         |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  Baseline & Executáveis     |  |  Quarentena & Evidências Forenses                |  |
|  |  - BaselineCriptografico    |  |  - Isolamento seguro (`quarantine\*.quar`)       |  |
|  |  - analisar_executavel (PE) |  |  - Validação SHA-256 pré-restauração             |  |
|  |  - Detecção de Drift        |  |  - Pacotes ZIP Forenses (.zip, .json, .csv, .xml)|  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  MotorRegras & ScoreRisco   |  |  Sessões, Diff & MultiDiretorio                  |  |
|  |  - Pontuação 0 a 100        |  |  - CorrelacaoSessoes & AtribuicaoForense         |  |
|  |  - 4 Níveis Determinísticos |  |  - Perfis persistentes em huginn_security.db     |  |
|  +-----------------------------+  +--------------------------------------------------+  |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                        APIS NATIVAS DO WINDOWS (windows-sys 0.61.2)                     |
|                                                                                         |
|   Win32_Storage_FileSystem (ReadDirectoryChangesW, NTFS ADS, Atributos de Arquivo)      |
|   Win32_Security_WinTrust & Win32_Security_Cryptography (Assinatura Authenticode X.509) |
|   Win32_UI_Shell & Win32_UI_WindowsAndMessaging & Win32_System_Registry                 |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Dependências Efetivas e Ambiente de Compilação

### 2.1. Linguagem, Perfil de Compilação e Binários
- **Edição da Linguagem:** Rust `edition = "2021"` gerenciada via `Cargo.toml` (`version = "2026.10.29"` / versão de exibição `2026.10.29.0`).
- **Toolchain Homologada:** `rustc` estável sobre `x86_64-pc-windows-msvc`.
- **Perfil Release (`[profile.release]`):** `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true` — gera binários nativos enxutos e estaticamente vinculados sem necessidade de DLLs externas de interface gráfica ou banco de dados.
- **Diretório de Saída (`.cargo/config.toml`):** `target-dir = "builds/huginnsentinel"`, produzindo os artefatos finais em `builds/huginnsentinel/release/`:
  - `huginnsentinel.exe` (`src/main.rs`, com `#![windows_subsystem = "windows"]`): aplicação gráfica desktop.
  - `huginnsentinel-cli.exe` (`src/bin/main_cli.rs`): utilitário de linha de comando headless para servidores e automação DevSecOps.

### 2.2. Crates Rust Efetivas (`Cargo.toml`)

| Crate | Versão | Papel Arquitetural |
|---|---|---|
| `eframe` / `egui` | `0.37.0` | Motor gráfico em modo imediato acelerado por hardware, múltiplas viewports nativas do SO, menus e planilha virtualizada. |
| `rusqlite` | `0.41.0` (`bundled`) | Motor SQLite3 embarcado estaticamente, operado em modo WAL para histórico de eventos e banco de segurança. |
| `notify` | `8.2.1` | Observação contínua e recursiva do sistema de arquivos sobre `ReadDirectoryChangesW` no Windows. |
| `crossbeam-channel` | `0.5.17` | Filas MPMC sem bloqueio para comunicação de alto rendimento entre observador, scanner, writer SQLite e UI. |
| `parking_lot` | `0.13.0` | Primitivas rápidas `Mutex` e `RwLock` para sincronização de caches, estado de tradução e candidatos de correlação. |
| `sha2` / `sha1` / `md-5` / `hex` | `0.11.1` / `0.11.0` / `0.11.0` / `0.4.3` | Cálculo criptográfico puro em Rust de hashes SHA-256, SHA-1 e MD5 para integridade, baseline, quarentena e custódia. |
| `windows-sys` | `0.61.2` | Bindings Win32 diretos (`WinTrust`, `crypt32` para Authenticode, atributos de arquivos, NTFS ADS, Registro e Shell). |
| `quick-xml` | `0.42.0` | Inspeção rápida de pacotes OOXML/ODF/SVG e estruturação de metadados XML. |
| `zip` | `9.1.2` | Inspeção de arquivos compactados (contagem interna e tamanho descompactado) e criação de pacotes forenses `.zip`. |
| `rfd` | `0.17.2` | Diálogos nativos do Windows para seleção de diretórios, exportação de relatórios e confirmações críticas. |
| `serde` / `serde_json` | `1.0.229` / `1.0.154` | Serialização tipada de eventos, dicionários JSON de idiomas e persistência de `painel_fixo_config.json`. |
| `chrono` / `image` / `uuid` | `0.4.45` / `0.25.10` / `1.10` | Carimbos temporais locais, decodificação de ícones/dimensões de imagens e identificadores únicos de sessão. |

Não há servidores HTTP embutidos, WebSockets remotos, motores de navegador ou conexões externas: o software opera **100% offline**.

---

## 3. Mapa de Módulos e Estrutura do Código-Fonte

| Caminho | Papel Arquitetural |
|---|---|
| `src/main.rs` | Ponto de entrada do executável gráfico `huginnsentinel.exe`; processa o diretório posicional opcional, configura o `ViewportBuilder` e inicia `eframe::run_native`. |
| `src/bin/main_cli.rs` | Ponto de entrada do utilitário headless `huginnsentinel-cli.exe` (`scan`, `metadata`, `baseline`, `quarantine`, `monitor`). |
| `src/lib.rs` | Raiz da biblioteca `huginn_rs`, reexportando `EventModel`, `FileCategory`, `OperationType`, `RiskLevel`, `Localizer`, `SecurityManager`, `BaseEvento` e `HuginnApp`. |
| `src/core/` | Contratos centrais (`event.rs` com `EventModel` de 58 campos, `types.rs` com `OperationType`, `FileCategory`, `RiskLevel` e `error.rs`). |
| `src/data/` | Motor analítico e estatístico (`charts.rs` com 17 variantes `TipoGrafico`, `generator.rs`, `render/raster.rs` e `render/export.rs`). |
| `src/locale/` + `src/language/*.json` | Internacionalização em 6 idiomas (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `de_DE`) com dicionários JSON compilados no binário via `include_str!` em `localizer.rs` e manual multilíngue em `manual.rs`. |
| `src/security/` | Fachada `SecurityManager` (`manager.rs`), `BaselineCriptografico` (`baseline.rs`), `MotorRegras` (`rules.rs`), `ScoreRisco` (`risk_score.rs`), `Quarentena` (`quarantine.rs`), `ExportacaoEvidencia` (`evidence.rs`), `analisar_executavel` (`executable_intel.rs`), `AtribuicaoForense` (`attribution.rs`), `DiffConteudo` (`diff.rs`), `CorrelacaoSessoes` (`sessions.rs`) e `MultiDiretorio` (`multi_directory.rs`). |
| `src/services/` | `Observador` (`observador.rs`), `DiretorioScanner` (`diretorio_scanner.rs`), `BaseEvento` (`base_evento.rs`), `DbWriter` (`db_writer.rs`), `GerenciadorColunas` (`gerenciador_colunas.rs`), `GerenciadorTabela` (`gerenciador_tabela.rs`) e extração/harmonização de 58 colunas em `metadados/` (`extractor.rs`, `harmonizacao.rs`, `archives.rs`, `documents.rs`, `executables.rs`, `images.rs`, `media.rs`, `types.rs`). |
| `src/tools/` | Motor de filtros multidimensionais (`filters.rs`: `FiltroEstado`, `ContadoresEventos`) e seletor de calendário (`calendar.rs`). |
| `src/ui/` | Orquestrador `HuginnApp` (`app.rs`), barra de menus (`menu_bar.rs`), barra de ferramentas (`toolbar.rs`), painel lateral (`filter_panel.rs`), widgets customizados e guardião de hover (`widgets.rs`), visualização de tabela (`table_view.rs`), 13 módulos de diálogos (`dialogs/`) e motor de planilha congelada (`spreadsheet/`). |
| `src/utils/` | Resolução resiliente de diretórios e assets (`paths.rs`), ícones (`icons.rs`), logger central (`logger.rs`), temas claro/escuro (`theme.rs`) e integração com o Windows Explorer (`app_association.rs`). |
| `src/assets/` | Ícones `.ico`/`.png` e documentos regulatórios multilíngues (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE`). A documentação técnica detalhada reside em `docs/`. |

---

## 4. Fluxo de Inicialização e Ciclo por Frame

```text
src/main.rs (huginnsentinel.exe)
  │
  ├── Lê argumento posicional opcional [directory_path] (se existir e for diretório válido)
  ├── Configura egui::ViewportBuilder (1280x800 inicial, mínimo 960x600, ícone da janela)
  └── eframe::run_native("Huginn Sentinel", native_options, ...)
        │
        ▼
      HuginnApp::new(pasta_inicial) (src/ui/app.rs)
        ├── Inicializa canais crossbeam_channel::unbounded (event_tx/rx e progress_tx/rx)
        ├── Instancia BaseEvento (abre DbWriter WAL em huginn_events.db e carrega histórico)
        ├── Instancia SecurityManager (abre Baseline, Quarentena e MultiDiretorio em security/huginn_security.db)
        ├── Instancia Localizer (carrega 6 dicionários JSON embutidos via include_str! + preferência salva)
        ├── Carrega GerenciadorColunas, GerenciadorTabela e GridLayout (painel_fixo_config.json)
        └── Se pasta_inicial foi fornecida, prepara diretório ativo para varredura/monitoramento
              │
              ▼
      Ciclo por Frame — eframe::App::update(&mut self, ctx, _frame)
        ├── iniciar_rastreamento_menus_frame(&ctx) (prepara MenuHoverState do frame)
        ├── processar_mensagens_background(&ctx) (drena até 512 eventos / orçamento de 4 ms por frame)
        ├── Atualização incremental de índices filtrados e contadores sem reordenar histórico inteiro
        ├── Cadência adaptativa de repintura: 16 ms (fila ativa), 33 ms (monitorando), 200 ms (repouso)
        ├── Painel Superior: renderizar_menu_bar + renderizar_toolbar
        ├── Painel Inferior: progresso, contadores, contador dinâmico e pílulas de até 16 janelas modeless
        ├── Painel Esquerdo: renderizar_painel_filtros
        ├── Painel Central: renderizar_tabela_eventos (planilha 2D sincronizada em 4 quadrantes)
        ├── Renderização independente das janelas filhas modeless abertas (show_modeless_dialog)
        └── finalizar_rastreamento_menus_frame(&ctx) (fecha menus automaticamente se o ponteiro saiu da região)
```

---

## 5. Arquitetura da Interface Gráfica (`src/ui/`)

### 5.1. Árvore de Menus Persistente por Hover (`src/ui/menu_bar.rs`, `src/ui/widgets.rs`, `src/ui/spreadsheet/context_menu.rs`)
Para permitir que o analista marque múltiplas caixas de seleção, alterne filtros, fixe diversas linhas/colunas ou acione comandos repetidos sem que o menu se feche a cada clique:
- **Configuração Persistente (`config_menu_persistente`):** Constrói `egui::containers::menu::MenuConfig` com `PopupCloseBehavior::IgnoreClicks` e `submenu_gap(0.0)`, eliminando frestas geométricas entre menus pais e submenus aninhados.
- **Construtores Unificados:** `menu_persistente`, `menu_contexto_persistente` e `combo_persistente_registrar` aplicam o comportamento persistente tanto na barra de menus principal quanto em submenus em cascata, menus de contexto de botão direito na planilha e caixas de combinação (`ComboBox`).
- **Rastreamento Geométrico por Frame (`MenuHoverState`):** Invocado no início (`iniciar_rastreamento_menus_frame`) e no encerramento (`finalizar_rastreamento_menus_frame`) de cada frame, registra os retângulos ativos de todos os menus e submenus abertos (com tolerância de costura de `6.0 px` e detecção de deslocamento em direção ao submenu `is_moving_towards_rect`).
- **Fechamento Automático sem Cliques Extras:** A árvore de menus permanece aberta durante múltiplos cliques internos e fecha automaticamente somente quando o ponteiro do mouse deixa a região ativa da árvore de menus aberta (ou quando a tecla `Esc` é pressionada).

### 5.2. Planilha 2D Congelada Sincronizada em 4 Quadrantes e Seleção Contínua (`src/ui/table_view.rs`, `src/ui/spreadsheet/grid_layout.rs`, `src/ui/spreadsheet/cell_selection.rs`)
A tabela central de eventos funciona como uma planilha forense de alta densidade capaz de fixar (congelar) simultaneamente cabeçalhos, linhas arbitrárias e colunas arbitrárias, além de oferecer seleção contínua interativa:
- **Seleção Interativa Contínua por Clique e Arraste (`cell_selection.rs`):**
  - Máquina de estados de seleção com suporte a faixas retangulares de células (`SelectionType::Cells`), faixas de colunas (`SelectionType::Columns`) e faixas de linhas (`SelectionType::Rows`).
  - Rastreamento dinâmico do ponteiro do mouse durante o arraste (`is_dragging`, `drag_start`, `drag_current`) com realce visual contínuo e translúcido renderizado em tempo real sobre a grade por `grid_renderer.rs`.
  - Preservação estrita das bordas e divisórias de cabeçalhos de linhas e colunas ao acionar o botão de seleção total (*Select All*).
- **Fixação Bidimensional Arbitrária (`GridLayout`):**
  - 1ª linha de cabeçalhos de colunas (`freeze_header_row: bool`);
  - Qualquer subconjunto arbitrário de linhas de dados (`pinned_rows: BTreeSet<usize>`), incluindo parser de intervalos 1-based (ex.: `1, 3, 5-8`) com proteção de expansão;
  - Coluna `#` de numeração de linhas (`freeze_row_headers: bool`);
  - Qualquer subconjunto arbitrário dentre as **58 colunas de metadados** (`pinned_cols: BTreeSet<String>`), sinalizadas visualmente com `📌`.
- **Renderização Sincronizada em 4 Quadrantes:**
  - **`TL` (Top-Left):** Canto superior esquerdo totalmente fixo (interseção de linhas fixadas e colunas fixadas);
  - **`TR` (Top-Right):** Faixa superior de linhas fixadas sincronizada horizontalmente com a rolagem X;
  - **`BL` (Bottom-Left):** Faixa lateral de colunas fixadas sincronizada verticalmente com a rolagem Y;
  - **`BR` (Bottom-Right):** Corpo principal com rolagem livre 2D (X e Y).
- **Virtualização `O(visíveis)` e Persistência:** `compute_visible_rows` calcula apenas a janela visível de linhas a cada frame, mantendo 60 FPS mesmo com centenas de milhares de registros. Toda a geometria de congelamento é salva e restaurada automaticamente via `PainelFixoConfig` em `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`.

### 5.3. Janelas Filhas Modeless, Sistema de Limpeza e Contador Dinâmico Localizado no Rodapé (`src/ui/app.rs`, `src/ui/dialogs/modeless.rs`, `src/ui/dialogs/cleanup.rs`)
- **Viewports Nativas Independentes (`show_modeless_dialog`):** Cada janela auxiliar utiliza `egui::ViewportId` próprio e `ModelessState`, gerando janelas reais do Windows 11 que não bloqueiam a janela principal, possuem botões nativos de minimizar/maximizar/fechar e podem ser arrastadas livremente entre múltiplos monitores.
- **Sistema de Limpeza de Dados Individual e Granular (`CleanupDialog` em `src/ui/dialogs/cleanup.rs`):** Diálogo modeless nativo acessível pela Toolbar e menus que permite purgar seletivamente a memória da grade da UI, o banco de eventos (`events/huginn_events.db`) com recriação e `VACUUM`, e a base de segurança (`security/huginn_security.db`). Inclui etapas de confirmação com destaque de perigo, telas de sucesso e aviso de erro, botões simétricos com centralização vertical no eixo da caixa e ajuste dinâmico de altura de viewport no Windows 11.
- **Até 16 Janelas Modeless Simultâneas:** Abrange os 12 diálogos especializados (Filtros Avançados, Estatísticas, Central de Alertas, Central de Sessões, Configurar Colunas, Cores, Recuperação/Quarentena, Multi-Diretório, Exportação de Evidências, Manual, Ajuda e Sobre) mais o Sistema de Limpeza e os painéis modeless de notificações de segurança (Alertas, Sessões, Quarentena e Resumo).
- **Contador Dinâmico Singular/Plural e Pílulas Z-Order:** O rodapé da janela principal monitora em tempo real quantas janelas filhas estão abertas, exibindo o contador localizado no singular ou plural nos 6 idiomas (`active_windows_count_one` / `active_windows_count_many`, ex.: `1 Janela:` vs. `2 Janelas:`, `1 Window:` vs. `2 Windows:`) acompanhado de botões-pílula (`render_centered_emoji_button`) que restauram e elevam instantaneamente o Z-order (`ViewportCommand::Focus`) da janela clicada.

---

## 6. Serviços de Observação, Varredura e Correlação Semântica (`src/services/`)

### 6.1. Observador em Tempo Real (`src/services/observador.rs`)
- Encapsula `notify::RecommendedWatcher` (`notify 8.2.1`) apoiado na API nativa `ReadDirectoryChangesW` do Windows em modo recursivo.
- Executa uma thread worker dedicada que desacopla a captura do sistema de arquivos da interface gráfica via `crossbeam_channel::unbounded`, suportando pausa instantânea (`AtomicBool`) sem perda de estado.
- Correlaciona eventos nativos de renomeação (`RenameMode::From` / `RenameMode::To` / `RenameMode::Both`) através de fila temporal `VecDeque<(PathBuf, Instant)>` (`localizar_registro_antigo`), distinguindo com precisão `OperationType::Renomeado` (mesmo diretório pai) de `OperationType::Movido` (diretórios pais distintos).

### 6.2. Scanner Recursivo Multithread (`src/services/diretorio_scanner.rs`)
- `DiretorioScanner::iniciar_scan` percorre recursivamente a árvore alvo (`std::fs::read_dir`) e distribui os arquivos por um canal limitado `crossbeam_channel::bounded::<PathBuf>(1024)` para um pool elástico de **2 a 8 threads nativas** (`std::thread::available_parallelism().clamp(2, 8)`).
- Emite telemetria interna de progresso (`ScanProgress`) para a barra inferior da UI e suporta pausa e cancelamento cooperativos via `AtomicBool`.

### 6.3. Normalização e Correlação Histórica (`src/services/base_evento.rs`)
- `BaseEvento` mantém um mapa protegido por `parking_lot::Mutex` de exclusões recentes (`pending_deletions`, janela de 3 segundos) para converter pares espúrios `Excluido` + `Adicionado` em eventos semânticos reais (`Movido` ou `Renomeado`).
- Quando um item é excluído do disco, `restaurar_campos_ausentes_de_anterior` recupera do cache e do histórico SQLite os metadados anteriores (tamanho, hashes, datas e contagens compatíveis com o perfil da categoria), preservando a rastreabilidade forense completa.

---

## 7. Persistência Local e Escrita Assíncrona em Lotes (`src/services/db_writer.rs`)

- **SQLite3 Embarcado em Modo WAL:** Operado via `rusqlite 0.41.0` (`features = ["bundled"]`), configurado com `PRAGMA journal_mode = WAL` e `PRAGMA synchronous = NORMAL` para máxima vazão de escrita concorrente à leitura.
- **Arquivos Persistentes em Subdiretórios Isolados (`%LOCALAPPDATA%\HuginnSentinel Sentinel`):**
  - `events/huginn_events.db`: registro transacional de todos os eventos de monitoramento e auditoria (com arquivos WAL/SHM isolados);
  - `security/huginn_security.db`: tabelas de baseline criptográfico, registro de quarentena e perfis multi-diretório (com arquivos WAL/SHM isolados);
  - `quarantine\*.quar`: artefatos isolados pelo cofre de quarentena;
  - `painel_fixo_config.json`: estado persistente das linhas e colunas congeladas na planilha.
- **`DbWriter` Transacional:** Recebe comandos (`InsertEvent`, `InsertBatch`, `ClearTable`, `Flush`, `Shutdown`) por canal assíncrono e acumula lotes de até **2048 eventos** (ou janela máxima de **50 ms** via `recv_timeout`), gravando em transação única SQLite sem bloquear o thread de renderização do `egui`.

---

## 8. Motor de 58 Colunas de Metadados e Harmonização Determinística (`src/services/metadados/`)

- **Extração Profunda (`metadados/extractor.rs` e submódulos):** Preenche as **58 colunas** do `EventModel`, incluindo tamanhos multiescala (`B`, `KB`, `MB`, `GB`, `TB`), carimbos de criação/modificação/acesso, atributos Win32 e NTFS Alternate Data Streams (ADS), detecção de Magic Numbers e tipo MIME real, dimensões de imagem (`images.rs`), duração e bitrate de áudio/vídeo (`media.rs`), páginas/palavras/linhas/slides/planilhas/tabelas (`documents.rs`), contagem de entradas e volume descompactado em arquivos `.zip` (`archives.rs`), metadados de cabeçalhos PE e hashes criptográficos puros em Rust (`sha2`, `sha1`, `md-5` em `executables.rs`).
- **Harmonização Determinística (`metadados/harmonizacao.rs`):** Aplica regras estritas de coerência por perfil de categoria (`perfil_evento`), garantindo que colunas incompatíveis com o tipo real do item (por exemplo, dimensões de imagem em um script de texto ou métricas de arquivo em diretórios) permaneçam limpas e consistentes tanto em eventos ao vivo quanto na recarga histórica do SQLite.

---

## 9. Subsistema de Segurança, Quarentena e Evidências (`src/security/`)

`SecurityManager` (`src/security/manager.rs`) coordena toda a pipeline determinística de segurança sobre cada `EventModel`:

```text
                                 SecurityManager (src/security/manager.rs)
                                                    │
       ┌────────────────────┬───────────────────────┼───────────────────────┬────────────────────┐
       ▼                    ▼                       ▼                       ▼                    ▼
BaselineCriptografico   MotorRegras &          analisar_executavel     Quarentena         ExportacaoEvidencia
(baseline.rs)           ScoreRisco             (executable_intel.rs)   (quarantine.rs)    (evidence.rs)
- Hash SHA-256          (rules.rs /            - WinTrust / crypt32    - Cofre *.quar     - IDs EV-YYYYMMDD-NNNNN
- Auditoria de Drift     risk_score.rs)        - Assinatura X.509      - Check SHA-256    - Pacotes ZIP Forenses
                        - Score 0..100         - Risco de Scripts        pré-restauração    (.json + manifesto)
```

- **Validação Authenticode Win32 (`executable_intel.rs`):** Invoca `WinVerifyTrust` (`wintrust.dll`) e APIs X.509 (`crypt32.dll`) via `windows-sys` para verificar assinaturas digitais e identificar o publicador real de binários PE (`.exe`, `.dll`, `.sys`, `.msi`), além de avaliar risco em scripts (`.ps1`, `.vbs`, `.bat`, `.cmd`, `.js`).
- **Motor de Regras e Score 0–100 (`rules.rs`, `risk_score.rs`):** Detecta extensões típicas de ransomware (`SEC-R01`), extensões duplas dissimuladas (`SEC-R02`), execução em pastas temporárias, desvios de baseline e binários sem assinatura, consolidando o score de `0` a `100` nos níveis `Baixo`, `Médio`, `Alto` e `Crítico`.
- **Quarentena Segura (`quarantine.rs`):** Move ou copia atomicamente o artefato suspeito para `%LOCALAPPDATA%\HuginnSentinel\quarantine\<nome>_<timestamp>.quar`, registra o hash SHA-256 original em `security/huginn_security.db` e exige validação criptográfica SHA-256 estrita antes de qualquer restauração.
- **Exportação e Cadeia de Custódia (`evidence.rs` e `app.rs`):** Atribui identificadores `EV-YYYYMMDD-NNNNN` e hash de custódia SHA-256 a cada evento, exportando relatórios tabulares (`.csv`, `.json`, `.html`, `.txt`, `.xml`) e pacotes forenses compactados (`.zip` contendo `eventos_auditoria.json` e manifesto assinado).

---

## 10. Motor Gráfico e Visualização Estatística (`src/data/` — 17 Gráficos)

O módulo `src/data/` implementa 17 visualizações estatísticas determinísticas definidas no enum `TipoGrafico` (`src/data/charts.rs`), calculadas por `generator.rs` e desenhadas diretamente no canvas vetorial/raster do `egui` (`render/raster.rs` e `render/export.rs`):

1. `PizzaOperacoes` — Pizza de Operações
2. `BarrasTipos` — Barras por Tipo de Arquivo
3. `TimelineOperacoes` — Linha do Tempo de Operações
4. `TreemapTipos` — Treemap por Tipos
5. `HistogramaHorarios` — Histograma por Horários
6. `ParetoExtensoes` — Gráfico de Pareto de Extensões
7. `LinhaEventosHora` — Evolução de Eventos por Hora
8. `BoxplotTamanhos` — Boxplot de Tamanhos de Arquivo
9. `RadarOperacoes` — Radar de Operações
10. `HeatmapDiaHora` — Mapa de Calor (Dia da Semana × Hora)
11. `ScatterTamanhoHora` — Dispersão Tamanho × Horário
12. `SankeyOperacaoTipo` — Fluxo Sankey (Operação → Tipo)
13. `RadarMultidimensional` — Radar Multidimensional de Risco e Volume
14. `DotplotDistribuicao` — Dotplot de Distribuição de Frequência
15. `SankeyEventoDiretorio` — Fluxo Sankey (Evento → Diretório)
16. `SankeyTipoDiretorio` — Fluxo Sankey (Tipo → Diretório)
17. `ArvoreDiretorios` — Árvore Hierárquica de Diretórios

- **Controles Interativos nos Diagramas Sankey:**
  - Menus suspensos com caixas de seleção (*checkboxes*) integradas para seleção dinâmica múltipla de operações/eventos.
  - Seletores de limite de itens (10 a 100 e Todos) com barras de rolagem independentes e ordenação determinística das fitas Bézier cúbicas, prevenindo oscilações ou instabilidades visuais de traçado.
- **Separação Estrita entre Renomeado e Movido:**
  - Garantia de discriminação completa entre arquivos renomeados (`OperationType::Renomeado`) e movidos (`OperationType::Movido`), refletida com exatidão na Linha do Tempo, no Radar Multidimensional de Risco e nos diagramas Sankey.
- **Adaptação Dinâmica aos Temas Claro e Escuro do Windows 11:**
  - `ChartTheme` avalia em tempo de execução o modo de cor ativo no egui/sistema operacional, ajustando dinamicamente paletas cromáticas, espessuras de contorno, contraste de rótulos e fundos translúcidos sem recarregar a aplicação.
- **Refinamentos Ergonômicos:**
  - Remoção de divisórias centrais indesejadas no gráfico de Pizza de Operações, centralização geométrica das legendas na Evolução de Eventos por Hora e fechamento automático de listas suspensas ao remover o cursor da área.

---

## 11. Concorrência, Sincronização e Responsividade

- **`std::thread` Dedicadas:** Threads nativas isoladas para o `Observador`, para o pool paralelo de 2 a 8 workers do `DiretorioScanner` e para a gravação em lote do `DbWriter`.
- **`crossbeam-channel` (`0.5.17`):** Comunicação assíncrona livre de contenção (`unbounded` para eventos em tempo real e comandos de banco; `bounded(1024)` para backpressure controlado durante varreduras massivas).
- **`parking_lot` (`0.13.0`):** `RwLock` no `Localizer` para leituras simultâneas sem bloqueio em todas as células da planilha, e `Mutex` leve para correlação de exclusões e controle de progresso.
- **Orçamento de Drenagem por Frame:** `HuginnApp::processar_mensagens_background` processa lotes limitados a `512` eventos ou `4 ms` por frame, mantendo 60 FPS constantes na planilha e nas janelas modeless secundárias.

---

## 12. Diretrizes de Extensão da Arquitetura

### 12.1. Adicionar um Novo Gráfico Estatístico
1. Adicionar a nova variante ao enum `TipoGrafico` em `src/data/charts.rs` (incluindo `TipoGrafico::all()`, `titulo()` e `descricao()`).
2. Implementar a agregação determinística em `src/data/generator.rs` e a primitiva visual em `src/data/render/raster.rs`.
3. Adicionar as chaves de tradução nos 6 arquivos `src/language/*.json`.

### 12.2. Adicionar uma Nova Coluna na Planilha
1. Incluir o campo correspondente em `EventModel` (`src/core/event.rs`, incluindo `get_field`) e no schema SQLite (`src/services/db_writer.rs`).
2. Preencher e harmonizar o campo em `src/services/metadados/extractor.rs` e `src/services/metadados/harmonizacao.rs`.
3. Registrar `ColunaInfo` em `LISTA_COLUNAS_TODAS` (`src/services/gerenciador_colunas.rs`) e adicionar a tradução nos 6 dicionários `src/language/*.json`.

### 12.3. Adicionar uma Nova Regra de Segurança
1. Implementar a verificação determinística em `MotorRegras::avaliar` (`src/security/rules.rs`) retornando `SecurityRuleMatch`.
2. Ajustar pesos adicionais, se necessário, em `ScoreRisco` (`src/security/risk_score.rs`) e incluir a chave localizada nos dicionários JSON.

### 12.4. Adicionar um Novo Idioma (i18n)
1. Criar o dicionário `src/language/<codigo>.json` espelhando as chaves de `pt_BR.json` e `en_US.json`.
2. Embutir o arquivo via `include_str!` e registrá-lo em `IDIOMAS_SUPORTADOS` dentro de `src/locale/localizer.rs`.
3. Adicionar o conteúdo traduzido do manual do usuário em `src/locale/manual.rs`.

---

## 13. Documentação Relacionada

- [Referência da Linha de Comando (CLI)](./cli-reference.md)
- [Modelo de Segurança e Forense](./security-model.md)
- [Guia do Usuário](./user-guide.md)
- [Conformidade e Privacidade Soberana](./compliance.md)
- [Licenciamento (GNU AGPLv3)](./licensing.md)
- [Sugestões de Issues para Contribuidores](./suggested-issues.md)
- [Texto Integral da Licença GNU AGPLv3](../../../LICENSE)

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the architecture in English</summary><br>

# Huginn Sentinel — System Architecture & Technical Specifications

**Version:** `2026.11.9.0`  
**Date:** `November 9, 2026`  
**Standard:** Rust (Edition 2021) | `egui 0.37.0` / `eframe 0.37.0` | Win32 API (`windows-sys 0.61.2`: `ReadDirectoryChangesW`, `WinTrust`, `crypt32`) | SQLite3 (`rusqlite 0.41.0` `bundled`, WAL mode)  
**Target Platform:** Windows 11 / Windows 10 / Windows Server 2022+ (x64)  
**Binaries (`builds/huginnsentinel/release/`):** `huginnsentinel.exe` (`src/main.rs`) | `huginnsentinel-cli.exe` (`src/bin/main_cli.rs`)  
**License:** GNU Affero General Public License v3.0 (`GNU AGPLv3`)  
**Author:** Fernando Nillsson Cidade  

---

## 1. Architectural Overview & Vision

**Huginn Sentinel** is a native Windows desktop platform engineered entirely in **Rust** (Edition `2021`) with a hardware-accelerated immediate-mode GUI powered by **`egui 0.37.0` / `eframe 0.37.0`**, built to deliver complete filesystem visibility, structured chronological auditing, deterministic non-AI file intelligence, and high-throughput local forensic security monitoring.

The architecture is anchored on **Sovereign Privacy (Local-First)**: all filesystem events, all **58 enriched and harmonized metadata columns**, local WAL-mode SQLite3 databases, frozen-pane layout configurations, and forensic custody bundles are processed and stored exclusively on the user's host machine (`%LOCALAPPDATA%\HuginnSentinel Sentinel`). There are **zero outbound network connections, zero telemetry beacons, zero cloud dependencies, and zero reliance on external AI APIs or Large Language Models (LLMs)**. Every classification, semantic move/rename correlation, metadata harmonization rule, and risk score (0–100) runs on pure deterministic mathematics and algorithmic logic.

Structurally, the platform operates as a **local data pipeline organized into 7 cooperating domains**:

```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          DATA PIPELINE - HUGINN SENTINEL                              │
└─────────────────────────────────────────────────────────────────────────────────────────┘
                                             │
 1. MONITORING DOMAIN (notify 8.2.1 / ReadDirectoryChangesW / DiretorioScanner)
                                             ↓
 2. EVENTS & CORRELATION DOMAIN (crossbeam-channel 0.5.17 / BaseEvento Move & Rename)
                                             ↓
 3. METADATA DOMAIN (58 Columns in extractor.rs + harmonizacao.rs, Magic Numbers, SHA-256)
                                             ↓
 4. PERSISTENCE DOMAIN (rusqlite 0.41.0 bundled in WAL Mode / 2048-Event Batch DbWriter)
                                             ↓
 5. ANALYTICS & STATISTICS DOMAIN (17 Native TipoGrafico Charts / Immediate Raster Engine)
                                             ↓
 6. SECURITY & AUDIT DOMAIN (SHA-256 Baseline, MotorRegras, ScoreRisco, Quarantine Vault)
                                             ↓
 7. PRESENTATION DOMAIN (egui/eframe: Hover-Open Menus, 4-Quadrant Grid, 16 Modeless Windows, CLI)
```

### 1.1. System Architecture Layer Diagram

```text
+-----------------------------------------------------------------------------------------+
|                                    PRESENTATION LAYER                                   |
|                   (src/ui/ — egui 0.37.0 / eframe 0.37.0 + Native CLI)                  |
|                                                                                         |
|  - Persistent Hover-Open Menu Tree (MenuConfig IgnoreClicks + MenuHoverState)           |
|  - 4-Quadrant Synchronized 2D Frozen Spreadsheet (TL, TR, BL, BR) + O(vis) Virtualizing |
|  - Up to 16 Independent Modeless Child Windows with Dynamic Counter & Z-Order Pills     |
|  - Standalone Headless Console Executable (`huginnsentinel-cli.exe`)                 |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                             CHART & ANALYTICS SUBSYSTEM                                 |
|                      (src/data/ — 17 Native TipoGrafico Visualizations)                 |
|                                                                                         |
|    generator.rs (Deterministic Aggregation) | render/raster.rs & render/export.rs       |
|    - Pie, Bar, Timeline, Treemap            | - Heatmap, Sankey, Pareto, Boxplot, Radar |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                    CORE SERVICES LAYER                                  |
|                                      (src/services/)                                    |
|                                                                                         |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  Observer & Scanner         |  |  Semantic Event Correlation (BaseEvento)         |  |
|  |  - notify 8.2.1 (Win32)     |  |  - Renamed vs. Moved across distinct directories |  |
|  |  - crossbeam-channel 0.5.17 |  |  - Post-delete historical attribute recovery     |  |
|  |  - std::thread Pool (2..8)  |  |  - GerenciadorColunas & GerenciadorTabela        |  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  58-Column Metadata Engine  |  |  Asynchronous SQLite3 Persistence                |  |
|  |  - metadados/extractor.rs   |  |  - rusqlite 0.41.0 (bundled) in WAL Mode         |  |
|  |  - metadados/harmonizacao.rs|  |  - Transactional DbWriter (2048 batch / 50 ms)   |  |
|  |  - Pure sha2 / sha1 / md-5  |  |  - %LOCALAPPDATA%\HuginnSentinel Sentinel               |  |
|  +-----------------------------+  +--------------------------------------------------+  |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                                    SECURITY SUBSYSTEM                                   |
|                                      (src/security/)                                    |
|                                                                                         |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  Baseline & Executable Intel|  |  Quarantine & Forensic Evidence                  |  |
|  |  - BaselineCriptografico    |  |  - Safe artifact isolation (`quarantine\*.quar`) |  |
|  |  - analisar_executavel (PE) |  |  - Pre-restoration SHA-256 verification          |  |
|  |  - Drift Auditing           |  |  - Forensic ZIP Bundles (.zip, .json, .csv, .xml)|  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  +-----------------------------+  +--------------------------------------------------+  |
|  |  MotorRegras & ScoreRisco   |  |  Sessions, Diff & MultiDiretorio                 |  |
|  |  - Risk Score 0 to 100      |  |  - CorrelacaoSessoes & AtribuicaoForense         |  |
|  |  - 4 Deterministic Tiers    |  |  - Persistent profiles in huginn_security.db     |  |
|  +-----------------------------+  +--------------------------------------------------+  |
+-----------------------------------------------------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                        NATIVE WINDOWS OS APIS (windows-sys 0.61.2)                      |
|                                                                                         |
|   Win32_Storage_FileSystem (ReadDirectoryChangesW, NTFS ADS, File Attributes)           |
|   Win32_Security_WinTrust & Win32_Security_Cryptography (Authenticode X.509 Signatures) |
|   Win32_UI_Shell & Win32_UI_WindowsAndMessaging & Win32_System_Registry                 |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Dependencies & Build Environment

### 2.1. Language Edition, Release Profile, and Output Binaries
- **Language Standard:** Rust `edition = "2021"` configured via `Cargo.toml` (`version = "2026.10.29"` / display release `2026.10.29.0`).
- **Target Toolchain:** Stable `rustc` targeting `x86_64-pc-windows-msvc`.
- **Release Optimization Profile (`[profile.release]`):** `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true` — produces compact, statically linked native Windows executables with zero external GUI or SQLite DLL dependencies.
- **Build Target Directory (`.cargo/config.toml`):** `target-dir = "builds/huginnsentinel"`, emitting production binaries in `builds/huginnsentinel/release/`:
  - `huginnsentinel.exe` (`src/main.rs`, `#![windows_subsystem = "windows"]`): native desktop graphical application.
  - `huginnsentinel-cli.exe` (`src/bin/main_cli.rs`): standalone headless console CLI for servers, Task Scheduler jobs, and DevSecOps pipelines.

### 2.2. Core Rust Crates (`Cargo.toml`)

| Crate | Version | Architectural Role |
|---|---|---|
| `eframe` / `egui` | `0.37.0` | Hardware-accelerated immediate-mode GUI, independent native OS viewports, persistent menus, and virtualized 2D spreadsheet. |
| `rusqlite` | `0.41.0` (`bundled`) | Statically bundled SQLite3 engine operating in WAL mode for event history and security ledgers. |
| `notify` | `8.2.1` | Continuous recursive filesystem observation backed by Win32 `ReadDirectoryChangesW`. |
| `crossbeam-channel` | `0.5.17` | Lock-free high-throughput MPMC channels connecting observer, scanner, SQLite writer, and UI thread. |
| `parking_lot` | `0.13.0` | Low-overhead `Mutex` and `RwLock` synchronization primitives for caches, localization dictionaries, and correlation queues. |
| `sha2` / `sha1` / `md-5` / `hex` | `0.11.1` / `0.11.0` / `0.11.0` / `0.4.3` | Pure Rust cryptographic hashing (SHA-256, SHA-1, MD5) for file integrity, baselines, quarantine verification, and custody chains. |
| `windows-sys` | `0.61.2` | Direct Win32 FFI bindings (`WinTrust` & `crypt32` Authenticode validation, NTFS ADS, file attributes, Registry, and Shell). |
| `quick-xml` | `0.42.0` | High-speed XML parsing for OOXML/ODF/SVG document metadata inspection and XML report generation. |
| `zip` | `9.1.2` | Archive entry inspection (entry counts and uncompressed byte totals) and forensic `.zip` evidence bundle creation. |
| `rfd` | `0.17.2` | Native Windows file/folder pickers and confirmation message dialogs. |
| `serde` / `serde_json` | `1.0.229` / `1.0.154` | Strongly typed serialization for events, language dictionaries, and `painel_fixo_config.json` persistence. |
| `chrono` / `image` / `uuid` | `0.4.45` / `0.25.10` / `1.10` | Local timestamps, window icon & image dimension decoding, and unique session identifiers. |

There are zero embedded HTTP servers, remote WebSockets, web browser runtimes, or outbound network sockets: Huginn Sentinel operates **100% offline**.

---

## 3. Module Tree & Codebase Breakdown

| Path | Architectural Role |
|---|---|
| `src/main.rs` | Entry point for `huginnsentinel.exe`; resolves optional positional `[directory_path]`, configures `ViewportBuilder`, and starts `eframe::run_native`. |
| `src/bin/main_cli.rs` | Entry point for headless `huginnsentinel-cli.exe` (`scan`, `metadata`, `baseline`, `quarantine`, `monitor`). |
| `src/lib.rs` | Library root (`huginn_rs`), re-exporting `EventModel`, `FileCategory`, `OperationType`, `RiskLevel`, `Localizer`, `SecurityManager`, `BaseEvento`, and `HuginnApp`. |
| `src/core/` | Core domain models (`event.rs` housing the 58-column `EventModel`, `types.rs` with `OperationType`, `FileCategory`, `RiskLevel`, and `error.rs`). |
| `src/data/` | Deterministic analytics and charting subsystem (`charts.rs` with 17 `TipoGrafico` variants, `generator.rs`, `render/raster.rs`, and `render/export.rs`). |
| `src/locale/` + `src/language/*.json` | 6-language localization (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `de_DE`) compiled directly into the binary via `include_str!` in `localizer.rs`, plus localized user manuals in `manual.rs`. |
| `src/security/` | `SecurityManager` facade (`manager.rs`), `BaselineCriptografico` (`baseline.rs`), `MotorRegras` (`rules.rs`), `ScoreRisco` (`risk_score.rs`), `Quarentena` (`quarantine.rs`), `ExportacaoEvidencia` (`evidence.rs`), `analisar_executavel` (`executable_intel.rs`), `AtribuicaoForense` (`attribution.rs`), `DiffConteudo` (`diff.rs`), `CorrelacaoSessoes` (`sessions.rs`), and `MultiDiretorio` (`multi_directory.rs`). |
| `src/services/` | `Observador` (`observador.rs`), `DiretorioScanner` (`diretorio_scanner.rs`), `BaseEvento` (`base_evento.rs`), `DbWriter` (`db_writer.rs`), `GerenciadorColunas` (`gerenciador_colunas.rs`), `GerenciadorTabela` (`gerenciador_tabela.rs`), and 58-column metadata extraction & deterministic harmonization in `metadados/` (`extractor.rs`, `harmonizacao.rs`, `archives.rs`, `documents.rs`, `executables.rs`, `images.rs`, `media.rs`, `types.rs`). |
| `src/tools/` | Multi-dimensional filtering engine (`filters.rs`: `FiltroEstado`, `ContadoresEventos`) and date/calendar picker (`calendar.rs`). |
| `src/ui/` | Main orchestrator `HuginnApp` (`app.rs`), menu bar (`menu_bar.rs`), toolbar (`toolbar.rs`), sidebar filter panel (`filter_panel.rs`), custom geometry widgets & hover menu guardian (`widgets.rs`), spreadsheet view (`table_view.rs`), 13 dialog modules (`dialogs/`), and 2D frozen spreadsheet engine (`spreadsheet/`). |
| `src/utils/` | Resilient user-data and asset path resolution (`paths.rs`), window icons (`icons.rs`), structured logger (`logger.rs`), dark/light themes (`theme.rs`), and Explorer file association helpers (`app_association.rs`). |
| `src/assets/` | Application `.ico`/`.png` icons and multilingual regulatory assets (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE`). Detailed technical documentation is located in `docs/`. |

---

## 4. Bootstrap Flow & Per-Frame Execution Lifecycle

```text
src/main.rs (huginnsentinel.exe)
  │
  ├── Parses optional positional [directory_path] (accepted if it exists and is a valid directory)
  ├── Configures egui::ViewportBuilder (1280x800 initial size, 960x600 min size, window icon)
  └── eframe::run_native("Huginn Sentinel", native_options, ...)
        │
        ▼
      HuginnApp::new(pasta_inicial) (src/ui/app.rs)
        ├── Creates crossbeam_channel::unbounded queues (event_tx/rx and progress_tx/rx)
        ├── Initializes BaseEvento (spawns WAL DbWriter on huginn_events.db & loads history)
        ├── Initializes SecurityManager (opens Baseline, Quarentena, MultiDiretorio on security/huginn_security.db)
        ├── Initializes Localizer (loads 6 embedded JSON dictionaries via include_str! + saved locale)
        ├── Loads GerenciadorColunas, GerenciadorTabela, and GridLayout (painel_fixo_config.json)
        └── Sets initial active directory if provided on startup
              │
              ▼
      Per-Frame Loop — eframe::App::update(&mut self, ctx, _frame)
        ├── iniciar_rastreamento_menus_frame(&ctx) (prepares frame MenuHoverState)
        ├── processar_mensagens_background(&ctx) (drains up to 512 events / 4 ms time budget)
        ├── Incremental filtered-index and counter updates without resorting full history
        ├── Adaptive repaint scheduling: 16 ms (active queue), 33 ms (monitoring), 200 ms (idle)
        ├── Top Panel: renderizar_menu_bar + renderizar_toolbar
        ├── Bottom Panel: scan progress, counters, dynamic localized counter & pills for up to 16 windows
        ├── Left Panel: renderizar_painel_filtros
        ├── Central Panel: renderizar_tabela_eventos (4-quadrant synchronized 2D frozen spreadsheet)
        ├── Independent OS viewport rendering for open modeless child windows (show_modeless_dialog)
        └── finalizar_rastreamento_menus_frame(&ctx) (auto-closes menus once pointer exits active tree)
```

---

## 5. User Interface Architecture (`src/ui/`)

### 5.1. Persistent Hover-Open Menu Tree (`src/ui/menu_bar.rs`, `src/ui/widgets.rs`, `src/ui/spreadsheet/context_menu.rs`)
To allow analysts to toggle multiple checkboxes, adjust filters, pin multiple rows/columns, or trigger repeated actions without menus snapping shut on click:
- **Persistent Menu Configuration (`config_menu_persistente`):** Configures `egui::containers::menu::MenuConfig` with `PopupCloseBehavior::IgnoreClicks` and `submenu_gap(0.0)`, eliminating dead-zone gaps between parent menus and cascading submenus.
- **Unified Menu Builders:** `menu_persistente`, `menu_contexto_persistente`, and `combo_persistente_registrar` enforce consistent multi-click persistence across top-bar menus, nested submenus, right-click spreadsheet context menus, and dropdown `ComboBox` popups.
- **Per-Frame Hover Region Guardian (`MenuHoverState`):** Bracketed by `iniciar_rastreamento_menus_frame` and `finalizar_rastreamento_menus_frame`, every open menu button and popup registers its bounding rectangle each frame (with a `6.0 px` seam tolerance and `is_moving_towards_rect` trajectory protection).
- **Hands-Free Auto-Close:** Any open menu, nested submenu, context menu, or dropdown stays open across unlimited clicks while the pointer remains inside the active menu tree region, closing automatically only when the pointer leaves the open menu region (or `Esc` is pressed).

### 5.2. 4-Quadrant Synchronized 2D Frozen Spreadsheet (`src/ui/table_view.rs`, `src/ui/spreadsheet/grid_layout.rs`)
The central event view is engineered as a high-density forensic spreadsheet supporting simultaneous arbitrary row and column freezing:
- **Arbitrary 2D Pinning (`GridLayout`):**
  - 1st column-header row (`freeze_header_row: bool`);
  - Any arbitrary data row(s) (`pinned_rows: BTreeSet<usize>`), including a 1-based range parser (e.g., `1, 3, 5-8`) with bounded range expansion;
  - The `#` row-number column (`freeze_row_headers: bool`);
  - Any of the **58 metadata columns** (`pinned_cols: BTreeSet<String>`), visually badged with `📌`.
- **4 Synchronized Quadrants:**
  - **`TL` (Top-Left):** Fully stationary intersection of frozen header/pinned rows and frozen `#`/pinned columns;
  - **`TR` (Top-Right):** Horizontally scrolled frozen/pinned rows locked vertically at the top;
  - **`BL` (Bottom-Left):** Vertically scrolled frozen `#` and pinned metadata columns locked horizontally on the left;
  - **`BR` (Bottom-Right):** Main 2D scrollable data viewport synchronized along both X and Y axes.
- **`O(visible)` Virtualization & Persistence:** `compute_visible_rows` renders strictly the visible slice of unpinned rows each frame, sustaining 60 FPS across massive event histories, while `PainelFixoConfig` persists all frozen-pane settings in `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`.

### 5.3. Modeless Child Windows, Data Cleanup System & Dynamic Localized Footer Counter (`src/ui/app.rs`, `src/ui/dialogs/modeless.rs`, `src/ui/dialogs/cleanup.rs`)
- **Independent Native OS Viewports (`show_modeless_dialog`):** Built on dedicated `egui::ViewportId` instances and `ModelessState`, child dialogs open as native Windows 11 top-level windows with standard minimize/maximize/close controls, free multi-monitor dragging, and zero modal blocking of the main window.
- **Granular Data Cleanup System (`CleanupDialog` in `src/ui/dialogs/cleanup.rs`):** A dedicated native modeless dialog accessible via the toolbar and menus that allows selective purging of the UI in-memory grid, the events database (`events/huginn_events.db`) with recreation and `VACUUM`, and the security database (`security/huginn_security.db`). Features danger-highlighted confirmation steps, success/error feedback views, symmetrical buttons with vertical axis centering, and dynamic viewport height resizing in Windows 11.
- **Up to 16 Concurrent Modeless Child Windows:** Covers all 12 core dialogs (Advanced Filters, Statistics, Alert Center, Session Center, Configure Columns, Colors, Quarantine Recovery, Multi-Directory Profiles, Evidence Export, User Manual, Help, and About) plus the Data Cleanup System and the 4 live security notification popups (Alerts, Sessions, Quarantine, and Summary).
- **Dynamic Localized Counter & Z-Order Elevation Pills:** The main window's status bar footer tracks every open child window in real time, rendering a localized singular/plural label across all 6 languages (`active_windows_count_one` / `active_windows_count_many`, e.g., `1 Janela:` vs. `2 Janelas:`, `1 Window:` vs. `2 Windows:`) alongside clickable emoji pills (`render_centered_emoji_button`) that unminimize and raise the target window's Z-order (`ViewportCommand::Focus`).

---

## 6. Observation, Parallel Scanning & Semantic Correlation (`src/services/`)

### 6.1. Real-Time Filesystem Observer (`src/services/observador.rs`)
- Wraps `notify::RecommendedWatcher` (`notify 8.2.1`) over Win32 `ReadDirectoryChangesW` in recursive mode.
- Spawns a dedicated worker thread streaming enriched `EventModel` instances over `crossbeam_channel::unbounded`, with lock-free pause/resume toggling via `AtomicBool`.
- Resolves native rename notifications (`RenameMode::From` / `RenameMode::To` / `RenameMode::Both`) through a bounded temporal queue (`localizar_registro_antigo`), cleanly separating `OperationType::Renomeado` (same parent directory) from `OperationType::Movido` (different parent directories).

### 6.2. Multi-Threaded Directory Scanner (`src/services/diretorio_scanner.rs`)
- `DiretorioScanner::iniciar_scan` traverses directories (`std::fs::read_dir`) and feeds file paths through `crossbeam_channel::bounded::<PathBuf>(1024)` into a worker pool of **2 to 8 OS threads** (`std::thread::available_parallelism().clamp(2, 8)`).
- Streams live `ScanProgress` updates to the status bar and supports instant cooperative pause and cancellation.

### 6.3. Event Normalization & Historical Recovery (`src/services/base_evento.rs`)
- `BaseEvento` tracks recent deletions in a `parking_lot::Mutex` map (`pending_deletions`, 3-second correlation window) to coalesce split delete+create notifications into true `Movido` or `Renomeado` events.
- For deleted items, `restaurar_campos_ausentes_de_anterior` restores prior known sizes, timestamps, SHA-256 hashes, and category-compatible content metrics from cache and SQLite history.

---

## 7. Local Persistence & Asynchronous Batch SQLite WAL (`src/services/db_writer.rs`)

- **Bundled SQLite3 in WAL Mode:** Powered by `rusqlite 0.41.0` (`bundled`), initialized with `PRAGMA journal_mode = WAL` and `PRAGMA synchronous = NORMAL` for non-blocking concurrent reads and high-speed transactional writes.
- **Persistent Storage Layout Across Isolated Subdirectories (`%LOCALAPPDATA%\HuginnSentinel Sentinel`):**
  - `events/huginn_events.db`: transactional database of all monitoring and audit events (with isolated WAL/SHM files);
  - `security/huginn_security.db`: cryptographic baselines, quarantine ledger, and multi-directory profiles (with isolated WAL/SHM files);
  - `quarantine\*.quar`: isolated suspicious artifacts;
  - `painel_fixo_config.json`: persisted 2D frozen-pane spreadsheet state.
- **Transactional Batch `DbWriter`:** Processes `DbCommand` messages on a dedicated background thread, committing transactions in batches of up to **2048 events** or every **50 ms** (`recv_timeout`) so disk I/O never stalls GUI rendering.

---

## 8. 58-Column Metadata Engine & Deterministic Harmonization (`src/services/metadados/`)

- **Deep Extraction (`metadados/extractor.rs` & submodules):** Populates all **58 columns** of `EventModel`, spanning multi-unit sizes (`B`, `KB`, `MB`, `GB`, `TB`), creation/modification/access timestamps, Win32 file attributes and NTFS Alternate Data Streams (ADS), binary Magic Numbers and true MIME detection, image dimensions (`images.rs`), media duration and bitrates (`media.rs`), document pages/words/lines/slides/sheets/tables (`documents.rs`), `.zip` entry counts and uncompressed sizes (`archives.rs`), PE header intelligence, and pure Rust SHA-256/SHA-1/MD5 cryptographic hashes (`executables.rs`).
- **Deterministic Harmonization (`metadados/harmonizacao.rs`):** Enforces strict category-profile invariants (`perfil_evento`) so fields inapplicable to an item's true category (e.g., image dimensions on a source file or file-only metrics on directories) are deterministically cleared and normalized across both live capture and SQLite reloads.

---

## 9. Security, Quarantine & Forensic Evidence Subsystem (`src/security/`)

`SecurityManager` (`src/security/manager.rs`) orchestrates the deterministic local security pipeline across every event:

```text
                                 SecurityManager (src/security/manager.rs)
                                                    │
       ┌────────────────────┬───────────────────────┼───────────────────────┬────────────────────┐
       ▼                    ▼                       ▼                       ▼                    ▼
BaselineCriptografico   MotorRegras &          analisar_executavel     Quarentena         ExportacaoEvidencia
(baseline.rs)           ScoreRisco             (executable_intel.rs)   (quarantine.rs)    (evidence.rs)
- SHA-256 Hashes        (rules.rs /            - WinTrust / crypt32    - *.quar Vault     - EV-YYYYMMDD-NNNNN IDs
- Drift Detection        risk_score.rs)        - Authenticode X.509    - Pre-restore      - Forensic ZIP Bundles
                        - Score 0..100         - Script Risk Heuristics  SHA-256 Check      (.json + manifest)
```

- **Win32 Authenticode Verification (`executable_intel.rs`):** Calls `WinVerifyTrust` (`wintrust.dll`) and `crypt32.dll` certificate routines via `windows-sys` to verify digital signatures and extract publisher identities on PE binaries (`.exe`, `.dll`, `.sys`, `.msi`), alongside heuristic risk inspection for scripts (`.ps1`, `.vbs`, `.bat`, `.cmd`, `.js`).
- **Rule Engine & 0–100 Risk Scoring (`rules.rs`, `risk_score.rs`):** Evaluates ransomware extension patterns (`SEC-R01`), disguised double extensions (`SEC-R02`), temp-folder binary drops, baseline drift, and unsigned executables, producing a deterministic `0`–`100` score classified into `Low`, `Medium`, `High`, and `Critical`.
- **Cryptographic Quarantine (`quarantine.rs`):** Isolates suspicious files into `%LOCALAPPDATA%\HuginnSentinel\quarantine\<filename>_<timestamp>.quar`, records the original SHA-256 digest in `security/huginn_security.db`, and enforces mandatory SHA-256 integrity verification prior to restoration.
- **Forensic Evidence & Multi-Format Export (`evidence.rs` & `app.rs`):** Stamps each event with a unique `EV-YYYYMMDD-NNNNN` custody identifier and SHA-256 evidence digest, supporting direct tabular exports (`.csv`, `.json`, `.html`, `.txt`, `.xml`) as well as compressed `.zip` forensic bundles (`eventos_auditoria.json` + custody manifest).

---

## 10. Native Immediate-Mode Charting Engine (`src/data/` — 17 Charts)

The `src/data/` subsystem implements 17 native statistical visualizations enumerated in `TipoGrafico` (`src/data/charts.rs`), aggregated deterministically in `generator.rs` and rendered via `egui` painter primitives (`render/raster.rs` and `render/export.rs`):

1. `PizzaOperacoes` — Operation Distribution Pie Chart
2. `BarrasTipos` — File Type Bar Chart
3. `TimelineOperacoes` — Chronological Operations Timeline
4. `TreemapTipos` — Hierarchical File Type Treemap
5. `HistogramaHorarios` — Hourly Activity Histogram
6. `ParetoExtensoes` — Cumulative Extension Pareto Chart
7. `LinhaEventosHora` — Hourly Event Trend Line
8. `BoxplotTamanhos` — File Size Dispersion Boxplot
9. `RadarOperacoes` — Multivariate Operations Radar
10. `HeatmapDiaHora` — Day-of-Week × Hour Heatmap
11. `ScatterTamanhoHora` — File Size × Hour Scatter Plot
12. `SankeyOperacaoTipo` — Operation → File Type Sankey Flow
13. `RadarMultidimensional` — Multidimensional Risk & Volume Radar
14. `DotplotDistribuicao` — Frequency Distribution Dotplot
15. `SankeyEventoDiretorio` — Event → Directory Sankey Flow
16. `SankeyTipoDiretorio` — File Type → Directory Sankey Flow
17. `ArvoreDiretorios` — Directory Occupancy Tree

---

## 11. Concurrency, Synchronization & Frame Budgeting

- **Dedicated OS Threads (`std::thread`):** Isolated background threads run `Observador`, the 2–8 worker pool inside `DiretorioScanner`, and the transactional `DbWriter`.
- **`crossbeam-channel` (`0.5.17`):** Contention-free message passing (`unbounded` for live events and DB commands; `bounded(1024)` for backpressure-controlled directory scans).
- **`parking_lot` (`0.13.0`):** `RwLock` inside `Localizer` enables concurrent zero-allocation string lookups across thousands of visible cells, paired with compact `Mutex` guards for correlation windows.
- **Bounded Per-Frame Drain Budget:** `HuginnApp::processar_mensagens_background` drains at most `512` events or `4 ms` per frame, guaranteeing smooth 60 FPS interaction across both the main 4-quadrant spreadsheet and all open modeless child windows.

---

## 12. Architectural Extension Guidelines

### 12.1. Adding a New Statistical Chart
1. Add the variant to `TipoGrafico` in `src/data/charts.rs` (updating `TipoGrafico::all()`, `titulo()`, and `descricao()`).
2. Implement deterministic dataset aggregation in `src/data/generator.rs` and canvas rendering in `src/data/render/raster.rs`.
3. Add localized title and description keys across all 6 `src/language/*.json` dictionaries.

### 12.2. Adding a New Spreadsheet Column
1. Add the field to `EventModel` (`src/core/event.rs`, including `get_field`) and the SQLite table schema (`src/services/db_writer.rs`).
2. Populate and harmonize the attribute in `src/services/metadados/extractor.rs` and `src/services/metadados/harmonizacao.rs`.
3. Register `ColunaInfo` in `LISTA_COLUNAS_TODAS` (`src/services/gerenciador_colunas.rs`) and add header translations in `src/language/*.json`.

### 12.3. Adding a New Security Rule
1. Add the deterministic rule check in `MotorRegras::avaliar` (`src/security/rules.rs`) returning a `SecurityRuleMatch`.
2. Adjust composite scoring weights if needed in `ScoreRisco` (`src/security/risk_score.rs`) and localize the rule label in `src/language/*.json`.

### 12.4. Adding a New Language (i18n)
1. Create `src/language/<locale>.json` matching the key set of `pt_BR.json` and `en_US.json`.
2. Embed the JSON file via `include_str!` and register the locale tuple in `IDIOMAS_SUPORTADOS` inside `src/locale/localizer.rs`.
3. Add the corresponding localized user manual section in `src/locale/manual.rs`.

---

## 13. Related Documentation

- [Command-Line Interface (CLI) Reference](./cli-reference.md)
- [Security & Forensic Model](./security-model.md)
- [User Guide](./user-guide.md)
- [Compliance & Sovereign Privacy](./compliance.md)
- [Licensing & Commercial Clarification (GNU AGPLv3)](./licensing.md)
- [Suggested Community Starter Issues](./suggested-issues.md)
- [Full GNU AGPLv3 License Text](../../../LICENSE)

</details>
