# Relatório Técnico de Migração Integral: Huginn Sentinel (C++20/Qt6 para Rust/egui)

## 1. Resumo da Migração Realizada

O projeto **Huginn Sentinel** (versão `v2026.11.9.0`), originalmente concebido e prototipado em C++20 com o framework Qt 6 na base legada `Huginn_cpp`, foi integralmente migrado e refatorado para a linguagem **Rust (Edição 2021)** no repositório de produção `HuginnSentinel_rs`.

A base de código em Rust adota o framework gráfico de modo imediato **`egui` (0.37) / `eframe` (0.37)** em conformidade com as diretrizes do projeto, substituindo completamente a infraestrutura legada do Qt 6. Foram preservadas e aprimoradas todas as 58 colunas de metadados da tabela de auditoria (agora com **harmonização determinística estrita** em `harmonizacao.rs`), as 17 visualizações gráficas analíticas nativas aceleradas por GPU, as 6 regras canônicas de segurança (`SEC-R01` a `SEC-R06`), o motor de linha de base criptográfica SHA-256, o sistema de quarentena com custódia em banco relacional SQLite WAL e extensão inerte `.quar`, a internacionalização instantânea para 6 idiomas e novos recursos exclusivos de alta produtividade (fixação 2D arbitrária de linhas e colunas em 4 quadrantes, menus persistentes por hover e contador dinâmico de janelas no rodapé).

---

## 2. Mapeamento dos Módulos C++20 para a Arquitetura Real em Rust

| Módulo Original (C++20 / Qt6) | Responsabilidade Principal | Implementação em Rust (`src/`) |
|---|---|---|
| `EventModel` (`src/core/eventmodel.*`) | Modelo canônico de dados com 58 colunas de metadados | `src/core/event.rs`, `src/core/types.rs` |
| `Observador` (`src/core/observador.*`) | Monitor de eventos em tempo real via Windows I/O assíncrono | `src/services/observador.rs` (via crate `notify 8.2`) |
| `Scanner` (`src/core/scanner.*`) | Varredura recursiva multithread com cancelamento cooperativo | `src/services/diretorio_scanner.rs` |
| `BaseEvento` (`src/core/baseevento.*`) | Desambiguação semântica de eventos (Move vs. Rename) | `src/services/base_evento.rs` |
| `DatabaseManager` (`src/services/databasemanager.*`) | Persistência assíncrona SQLite WAL em lote | `src/services/db_writer.rs` (via crate `rusqlite 0.41`) |
| `GerenciadorTabela` / `GerenciadorColunas` | Gerenciamento de estado, larguras, ordenação e cores da tabela | `src/services/gerenciador_tabela.rs`, `src/services/gerenciador_colunas.rs` |
| `ExtratorMetadados` (`src/services/*`) | Extração forense de atributos, magic numbers e hashes | `src/services/metadados/extractor.rs`, `archives.rs`, `documents.rs`, `executables.rs`, `images.rs`, `media.rs` |
| *Novo no Rust* | Harmonização determinística estrita das 58 colunas para as 6 operações | `src/services/metadados/harmonizacao.rs` |
| `SecurityManager` (`src/security/securitymanager.*`) | Coordenação de regras de segurança e avaliação de incidentes | `src/security/manager.rs` |
| `SecurityRulesEngine` (`src/security/rules.*`) | Avaliação contínua das regras `SEC-R01` a `SEC-R06` | `src/security/rules.rs` |
| `BaselineEngine` (`src/security/baseline.*`) | Fotografia de integridade e auditoria de drift SHA-256 | `src/security/baseline.rs`, `src/security/diff.rs` |
| `QuarantineManager` (`src/security/quarantine.*`) | Isolamento `.quar` e restauração validada por hash | `src/security/quarantine.rs` |
| `RiskScoreCalculator` | Avaliação quantitativa de risco por evento (0 a 100) | `src/security/risk_score.rs` |
| `ExecutableIntel` | Inspeção de cabeçalhos PE, entropia e Authenticode | `src/security/executable_intel.rs` (via `windows-sys`) |
| `GeradorEstatisticas` (`src/data/*`) | Processamento estatístico determinístico para os 17 gráficos | `src/data/generator.rs`, `src/data/charts.rs` |
| `RenderizadorEstatisticas` | Rasterização offscreen e exportação PNG dos gráficos | `src/data/render/raster.rs`, `src/data/render/export.rs` |
| `FiltrosAvancados` (`src/tools/*`) | Filtros multicritério com flags de exceção por operação | `src/tools/filters.rs`, `src/tools/calendar.rs` |
| `Localizador` (`src/locale/*`) | Dicionários embutidos JSON para 6 idiomas e manual interativo | `src/locale/localizer.rs`, `src/locale/manual.rs`, `src/language/*.json` |
| `MainWindow` (`src/ui/mainwindow.*`) | Janela principal, barra de ferramentas e painel de filtros | `src/ui/app.rs`, `src/ui/toolbar.rs`, `src/ui/filter_panel.rs` |
| `MenuBar` & Widgets | Menus superiores e submenus com comportamento persistente por hover | `src/ui/menu_bar.rs`, `src/ui/widgets.rs` |
| `TableView` & Planilha | Planilha em 4 quadrantes com fixação 2D arbitrária (Freeze Panes) e seleção contínua | `src/ui/table_view.rs`, `src/ui/spreadsheet/*` (12 módulos) |
| 17 Diálogos Modeless | Janelas filhas independentes estilo Windows 11 com Z-order nativo (incluindo Sistema de Limpeza Granular) | `src/ui/dialogs/*` (16 módulos de diálogo) |
| `cli_main.cpp` / `main.cpp` | Pontos de entrada para utilitário de console e GUI desktop | `src/bin/main_cli.rs`, `src/main.rs`, `src/lib.rs` |

---

## 3. Relação Consolidada da Estrutura Criada

- **Código-Fonte em `src/`:** **87 arquivos `.rs`** organizados em camadas modulares limpas (`core`, `services`, `security`, `data`, `tools`, `locale`, `ui`, `utils`, `bin`).
- **Suíte de Testes Automatizados em `tests/`:** **12 suítes de teste `.rs`** cobrindo consistência de metadados, eventos, menus, ajuda, banco SQLite, escaneamento, segurança, planilha virtualizada, estatísticas e internacionalização.
- **Dicionários de Tradução em `src/language/`:** 6 arquivos JSON embutidos (`pt_BR.json`, `en_US.json`, `es_ES.json`, `fr_FR.json`, `it_IT.json`, `de_DE.json`).
- **Configurações e Scripts:** `.cargo/config.toml` (redirecionando saída para `builds/huginnsentinel/release/`), `.vscode/tasks.json` (tarefas PowerShell estruturadas), `.gitattributes` (normalização e diff Rust) e `.github/workflows/` (esteiras de CI validadas).

---

## 4. Comparativo de Dependências e Tecnologias

| Aspecto | Projeto Legado (C++20 / Qt6) | Projeto Refatorado (Rust / egui) | Benefício Alcançado |
|---|---|---|---|
| **Linguagem & Toolchain** | C++20 (MSVC / CMake) | Rust 2021 (Cargo / rustc) | Segurança de memória sem garbage collector, ausência de referências nulas e data races eliminadas em compilação. |
| **Framework Gráfico** | Qt 6.7+ (QtWidgets / QtCharts) | `egui 0.37` + `eframe 0.37` | Eliminação de mais de 100 MB em DLLs Qt; renderização imediata nativa acelerada na GPU a 60 FPS com Z-Order dinâmico. |
| **Persistência de Dados** | QSqlDatabase / SQLite C API | `rusqlite 0.41` (bundled) | Banco embutido no binário com modo WAL seguro e gravação assíncrona desacoplada em lote via thread `DbWriter`. |
| **Monitoramento do Disco** | Win32 ReadDirectoryChangesW manual | `notify 8.2` + `crossbeam-channel` | Desacoplamento perfeito entre o kernel do Windows e a thread gráfica sem bloqueio de interface. |
| **Internacionalização** | Arquivos XML `.ts` / lupdate | Dicionários JSON via `include_str!` | Troca instantânea de idioma em microssegundos sem overhead de parsing XML em tempo de execução. |
| **Quarentena** | Isolamento manual básico | Isolamento `.quar` + tabela `quarentena` SQLite | Garantia criptográfica de integridade SHA-256 e custódia rastreável antes de qualquer restauração. |
| **Distribuição e Execução** | Dependente de dezenas de DLLs Qt | Dois executáveis nativos independentes | `huginnsentinel.exe` (GUI) e `huginnsentinel-cli.exe` (CLI) autônomos. |

---

## 5. Inovações Exclusivas da Versão Rust

1. **Menus Persistentes por Hover:** Ao contrário dos menus convencionais que se fecham a cada clique em uma opção, a árvore de menus permanece aberta enquanto o usuário mantiver o ponteiro do mouse em sua área, permitindo múltiplas seleções de forma ultraprodutiva.
2. **Planilha em 4 Quadrantes com Fixação 2D Arbitrária:** Capacidade de congelar qualquer linha (inclusive a primeira de cabeçalhos e intervalos como `1, 3, 5-8`) e qualquer coluna (inclusive a de numeração `#` e colunas selecionadas), mantendo as referências visíveis durante a rolagem.
3. **Harmonização Determinística de Metadados:** Algoritmo estrito em `harmonizacao.rs` que garante que todas as 58 colunas mantenham padrões previsíveis e integridade formal em todas as operações de auditoria.
4. **Contador Dinâmico no Rodapé:** Identificação no rodapé traduzida em tempo real indicando exatamente a quantidade de janelas filhas abertas (**"1 Janela:"**, **"2 Janelas:"**, etc.).
