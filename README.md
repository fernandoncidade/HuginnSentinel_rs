# Huginn Sentinel — Windows Real-Time File System Auditor & Security Monitor

**Plataforma de alta performance para auditoria de integridade de arquivos, monitoramento em tempo real do sistema de arquivos Windows, detecção autônoma de ransomware e inteligência analítica local. Desenvolvido em Rust (Edição 2021) com interface gráfica imediata em egui e persistência SQLite WAL.**

[![Latest Release](https://img.shields.io/badge/Release-v2026.11.9.0-blue.svg)](../../releases/latest)
[![Rust 2021](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust)](Cargo.toml)
[![egui 0.37](https://img.shields.io/badge/egui-0.37-5C7CFA?logo=rust)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows_11_%2F_10_%2F_Server-0078D4?logo=windows)](https://microsoft.com)
[![License AGPLv3](https://img.shields.io/badge/License-GNU_AGPLv3-blue.svg)](LICENSE)
[![100% Offline](https://img.shields.io/badge/Security-100%25_Offline-success.svg)](docs/security-model.md)

<p align="center">
  <a href="docs/">📖 Docs</a> •
  <a href="#-quick-start">🚀 Quick Start</a> •
  <a href="../../discussions">💬 Discussions</a> •
  <a href="../../issues">🐛 Issues</a><br>
  <b>Selecione o idioma / Select language / Seleccione el idioma / Sélectionnez la langue / Seleziona la lingua / Sprache wählen:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> •
  <a href="#enus">🇺🇸 English (US)</a> •
  <a href="#eses">🇪🇸 Español (ES)</a> •
  <a href="#frfr">🇫🇷 Français (FR)</a> •
  <a href="#itit">🇮🇹 Italiano (IT)</a> •
  <a href="#dede">🇩🇪 Deutsch (DE)</a>
</p>

---
<h2 id="ptbr">🇧🇷 Português (BR)</h2>

> **Observação:** Este repositório refere-se à versão estável **`2026.11.9.0`** do Projeto Huginn Sentinel (implementação nativa em **Rust**). Apoie o projeto e instale a versão oficial assinada digitalmente através do link: [Instalar via Microsoft Store](https://apps.microsoft.com/detail/9NHQTGB26LMK)

<details>
<summary><b>Clique para expandir / recolher o README em português</b></summary><br>

<p align="center">
  <img src="repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153655.png" alt="Huginn Sentinel - Interface Desktop em Português" width="850">
</p>

<h4 align="center">📸 Galeria de Capturas de Tela Oficiais (Microsoft Store / Interface Desktop)</h4>

| 1. Interface Principal de Auditoria (58 Colunas Forenses) | 2. Menu Arquivo (Estrutura Persistente por Hover) |
|:---:|:---:|
| [![1. Interface Principal](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153655.png)](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153655.png) | [![2. Menu Arquivo](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153714.png)](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153714.png) |
| **3. Menu Configurações (Submenus em Cascata)** | **4. Menu Opções (Idioma, Manual e Informações)** |
| [![3. Menu Configurações](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153723.png)](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153723.png) | [![4. Menu Opções](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153734.png)](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153734.png) |
| **5. Inteligência Visual (Gráfico Pizza de Operações)** | **6. Estatísticas Avançadas na GPU (Boxplot de Tamanhos)** |
| [![5. Inteligência Visual](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153824.png)](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153824.png) | [![6. Estatísticas Avançadas](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153837.png)](repo/LISTAGEM/pt_BR/Captura%20de%20tela%202026-11-09%20153837.png) |

---

### ⚖️ Manifesto e Autorização Legal do Titular dos Direitos Autorais

> **Declaração Expressa de Fernando Nillsson Cidade:**  
> Eu, **Fernando Nillsson Cidade**, na qualidade de único autor, criador original e detentor dos direitos autorais do projeto **Huginn Sentinel**, **autorizo expressamente o uso, estudo, cópia, modificação, compilação e distribuição do código-fonte deste repositório** estritamente nos termos e condições estabelecidos no arquivo de licença **[LICENSE](LICENSE)** (**GNU Affero General Public License v3 - GNU AGPLv3**).  
> Toda versão modificada ou trabalho derivado deve obrigatoriamente manter os créditos da autoria original e disponibilizar o código-fonte correspondente sob os mesmos termos da licença GNU AGPLv3.

---

### 📌 Ficha Técnica do Projeto

- **Projeto:** HUGINN SENTINEL<br>
- **Versão:** `2026.11.9.0` (Data: 9 de novembro de 2026)<br>
- **Autor e Detentor dos Direitos:** Fernando Nillsson Cidade<br>
- **Linguagem & Ecossistema:** Rust (Edição 2021) | Cargo<br>
- **Framework de Interface:** `egui 0.37` / `eframe 0.37` (Modo imediato acelerado na GPU via DirectX/OpenGL)<br>
- **Banco de Dados Relacional:** SQLite 3 em modo Write-Ahead Logging (WAL) via `rusqlite 0.41` (embutido)<br>
- **I/O e Kernel do Windows:** `notify 8.2` (`ReadDirectoryChangesW`), `windows-sys 0.61`, `crossbeam-channel 0.5`<br>
- **Criptografia e Integridade:** SHA-256, SHA-1, MD5, WinTrust Authenticode (`WinVerifyTrust`)<br>
- **Binários Gerados:** `huginnsentinel.exe` (GUI Desktop) e `huginnsentinel-cli.exe` (Console CLI Headless)<br>
- **Licença do Código-Fonte:** [GNU Affero General Public License v3 (AGPLv3)](LICENSE)<br>

---

### 🔍 Descrição e Proposta de Valor

> **"Saiba com exatidão o que aconteceu com seus arquivos — localmente, visualmente e com histórico pesquisável."**

O **Huginn Sentinel** é uma plataforma corporativa desktop de alta performance desenvolvida para fornecer visibilidade absoluta, rastreamento forense de histórico e auditoria determinística sobre o sistema de arquivos do Windows. 

Diferente de monitores efêmeros ou ferramentas restritivas convencionais, o Huginn Sentinel unifica três pilares essenciais:
1. **Monitoramento Contínuo e Semântico:** Captura em tempo real com desambiguação semântica real entre operações de `RENOMEADO` e `MOVIDO` (eliminando falsos pares de exclusão/criação).
2. **Histórico Pesquisável e Inteligência Forense sem IA:** Banco de dados relacional SQLite WAL com 58 colunas de metadados harmonizados, acompanhado por 17 visualizações gráficas nativas na GPU e relatórios estatísticos determinísticos.
3. **Segurança Local, Quarentena e Auditoria de Evidências:** Linha de base criptográfica SHA-256, detecção autônoma de ransomware e scripts suspeitos via 6 regras canônicas, quarentena transacional em formato inerte `.quar` e exportação auditável multiformato.

#### 🛡️ Privacidade Soberana (Local-First)
* **100% Local e Offline:** Seus arquivos, metadados, históricos e relatórios residem exclusivamente na sua máquina (`%LOCALAPPDATA%\\Huginn Sentinel`).
* **Zero Nuvem, Zero Telemetria, Zero Contas:** Nenhuma informação operacional ou pessoal é transmitida para a rede.
* **Inteligência Determinística sem IA:** Toda a análise, classificação, detecção de incidentes e pontuação de risco (0–100) utilizam lógica algorítmica e matemática pura — sem modelos de linguagem (LLMs), sem RAG e sem dependências externas.

---

### 🧭 Os 5 Modos de Operação

| Modo | Foco Central | Benefício Principal |
|---|---|---|
| **Modo Monitoramento** | *O que está acontecendo agora?* | Visualização imediata de eventos em tempo real com coloração semântica e buffers desacoplados thread-safe. |
| **Modo Histórico** | *O que aconteceu com meu arquivo?* | Consulta cronológica reversa detalhada (*File Genealogy*), exibindo a árvore de modificações, caminhos prévios e hashes. |
| **Modo Analytics (Sem IA)** | *Quais são os padrões de uso?* | 17 gráficos estatísticos GPU (Heatmaps 7x24, diagramas Sankey, histogramas, Pareto, dispersões e boxplots). |
| **Modo Segurança** | *Houve alguma alteração suspeita?* | Avaliação de 6 regras canônicas contra ransomware, extensões duplas, scripts ocultos e desvios de linha de base. |
| **Modo Evidência** | *Como comprovar as alterações?* | Exportação formal de relatórios estruturados e imutáveis em `.xlsx` (OpenXML), `.csv`, `.txt`, `.json`, `.xml` e `.db`. |

---

### ⚙️ Como o Huginn Sentinel Funciona (Fluxo de Dados)

```text
Sistema de Arquivos (Windows)
      ↓
APIs Nativas Assíncronas (ReadDirectoryChangesW / notify)
      ↓
Fila Thread-Safe & Desacoplamento de Pico (crossbeam-channel)
      ↓
Desambiguação Semântica (Detecção Real de Move vs. Rename)
      ↓
Extração Forense & Harmonização Determinística (58 Colunas de Metadados)
      ↓
Persistência Assíncrona em Lote no SQLite Local WAL (DbWriter / rusqlite)
      ↓
Interface Gráfica egui 0.37 / eframe (GPU) | 17 Gráficos Nativos | Exportação Multiformato
```

---

### ✨ Inovações Exclusivas da Arquitetura em Rust

1. **Menus Persistentes por Hover (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):**
   A árvore de menus superiores, submenus em cascata, menus de contexto e ComboBoxes **permanece aberta** enquanto o ponteiro do mouse estiver sobre sua área. Clicar em opções (ativar/desativar colunas, fixar linhas ou alternar filtros) **não fecha o menu**, viabilizando seleções consecutivas com agilidade máxima. O menu fecha suavemente assim que o mouse deixa a região.
2. **Planilha Virtualizada em 4 Quadrantes com Seleção Interativa e Fixação 2D Arbitrária (`src/ui/spreadsheet/*`):**
   Seleção contínua de blocos de células, faixas de colunas ou faixas de linhas arrastando o mouse com realce visual em tempo real, preservação de divisórias de cabeçalhos no Selecionar Tudo e congelamento independente de qualquer linha (inclusive a 1ª de cabeçalhos e intervalos como `1, 3, 5-8`) e qualquer coluna (inclusive `#` e colunas selecionadas), com persistência em `painel_fixo_config.json`.
3. **Harmonização Determinística de Metadados (`src/services/metadados/harmonizacao.rs`):**
   Algoritmo matemático que garante o preenchimento coerente de todas as 58 colunas em todas as 6 operações auditadas (`Adicionado`, `Modificado`, `Excluído`, `Renomeado`, `Movido` e `Escaneado`).
4. **Workspace Modeless com até 16 Janelas Filhas Independentes (`src/ui/dialogs/*`):**
   Diálogos flutuantes estilo Windows 11 com Z-Order dinâmico, controles clássicos de janela e indicador no rodapé traduzido em tempo real: **"1 Janela:"**, **"2 Janelas:"**, **"3 Janelas:"**, etc.
5. **Motor Heurístico de Segurança (`SEC-R01` a `SEC-R06`):**
   Detecção em tempo real de rajadas de ransomware, duplas extensões, criação de scripts não assinados, executáveis em pastas temporárias e deriva de integridade.
6. **Linha de Base Criptográfica SHA-256 e Drift (`src/security/baseline.rs`, `src/security/diff.rs`):**
   Fotografia pericial com cálculo de hashes e comparação instantânea para apontar violações de integridade.
7. **Quarentena com Custódia Auditada (`src/security/quarantine.rs`):**
   Isolamento de arquivos com extensão inerte `.quar` em `%LOCALAPPDATA%\\HuginnSentinel\\quarantine` com cadeia de custódia no SQLite WAL e restauração condicionada à validação de hash.
8. **Inteligência de Executáveis PE & Authenticode (`src/security/executable_intel.rs`):**
   Inspeção de cabeçalhos de executáveis, cálculo de entropia de Shannon e verificação Authenticode via Win32 `WinTrust`.
9. **17 Gráficos Analíticos Nativos na GPU e Adaptação a Temas (`src/data/charts.rs`, `src/data/render/*`):**
   Pizza (sem divisórias espúrias), Barras, Timeline, Treemap, Histograma, Pareto, Linha, Boxplot, Radar, Heatmap 7x24, Scatter, Sankey Operação-Extensão, Dotplot, Sankey Evento-Caminho, Sankey Tipo-Caminho, Árvore de Diretórios e Radar Multivariado. Inclui filtros interativos por checkboxes para operações nos diagramas Sankey, seletores de limite de itens (10 a 100 e Todos) com barras de rolagem e fitas Bézier estáveis, separação precisa entre eventos Renomeados e Movidos, legendas centralizadas no gráfico horário e adaptação dinâmica imediata aos temas Claro e Escuro do Windows 11 em tempo de execução.
10. **Utilitário Autônomo de Linha de Comando (`huginnsentinel-cli.exe`):**
    Executável headless completo para automação em PowerShell, scripts bat, Task Scheduler e servidores sem interface gráfica.

---

### 📂 Estrutura de Diretórios do Código-Fonte

```text
HuginnSentinel_rs/
├── .cargo/config.toml              # Configurações de build do Cargo
├── .vscode/                        # Tarefas (tasks.json), debug (launch.json) e configurações (settings.json)
├── repo/LISTAGEM/                  # Capturas de tela oficiais por idioma (pt_BR, en_US, es_ES, fr_FR, it_IT, de_DE)
├── scripts/                        # Scripts PowerShell de automação e distribuição limpa
├── src/                            # Código-fonte Rust (87 módulos modulares)
│   ├── main.rs                     # Ponto de entrada da aplicação desktop (GUI egui)
│   ├── lib.rs                      # Biblioteca central e re-exportação de módulos
│   ├── bin/main_cli.rs             # Ponto de entrada do utilitário headless de console (CLI)
│   ├── core/                       # Modelo canônico de eventos (58 colunas) e tipos primitivos
│   ├── services/                   # Observador kernel (notify), scanner multithread e banco SQLite (DbWriter)
│   │   └── metadados/              # Extração forense de atributos, magic numbers e harmonização estrita
│   ├── security/                   # Motor canônico de segurança (SEC-R01..R06), baseline SHA-256, quarentena e PE
│   ├── data/                       # Gerador estatístico e renderização imediata dos 17 gráficos
│   ├── tools/                      # Mecanismos de filtros multicritério e calendário
│   ├── locale/                     # Motor de internacionalização e manual interativo
│   ├── language/                   # Dicionários de tradução embutidos JSON (6 idiomas)
│   ├── ui/                         # Interface gráfica: MainWindow, MenuBar com hover, modais e planilha 2D
│   │   ├── spreadsheet/            # Planilha virtualizada em 4 quadrantes, seleção contínua e fixação 2D (12 módulos)
│   │   └── dialogs/                # 16 janelas filhas independentes modeless estilo Windows 11
│   ├── utils/                      # Resolução de caminhos no Windows, ícones e logs ACID
│   └── assets/                     # Recursos visuais (ícones, imagens) e termos legais (ABOUT, EULA, CLC, NOTICES)
├── docs/                           # Documentação técnica e regulatória oficial (arquitetura, compliance, CLI)
├── tests/                          # 12 suítes de testes de integração automatizados
├── Cargo.toml                      # Manifesto de dependências e perfis de otimização
├── LICENSE                         # Texto integral da licença GNU AGPLv3
├── MANUAL.md                       # Manual do usuário completo (Português e Inglês)
├── OBSERVACAO.md                   # Relatório de posicionamento técnico e auditoria
├── CONTRIBUTING.md                 # Guia de contribuição técnica
└── SECURITY.md                     # Política de segurança e reporte de vulnerabilidades
```

---

### 🚀 Compilação e Execução

#### Requisitos
- **Sistema Operacional:** Windows 10, Windows 11 ou Windows Server 2022+ (x64)
- **Toolchain:** Rust 1.80+ (Canal Stable, toolchain `x86_64-pc-windows-msvc` recomendada)

#### Compilar via Linha de Comando (Cargo)
```powershell
# Compilar binários otimizados de produção (GUI e CLI)
cargo build --release --bin huginn --bin huginnsentinel-cli

# Os executáveis finais são gerados em:
# builds\\huginnsentinel\\release\\huginnsentinel.exe
# builds\\huginnsentinel\\release\\huginnsentinel-cli.exe
```

#### Executar Testes Automatizados
```powershell
cargo test
```

#### Executar via Tarefas do VS Code
O projeto disponibiliza tarefas manuais pré-configuradas em `.vscode/tasks.json`. Para executar:
1. Pressione `Ctrl + Shift + P` no VS Code.
2. Digite `Tasks: Run Task` (ou `Tarefas: Executar Tarefa`).
3. Selecione:
   - **Compilar Huginn Sentinel:** compila conjuntamente em Debug e Release e gera as distribuições limpas (local e externa).
   - **Executar Huginn Sentinel:** inicia o executável compilado.
   - **Compilar e Executar Huginn Sentinel:** compila (Debug e Release), sincroniza as distribuições e inicia a aplicação.

> **Importante:** As tarefas do VS Code estão configuradas como `"group": "none"` e direcionam todas as compilações para `builds/` (Debug e Release), garantindo que **nenhuma compilação seja gerada em `target`** e que **nenhuma recompilação ocorra automaticamente** ao excluir o diretório `builds`.

---

### 💻 Referência Rápida do Utilitário CLI (`huginnsentinel-cli.exe`)

```powershell
# Modo Scan: Varredura inicial e snapshot forense de um diretório
.\\builds\\huginnsentinel\release\huginnsentinel-cli.exe scan "C:\Auditoria"

# Modo Watch: Monitoramento contínuo em tempo real via console
.\\builds\\huginnsentinel\release\huginnsentinel-cli.exe watch "C:\Auditoria"

# Modo Baseline: Criar linha de base criptográfica de integridade SHA-256
.\\builds\\huginnsentinel\release\huginnsentinel-cli.exe baseline create "C:\Auditoria" "baseline_v1.json"

# Modo Diff: Comparar diretório contra a linha de base e detectar conformidade
.\\builds\\huginnsentinel\release\huginnsentinel-cli.exe baseline check "C:\Auditoria" "baseline_v1.json"
```

---

### 📚 Documentação Complementar

- **[MANUAL.md](MANUAL.md):** Manual completo de operação da interface, menus e diagnósticos.
- **[OBSERVACAO.md](OBSERVACAO.md):** Relatório analítico detalhado da arquitetura e subsistemas.
- **[docs/architecture.md](docs/architecture.md):** Especificação técnica formal da engine em Rust.
- **[docs/cli-reference.md](docs/cli-reference.md):** Guia exaustivo de todos os comandos do utilitário de console.
- **[docs/security-model.md](docs/security-model.md):** Detalhamento das regras `SEC-R01` a `SEC-R06` e linha de base.
- **[LICENSE](LICENSE):** Texto da licença GNU Affero General Public License v3.

</details>

---

<h2 id="enus">🇺🇸 English (US)</h2>

> **Note:** This repository refers to stable version **`2026.11.9.0`** of the Huginn Sentinel Project (native **Rust** implementation). Support the project and get the officially signed release via: [Install via Microsoft Store](https://apps.microsoft.com/detail/9NHQTGB26LMK)

<details>
<summary><b>Click to expand / collapse the README in English</b></summary><br>

<p align="center">
  <img src="repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153915.png" alt="Huginn Sentinel - Desktop Interface in English" width="850">
</p>

<h4 align="center">📸 Official Screenshot Gallery (Microsoft Store / Desktop Interface)</h4>

| 1. Main Auditing Interface (58 Forensic Columns) | 2. File Menu (Hover-Persistent Menu Structure) |
|:---:|:---:|
| [![1. Main Auditing Interface](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153915.png)](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153915.png) | [![2. File Menu](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153925.png)](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153925.png) |
| **3. Settings Menu (Cascading Hover Submenus)** | **4. Options Menu (Language, User Manual & Info)** |
| [![3. Settings Menu](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153935.png)](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153935.png) | [![4. Options Menu](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153953.png)](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20153953.png) |
| **5. Visual Intelligence (Operations Pie Chart)** | **6. Advanced GPU Analytics (File Size Boxplot)** |
| [![5. Visual Intelligence](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20154012.png)](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20154012.png) | [![6. Advanced GPU Analytics](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20154025.png)](repo/LISTAGEM/en_US/Captura%20de%20tela%202026-11-09%20154025.png) |

---

### ⚖️ Manifesto & Legal Authorization of the Copyright Holder

> **Express Declaration of Fernando Nillsson Cidade:**  
> I, **Fernando Nillsson Cidade**, as the sole author, original creator, and copyright holder of the **Huginn Sentinel** project, **expressly authorize the use, study, copying, modification, compilation, and distribution of the source code in this repository** strictly under the terms and conditions established in the **[LICENSE](LICENSE)** file (**GNU Affero General Public License v3 - GNU AGPLv3**).  
> Any modified version or derivative work must retain original authorship attribution and make its corresponding source code available under the exact same terms of the GNU AGPLv3.

---

### 📌 Project Technical Specifications

- **Project:** HUGINN SENTINEL<br>
- **Version:** `2026.11.9.0` (Date: November 9, 2026)<br>
- **Author & Copyright Holder:** Fernando Nillsson Cidade<br>
- **Language & Ecosystem:** Rust (Edition 2021) | Cargo<br>
- **UI Framework:** `egui 0.37` / `eframe 0.37` (Immediate mode hardware-accelerated via DirectX/OpenGL)<br>
- **Relational Database:** SQLite 3 in Write-Ahead Logging (WAL) mode via `rusqlite 0.41` (bundled)<br>
- **Windows Kernel & I/O:** `notify 8.2` (`ReadDirectoryChangesW`), `windows-sys 0.61`, `crossbeam-channel 0.5`<br>
- **Cryptography & Integrity:** SHA-256, SHA-1, MD5, WinTrust Authenticode (`WinVerifyTrust`)<br>
- **Generated Binaries:** `huginnsentinel.exe` (Desktop GUI) and `huginnsentinel-cli.exe` (Headless Console CLI)<br>
- **Source Code License:** [GNU Affero General Public License v3 (AGPLv3)](LICENSE)<br>

---

### 🔍 Overview and Value Proposition

> **"Know exactly what happened to your files — locally, visually, deterministically."**

**Huginn Sentinel** is a high-performance corporate desktop platform engineered to provide absolute visibility, forensic historical tracking, and deterministic auditing over the Windows file system. 

Unlike ephemeral file monitors or restrictive conventional tools, Huginn Sentinel unifies three core pillars:
1. **Continuous & Semantic Activity Monitoring:** Real-time event capture with semantic disambiguation between real `RENAMED` and `MOVED` operations (eliminating artificial deletion/creation pairs).
2. **Searchable History & Forensic Intelligence without AI:** Local SQLite WAL relational database enriched with 58 harmonized metadata columns, 17 native GPU statistical charts, and deterministic forensic reports.
3. **Local Security, Quarantine & Audit Evidence:** SHA-256 cryptographic baseline, autonomous ransomware and script detection via 6 canonical rules, transactional quarantine in inert `.quar` format, and multi-format evidence export.

#### 🛡️ Sovereign Privacy (Local-First)
* **100% Local & Offline:** Your files, metadata, audit histories, and reports reside exclusively on your machine (`%LOCALAPPDATA%\\Huginn Sentinel`).
* **Zero Cloud, Zero Telemetry, Zero Accounts:** No operational or private data is ever transmitted across the network.
* **Deterministic Intelligence without AI:** All risk scoring (0–100), categorization, incident detection, and forensic reporting rely purely on algorithmic and mathematical logic — no LLMs, no RAG, and no external cloud services.

---

### 🧭 The 5 Operating Modes

| Mode | Core Question | Main Benefit |
|---|---|---|
| **Monitoring Mode** | *What is happening right now?* | Real-time event stream with semantic color-coding and lockless thread-safe buffers. |
| **History Mode** | *What happened to my file?* | Reverse chronological lookup (*File Genealogy*), tracing the entire tree of modifications, previous paths, and hashes. |
| **Analytics Mode (No AI)** | *What are the usage patterns?* | 17 GPU statistical charts (7x24 heatmaps, Sankey diagrams, histograms, Pareto, scatter, and boxplots). |
| **Security Mode** | *Was there any suspicious change?* | Evaluation against 6 canonical rules detecting ransomware bursts, double extensions, unsigned scripts, and baseline drift. |
| **Evidence Mode** | *How to prove the file changes?* | Formal export of immutable structured audit reports in `.xlsx` (OpenXML), `.csv`, `.txt`, `.json`, `.xml`, and `.db`. |

---

### ⚙️ System Architecture & Data Flow

```text
Windows File System
      ↓
Asynchronous Native APIs (ReadDirectoryChangesW / notify)
      ↓
Thread-Safe Queue & Peak Buffering (crossbeam-channel)
      ↓
Semantic Disambiguation (True Move vs. Rename Detection)
      ↓
Forensic Extraction & Deterministic Harmonization (58 Metadata Columns)
      ↓
Batch Asynchronous Persistence in Local SQLite WAL (DbWriter / rusqlite)
      ↓
Hardware-Accelerated egui 0.37 / eframe (GPU) | 17 Native Charts | Multi-Format Export
```

---

### ✨ Exclusive Features of the Rust Architecture

1. **Hover-Persistent Menus (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):**  
   Top menus, cascading submenus, context menus, and ComboBoxes **stay open** while the mouse pointer hovers over their area. Clicking options (toggling columns, pinning rows, or switching filters) **does not close the menu**, enabling rapid consecutive selections. The menu automatically collapses as soon as the cursor leaves the area.
2. **4-Quadrant Virtualized Spreadsheet with Interactive Drag Selection & 2D Pinning (`src/ui/spreadsheet/*`):**  
   Continuous click-and-drag range selection with real-time visual feedback across cells, columns, and rows, preserved header dividers on Select All, and independent freezing of any row (including header row 1 and arbitrary intervals like `1, 3, 5-8`) and any column (including `#` and selected columns), persisted in `painel_fixo_config.json`.
3. **Deterministic Metadata Harmonization (`src/services/metadados/harmonizacao.rs`):**  
   Strict algorithm ensuring consistent population of all 58 columns across all 6 audited operations (`Added`, `Modified`, `Deleted`, `Renamed`, `Moved`, and `Scanned`).
4. **Modeless Workspace with up to 16 Independent Child Windows (`src/ui/dialogs/*`):**  
   Windows 11-style floating dialogs with dynamic Z-Order, classic window controls, and a translated live footer counter: **"1 Window:"**, **"2 Windows:"**, **"3 Windows:"**, etc.
5. **Heuristic Security Rules Engine (`SEC-R01` to `SEC-R06`):**  
   Real-time detection of ransomware bursts, disguised double extensions, unsigned scripts in temp folders, hidden executables, and integrity drift.
6. **SHA-256 Cryptographic Baseline & Drift Audit (`src/security/baseline.rs`, `src/security/diff.rs`):**  
   Forensic snapshot with hash calculation and instant comparison to detect unauthorized file tampering.
7. **Audited Quarantine with Secure Custody (`src/security/quarantine.rs`):**  
   Atomic isolation of suspicious files with inert extension `.quar` in `%LOCALAPPDATA%\\HuginnSentinel\\quarantine` backed by SQLite WAL custody tracking and hash verification upon restore.
8. **PE Executable Intelligence & Authenticode (`src/security/executable_intel.rs`):**  
   Deep inspection of PE headers, Shannon entropy calculation, and Authenticode digital signature validation via Win32 `WinTrust`.
9. **17 Native GPU-Accelerated Charts & Theme Adaptation (`src/data/charts.rs`, `src/data/render/*`):**  
   Clean Pie chart, Bar, Timeline, Treemap, Histogram, Pareto, Line, Boxplot, Radar, 7x24 Heatmap, Scatter, Operation-Extension Sankey, Dotplot, Event-Path Sankey, Type-Path Sankey, Directory Tree, and Multivariate Radar. Includes interactive checkbox filtering for operations in Sankey diagrams, item limiters (10 to 100 and All) with scrollbars and stable Bézier ribbons, strict separation between Renamed and Moved events, centered legends in the hourly event chart, and dynamic on-the-fly adaptation to Windows 11 Light and Dark themes.
10. **Autonomous Headless Console CLI (`huginnsentinel-cli.exe`):**  
    Dedicated console utility for CI/CD pipelines, PowerShell scripting, and headless Windows Server deployments.

---

### 🚀 Build and Execution

#### Prerequisites
- **OS:** Windows 10, Windows 11, or Windows Server 2022+ (x64)
- **Toolchain:** Rust 1.80+ (Stable channel, `x86_64-pc-windows-msvc` recommended)

#### Command Line Compilation (Cargo)
```powershell
# Build production-optimized binaries (GUI and CLI)
cargo build --release --bin huginn --bin huginnsentinel-cli

# Output executables are created in:
# builds\\huginnsentinel\\release\\huginnsentinel.exe
# builds\\huginnsentinel\\release\\huginnsentinel-cli.exe
```

#### Running Automated Tests
```powershell
cargo test
```

#### Running via VS Code Tasks
Configured in `.vscode/tasks.json`:
1. Press `Ctrl + Shift + P` in VS Code.
2. Select `Tasks: Run Task`.
3. Choose **Compilar Huginn Sentinel**, **Executar Huginn Sentinel**, or **Compilar e Executar Huginn Sentinel**.

> **Note:** VS Code tasks are set with `"group": "none"` and route all builds directly to `builds/` (Debug and Release), guaranteeing that **no artifacts are built in `target`** and that **no automatic compilation is triggered** when deleting the `builds` directory.

---

### 📚 Additional Documentation

- **[MANUAL.md](MANUAL.md):** User manual and comprehensive diagnostic guide.
- **[OBSERVACAO.md](OBSERVACAO.md):** Architecture and engineering positioning report.
- **[LICENSE](LICENSE):** GNU Affero General Public License v3 terms.

</details>

---

<h2 id="eses">🇪🇸 Español (ES)</h2>

> **Nota:** Este repositorio se refiere a la versión estable **`2026.11.9.0`** del Proyecto Huginn Sentinel (implementación nativa en **Rust**). Apoye el proyecto e instale la versión oficial firmada digitalmente a través de: [Instalar vía Microsoft Store](https://apps.microsoft.com/detail/9NHQTGB26LMK)

<details>
<summary><b>Haga clic para expandir / contraer el README en español</b></summary><br>

<p align="center">
  <img src="repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154049.png" alt="Huginn Sentinel - Interfaz Desktop en Español" width="850">
</p>

<h4 align="center">📸 Galería de Capturas de Pantalla Oficiales (Microsoft Store / Interfaz Desktop)</h4>

| 1. Interfaz Principal de Auditoría (58 Columnas Forenses) | 2. Menú Archivo (Estructura Persistente por Hover) |
|:---:|:---:|
| [![1. Interfaz Principal](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154049.png)](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154049.png) | [![2. Menú Archivo](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154057.png)](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154057.png) |
| **3. Menú Configuración (Submenús en Cascada)** | **4. Menú Opciones (Idioma, Manual e Información)** |
| [![3. Menú Configuración](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154106.png)](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154106.png) | [![4. Menú Opciones](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154115.png)](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154115.png) |
| **5. Inteligencia Visual (Gráfico Circular de Operaciones)** | **6. Estadísticas Avanzadas en GPU (Boxplot de Tamaños)** |
| [![5. Inteligencia Visual](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154127.png)](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154127.png) | [![6. Estadísticas Avanzadas](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154140.png)](repo/LISTAGEM/es_ES/Captura%20de%20tela%202026-11-09%20154140.png) |

---

### ⚖️ Manifiesto y Autorización Legal del Titular de los Derechos de Autor

> **Declaración Expresa de Fernando Nillsson Cidade:**  
> Yo, **Fernando Nillsson Cidade**, en calidad de único autor, creador original y titular de los derechos de autor del proyecto **Huginn Sentinel**, **autorizo expresamente el uso, estudio, copia, modificación, compilación y distribución del código fuente de este repositorio** estrictamente bajo los términos y condiciones establecidos en el archivo de licencia **[LICENSE](LICENSE)** (**GNU Affero General Public License v3 - GNU AGPLv3**).  
> Cualquier versión modificada o trabajo derivado debe conservar obligatoriamente los créditos de la autoría original y poner a disposición el código fuente correspondiente bajo los mismos términos de la GNU AGPLv3.

---

### 📌 Ficha Técnica del Proyecto

- **Proyecto:** HUGINN SENTINEL<br>
- **Versión:** `2026.11.9.0` (Fecha: 9 de noviembre de 2026)<br>
- **Autor y Titular de Derechos:** Fernando Nillsson Cidade<br>
- **Lenguaje y Ecosistema:** Rust (Edición 2021) | Cargo<br>
- **Interfaz Gráfica:** `egui 0.37` / `eframe 0.37` (Modo inmediato acelerado en GPU vía DirectX/OpenGL)<br>
- **Base de Datos Relacional:** SQLite 3 en modo Write-Ahead Logging (WAL) mediante `rusqlite 0.41` (embebido)<br>
- **Kernel de Windows e I/O:** `notify 8.2` (`ReadDirectoryChangesW`), `windows-sys 0.61`, `crossbeam-channel 0.5`<br>
- **Criptografía e Integridad:** SHA-256, SHA-1, MD5, WinTrust Authenticode (`WinVerifyTrust`)<br>
- **Binarios Generados:** `huginnsentinel.exe` (GUI Desktop) y `huginnsentinel-cli.exe` (Consola CLI Headless)<br>
- **Licencia del Código Fuente:** [GNU Affero General Public License v3 (AGPLv3)](LICENSE)<br>

---

### 🔍 Descripción y Propuesta de Valor

> **"Sepa con exactitud qué ocurrió con sus archivos — localmente, visualmente y con historial consultable."**

**Huginn Sentinel** es una plataforma corporativa desktop de alto rendimiento desarrollada para proporcionar visibilidad absoluta, rastreo forense de historial y auditoría determinista sobre el sistema de archivos de Windows.

A diferencia de los monitores de archivos efímeros o herramientas restrictivas tradicionales, Huginn Sentinel integra tres pilares fundamentales:
1. **Monitoreo Continuo y Semántico:** Captura en tiempo real con desambiguación real entre operaciones de `RENOMBRADO` y `MOVIDO` (eliminando falsos pares de eliminación/creación).
2. **Historial Consultable e Inteligencia Forense sin IA:** Base de datos relacional SQLite WAL enriquecida con 58 columnas de metadatos armonizados, 17 gráficos estadísticos nativos en GPU e informes deterministas.
3. **Seguridad Local, Cuarentena y Evidencias de Auditoría:** Línea base criptográfica SHA-256, detección autónoma de ransomware y scripts sospechosos mediante 6 reglas canónicas, cuarentena en formato inerte `.quar` y exportación multiformato.

#### 🛡️ Privacidad Soberana (Local-First)
* **100% Local y Offline:** Sus archivos, metadados, historiales e informes residen exclusivamente en su equipo (`%LOCALAPPDATA%\\Huginn Sentinel`).
* **Cero Nube, Cero Telemetría, Cero Cuentas:** Ninguna información operativa o personal se transmite a través de la red.
* **Inteligencia Determinista sin IA:** Todo el cálculo de riesgo (0–100), categorización y detección de incidentes utiliza lógica algorítmica y matemática pura — sin LLMs, sin RAG y sin dependencias externas.

---

### 🧭 Los 5 Modos de Operación

| Modo | Pregunta Central | Beneficio Principal |
|---|---|---|
| **Modo Monitoreo** | *¿Qué está ocurriendo ahora?* | Flujo de eventos en tiempo real con coloración semántica y buffers desacoplados thread-safe. |
| **Modo Historial** | *¿Qué ocurrió con mi archivo?* | Consulta cronológica inversa (*File Genealogy*), rastreando la genealogía completa de modificaciones, rutas previas y hashes. |
| **Modo Analytics (Sin IA)** | *¿Cuáles son los patrones de uso?* | 17 gráficos estadísticos GPU (Mapas de calor 7x24, diagramas Sankey, histogramas, Pareto y boxplots). |
| **Modo Seguridad** | *¿Hubo alguna alteración sospechosa?* | Verificación contra 6 reglas canónicas para detectar ransomware, extensiones dobles, scripts no firmados y deriva de línea base. |
| **Modo Evidencia** | *¿Cómo comprobar las alteraciones?* | Exportación formal de informes inmutables en `.xlsx` (OpenXML), `.csv`, `.txt`, `.json`, `.xml` y `.db`. |

---

### ✨ Innovaciones Exclusivas de la Versión en Rust

1. **Menús Persistentes por Hover (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):** La estructura de menús superiores, submenús y ComboBoxes permanece abierta mientras el cursor esté en su área. Hacer clic no cierra el menú, permitiendo selecciones continuas con máxima agilidad.
2. **Hoja Virtualizada en 4 Cuadrantes con Selección por Arrastre y Fijación 2D (`src/ui/spreadsheet/*`):** Selección continua de celdas, columnas y filas arrastrando con resaltado en tiempo real, divisores intactos y congelación independiente de cualquier fila y columna.
3. **Armonización Determinista de Metadados (`src/services/metadados/harmonizacao.rs`):** Consistencia estricta de las 58 columnas en las 6 operaciones auditadas.
4. **Espacio de Trabajo Modeless con hasta 16 Ventanas Hijas (`src/ui/dialogs/*`):** Diálogos estilo Windows 11 con Z-Order dinámico y contador en el pie de página traducido en tiempo real.
5. **Motor Heurístico de Seguridad (`SEC-R01` a `SEC-R06`):** Detección temprana de ransomware, ejecutables ocultos y manipulación de archivos.
6. **Línea Base Criptográfica SHA-256 y Auditoría de Deriva:** Comparación instantánea para identificar alteraciones no autorizadas.
7. **Cuarentena con Custodia Auditada:** Aislamiento seguro `.quar` y registro transaccional en SQLite WAL.
8. **Inteligencia de Ejecutables PE y Authenticode:** Inspección de cabeceras PE, entropía de Shannon y firmas digitales.
9. **17 Gráficos Estadísticos Nativos en GPU y Adaptación de Temas:** Visualizaciones analíticas aceleradas por hardware, diagramas Sankey con filtros interactivos por casillas de verificación y límites con barra de desplazamiento, separación estricta de Renombrado/Movido y adaptación dinámica a los temas Claro y Oscuro de Windows 11.
10. **Utilidad de Consola Headless (`huginnsentinel-cli.exe`):** Herramienta para automatización en scripts PowerShell, servidores y CI/CD.

---

### 🚀 Compilación y Ejecución

```powershell
# Compilar binarios optimizados (GUI y CLI)
cargo build --release --bin huginn --bin huginnsentinel-cli

# Ejecutar pruebas automatizadas
cargo test
```

> **Nota sobre VS Code:** Las tareas en `.vscode/tasks.json` usan `"group": "none"` y enrutan todas las compilaciones a `builds/` (Debug y Release), garantizando que **no se genere nada en `target`** e impidiendo cualquier recompilación automática al borrar el directorio `builds`.

---

### 📚 Documentación Adicional

- **[MANUAL.md](MANUAL.md):** Manual completo de usuario y guía de diagnóstico.
- **[OBSERVACAO.md](OBSERVACAO.md):** Informe de arquitectura y posicionamiento técnico.
- **[LICENSE](LICENSE):** Términos de la licencia GNU AGPLv3.

</details>

---

<h2 id="frfr">🇫🇷 Français (FR)</h2>

> **Remarque :** Ce dépôt fait référence à la version stable **`2026.11.9.0`** du Projet Huginn Sentinel (implémentation native en **Rust**). Soutenez le projet et installez la version officielle signée numériquement via : [Installer via Microsoft Store](https://apps.microsoft.com/detail/9NHQTGB26LMK)

<details>
<summary><b>Cliquez pour afficher / masquer le README en français</b></summary><br>

<p align="center">
  <img src="repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154159.png" alt="Huginn Sentinel - Interface Desktop en Français" width="850">
</p>

<h4 align="center">📸 Galerie de Captures d'Écran Officielles (Microsoft Store / Interface Desktop)</h4>

| 1. Interface Principale d'Audit (58 Colonnes Forensiques) | 2. Menu Fichier (Structure Persistante au Survol) |
|:---:|:---:|
| [![1. Interface Principale](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154159.png)](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154159.png) | [![2. Menu Fichier](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154208.png)](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154208.png) |
| **3. Menu Configuration (Sous-menus en Cascade)** | **4. Menu Options (Langue, Manuel et Informations)** |
| [![3. Menu Configuration](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154222.png)](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154222.png) | [![4. Menu Options](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154234.png)](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154234.png) |
| **5. Renseignement Visuel (Camembert des Opérations)** | **6. Statistiques GPU Avancées (Boîte à Moustaches)** |
| [![5. Renseignement Visuel](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154249.png)](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154249.png) | [![6. Statistiques GPU Avancées](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154301.png)](repo/LISTAGEM/fr_FR/Captura%20de%20tela%202026-11-09%20154301.png) |

---

### ⚖️ Manifeste et Autorisation Légale du Titulaire des Droits d'Auteur

> **Déclaration Expresse de Fernando Nillsson Cidade :**  
> Moi, **Fernando Nillsson Cidade**, en qualité d'unique auteur, créateur original et titulaire des droits d'auteur du projet **Huginn Sentinel**, **j'autorise expressément l'utilisation, l'étude, la copie, la modification, la compilation et la distribution du code source de ce dépôt** strictement selon les termes et conditions établis dans le fichier de licence **[LICENSE](LICENSE)** (**GNU Affero General Public License v3 - GNU AGPLv3**).  
> Toute version modifiée ou œuvre dérivée doit obligatoirement préserver la mention de l'auteur original et rendre son code source accessible sous les mêmes termes de la GNU AGPLv3.

---

### 📌 Fiche Technique du Projet

- **Projet :** HUGINN SENTINEL<br>
- **Version :** `2026.11.9.0` (Date : 9 novembre 2026)<br>
- **Auteur et Titulaire des Droits :** Fernando Nillsson Cidade<br>
- **Langage & Écosystème :** Rust (Édition 2021) | Cargo<br>
- **Framework Graphique :** `egui 0.37` / `eframe 0.37` (Mode immédiat accéléré GPU via DirectX/OpenGL)<br>
- **Base de Données Relationnelle :** SQLite 3 en mode Write-Ahead Logging (WAL) via `rusqlite 0.41` (embarqué)<br>
- **Kernel Windows et E/S :** `notify 8.2` (`ReadDirectoryChangesW`), `windows-sys 0.61`, `crossbeam-channel 0.5`<br>
- **Cryptographie et Intégrité :** SHA-256, SHA-1, MD5, WinTrust Authenticode (`WinVerifyTrust`)<br>
- **Fichiers Binaires :** `huginnsentinel.exe` (GUI Desktop) et `huginnsentinel-cli.exe` (Console CLI Headless)<br>
- **Licence du Code Source :** [GNU Affero General Public License v3 (AGPLv3)](LICENSE)<br>

---

### 🔍 Description et Proposition de Valeur

> **"Sachez avec exactitude ce qui est arrivé à vos fichiers — localement, visuellement et avec un historique consultable."**

**Huginn Sentinel** est une plateforme d'audit d'intégrité et de surveillance continue des systèmes de fichiers Windows en temps réel, conçue pour les environnements professionnels et techniques exigeants.

Contrairement aux outils de surveillance éphémères, Huginn Sentinel unifie trois piliers essentiels :
1. **Surveillance Continue et Sémantique :** Capture en temps réel avec distinction sémantique claire entre opérations de `RENOMMÉ` et `DÉPLACÉ` (évitant les faux couples suppression/création).
2. **Historique Consultable et Renseignement sans IA :** Base relationnelle SQLite WAL intégrant 58 colonnes de métadonnées harmonisées, 17 graphiques statistiques natifs sur GPU et rapports d'audit déterministes.
3. **Sécurité Locale, Quarantaine et Preuves d'Audit :** Ligne de base cryptographique SHA-256, détection autonome de ransomwares et scripts suspects via 6 règles canoniques, quarantaine transactionnelle en format inerte `.quar` et export multiformat.

#### 🛡️ Confidentialité Souveraine (Local-First)
* **100% Local et Hors-Ligne :** Vos fichiers, métadonnées, historiques et rapports résident exclusivement sur votre machine (`%LOCALAPPDATA%\\Huginn Sentinel`).
* **Zéro Cloud, Zéro Télémétrie, Zéro Compte :** Aucune donnée opérationnelle ou personnelle n'est envoyée sur le réseau.
* **Déterminisme Pur sans IA :** L'évaluation des scores de risque (0–100) et la détection d'incidents reposent sur une logique algorithmique et mathématique stricte — sans LLM ni dépendance cloud externe.

---

### 🧭 Les 5 Modes de Fonctionnement

| Mode | Question Clé | Bénéfice Principal |
|---|---|---|
| **Mode Surveillance** | *Que se passe-t-il actuellement ?* | Flux d'événements en direct avec coloration sémantique et tampons non bloquants. |
| **Mode Historique** | *Qu'est-il arrivé à mon fichier ?* | Analyse chronologique inversée (*File Genealogy*), retraçant l'arbre complet des modifications et des hashes. |
| **Mode Analytics (Sans IA)** | *Quels sont les modèles d'activité ?* | 17 visualisations GPU (Cartes thermiques 7x24, diagrammes Sankey, histogrammes et Pareto). |
| **Mode Sécurité** | *Y a-t-il une modification suspecte ?* | Évaluation en direct via 6 règles canoniques pour détecter les attaques par ransomware et les scripts malveillants. |
| **Mode Preuve** | *Comment certifier les changements ?* | Exportation formelle de rapports structurés en `.xlsx` (OpenXML), `.csv`, `.txt`, `.json`, `.xml` et `.db`. |

---

### ✨ Caractéristiques Uniques de l'Architecture Rust

1. **Menus Persistants au Survol (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`) :** Les menus et sous-menus restent ouverts tant que la souris est présente, permettant des sélections multiples sans fermeture intempestive.
2. **Feuille Virtualisée à 4 Quadrants avec Sélection par Glisser et Figeage 2D (`src/ui/spreadsheet/*`) :** Sélection continue de cellules, colonnes et lignes par glisser avec surbrillance en temps réel, séparateurs d'en-tête préservés et verrouillage indépendant de lignes et colonnes.
3. **Harmonisation Déterministe des Métadonnées :** Garantit la cohérence stricte des 58 colonnes sur les 6 opérations auditées.
4. **Espace de Travail Modeless avec jusqu'à 16 Fenêtres :** Boîtes de dialogue flottantes style Windows 11 avec compteur dynamique au pied de page.
5. **Moteur Heuristique de Sécurité (`SEC-R01` à `SEC-R06`) :** Protection proactive contre les ransomwares, scripts suspects et dérives d'intégrité.
6. **Ligne de Base SHA-256 et Audit de Dérive :** Comparaison instantanée avec la ligne de base d'intégrité.
7. **Quarantaine Sécurisée avec Traçabilité :** Isolation atomique sous extension `.quar` et historique vérifié en SQLite WAL.
8. **Analyse des Exécutables PE et Authenticode :** Vérification des en-têtes PE, entropie de Shannon et signatures numériques.
9. **17 Graphiques Statistiques Natifs sur GPU et Adaptation aux Thèmes :** Visualisations analytiques dessinées avec `egui::Painter`, diagrammes Sankey avec filtres interactifs par cases à cocher et limites d'éléments avec défilement, séparation stricte entre Renommé et Déplacé et adaptation dynamique aux thèmes Clair et Sombre de Windows 11.
10. **Utilitaire Console Autonome (`huginnsentinel-cli.exe`) :** Outil headless pour l'automatisation en scripts PowerShell et serveurs.

---

### 🚀 Compilation et Exécution

```powershell
# Compiler les binaires de production (GUI et CLI)
cargo build --release --bin huginn --bin huginnsentinel-cli

# Exécuter les tests unitaires et d'intégration
cargo test
```

> **Configuration VS Code :** Les tâches de `.vscode/tasks.json` sont définies avec `"group": "none"` et dirigent toutes les compilations vers `builds/` (Debug et Release), garantissant **qu'aucun fichier n'est généré dans `target`** et éliminant toute recompilation automatique lors de la suppression du dossier `builds`.

---

### 📚 Documentation Complémentaire

- **[MANUAL.md](MANUAL.md) :** Guide d'utilisation et manuel de référence.
- **[OBSERVACAO.md](OBSERVACAO.md) :** Rapport de positionnement technique et d'architecture.
- **[LICENSE](LICENSE) :** Termes de la licence GNU AGPLv3.

</details>

---

<h2 id="itit">🇮🇹 Italiano (IT)</h2>

> **Nota:** Questo repository fa riferimento alla versione stabile **`2026.11.9.0`** del Progetto Huginn Sentinel (implementazione nativa in **Rust**). Sostieni il progetto e installa la versione ufficiale con firma digitale tramite: [Installa via Microsoft Store](https://apps.microsoft.com/detail/9NHQTGB26LMK)

<details>
<summary><b>Clicca per espandere / comprimere il README in italiano</b></summary><br>

<p align="center">
  <img src="repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154322.png" alt="Huginn Sentinel - Interfaccia Desktop in Italiano" width="850">
</p>

<h4 align="center">📸 Galleria delle Schermate Ufficiali (Microsoft Store / Interfaccia Desktop)</h4>

| 1. Interfaccia Principale di Controllo (58 Colonne Forensi) | 2. Menu File (Struttura Persistente al Passaggio del Mouse) |
|:---:|:---:|
| [![1. Interfaccia Principale](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154322.png)](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154322.png) | [![2. Menu File](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154332.png)](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154332.png) |
| **3. Menu Impostazioni (Sottomenu a Cascata)** | **4. Menu Opzioni (Lingua, Manuale e Informazioni)** |
| [![3. Menu Impostazioni](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154344.png)](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154344.png) | [![4. Menu Opzioni](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154354.png)](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154354.png) |
| **5. Intelligenza Visiva (Grafico a Torta delle Operazioni)** | **6. Statistiche GPU Avanzate (Boxplot delle Dimensioni)** |
| [![5. Intelligenza Visiva](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154416.png)](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154416.png) | [![6. Statistiche GPU Avanzate](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154430.png)](repo/LISTAGEM/it_IT/Captura%20de%20tela%202026-11-09%20154430.png) |

---

### ⚖️ Manifesto e Autorizzazione Legale del Titolare dei Diritti d'Autore

> **Dichiarazione Espressa di Fernando Nillsson Cidade:**  
> Io, **Fernando Nillsson Cidade**, in qualità di unico autore, creatore originale e titolare dei diritti d'autore del progetto **Huginn Sentinel**, **autorizzo espressamente l'uso, lo studio, la copia, la modifica, la compilazione e la distribuzione del codice sorgente di questo repository** rigorosamente secondo i termini e le condizioni stabiliti nel file di licenza **[LICENSE](LICENSE)** (**GNU Affero General Public License v3 - GNU AGPLv3**).  
> Qualsiasi versione modificata o opera derivata deve obbligatoriamente mantenere i crediti dell'autore originale e rendere disponibile il codice sorgente corrispondente sotto i medesimi termini della GNU AGPLv3.

---

### 📌 Specifiche Tecniche del Progetto

- **Progetto:** HUGINN SENTINEL<br>
- **Versione:** `2026.11.9.0` (Data: 9 novembre 2026)<br>
- **Autore e Titolare dei Diritti:** Fernando Nillsson Cidade<br>
- **Linguaggio ed Ecosistema:** Rust (Edizione 2021) | Cargo<br>
- **Interfaccia Grafica:** `egui 0.37` / `eframe 0.37` (Modalità immediata accelerata su GPU tramite DirectX/OpenGL)<br>
- **Database Relazionale:** SQLite 3 in modalità Write-Ahead Logging (WAL) tramite `rusqlite 0.41` (incorporato)<br>
- **Kernel di Windows e I/O:** `notify 8.2` (`ReadDirectoryChangesW`), `windows-sys 0.61`, `crossbeam-channel 0.5`<br>
- **Crittografia e Integrità:** SHA-256, SHA-1, MD5, WinTrust Authenticode (`WinVerifyTrust`)<br>
- **Binari Generati:** `huginnsentinel.exe` (Desktop GUI) e `huginnsentinel-cli.exe` (Console CLI Headless)<br>
- **Licenza del Codice Sorgente:** [GNU Affero General Public License v3 (AGPLv3)](LICENSE)<br>

---

### 🔍 Descrizione e Proposta di Valore

> **"Sappi esattamente cosa è accaduto ai tuoi file — localmente, visivamente e con cronologia ricercabile."**

**Huginn Sentinel** è una piattaforma desktop aziendale ad alte prestazioni progettata per fornire visibilità assoluta, tracciamento forense e auditing deterministico sul file system di Windows.

A differenza dei semplici monitor effimeri o dei software restrittivi convenzionali, Huginn Sentinel unisce tre pilastri essenziali:
1. **Monitoraggio Continuo e Semantico:** Riconoscimento in tempo reale con distinzione semantica reale tra operazioni di `RINOMINATO` e `SPOSTATO` (evitando coppie artificiali di cancellazione/creazione).
2. **Cronologia Ricercabile e Analisi senza IA:** Database relazionale SQLite WAL locale arricchito con 58 colonne di metadati armonizzati, 17 grafici statistici nativi su GPU e report deterministici.
3. **Sicurezza Locale, Quarantena e Prove di Audit:** Baseline crittografica SHA-256, rilevamento autonomo di ransomware e script sospetti tramite 6 regole canoniche, quarantena transazionale in formato inerte `.quar` ed esportazione multiformato.

#### 🛡️ Privacy Sovrana (Local-First)
* **100% Locale e Offline:** I tuoi file, metadati, cronologie e report risiedono esclusivamente sul tuo computer (`%LOCALAPPDATA%\\Huginn Sentinel`).
* **Zero Cloud, Zero Telemetria, Zero Account:** Nessuna informazione operativa o privata viene trasmessa sulla rete.
* **Intelligenza Deterministica senza IA:** Tutto il calcolo dei punteggi di rischio (0–100) e il rilevamento degli incidenti utilizzano logica algoritmica e matematica pura — senza modelli LLM né dipendenze cloud esterne.

---

### 🧭 Le 5 Modalità Operative

| Modalità | Domanda Centrale | Vantaggio Principale |
|---|---|---|
| **Modalità Monitoraggio** | *Cosa sta accadendo adesso?* | Flusso di eventi in tempo reale con colorazione semantica e buffer thread-safe non bloccanti. |
| **Modalità Cronologia** | *Cosa è successo al mio file?* | Consultazione cronologica inversa (*File Genealogy*), ricostruendo l'albero completo delle modifiche e degli hash. |
| **Modalità Analytics (Senza IA)** | *Quali sono i pattern di utilizzo?* | 17 grafici statistici GPU (Mappe termiche 7x24, diagrammi Sankey, istogrammi, Pareto e boxplot). |
| **Modalità Sicurezza** | *C'è stata qualche modifica sospetta?* | Valutazione continua tramite 6 regole canoniche contro ransomware, estensioni doppie e script nascosti. |
| **Modalità Prove** | *Come certificare le modifiche?* | Esportazione formale di report strutturati in `.xlsx` (OpenXML), `.csv`, `.txt`, `.json`, `.xml` e `.db`. |

---

### ✨ Innovazioni Esclusive dell'Architettura Rust

1. **Menu Persistenti al Passaggio del Mouse (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):** La struttura dei menu e dei sottomenu rimane aperta finché il cursore è presente, consentendo selezioni multiple rapide.
2. **Foglio Virtualizzato a 4 Quadranti con Selezione a Trascinamento e Blocco 2D (`src/ui/spreadsheet/*`):** Selezione continua di celle, colonne e righe tramite trascinamento con evidenziazione in tempo reale, divisori preservati e blocco indipendente di righe e colonne.
3. **Armonizzazione Deterministica dei Metadati:** Consistenza rigorosa delle 58 colonne nelle 6 operazioni monitorate.
4. **Workspace Modeless con fino a 16 Finestre Figlie:** Finestre di dialogo stile Windows 11 con contatore dinamico nel piè di pagina.
5. **Motore Euristico di Sicurezza (`SEC-R01` a `SEC-R06`):** Rilevamento tempestivo di ransomware e script non firmati.
6. **Baseline Crittografica SHA-256 e Audit di Deriva:** Confronto istantaneo per individuare violazioni di integrità.
7. **Quarantena con Custodia Tracciabile:** Isolamento in formato `.quar` con registrazione transazionale in SQLite WAL.
8. **Analisi degli Eseguibili PE e Authenticode:** Ispezione intestazioni PE, entropia di Shannon e firme digitali.
9. **17 Grafici Statistici Nativi su GPU e Adattamento ai Temi:** Visualizzazioni analitiche accelerate via hardware, diagrammi Sankey con filtri interattivi tramite caselle di controllo e limiti di elementi con barra di scorrimento, separazione rigorosa tra Rinominato e Spostato e adattamento dinamico ai temi Chiaro e Scuro di Windows 11.
10. **Utility Console Autonoma (`huginnsentinel-cli.exe`):** Eseguibile headless per automazioni in script PowerShell e server.

---

### 🚀 Compilazione ed Esecuzione

```powershell
# Compilazione binari di produzione (GUI e CLI)
cargo build --release --bin huginn --bin huginnsentinel-cli

# Esecuzione dei test automatizzati
cargo test
```

> **Configurazione VS Code:** I task in `.vscode/tasks.json` sono impostati con `"group": "none"` e indirizzano tutte le compilazioni in `builds/` (Debug e Release), garantendo che **nessun file sia generato in `target`** e prevenendo qualsiasi ricompilazione automatica alla cancellazione della cartella `builds`.

---

### 📚 Documentazione Aggiuntiva

- **[MANUAL.md](MANUAL.md):** Manuale utente completo e guida diagnostica.
- **[OBSERVACAO.md](OBSERVACAO.md):** Relazione tecnica sull'architettura.
- **[LICENSE](LICENSE):** Termini della licenza GNU AGPLv3.

</details>

---

<h2 id="dede">🇩🇪 Deutsch (DE)</h2>

> **Hinweis:** Dieses Repository bezieht sich auf die stabile Version **`2026.11.9.0`** des Projekts Huginn Sentinel (native Implementierung in **Rust**). Unterstützen Sie das Projekt und installieren Sie die offiziell digital signierte Version über: [Über Microsoft Store installieren](https://apps.microsoft.com/detail/9NHQTGB26LMK)

<details>
<summary><b>Klicken Sie hier, um das README auf Deutsch ein-/auszublenden</b></summary><br>

<p align="center">
  <img src="repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154450.png" alt="Huginn Sentinel - Desktop-Benutzeroberfläche auf Deutsch" width="850">
</p>

<h4 align="center">📸 Offizielle Screenshot-Galerie (Microsoft Store / Desktop-Benutzeroberfläche)</h4>

| 1. Haupt-Audit-Benutzeroberfläche (58 Forensische Spalten) | 2. Menü Datei (Hover-Persistente Menüstruktur) |
|:---:|:---:|
| [![1. Haupt-Audit-Benutzeroberfläche](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154450.png)](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154450.png) | [![2. Menü Datei](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154500.png)](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154500.png) |
| **3. Menü Einstellungen (Kaskadierende Hover-Untermenüs)** | **4. Menü Optionen (Sprachauswahl, Handbuch und Info)** |
| [![3. Menü Einstellungen](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154511.png)](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154511.png) | [![4. Menü Optionen](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154521.png)](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154521.png) |
| **5. Visuelle Intelligenz (Kreisdiagramm der Vorgänge)** | **6. Erweiterte GPU-Statistiken (Dateigrößen-Boxplot)** |
| [![5. Visuelle Intelligenz](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154533.png)](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154533.png) | [![6. Erweiterte GPU-Statistiken](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154545.png)](repo/LISTAGEM/de_DE/Captura%20de%20tela%202026-11-09%20154545.png) |

---

### ⚖️ Manifest und Gesetzliche Autorisierung des Urheberrechtsinhabers

> **Ausdrückliche Erklärung von Fernando Nillsson Cidade:**  
> Ich, **Fernando Nillsson Cidade**, als alleiniger Autor, ursprünglicher Schöpfer und Inhaber der Urheberrechte des Projekts **Huginn Sentinel**, **autorisiere hiermit ausdrücklich die Nutzung, Untersuchung, Vervielfältigung, Änderung, Kompilierung und Weitergabe des Quellcodes dieses Repositories** streng nach den Bedingungen der Lizenzdatei **[LICENSE](LICENSE)** (**GNU Affero General Public License v3 - GNU AGPLv3**).  
> Jede modifizierte Version oder abgeleitete Arbeit muss die ursprüngliche Urheberschaft beibehalten und den entsprechenden Quellcode unter denselben Bedingungen der GNU AGPLv3 öffentlich zugänglich machen.

---

### 📌 Technische Projektdaten

- **Projekt:** HUGINN SENTINEL<br>
- **Version:** `2026.11.9.0` (Datum: 9. November 2026)<br>
- **Autor und Urheberrechtsinhaber:** Fernando Nillsson Cidade<br>
- **Programmiersprache & Ökosystem:** Rust (Edition 2021) | Cargo<br>
- **Grafik-Framework:** `egui 0.37` / `eframe 0.37` (Immediate-Mode GPU-beschleunigt über DirectX/OpenGL)<br>
- **Relationale Datenbank:** SQLite 3 im Write-Ahead-Logging (WAL)-Modus über `rusqlite 0.41` (eingebettet)<br>
- **Windows-Kernel und E/A:** `notify 8.2` (`ReadDirectoryChangesW`), `windows-sys 0.61`, `crossbeam-channel 0.5`<br>
- **Kryptographie und Integrität:** SHA-256, SHA-1, MD5, WinTrust Authenticode (`WinVerifyTrust`)<br>
- **Kompilierte Binärdateien:** `huginnsentinel.exe` (Desktop-GUI) und `huginnsentinel-cli.exe` (Headless-Konsole)<br>
- **Quellcode-Lizenz:** [GNU Affero General Public License v3 (AGPLv3)](LICENSE)<br>

---

### 🔍 Beschreibung und Wertversprechen

> **"Wissen Sie genau, was mit Ihren Dateien geschehen ist — lokal, visuell und mit durchsuchbarem Verlauf."**

**Huginn Sentinel** ist eine hochperformante Desktop-Plattform für Unternehmen, die entwickelt wurde, um absolute Transparenz, forensische Nachverfolgung und deterministische Überwachung des Windows-Dateisystems zu gewährleisten.

Im Gegensatz zu flüchtigen Überwachungswerkzeugen vereint Huginn Sentinel drei Kernbereiche:
1. **Kontinuierliche & Semantische Dateiüberwachung:** Echtzeit-Erfassung mit semantischer Erkennung tatsächlicher `UMBENANNT`- und `VERSCHOBEN`-Vorgänge (ohne falsche Lösch-/Erstellungspaare).
2. **Durchsuchbarer Verlauf & Forensik ohne KI:** Lokale relationale SQLite WAL-Datenbank mit 58 harmonisierten Metadatenspalten, 17 nativen GPU-Statistikdiagrammen und deterministischen Berichten.
3. **Lokale Sicherheit, Quarantäne & Revisionssicherheit:** Kryptographische SHA-256-Baseline, autonome Ransomware- und Skripterkennung über 6 kanonische Regeln, transaktionale Quarantäne im inerten `.quar`-Format und strukturierter Multi-Format-Export.

#### 🛡️ Souveräner Datenschutz (Local-First)
* **100% Lokal & Offline:** Ihre Dateien, Metadaten, Historien und Berichte verbleiben ausschließlich auf Ihrem Computer (`%LOCALAPPDATA%\\Huginn Sentinel`).
* **Keine Cloud, Keine Telemetrie, Keine Benutzerkonten:** Es werden keinerlei operative oder persönliche Daten übertragen.
* **Deterministische Intelligenz ohne KI:** Die Risikobewertung (0–100) und Vorfallserkennung basieren rein auf mathematischer und algorithmischer Logik — ohne LLMs oder externe Cloud-Dienste.

---

### 🧭 Die 5 Betriebsmodi

| Modus | Kernfrage | Hauptvorteil |
|---|---|---|
| **Überwachungsmodus** | *Was passiert jetzt gerade?* | Echtzeit-Ereignisstrom mit semantischer Farbkodierung und blockierungsfreien Puffern. |
| **Verlaufsmodus** | *Was ist mit meiner Datei passiert?* | Umfassende chronologische Rückschau (*File Genealogy*) mit lückenlosem Änderungsverlauf und Hashes. |
| **Analytics-Modus (Ohne KI)** | *Welche Nutzungsmuster liegen vor?* | 17 GPU-Diagramme (7x24 Heatmaps, Sankey-Diagramme, Histogramme, Pareto und Boxplots). |
| **Sicherheitsmodus** | *Gab es verdächtige Änderungen?* | Kontinuierliche Prüfung anhand von 6 kanonischen Regeln gegen Ransomware und schädliche Skripte. |
| **Beweismodus** | *Wie lassen sich Änderungen belegen?* | Formelle Berichterstellung in `.xlsx` (OpenXML), `.csv`, `.txt`, `.json`, `.xml` und `.db`. |

---

### ✨ Besondere Merkmale der Rust-Architektur

1. **Hover-Persistente Menüs (`src/ui/widgets.rs`, `src/ui/menu_bar.rs`):** Menüs bleiben geöffnet, solange sich die Maus darüber befindet, was mehrfache Auswahlen ohne erneutes Öffnen ermöglicht.
2. **4-Quadranten-Tabelle mit Ziehauswahl und 2D-Fixierung (`src/ui/spreadsheet/*`):** Kontinuierliche Markierung von Zellen, Spalten und Zeilen durch Klicken und Ziehen mit visueller Echtzeit-Hervorhebung, intakten Trennlinien und beliebiger Zeilen- und Spaltenfixierung.
3. **Deterministische Metadaten-Harmonisierung:** Einheitliche Konsistenz aller 58 Spalten für alle 6 überwachten Dateioperationen.
4. **Modelloser Arbeitsbereich mit bis zu 16 Fenstern:** Unabhängige Fenster im Windows-11-Design mit dynamischem Fensterzähler in der Statusleiste.
5. **Heuristische Sicherheits-Engine (`SEC-R01` bis `SEC-R06`):** Frühzeitige Erkennung von Ransomware-Mustern, getarnten Erweiterungen und Integritätsabweichungen.
6. **Kryptographische SHA-256-Baseline:** Sofortiger Abgleich zur Erkennung unerlaubter Dateiänderungen.
7. **Revisionssichere Quarantäne:** Sichere Isolierung als `.quar` mit lückenlosem Nachweis in SQLite WAL.
8. **PE-Ausführungsanalyse & Authenticode:** Prüfung von PE-Headern, Shannon-Entropie und digitalen Signaturen.
9. **17 Native GPU-Diagramme und Designanpassung:** Leistungsstarke statistische Darstellungen, Sankey-Diagramme mit interaktiven Kontrollkästchen-Filtern und Elementbegrenzungen mit Bildlaufleiste, strikte Trennung von Umbenannt/Verschoben und dynamische Windows 11-Designanpassung.
10. **Autonomes Headless-Konsolenwerkzeug (`huginnsentinel-cli.exe`):** Ausführbare Datei für Automatisierung, PowerShell-Skripte und Server.

---

### 🚀 Kompilierung und Ausführung

```powershell
# Produktionsoptimierte Binärdateien erstellen (GUI und CLI)
cargo build --release --bin huginn --bin huginnsentinel-cli

# Automatisierte Tests ausführen
cargo test
```

> **VS Code-Hinweis:** Aufgaben in `.vscode/tasks.json` sind mit `"group": "none"` konfiguriert und leiten alle Builds nach `builds/` (Debug und Release) weiter, sodass **keine Dateien in `target` erzeugt werden** und **keine automatische Neukompilierung ausgelöst wird**, wenn der Ordner `builds` gelöscht wird.

---

### 📚 Weitere Dokumentation

- **[MANUAL.md](MANUAL.md):** Umfassendes Benutzerhandbuch und Diagnosehilfe.
- **[OBSERVACAO.md](OBSERVACAO.md):** Technischer Bericht zur Architektur.
- **[LICENSE](LICENSE):** Bedingungen der GNU AGPLv3-Lizenz.

</details>


---

## 🤝 Contributing & Community / Contribuição e Comunidade

> [!NOTE]
> **Sobre o Desenvolvimento e Contribuições Externas / Project Maintenance & Contributions:**  
> 🇧🇷 *Este projeto é desenvolvido primariamente como uma iniciativa pessoal e educacional pelo autor, que é iniciante e está em contínuo aprendizado em programação. Sugestões, issues e PRs são bem-vindos, mas a análise técnica e incorporação de código ocorrem voluntariamente no ritmo do autor. Caso não haja resposta imediata ou merge, não reaja com alarde; a licença GNU AGPLv3 garante total liberdade para você evoluir o projeto em seu próprio fork.*  
> 🇺🇸 *This project is developed primarily as a personal and educational initiative by the author, who is a beginner and in continuous software engineering learning. Suggestions, issues, and PRs are welcome, but technical reviews and upstream merges occur voluntarily at the author's own pace. If there is no immediate response or merge, please do not be alarmed; the GNU AGPLv3 license guarantees complete freedom to evolve the project in your own fork.*

Contributions are welcome! / Contribuições são muito bem-vindas!  
Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md). / Por favor, consulte nosso [Guia de Contribuição](CONTRIBUTING.md) e [Código de Conduta](CODE_OF_CONDUCT.md).

- 💡 Have an idea or question? Open a topic in [GitHub Discussions](../../discussions)! / Tem uma ideia ou dúvida? Abra um tópico no [GitHub Discussions](../../discussions)!
- 🐛 Found a bug? Use our [Bug Report Template](../../issues/new?template=bug_report.md). / Encontrou um problema? Utilize nosso [Modelo de Relato de Bug](../../issues/new?template=bug_report.md).
- 🛡️ Security vulnerability? Report privately via [GitHub Security Advisories](../../security/advisories/new). / Vulnerabilidade de segurança? Reporte com privacidade via [GitHub Security Advisories](../../security/advisories/new).