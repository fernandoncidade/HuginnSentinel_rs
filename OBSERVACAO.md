<!-- Multilanguage OBSERVACAO.md for HuginnSentinel_rs -->

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> |
  <a href="#enus">🇺🇸 English (US)</a> |
  <a href="#tree">🌳 Árvore de Arquivos / File Tree</a>
</p>

---

<h2 id="ptbr">🇧🇷 Português (BR)</h2>

<details>
<summary>Clique para expandir / recolher a Observação Técnica em português</summary>

# HUGINN SENTINEL - OBSERVAÇÃO TÉCNICA E POSICIONAMENTO ESTRATÉGICO

Versão documentada: `2026.11.9.0`  
Data do snapshot técnico: `9 de novembro de 2026`  
Repositório: `Huginn_rs`  
Autor identificado no projeto: `Fernando Nillsson Cidade`  
Licença do repositório: `GNU AGPLv3`  

## 1. Estado atual do projeto e Posicionamento

O repositório do **Huginn Sentinel** consolida uma plataforma desktop corporativa **nativa em Rust (Edição 2021)** com interface gráfica em modo imediato acelerada por hardware **`egui 0.37` / `eframe 0.37`** e persistência relacional local SQLite em modo WAL (`rusqlite`).

O produto transcende a categoria de simples visualizador de arquivos ou utilitário passivo de logs, posicionando-se como uma **Plataforma Local de Auditoria de Integridade de Arquivos, Monitoramento em Tempo Real do Windows e Detecção Autônoma de Ameaças** (*Local File System Integrity Audit, Real-Time Monitoring & Threat Detection Platform*).

Sua proposta de valor central responde diretamente à pergunta de governança e resposta a incidentes mais crítica em infraestruturas Windows:
> **“Quais alterações exatas ocorreram no sistema de arquivos, quem as realizou e esse comportamento corresponde a ransomware, desvio de linha de base ou modificação não autorizada?”**

O produto atua como uma solução **100% Local-First** (sem nuvem, sem telemetria, sem contas e **sem qualquer dependência de modelos de IA/LLMs externos**), calculando todas as métricas, categorizações, harmonização de 58 colunas de metadados e laudos periciais através de lógica algorítmica e matemática determinística pura.

## 2. Linha Técnica e Destaques da Versão 2026.11.9.0

Esta versão consolida a maturidade técnica da engine Rust e a estabilização completa dos subsistemas de auditoria corporativa:

- **Motor Kernel Assíncrono (`src/services/observador.rs`):** Monitoramento em tempo real do sistema de arquivos via Windows `ReadDirectoryChangesW` (através do crate `notify`), desacoplado com canais `crossbeam-channel` para garantir taxa de quadros estável na interface gráfica;
- **Varredura Recursiva Multithread (`src/services/diretorio_scanner.rs`):** Escaneamento em pool assíncrono com cancelamento gracioso, extração profunda de metadados forenses e harmonização estrita;
- **Menus Persistentes por Hover (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):** Sistema inteligente de menus, submenus em cascata, menus de contexto e ComboBoxes que permanecem abertos durante múltiplos cliques de seleção e fecham automaticamente assim que o ponteiro do mouse deixa a área do menu;
- **Planilha Virtualizada em 4 Quadrantes com Fixação 2D Arbitrária (`src/ui/table_view.rs`, `src/ui/spreadsheet/*`):** Renderização imediata fluida de centenas de milhares de eventos com fixação bidimensional de quaisquer linhas (inclusive a 1ª linha de cabeçalhos e intervalos como `1, 3, 5-8`) e quaisquer colunas (inclusive a coluna `#`), com menus de contexto integrados e persistência em `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`;
- **Harmonização Determinística de Metadados (`src/services/metadados/harmonizacao.rs`):** Preenchimento e consistência estrita das 58 colunas em todas as 6 operações auditadas (`Adicionado`, `Modificado`, `Excluído`, `Renomeado`, `Movido` e `Escaneado`);
- **Workspace Modeless com Contador Dinâmico no Rodapé (`src/ui/dialogs/*`, `src/ui/app.rs`):** Até 16 janelas filhas independentes no estilo Windows 11 com Z-Order dinâmico, controles de janela clássicos e indicador no rodapé traduzido dinamicamente em tempo de execução: **"1 Janela:"**, **"2 Janelas:"**, **"3 Janelas:"**, etc.;
- **Motor Heurístico de Segurança (`src/security/rules.rs`):** Avaliação contínua de 6 regras canônicas determinísticas (`SEC-R01` a `SEC-R06`), detectando rajadas de ransomware, duplas extensões, criação de scripts não assinados, executáveis ocultos e discrepâncias de integridade;
- **Linha de Base Criptográfica e Deriva (`src/security/baseline.rs`, `src/security/diff.rs`):** Fotografia pericial com cálculo de hashes **SHA-256** e identificação automática de deriva de conformidade (*compliance drift*);
- **Quarentena com Custódia Auditada (`src/security/quarantine.rs`):** Isolamento atômico de arquivos suspeitos na pasta `%LOCALAPPDATA%\HuginnSentinel\quarantine` com extensão inerte `.quar` e cadeia de custódia transacional na tabela `quarentena` do SQLite WAL, condicionando restaurações à revalidação estrita de hash;
- **Inteligência PE e Assinatura Digital (`src/security/executable_intel.rs`):** Análise de cabeçalhos PE, seções, arquitetura, entropia de Shannon e verificação Authenticode via Win32 `WinTrust` (`windows-sys`);
- **17 Gráficos Analíticos Acelerados na GPU (`src/data/charts.rs`, `src/data/render/*`):** Visualizações estatísticas nativas (mapas de calor 7x24, diagramas Sankey, histogramas, dispersões e boxplots) desenhadas com `egui::Painter` e exportáveis para PNG em alta resolução;
- **Persistência Relacional Segura e Sistema de Limpeza Granular (`src/services/base_evento.rs`, `src/services/db_writer.rs`, `src/ui/dialogs/cleanup.rs`):** Armazenamento SQLite local em subdiretórios individuais isolados (`%LOCALAPPDATA%\HuginnSentinel\events\huginn_events.db` e `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`) em modo Write-Ahead Logging (WAL) com commits em lote via thread `DbWriter`, inicialização da GUI centralizada no monitor principal e novo diálogo modeless para purga seletiva de dados.

## 3. Resumo Estrutural Consolidado do Código-Fonte Rust

Contagem validada diretamente na árvore `src/`:

| Módulo / Camada | Arquivos `.rs` | Responsabilidade Técnica Principal |
|---|---:|---|
| `src/core/` | 4 | Modelo canônico de eventos (58 colunas), tipos de dados globais e erros tipados |
| `src/services/` | 16 | Observador `notify`, scanner recursivo, persistência SQLite WAL, gerenciadores de tabela/colunas e 9 módulos de metadados/harmonização |
| `src/security/` | 12 | Regras `SEC-R01`..`SEC-R06`, score de risco 0–100, baseline SHA-256, quarentena `.quar`, inteligência PE e evidências seladas em ZIP |
| `src/data/` | 6 | Processamento estatístico determinístico, gerador de feeds dos 17 gráficos e renderizador/exportador de raster PNG |
| `src/tools/` | 3 | Filtros avançados multicritério com exceções por operação e utilitários de calendário |
| `src/locale/` | 3 | Localizador em tempo de execução com 6 idiomas embutidos e manual interativo integrado |
| `src/ui/` | 33 | Interface principal, barra de menus persistente por hover, tabela virtualizada em 4 quadrantes, widgets, componentes de planilha e 15 diálogos modeless |
| `src/utils/` | 6 | Associação de aplicativos, ícones, logging, caminhos de dados e suporte a temas |
| `src/bin/` | 1 | Utilitário de linha de comando autônomo (`src/bin/main_cli.rs`) |
| Raiz de `src/` | 3 | Script de build (`src/build.rs`), biblioteca raiz (`src/lib.rs`) e ponto de entrada da GUI Desktop (`src/main.rs`) |
| **Total no Projeto Principal (`src/`)** | **87** | **Base completa em Rust (Edição 2021) e egui (0.37)** |
| **Suíte de Testes de Integração (`tests/`)** | **12** | **Testes de eventos, consistência de metadados, planilha, menus, segurança, localizer, etc.** |

## 4. Build, Toolchain e Dependências Reais

Dados validados no arquivo `Cargo.toml`:
- Edição Rust: `edition = "2021"`
- Interface Gráfica: `egui 0.37.0`, `eframe 0.37.0`
- Persistência Relacional: `rusqlite 0.41.0` (bundled, modo WAL)
- Monitoramento de Disco: `notify 8.2.1`
- Criptografia Determinística: `sha2 0.11.1`, `sha1 0.11.0`, `md-5 0.11.0`, `hex 0.4.3`
- Concorrência e Sincronização: `crossbeam-channel 0.5.17`, `parking_lot 0.13.0`
- Manipulação de Datas: `chrono 0.4.45`
- Integração Nativa Windows: `windows-sys 0.61.2` (`ReadDirectoryChangesW`, `WinTrust`, `crypt32`)
- Exportação & Empacotamento: `quick-xml 0.42.0`, `zip 9.1.2`, `rfd 0.17.2`
- Compilação de Recursos Windows: `winres 0.1` (`src/build.rs`, embutimento de `huginn.ico`)

## 5. Atualização Funcional Observada na Versão 2026.11.9.0

- Migração integral e definitiva da base de código legada C++20/Qt6 para Rust e egui;
- Implementação de menus, submenus e menus de contexto persistentes por hover (não fecham com cliques em opções; fecham apenas ao sair da área);
- Fixação bidimensional arbitrária de linhas e colunas (Freeze Panes em 4 quadrantes) com suporte a intervalos numéricos (`1, 3, 5-8`) e persistência JSON;
- Harmonização determinística completa das 58 colunas de metadados para as 6 operações de auditoria;
- Contador dinâmico de janelas modeless ativas traduzido no rodapé (`1 Janela:`, `2 Janelas:`, etc.);
- Encadeamento inteligente e configurável entre escaneamento recursivo e monitoramento contínuo em tempo real (`Configurações > Escaneamento` e `Configurações > Monitoramento`);
- Isolamento estrito entre escaneamento e monitoramento ativo, evitando que eventos em tempo real interfiram durante o escaneamento;
- Contador dinâmico do total de eventos no painel de filtros somando em tempo real os tipos de operações marcados;
- Configuração do comportamento da seleção de células/linhas (modo estático vs. acompanhar dados);
- Janela de personalização da cor de seleção com paleta recomendada, contraste adaptativo dinâmico de texto nos temas Claro e Escuro do Windows 11, código hexadecimal HTML editável (`#00ff00`), pré-visualização comparativa lado a lado e diálogo modal de confirmação;
- Paleta de cores com campo de texto HTML editável, botão "❌" para fechamento rápido e alinhamento vertical dos botões U8 e 📋;
- Gerenciador de colunas simplificado em lista vertical única;
- Botão "🔄 Inverter" de ordenação da coluna selecionada na grade;
- Alinhamento vertical dos botões da barra de ferramentas e das caixas de diálogo, com remoção de linhas divisórias desnecessárias;
- Isolamento seguro de quarentena em arquivos neutros `.quar` acoplado à tabela relacional SQLite `quarentena`;
- Total eliminação de console de terminal na execução da interface gráfica (`#![windows_subsystem = "windows"]`);
- Atualização e alinhamento completo de toda a suíte de documentações técnicas em `docs/`.

</details>

---

<h2 id="enus">🇺🇸 English (US)</h2>

<details>
<summary>Click to expand / collapse Technical Observations in English</summary>

# HUGINN SENTINEL - TECHNICAL OBSERVATION AND STRATEGIC POSITIONING

Documented version: `2026.11.9.0`  
Technical snapshot date: `November 9, 2026`  
Repository: `Huginn_rs`  
Author identified in the project: `Fernando Nillsson Cidade`  
Repository License: `GNU AGPLv3`  

## 1. Current Project State & Positioning

The **Huginn Sentinel** repository consolidates an enterprise desktop platform **native to Rust (2021 Edition)** with a hardware-accelerated immediate-mode graphical interface using **`egui 0.37` / `eframe 0.37`** and local relational SQLite storage in WAL mode (`rusqlite`).

The product transcends the category of a simple file viewer or passive log utility, positioning itself as a **Local File System Integrity Audit, Real-Time Windows Monitoring & Autonomous Threat Detection Platform** (*Plataforma Local de Auditoria de Integridade de Arquivos, Monitoramento em Tempo Real do Windows e Detecção Autônoma de Ameaças*).

Its core value proposition directly answers the most critical governance and incident response question in Windows infrastructures:
> **“What exact changes occurred in the file system, who performed them, and does this behavior correspond to ransomware, baseline drift, or unauthorized modification?”**

The product operates as a **100% Local-First** solution (no cloud, no telemetry, no accounts, and **zero dependency on external AI/LLMs**), computing all metrics, categorizations, 58-column metadata harmonization, and forensic reports through pure algorithmic logic and deterministic mathematics.

## 2. Technical Highlights of Version 2026.11.9.0

This version consolidates the technical maturity of the Rust engine and the complete stabilization of enterprise audit subsystems:

- **Asynchronous Kernel Engine (`src/services/observador.rs`):** Real-time file system monitoring via Windows `ReadDirectoryChangesW` (through the `notify` crate), decoupled with `crossbeam-channel` channels to ensure stable frame rates on the graphical interface;
- **Multithreaded Recursive Scanner (`src/services/diretorio_scanner.rs`):** Asynchronous pool scanning with graceful cancellation, deep forensic metadata extraction, and strict schema harmonization;
- **Persistent Hover-Open Menus (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):** Intelligent system of menus, cascading submenus, context menus, and ComboBoxes that remain open across multiple selection clicks and automatically close as soon as the mouse pointer leaves the menu area;
- **4-Quadrant Virtualized Grid with Arbitrary 2D Pinning (`src/ui/table_view.rs`, `src/ui/spreadsheet/*`):** Fluid immediate-mode rendering of hundreds of thousands of events with 2D freezing of arbitrary rows (including the 1st header row and ranges like `1, 3, 5-8`) and arbitrary columns (including the `#` index column), with integrated context menus and persistence in `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`;
- **Deterministic Metadata Harmonization (`src/services/metadados/harmonizacao.rs`):** Strict completeness and consistency across all 58 columns in all 6 audited operations (`Added`, `Modified`, `Deleted`, `Renamed`, `Moved`, and `Scanned`);
- **Modeless Workspace with Dynamic Footer Counter (`src/ui/dialogs/*`, `src/ui/app.rs`):** Up to 16 independent Windows 11-style child windows with dynamic Z-Order, classic window controls, and a footer indicator translated dynamically at runtime: **"1 Window:"**, **"2 Windows:"**, **"3 Windows:"**, etc.;
- **Heuristic Security Engine (`src/security/rules.rs`):** Continuous evaluation of 6 canonical deterministic rules (`SEC-R01` through `SEC-R06`), detecting ransomware bursts, double extensions, unsigned script creation, hidden binaries, and integrity discrepancies;
- **Cryptographic Baseline & Drift (`src/security/baseline.rs`, `src/security/diff.rs`):** Forensic snapshot calculation with **SHA-256** hashes and automated detection of compliance drift;
- **Audited Quarantine with Custody (`src/security/quarantine.rs`):** Atomic isolation of suspicious files in `%LOCALAPPDATA%\HuginnSentinel\quarantine` with inert `.quar` extensions and transactional chain of custody in the `quarentena` SQLite WAL table, conditioning restoration upon strict hash re-validation;
- **PE Intelligence & Digital Signatures (`src/security/executable_intel.rs`):** Analysis of PE headers, sections, architecture, Shannon entropy, and Authenticode verification via Win32 `WinTrust` (`windows-sys`);
- **17 GPU-Accelerated Analytical Charts (`src/data/charts.rs`, `src/data/render/*`):** Native statistical visualizations (7x24 heatmaps, Sankey diagrams, histograms, scatter plots, and boxplots) drawn with `egui::Painter` and exportable to high-resolution PNG;
- **Secure Relational Persistence & Granular Cleanup System (`src/services/base_evento.rs`, `src/services/db_writer.rs`, `src/ui/dialogs/cleanup.rs`):** Local embedded SQLite storage across isolated subdirectories (`%LOCALAPPDATA%\HuginnSentinel\events\huginn_events.db` and `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`) in Write-Ahead Logging (WAL) mode with batched commit processing via thread `DbWriter`, centered primary monitor startup positioning, and new modeless dialog for selective data purging.

## 3. Consolidated Structural Summary of the Rust Codebase

Count verified directly in the `src/` directory tree:

| Module / Layer | `.rs` Files | Primary Technical Responsibility |
|---|---:|---|
| `src/core/` | 4 | Canonical event model (58 columns), global domain types, and typed errors |
| `src/services/` | 16 | `notify` watcher, recursive scanner, SQLite WAL persistence, table/column managers, and 9 metadata/harmonization modules |
| `src/security/` | 12 | Rules `SEC-R01`..`SEC-R06`, 0–100 risk scoring, SHA-256 baseline, `.quar` quarantine, PE intel, and sealed ZIP evidence |
| `src/data/` | 6 | Deterministic statistical processing, feeds for the 17 charts, and PNG rasterizer/exporter |
| `src/tools/` | 3 | Multi-criteria advanced filters with per-operation exceptions and calendar utilities |
| `src/locale/` | 3 | Runtime localizer with 6 embedded languages and built-in interactive manual |
| `src/ui/` | 33 | Main interface, persistent hover-open menu bar, 4-quadrant virtualized table, widgets, spreadsheet components, and 15 modeless dialogs |
| `src/utils/` | 6 | Application associations, icons, logging, data paths, and theme support |
| `src/bin/` | 1 | Standalone command-line interface utility (`src/bin/main_cli.rs`) |
| Root of `src/` | 3 | Build script (`src/build.rs`), root library crate (`src/lib.rs`), and Desktop GUI entry point (`src/main.rs`) |
| **Total in Main Project (`src/`)** | **87** | **Complete codebase in Rust (2021 Edition) and egui (0.37)** |
| **Integration Test Suite (`tests/`)** | **12** | **Automated tests covering events, metadata consistency, spreadsheet grid, menus, security, localizer, etc.** |

## 4. Build, Toolchain & Actual Dependencies

Data verified in the `Cargo.toml` manifest:
- Rust Edition: `edition = "2021"`
- Graphical Interface: `egui 0.37.0`, `eframe 0.37.0`
- Relational Persistence: `rusqlite 0.41.0` (bundled, WAL mode)
- Disk Monitoring: `notify 8.2.1`
- Deterministic Cryptography: `sha2 0.11.1`, `sha1 0.11.0`, `md-5 0.11.0`, `hex 0.4.3`
- Concurrency & Synchronization: `crossbeam-channel 0.5.17`, `parking_lot 0.13.0`
- Date & Time Handling: `chrono 0.4.45`
- Native Windows Integration: `windows-sys 0.61.2` (`ReadDirectoryChangesW`, `WinTrust`, `crypt32`)
- Export & Packaging: `quick-xml 0.42.0`, `zip 9.1.2`, `rfd 0.17.2`
- Windows Resource Compilation: `winres 0.1` (`src/build.rs`, embedding `huginn.ico`)

## 5. Functional Updates Observed in Version 2026.11.3.0

- Complete and definitive migration of the legacy C++20/Qt6 codebase to Rust and egui;
- Implementation of persistent hover-open menus, submenus, and context menus (they do not close upon clicking options; they close only when the cursor leaves the area);
- Arbitrary two-dimensional pinning of rows and columns (Freeze Panes across 4 quadrants) with numerical range support (`1, 3, 5-8`) and JSON persistence;
- Complete deterministic 58-column metadata harmonization across all 6 audit operations;
- Dynamic runtime-translated active modeless window counter in the footer status bar (`1 Window:`, `2 Windows:`, etc.);
- Intelligent and configurable workflow chaining between recursive scanning and real-time monitoring (`Settings > Scanning` and `Settings > Monitoring`);
- Strict scan isolation ensuring only "Scanned" events are logged during recursive traversals;
- Dynamic total event counter in the quick filter panel computing live totals for active operation checkboxes;
- Configurable cell/row selection behavior (static coordinate vs. follow selection on updates);
- Cell selection color customization with recommended presets, adaptive font contrast under Windows 11 Dark and Light themes, editable HTML hex `#00ff00`, side-by-side comparative preview, and confirmation dialogs;
- Operation color palette with editable HTML text entry, quick "❌" close button, and vertically centered U8 / 📋 actions;
- Streamlined single-column layout for the column visibility manager;
- "🔄 Invert" sort order button on the table control bar;
- Precise vertical centering for toolbar and dialog buttons with seamless frame geometry;
- Secure quarantine isolation into neutral `.quar` files coupled with the SQLite WAL relational `quarentena` table;
- Total elimination of terminal console windows when running the graphical interface (`#![windows_subsystem = "windows"]`);
- Complete update and alignment of the entire technical documentation suite in `docs/`.

</details>

---

<h2 id="tree">🌳 Árvore de Arquivos / File Tree</h2>

<details>
<summary>Clique para expandir a estrutura do repositório / Click to expand repository tree</summary><br>

```text
**Root Path:** `.\src`

```
├── 📁 assets
│   ├── 📁 ABOUT
│   │   ├── 📄 ABOUT_de_DE.txt
│   │   ├── 📄 ABOUT_en_US.txt
│   │   ├── 📄 ABOUT_es_ES.txt
│   │   ├── 📄 ABOUT_fr_FR.txt
│   │   ├── 📄 ABOUT_it_IT.txt
│   │   ├── 📄 ABOUT_pt_BR.txt
│   │   ├── 📄 History_APP_de_DE.txt
│   │   ├── 📄 History_APP_en_US.txt
│   │   ├── 📄 History_APP_es_ES.txt
│   │   ├── 📄 History_APP_fr_FR.txt
│   │   ├── 📄 History_APP_it_IT.txt
│   │   └── 📄 History_APP_pt_BR.txt
│   ├── 📁 CLC
│   │   ├── 📄 CLC_de_DE - Huginn.txt
│   │   ├── 📄 CLC_en_US - Huginn.txt
│   │   ├── 📄 CLC_es_ES - Huginn.txt
│   │   ├── 📄 CLC_fr_FR - Huginn.txt
│   │   ├── 📄 CLC_it_IT - Huginn.txt
│   │   └── 📄 CLC_pt_BR - Huginn.txt
│   ├── 📁 COPYRIGHT
│   │   ├── 📄 AVIS DE COPYRIGHT ET DE MARQUE DÉPOSÉE_fr_FR.txt
│   │   ├── 📄 AVISO DE COPYRIGHT E MARCA REGISTRA_pt_BR.txt
│   │   ├── 📄 AVISO DE DERECHOS DE AUTOR Y MARCA REGISTRADA_es_ES.txt
│   │   ├── 📄 AVVISO SU COPYRIGHT E MARCHIO REGISTRATO_it_IT.txt
│   │   ├── 📄 COPYRIGHT AND TRADEMARK NOTICE_en_US.txt
│   │   └── 📄 HINWEIS ZU URHEBERRECHT UND MARKENZEICHEN_de_DE.txt
│   ├── 📁 EULA
│   │   ├── 📄 EULA_de_DE - Huginn.txt
│   │   ├── 📄 EULA_en_US - Huginn.txt
│   │   ├── 📄 EULA_es_ES - Huginn.txt
│   │   ├── 📄 EULA_fr_FR - Huginn.txt
│   │   ├── 📄 EULA_it_IT - Huginn.txt
│   │   └── 📄 EULA_pt_BR - Huginn.txt
│   ├── 📁 LICENSES
│   │   ├── 📄 Apache License Version 2.0.txt
│   │   ├── 📄 BSD License Clause-2.txt
│   │   ├── 📄 BSD License Clause-3.txt
│   │   ├── 📄 GNU AFFERO GENERAL PUBLIC LICENSE Version3.txt
│   │   ├── 📄 GNU GENERAL PUBLIC LICENSE Version 3.txt
│   │   ├── 📄 GNU LESSER GENERAL PUBLIC LICENSE Version 2.1.txt
│   │   ├── 📄 GNU LESSER GENERAL PUBLIC LICENSE Version 3.txt
│   │   ├── 📄 ISC LICENSE.txt
│   │   ├── 📄 MIT License.txt
│   │   ├── 📄 PYTHON SOFTWARE FOUNDATION LICENSE.txt
│   │   └── 📄 SQLite Is Public Domain.txt
│   ├── 📁 NOTICES
│   │   ├── 📄 NOTICE_de_DE.txt
│   │   ├── 📄 NOTICE_en_US.txt
│   │   ├── 📄 NOTICE_es_ES.txt
│   │   ├── 📄 NOTICE_fr_FR.txt
│   │   ├── 📄 NOTICE_it_IT.txt
│   │   └── 📄 NOTICE_pt_BR.txt
│   ├── 📁 PRIVACY_POLICY
│   │   ├── 📄 Privacy_Policy_de_DE.txt
│   │   ├── 📄 Privacy_Policy_en_US.txt
│   │   ├── 📄 Privacy_Policy_es_ES.txt
│   │   ├── 📄 Privacy_Policy_fr_FR.txt
│   │   ├── 📄 Privacy_Policy_it_IT.txt
│   │   └── 📄 Privacy_Policy_pt_BR.txt
│   ├── 📁 RELEASE
│   │   ├── 📄 RELEASE NOTES_de_DE.txt
│   │   ├── 📄 RELEASE NOTES_en_US.txt
│   │   ├── 📄 RELEASE NOTES_es_ES.txt
│   │   ├── 📄 RELEASE NOTES_fr_FR.txt
│   │   ├── 📄 RELEASE NOTES_it_IT.txt
│   │   └── 📄 RELEASE NOTES_pt_BR.txt
│   └── 📁 images
│       ├── 📁 icons
│       │   ├── 📄 analyze.ico
│       │   ├── 📄 atualizar.ico
│       │   ├── 📄 blocos-de-cores.ico
│       │   ├── 📄 calendar.ico
│       │   ├── 📄 clear.ico
│       │   ├── 📄 clear_button3.ico
│       │   ├── 📄 filtro.ico
│       │   ├── 📄 huginn.ico
│       │   ├── 📄 huginn_1080-1080.ico
│       │   ├── 📄 huginn_1440-2160.ico
│       │   ├── 📄 huginn_150-150.ico
│       │   ├── 📄 huginn_1920-1080.ico
│       │   ├── 📄 huginn_2160-2160.ico
│       │   ├── 📄 huginn_300-300.ico
│       │   ├── 📄 huginn_3840-2160.ico
│       │   ├── 📄 huginn_584-800.ico
│       │   ├── 📄 huginn_71-71.ico
│       │   ├── 📄 paleta-de-cores.ico
│       │   ├── 📄 pause1.ico
│       │   ├── 📄 pause2.ico
│       │   ├── 📄 pause3.ico
│       │   ├── 📄 pause4.ico
│       │   ├── 📄 salvar.ico
│       │   ├── 📄 save_as.ico
│       │   ├── 📄 scan.ico
│       │   ├── 📄 selecione.ico
│       │   ├── 📄 start.ico
│       │   ├── 📄 statistics.ico
│       │   └── 📄 stop.ico
│       └── 📁 imgs
│           ├── 🖼️ analyze.png
│           ├── 🖼️ atualizar.png
│           ├── 🖼️ blocos-de-cores.png
│           ├── 🖼️ calendar.png
│           ├── 🖼️ clear.png
│           ├── 🖼️ clear_button3.png
│           ├── 🖼️ filtro.png
│           ├── 🖼️ huginn.png
│           ├── 🖼️ huginn_1080-1080.png
│           ├── 🖼️ huginn_1440-2160.png
│           ├── 🖼️ huginn_150-150.png
│           ├── 🖼️ huginn_1920-1080.png
│           ├── 🖼️ huginn_2160-2160.png
│           ├── 🖼️ huginn_300-300.png
│           ├── 🖼️ huginn_3840-2160.png
│           ├── 🖼️ huginn_584-800.png
│           ├── 🖼️ huginn_71-71.png
│           ├── 🖼️ paleta-de-cores.png
│           ├── 🖼️ pause1.png
│           ├── 🖼️ pause2.png
│           ├── 🖼️ pause3.png
│           ├── 🖼️ pause4.png
│           ├── 🖼️ salvar.png
│           ├── 🖼️ save_as.png
│           ├── 🖼️ scan.png
│           ├── 🖼️ selecione.png
│           ├── 🖼️ start.png
│           ├── 🖼️ statistics.png
│           └── 🖼️ stop.png
├── 📁 core
│   ├── 🦀 error.rs
│   ├── 🦀 event.rs
│   ├── 🦀 mod.rs
│   ├── 🦀 scan_monitor_config.rs
│   └── 🦀 types.rs
├── 📁 data
│   ├── 📁 render
│   │   ├── 🦀 export.rs
│   │   ├── 🦀 mod.rs
│   │   └── 🦀 raster.rs
│   ├── 🦀 charts.rs
│   ├── 🦀 generator.rs
│   └── 🦀 mod.rs
├── 📁 language
│   ├── ⚙️ de_DE.json
│   ├── ⚙️ en_US.json
│   ├── ⚙️ es_ES.json
│   ├── ⚙️ fr_FR.json
│   ├── ⚙️ it_IT.json
│   └── ⚙️ pt_BR.json
├── 📁 locale
│   ├── 🦀 localizer.rs
│   ├── 🦀 manual.rs
│   └── 🦀 mod.rs
├── 📁 security
│   ├── 🦀 attribution.rs
│   ├── 🦀 baseline.rs
│   ├── 🦀 diff.rs
│   ├── 🦀 evidence.rs
│   ├── 🦀 executable_intel.rs
│   ├── 🦀 manager.rs
│   ├── 🦀 mod.rs
│   ├── 🦀 multi_directory.rs
│   ├── 🦀 quarantine.rs
│   ├── 🦀 risk_score.rs
│   ├── 🦀 rules.rs
│   └── 🦀 sessions.rs
├── 📁 services
│   ├── 📁 metadados
│   │   ├── 🦀 archives.rs
│   │   ├── 🦀 documents.rs
│   │   ├── 🦀 executables.rs
│   │   ├── 🦀 extractor.rs
│   │   ├── 🦀 harmonizacao.rs
│   │   ├── 🦀 images.rs
│   │   ├── 🦀 media.rs
│   │   ├── 🦀 mod.rs
│   │   └── 🦀 types.rs
│   ├── 🦀 base_evento.rs
│   ├── 🦀 db_writer.rs
│   ├── 🦀 diretorio_scanner.rs
│   ├── 🦀 gerenciador_colunas.rs
│   ├── 🦀 gerenciador_tabela.rs
│   ├── 🦀 mod.rs
│   └── 🦀 observador.rs
├── 📁 tools
│   ├── 🦀 calendar.rs
│   ├── 🦀 filters.rs
│   └── 🦀 mod.rs
├── 📁 ui
│   ├── 📁 dialogs
│   │   ├── 🦀 about.rs
│   │   ├── 🦀 advanced_filters.rs
│   │   ├── 🦀 alerts_center.rs
│   │   ├── 🦀 cleanup.rs
│   │   ├── 🦀 colors.rs
│   │   ├── 🦀 columns.rs
│   │   ├── 🦀 evidence_export.rs
│   │   ├── 🦀 help.rs
│   │   ├── 🦀 manual.rs
│   │   ├── 🦀 mod.rs
│   │   ├── 🦀 modeless.rs
│   │   ├── 🦀 multi_dir.rs
│   │   ├── 🦀 notify_config.rs
│   │   ├── 🦀 recovery.rs
│   │   ├── 🦀 session_center.rs
│   │   └── 🦀 statistics.rs
│   ├── 📁 spreadsheet
│   │   ├── 🦀 cell_selection.rs
│   │   ├── 🦀 col_header.rs
│   │   ├── 🦀 context_menu.rs
│   │   ├── 🦀 grid.rs
│   │   ├── 🦀 grid_layout.rs
│   │   ├── 🦀 grid_navigation.rs
│   │   ├── 🦀 grid_renderer.rs
│   │   ├── 🦀 mod.rs
│   │   ├── 🦀 row_col_manager.rs
│   │   ├── 🦀 row_header.rs
│   │   └── 🦀 select_all.rs
│   ├── 🦀 app.rs
│   ├── 🦀 filter_panel.rs
│   ├── 🦀 menu_bar.rs
│   ├── 🦀 mod.rs
│   ├── 🦀 table_view.rs
│   ├── 🦀 toolbar.rs
│   └── 🦀 widgets.rs
├── 📁 utils
│   ├── 🦀 app_association.rs
│   ├── 🦀 icons.rs
│   ├── 🦀 logger.rs
│   ├── 🦀 mod.rs
│   ├── 🦀 paths.rs
│   └── 🦀 theme.rs
├── 🦀 build.rs
├── 🦀 lib.rs
└── 🦀 main.rs
```

**Root Path:** `.\scripts`

```
└── 📄 gerar_distribuicao_limpa.ps1
```

**Root Path:** `.\version`

```
├── 📁 .cargo
│   └── ⚙️ config.toml
├── 📁 src
│   ├── 🦀 app.rs
│   ├── 🦀 engine.rs
│   └── 🦀 main.rs
├── 📄 Cargo.lock
├── ⚙️ Cargo.toml
└── 📝 README.md
```

**Root Path:** `.\docs`

```
├── 📁 images
│   ├── 🖼️ huginn_1920-1080.png
│   └── 🖼️ huginn_300-300.png
├── 📝 MIGRATION_REPORT.md
├── 📝 architecture.md
├── 📝 cli-reference.md
├── 📝 compliance.md
├── 📝 licensing.md
├── 📝 release-v2026.11.6.0.md
├── 📝 security-model.md
├── 📝 suggested-issues.md
└── 📝 user-guide.md
```

**Root Path:** `.\.vscode`

```
├── ⚙️ launch.json
├── ⚙️ settings.json
└── ⚙️ tasks.json
```

**Root Path:** `.\.github`

```
├── 📁 ISSUE_TEMPLATE
│   ├── 📝 bug_report.md
│   ├── ⚙️ config.yml
│   ├── 📝 feature_request.md
│   └── 📝 question.md
├── 📁 workflows
│   └── ⚙️ update-readme-version.yml
└── 📝 PULL_REQUEST_TEMPLATE.md
```

**Root Path:** `.\.cargo`

```
└── ⚙️ config.toml
```

**Root Path:** `.\repo\LISTAGEM`

```
├── 📁 de_DE
│   ├── 🖼️ Captura de tela 2026-11-09 154450.png
│   ├── 🖼️ Captura de tela 2026-11-09 154500.png
│   ├── 🖼️ Captura de tela 2026-11-09 154511.png
│   ├── 🖼️ Captura de tela 2026-11-09 154521.png
│   ├── 🖼️ Captura de tela 2026-11-09 154533.png
│   └── 🖼️ Captura de tela 2026-11-09 154545.png
├── 📁 en_US
│   ├── 🖼️ Captura de tela 2026-11-09 153915.png
│   ├── 🖼️ Captura de tela 2026-11-09 153925.png
│   ├── 🖼️ Captura de tela 2026-11-09 153935.png
│   ├── 🖼️ Captura de tela 2026-11-09 153953.png
│   ├── 🖼️ Captura de tela 2026-11-09 154012.png
│   └── 🖼️ Captura de tela 2026-11-09 154025.png
├── 📁 es_ES
│   ├── 🖼️ Captura de tela 2026-11-09 154049.png
│   ├── 🖼️ Captura de tela 2026-11-09 154057.png
│   ├── 🖼️ Captura de tela 2026-11-09 154106.png
│   ├── 🖼️ Captura de tela 2026-11-09 154115.png
│   ├── 🖼️ Captura de tela 2026-11-09 154127.png
│   └── 🖼️ Captura de tela 2026-11-09 154140.png
├── 📁 fr_FR
│   ├── 🖼️ Captura de tela 2026-11-09 154159.png
│   ├── 🖼️ Captura de tela 2026-11-09 154208.png
│   ├── 🖼️ Captura de tela 2026-11-09 154222.png
│   ├── 🖼️ Captura de tela 2026-11-09 154234.png
│   ├── 🖼️ Captura de tela 2026-11-09 154249.png
│   └── 🖼️ Captura de tela 2026-11-09 154301.png
├── 📁 it_IT
│   ├── 🖼️ Captura de tela 2026-11-09 154322.png
│   ├── 🖼️ Captura de tela 2026-11-09 154332.png
│   ├── 🖼️ Captura de tela 2026-11-09 154344.png
│   ├── 🖼️ Captura de tela 2026-11-09 154354.png
│   ├── 🖼️ Captura de tela 2026-11-09 154416.png
│   └── 🖼️ Captura de tela 2026-11-09 154430.png
└── 📁 pt_BR
    ├── 🖼️ Captura de tela 2026-11-09 153655.png
    ├── 🖼️ Captura de tela 2026-11-09 153714.png
    ├── 🖼️ Captura de tela 2026-11-09 153723.png
    ├── 🖼️ Captura de tela 2026-11-09 153734.png
    ├── 🖼️ Captura de tela 2026-11-09 153824.png
    └── 🖼️ Captura de tela 2026-11-09 153837.png
```

```
</details>
