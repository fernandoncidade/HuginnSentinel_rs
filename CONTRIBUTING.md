# Guia de Contribuição / Contributing Guide — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher as instruções em português</summary><br>

# Contribuindo com o Huginn Sentinel

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  

Antes de tudo, agradecemos seu interesse em contribuir com o **Huginn Sentinel**! Seu apoio torna o monitoramento de integridade e auditoria de segurança do Windows mais transparente, robusto e acessível para todos.

> [!NOTE]
> **Sobre o Desenvolvimento, Suporte e Contribuições Externas:**  
> Este projeto é concebido, desenvolvido e mantido primariamente como uma iniciativa pessoal e educacional pelo autor, que é iniciante e está em processo contínuo de aprendizado em programação e engenharia de software em Rust.  
> O envio de sugestões, abertura de *Issues* e submissão de *Pull Requests (PRs)* são sempre bem-vindos! No entanto, ressalta-se que a análise de código, interação em tópicos técnicos e incorporação de contribuições são realizadas no próprio ritmo do autor e de forma estritamente voluntária. Caso um chamado ou PR demore para ser respondido ou não venha a ser incorporado à branch principal, por favor não reaja com alarde ou frustração. Conforme estabelecido pela licença GNU AGPLv3, você possui total liberdade para criar um *Fork* independente deste repositório e evoluir suas próprias ideias e versões.

---

## 1. Código de Conduta

Ao participar deste projeto, você concorda em cumprir nosso [Código de Conduta](CODE_OF_CONDUCT.md). Trate todos os colaboradores com respeito, cortesia e profissionalismo.

---

## 2. Formas de Contribuir

Você pode colaborar com o Huginn Sentinel de várias maneiras:
- **Relatar Bugs:** Submeta relatórios de bugs claros e reproduzíveis descrevendo o comportamento inesperado em monitoramento, auditoria, congelamento bidimensional de linhas/colunas, menus persistentes por hover ou janelas modeless egui.
- **Sugerir Funcionalidades:** Proponha novas regras de segurança (`SEC-R0*`), heurísticas determinísticas contra ransomware ou visualizações analíticas de arquivos.
- **Melhorar a Documentação:** Esclareça instruções técnicas em `docs/` e `MANUAL.md`, documente casos de uso forenses ou expanda o manual interativo em `src/locale/manual.rs`.
- **Enviar Pull Requests:** Implemente melhorias de performance em Rust, novas consultas analíticas no SQLite WAL ou novos subcomandos em `src/bin/main_cli.rs`.
- **Traduções (i18n):** Contribua aprimorando ou adicionando chaves nos dicionários JSON embutidos em `src/language/*.json` (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `de_DE`) gerenciados dinamicamente em tempo de execução pelo `Localizer` (`src/locale/localizer.rs`).

---

## 3. Configuração do Ambiente de Desenvolvimento

### 3.1. Pré-requisitos
- **Sistema Operacional:** Windows 10 / 11 x64 ou Windows Server 2022+
- **Rust Toolchain:** Versão estável 1.80+ (`rustup default stable`, alvo `x86_64-pc-windows-msvc`)
- **Compilador C++ / Linker:** Visual Studio C++ Build Tools 2022 (MSVC Linker)
- **Gerenciador de Pacotes e Build:** Cargo (incluso na instalação padrão do Rust)

### 3.2. Fluxo de Trabalho com Cargo

```powershell
# Clonar o repositório
git clone https://github.com/fernandoncidade/HuginnSentinel_rs.git
cd HuginnSentinel_rs

# Verificar formatação e lints
cargo fmt --all -- --check
cargo clippy --bins --tests -- -D warnings

# Checagem estática de integridade
cargo check --bins --tests

# Executar toda a suíte de testes unitários e de integração
cargo test --no-fail-fast

# Compilar binários otimizados de produção (Release)
cargo build --release --bin huginn --bin huginnsentinel-cli
```

Conforme configurado em `.cargo/config.toml`, os binários compilados são gerados em `builds/huginnsentinel/`:
- Aplicação Gráfica Desktop: `builds\huginnsentinel\release\huginnsentinel.exe`
- Utilitário Console CLI: `builds\huginnsentinel\release\huginnsentinel-cli.exe`

---

## 4. Padrões de Código e Arquitetura Rust

- **Edição da Linguagem:** Rust 2021 (`edition = "2021"` no `Cargo.toml`).
- **Formatação e Lints:**
  - Execute `cargo fmt --all -- --check` antes de submeter código.
  - O código deve compilar sem avisos em `cargo clippy --bins --tests -- -D warnings`.
- **Integridade da Arquitetura em Camadas (`87` arquivos `.rs` em `src/` + `12` suítes em `tests/`):**
  - `src/core/`: Modelo canônico de eventos de 58 colunas (`event.rs`), tipos globais (`types.rs`) e erros tipados (`error.rs`).
  - `src/services/`: Observador assíncrono em tempo real via `notify` (`observador.rs`), scanner recursivo multithread (`diretorio_scanner.rs`), persistência SQLite WAL em lote (`base_evento.rs`, `db_writer.rs`), gerenciadores de colunas/tabela (`gerenciador_colunas.rs`, `gerenciador_tabela.rs`) e extração/harmonização determinística de metadados (`metadados/extractor.rs`, `metadados/harmonizacao.rs`).
  - `src/security/`: Regras canônicas `SEC-R01`..`SEC-R06` (`rules.rs`), pontuação de risco 0–100 (`risk_score.rs`), baseline criptográfico SHA-256 (`baseline.rs`, `diff.rs`), quarentena `.quar` (`quarantine.rs`), inteligência PE/Authenticode (`executable_intel.rs`), atribuição (`attribution.rs`), sessões (`sessions.rs`) e evidências seladas ZIP (`evidence.rs`).
  - `src/data/`: Geração estatística determinística (`generator.rs`), 17 gráficos nativos (`charts.rs`), diagramas de fluxo Sankey com controles interativos (checkboxes e limitadores), adaptação dinâmica a temas do Windows 11 e rasterização PNG offscreen (`render/raster.rs`, `render/export.rs`).
  - `src/tools/`: Filtros avançados multicritério com exceções de data (`filters.rs`) e calendário (`calendar.rs`).
  - `src/locale/` & `src/language/`: Internacionalização instantânea em 6 idiomas via dicionários JSON (`localizer.rs`) e manual interativo (`manual.rs`).
  - `src/ui/`: Aplicação principal (`app.rs`), barra de menus e widgets com abertura persistente por hover (`menu_bar.rs`, `widgets.rs`), barra de ferramentas (`toolbar.rs`), painel de filtros (`filter_panel.rs`), planilha virtualizada em 4 quadrantes com seleção contínua por clique e arraste de células, colunas e linhas (`spreadsheet/cell_selection.rs`), fixação bidimensional arbitrária (`table_view.rs`, `spreadsheet/*`) e 16 janelas modeless independentes estilo Windows 11 (`dialogs/*`).
  - `src/main.rs` & `src/bin/main_cli.rs`: Pontos de entrada da GUI (`huginnsentinel.exe`) e do CLI headless (`huginnsentinel-cli.exe`).
- **Internacionalização Obrigatória:**
  - Qualquer nova string visível na interface gráfica deve possuir chave correspondente nos 6 arquivos `src/language/*.json` e ser consultada via `Localizer` (`loc.t(...)` / `loc.t_args(...)`).
- **Tratamento Seguro de Erros e Concorrência:**
  - Evite `unwrap()` ou `expect()` em código de produção da engine; propague erros usando `Result<T, E>` com `?`.
  - O monitor de disco e o gravador SQLite jamais devem bloquear a thread de renderização `egui`; utilize canais `crossbeam-channel` e sincronização enxuta com `parking_lot`.

---

## 5. Como Submeter um Pull Request

1. Faça um **Fork** do repositório e crie uma branch a partir de `main`:
   ```bash
   git checkout -b feature/minha-melhoria
   ```
2. Realize seus **Commits** com mensagens claras e semânticas:
   - `feat: adiciona nova regra de detecção de ransomware SEC-R07`
   - `fix: ajusta cálculo de quadrantes fixos na tabela de auditoria`
   - `docs: atualiza guia de operações forenses`
3. Execute e valide a compilação, lints e suíte de testes:
   ```powershell
   cargo fmt --all -- --check
   cargo clippy --bins --tests -- -D warnings
   cargo test --no-fail-fast
   ```
4. Envie a branch para o seu fork e abra um **Pull Request** apontando para a branch `main`. Certifique-se de que a compilação Release MSVC, o Clippy e todos os testes automatizados passaram localmente sem falhas ou avisos.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the instructions in English</summary><br>

# Contributing to Huginn Sentinel

**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  

First off, thank you for considering contributing to **Huginn Sentinel**! Your help makes Windows file system integrity auditing, real-time monitoring, and forensic intelligence more robust, transparent, and accessible for everyone.

> [!NOTE]
> **About Project Development, Support, and External Contributions:**  
> This project is designed, developed, and maintained primarily as a personal and educational initiative by the author, who is a beginner and in an ongoing learning journey in software engineering with Rust.  
> Feedback, suggestions, *Issues*, and *Pull Requests (PRs)* are always welcomed from the community! However, please be aware that reviewing technical inquiries, code submissions, and upstream merges is carried out strictly at the author's own pace on a voluntary basis. If a ticket or PR is not answered promptly or is not merged into the mainline branch, please do not react with alarm or frustration. Under the terms of the GNU AGPLv3 License, you have complete freedom to create an independent *Fork* of this repository to evolve your own ideas and customized builds.

---

## 1. Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please treat all contributors with respect, courtesy, and professionalism.

---

## 2. Ways to Contribute

You can contribute to Huginn Sentinel in many ways:
- **Reporting Bugs:** Submit clear, reproducible issue reports describing unexpected behavior in real-time monitoring, forensic extraction, 2D row/column pinning, persistent hover-open menus, or egui modeless dialogs.
- **Requesting Features:** Propose new security rules (`SEC-R0*`), deterministic ransomware heuristics, or analytical visualizations.
- **Improving Documentation:** Clarify technical instructions in `docs/` and `MANUAL.md`, document forensic scenarios, or expand the built-in interactive manual (`src/locale/manual.rs`).
- **Submitting Pull Requests:** Implement performance optimizations in Rust, new SQLite WAL analytic queries, or CLI subcommands (`src/bin/main_cli.rs`).
- **Translations (i18n):** Help refine or expand runtime translations across the 6 embedded JSON language dictionaries in `src/language/*.json` (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `de_DE`) managed by `Localizer` (`src/locale/localizer.rs`).

---

## 3. Development Environment Setup

### 3.1. Prerequisites
- **OS:** Windows 10 / 11 x64 or Windows Server 2022+
- **Rust Toolchain:** Stable 1.80+ (`rustup default stable`, target `x86_64-pc-windows-msvc`)
- **C++ Compiler / Linker:** Visual Studio C++ Build Tools 2022 (MSVC Linker)
- **Build System:** Cargo (included with default Rust installation)

### 3.2. Building with Cargo

```powershell
# Clone the repository
git clone https://github.com/fernandoncidade/HuginnSentinel_rs.git
cd HuginnSentinel_rs

# Verify formatting and lints
cargo fmt --all -- --check
cargo clippy --bins --tests -- -D warnings

# Static code check
cargo check --bins --tests

# Run all unit and integration tests
cargo test --no-fail-fast

# Build optimized release binaries
cargo build --release --bin huginn --bin huginnsentinel-cli
```

As configured in `.cargo/config.toml`, output binaries are placed in `builds/huginnsentinel/`:
- Desktop GUI: `builds\huginnsentinel\release\huginnsentinel.exe`
- Console CLI: `builds\huginnsentinel\release\huginnsentinel-cli.exe`

---

## 4. Rust Coding Standards & Architecture

- **Language Edition:** Rust 2021 (`edition = "2021"`).
- **Formatting and Lints:**
  - Run `cargo fmt --all -- --check` before opening a pull request.
  - Code must compile with zero warnings under `cargo clippy --bins --tests -- -D warnings`.
- **Layered Architecture (`87` `.rs` files in `src/` + `12` test suites in `tests/`):**
  - `src/core/`: 58-column canonical event model (`event.rs`), shared domain types (`types.rs`), and typed errors (`error.rs`).
  - `src/services/`: Asynchronous real-time watcher via `notify` (`observador.rs`), multithreaded recursive scanner (`diretorio_scanner.rs`), batched SQLite WAL persistence (`base_evento.rs`, `db_writer.rs`), column/table managers (`gerenciador_colunas.rs`, `gerenciador_tabela.rs`), and deterministic metadata extraction/harmonization (`metadados/extractor.rs`, `metadados/harmonizacao.rs`).
  - `src/security/`: Canonical rules `SEC-R01`..`SEC-R06` (`rules.rs`), 0–100 risk scoring (`risk_score.rs`), SHA-256 cryptographic baselines (`baseline.rs`, `diff.rs`), `.quar` quarantine isolation (`quarantine.rs`), PE header & Authenticode intel (`executable_intel.rs`), attribution (`attribution.rs`), sessions (`sessions.rs`), and sealed ZIP evidence bundles (`evidence.rs`).
  - `src/data/`: Deterministic statistics generation (`generator.rs`), 17 native charts (`charts.rs`), interactive Sankey flow controls (checkboxes and item limiters), dynamic Windows 11 theme adaptation, and offscreen PNG rasterization (`render/raster.rs`, `render/export.rs`).
  - `src/tools/`: Multi-criteria advanced filters with per-operation date exceptions (`filters.rs`) and calendar helpers (`calendar.rs`).
  - `src/locale/` & `src/language/`: Dynamic 6-language runtime localization via embedded JSON dictionaries (`localizer.rs`) and built-in interactive manual (`manual.rs`).
  - `src/ui/`: Main application shell (`app.rs`), persistent hover-open menu/submenu & popup tracking (`menu_bar.rs`, `widgets.rs`), toolbar (`toolbar.rs`), quick filter panel (`filter_panel.rs`), 4-quadrant virtualized 58-column spreadsheet with continuous click-and-drag range selection (`spreadsheet/cell_selection.rs`), arbitrary 2D row/column pinning (`table_view.rs`, `spreadsheet/*`), and 16 independent Windows 11 modeless dialogs (`dialogs/*`).
  - `src/main.rs` & `src/bin/main_cli.rs`: Entry points for the Desktop GUI (`huginnsentinel.exe`) and headless CLI (`huginnsentinel-cli.exe`).
- **Mandatory Runtime Localization:**
  - Every new user-facing UI string must be added to all 6 `src/language/*.json` dictionaries and resolved dynamically via `Localizer` (`loc.t(...)` / `loc.t_args(...)`).
- **Safe Error Handling & Concurrency:**
  - Avoid raw `unwrap()` or `expect()` in core engine pipelines; use `Result<T, E>` with the `?` operator.
  - Background watchers and SQLite writers must never block the main `egui` thread; communicate via `crossbeam-channel` and `parking_lot` primitives.

---

## 5. Submitting a Pull Request

1. **Fork** the repository and create your branch from `main`:
   ```bash
   git checkout -b feature/my-cool-feature
   ```
2. **Commit** your changes with clear semantic messages:
   - `feat: add ransomware detection rule SEC-R07`
   - `fix: refine pinned row/column quadrant geometry`
   - `docs: update forensic operations manual`
3. **Verify** formatting, lints, and tests locally:
   ```powershell
   cargo fmt --all -- --check
   cargo clippy --bins --tests -- -D warnings
   cargo test --no-fail-fast
   ```
4. **Push** to your fork and submit a Pull Request targeting the `main` branch. Ensure that formatting, Clippy, MSVC Release builds, and the full test suite pass locally with zero errors or warnings.

</details>
