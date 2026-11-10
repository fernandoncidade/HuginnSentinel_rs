# Referência da Linha de Comando / CLI Reference — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher a referência do CLI em português</summary><br>

# Huginn Sentinel — Referência da Linha de Comando (CLI) & Console

**Versão:** `2026.11.9.0`  
**Data:** `9 de novembro de 2026`  
**Executáveis:** `huginnsentinel.exe` (Interface gráfica desktop) | `huginnsentinel-cli.exe` (Utilitário de console headless)  
**Plataforma Alvo:** Windows 10 / Windows 11 / Windows Server 2022+ (x64)  
**Diretório dos Binários:** `builds\huginnsentinel\release\`  

---

## 1. Visão Geral

O **Huginn Sentinel** oferece dois modelos complementares de execução:
1. **Aplicação Gráfica Desktop (`huginnsentinel.exe`)**: Interface imediata rica em Rust e egui 0.37 com planilha virtualizada em 4 quadrantes, fixação 2D de linhas e colunas, menus persistentes por hover, 17 gráficos GPU e 16 janelas modeless estilo Windows 11. Suporta inicialização direta apontando para uma pasta via argumento posicional.
2. **Utilitário de Console Headless (`huginnsentinel-cli.exe`)**: Executável de console ultraleve compilado em Rust nativo para automação em scripts PowerShell, tarefas agendadas do Windows (Task Scheduler) e esteiras de CI/CD em servidores sem ambiente gráfico ativo.

Ambos os executáveis compartilham a mesma base algorítmica determinística, motor canônico de segurança (`SEC-R01` a `SEC-R06`), cálculo de hashes criptográficos SHA-256 e os bancos de dados relacionais SQLite WAL em `%LOCALAPPDATA%\HuginnSentinel\events\huginn_events.db` e `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`.

---

## 2. Aplicação Gráfica Desktop (`huginnsentinel.exe`)

Permite abrir a interface apontando diretamente para um diretório alvo para início imediato:

```cmd
huginnsentinel.exe [caminho_diretório]
```

### 2.1. Exemplos de Inicialização
- Iniciar a aplicação na pasta raiz de auditoria:
  ```powershell
  .\builds\huginnsentinel\release\huginnsentinel.exe "D:\DadosCriticos"
  ```
- Iniciar no diretório corrente:
  ```powershell
  .\builds\huginnsentinel\release\huginnsentinel.exe .
  ```

---

## 3. Utilitário Standalone de Console (`huginnsentinel-cli.exe`)

O utilitário `huginnsentinel-cli.exe` executa no terminal sem dependências externas, fornecendo saídas estruturadas em JSON ou texto formatado com códigos de saída (*exit codes*) para automação robusta.

### 3.1. Sintaxe Geral

```cmd
huginnsentinel-cli.exe <subcomando> [argumentos...] [opções]
```

### 3.2. Subcomandos Disponíveis

| Subcomando | Descrição |
|---|---|
| `scan <caminho>` | Realiza varredura recursiva de diretório, catalogando arquivos, atributos físicos, magic numbers e hashes SHA-256. |
| `metadata <arquivo>` | Extrai metadados forenses aprofundados de um arquivo específico (atributos, hashes, dados de execução PE e Authenticode). |
| `baseline <caminho>` | Compara o diretório alvo contra a linha de base SHA-256 gravada no SQLite WAL para detectar desvios (*drift*). Com a flag `--update`, atualiza ou grava o novo snapshot. |
| `quarantine <ação>` | Gerencia o cofre criptográfico de quarentena. Ações: `list` (listar itens isolados) ou `restore <id> [destino]` (restauração validada por hash SHA-256). |
| `monitor <caminho>` | Inicia monitoramento contínuo em tempo real via Windows `ReadDirectoryChangesW` exibindo eventos e alertas. A flag `--duration <segundos>` define um limite temporal opcional. |
| `--version` / `-v` | Exibe a versão oficial do utilitário CLI. |
| `--help` / `-h` | Exibe o manual integrado de sintaxe. |

---

## 4. Exemplos Práticos de Automação

### 4.1. Varredura Recursiva em PowerShell
```powershell
# Executa varredura e armazena resultado em formato JSON
.\builds\huginnsentinel\release\huginnsentinel-cli.exe scan "C:\MeusProjetos" | Out-File -Encoding utf8 scan_resultado.json
```

### 4.2. Inspeção de Binários e Assinatura Authenticode
```powershell
.\builds\huginnsentinel\release\huginnsentinel-cli.exe metadata "C:\Windows\System32\notepad.exe"
```

### 4.3. Monitoramento em Tempo Real com Duração Limitada
```powershell
# Monitora a pasta por 60 segundos e encerra automaticamente
.\builds\huginnsentinel\release\huginnsentinel-cli.exe monitor "C:\PastasSensiveis" --duration 60
```

### 4.4. Governança de Baseline e Quarentena
```powershell
# Atualizar baseline de integridade do diretório
.\builds\huginnsentinel\release\huginnsentinel-cli.exe baseline "C:\PastasSensiveis" --update

# Listar arquivos em quarentena
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine list

# Restaurar arquivo isolado de ID 1 para pasta segura
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine restore 1 "C:\Restaurados"
```

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the CLI reference in English</summary><br>

# Huginn Sentinel — Command-Line Interface (CLI) Reference

**Version:** `2026.11.9.0`  
**Date:** `November 9, 2026`  
**Binaries:** `huginnsentinel.exe` (Desktop GUI) | `huginnsentinel-cli.exe` (Headless Console CLI)  
**Target Platform:** Windows 10 / Windows 11 / Windows Server 2022+ (x64)  
**Binary Output Path:** `builds\huginnsentinel\release\`  

---

## 1. Overview

**Huginn Sentinel** provides two distinct, complementary runtime modalities:
1. **Desktop Graphical Application (`huginnsentinel.exe`)**: Immediate-mode GUI built with Rust and egui 0.37 featuring a 4-quadrant virtualized spreadsheet, arbitrary 2D row/column pinning, persistent hover-open menus, 17 native GPU charts, and 16 Windows 11 modeless dialogs. Accepts an optional target directory as a positional command-line argument.
2. **Headless Console CLI (`huginnsentinel-cli.exe`)**: Ultralight native console executable purpose-built for PowerShell scripts, Windows Task Scheduler, and CI/CD pipelines without graphical shell dependencies.

Both executables share the exact same deterministic security rule engine (`SEC-R01`–`SEC-R06`), SHA-256 cryptographic verification, and local SQLite WAL databases at `%LOCALAPPDATA%\HuginnSentinel\events\huginn_events.db` and `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`.

---

## 2. Desktop GUI Application (`huginnsentinel.exe`)

Launches the application directly with an optional target path:

```cmd
huginnsentinel.exe [directory_path]
```

### 2.1. Usage Examples
- Open GUI targeting a critical data folder:
  ```powershell
  .\builds\huginnsentinel\release\huginnsentinel.exe "D:\ProductionData"
  ```
- Launch GUI targeting current working folder:
  ```powershell
  .\builds\huginnsentinel\release\huginnsentinel.exe .
  ```

---

## 3. Standalone Console CLI (`huginnsentinel-cli.exe`)

The `huginnsentinel-cli.exe` tool executes natively inside the Windows console, returning structured JSON or human-readable formats with deterministic exit codes.

### 3.1. General Syntax

```cmd
huginnsentinel-cli.exe <subcommand> [arguments...] [options]
```

### 3.2. Available Subcommands

| Subcommand | Description |
|---|---|
| `scan <path>` | Recursively scans directory, extracting physical file attributes, magic numbers, and SHA-256 hashes. |
| `metadata <file>` | Extracts deep forensic metadata for a specific file (PE structure, Authenticode signature status, entropy). |
| `baseline <path>` | Compares current directory state against the SHA-256 baseline stored in SQLite WAL. Pass `--update` to record/update the baseline snapshot. |
| `quarantine <action>` | Manages the quarantine sandbox. Actions: `list` or `restore <id> [destination]`. |
| `monitor <path>` | Starts continuous real-time monitoring via Windows `ReadDirectoryChangesW`. Accepts optional `--duration <sec>`. |
| `--version` / `-v` | Displays the official compiled version string. |
| `--help` / `-h` | Displays the built-in manual and usage syntax. |

---

## 4. Automation Examples

### 4.1. Recursive Directory Scan in PowerShell
```powershell
.\builds\huginnsentinel\release\huginnsentinel-cli.exe scan "C:\Projects" | Out-File -Encoding utf8 scan_report.json
```

### 4.2. Binary Analysis and Authenticode Inspection
```powershell
.\builds\huginnsentinel\release\huginnsentinel-cli.exe metadata "C:\Windows\System32\cmd.exe"
```

### 4.3. Real-Time Monitoring with Duration Limit
```powershell
.\builds\huginnsentinel\release\huginnsentinel-cli.exe monitor "C:\TargetFolder" --duration 60
```

### 4.4. Integrity Baseline & Quarantine Management
```powershell
# Take baseline snapshot
.\builds\huginnsentinel\release\huginnsentinel-cli.exe baseline "C:\TargetFolder" --update

# List quarantined items
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine list

# Restore quarantined item 1 to a designated destination
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine restore 1 "C:\RestoredFolder"
```

</details>
