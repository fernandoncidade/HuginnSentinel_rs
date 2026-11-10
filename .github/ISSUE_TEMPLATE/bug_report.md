---
name: Relato de Bug / Bug Report
about: Crie um relatório para ajudar a diagnosticar e corrigir uma falha no Huginn Sentinel / Create a report to help diagnose and fix a bug in Huginn Sentinel
title: '[BUG] '
labels: ['bug', 'triage']
assignees: ''
---

<details>
<summary>Clique para expandir / recolher o formulário em Português</summary><br>

## Descrição do Problema
Uma descrição clara e concisa do comportamento inesperado ou falha encontrada no **Huginn Sentinel**.

## Informações do Ambiente
- **Versão do Sistema Operacional:** Windows 11 / Windows 10 / Windows Server (ex: Windows 11 24H2 x64)
- **Versão do Huginn Sentinel:** (ex: `v2026.10.29.0`)
- **Binário Afetado:** [ ] Desktop GUI (`huginnsentinel.exe`)  [ ] Utilitário CLI (`huginnsentinel-cli.exe`)  [ ] Ambos
- **Tipo de Distribuição:** [ ] Release Oficial GitHub  [ ] Pacote ZIP Portátil  [ ] Compilado do Código-Fonte (`cargo build --release`)
- **Toolchain Rust (se compilado do fonte):** Saída de `rustc --version` e `cargo --version` (ex: `stable-x86_64-pc-windows-msvc`)
- **Nível de Privilégio:** [ ] Usuário Padrão  [ ] Elevado (Executar como Administrador)

## Módulo / Subsistema Relacionado
- [ ] Monitoramento em Tempo Real (`ReadDirectoryChangesW` / `notify`)
- [ ] Escaneamento Recursivo de Diretórios (`Scanner`)
- [ ] Tabela de Auditoria / Planilha (`SpreadsheetGrid`, filtros, ordenação, painéis fixos)
- [ ] Extração das 58 Colunas de Metadados / PE / Authenticode (`WinTrust`)
- [ ] Motor de Segurança (`SEC-R01` a `SEC-R06`), Linha de Base SHA-256 ou Quarentena
- [ ] Persistência Local SQLite WAL (`%LOCALAPPDATA%\HuginnSentinel\monitoramento.db`)
- [ ] Gráficos Analíticos (17 gráficos `egui::Painter` / exportação PNG)
- [ ] Janelas Filhas Modeless Multi-Viewport / Internacionalização (i18n)

## Passos para Reproduzir o Erro
1. Selecione ou monitore o diretório '...'
2. Acesse o menu ou execute o comando '...'
3. Realize a ação '...'
4. Observe a falha ou comportamento inesperado

## Comportamento Esperado
Uma descrição clara do que você esperava que acontecesse.

## Logs, Mensagens de Erro ou Capturas de Tela
Se aplicável, anexe capturas de tela ou trechos relevantes do log local em `%LOCALAPPDATA%\HuginnSentinel Sentinel` (remova caminhos ou nomes confidenciais):
```text
Cole aqui mensagens de erro do console, backtrace (RUST_BACKTRACE=1) ou logs relevantes
```

## Contexto do Sistema de Arquivos Monitorado (se aplicável)
- Tipo de Sistema de Arquivos: [ ] NTFS Local  [ ] ReFS  [ ] Compartilhamento de Rede SMB  [ ] Unidade Externa / Removível
- Volume aproximado de arquivos/eventos na sessão: (ex: ~25.000 arquivos)

## Informações Adicionais
Adicione qualquer outro detalhe relevante sobre o problema aqui.

</details>

<details>
<summary>Click to expand / collapse the form in English</summary><br>

## Description
A clear and concise description of the unexpected behavior or bug encountered in **Huginn Sentinel**.

## Environment Information
- **OS Version:** Windows 11 / Windows 10 / Windows Server (e.g., Windows 11 24H2 x64)
- **Huginn Sentinel Version:** (e.g., `v2026.10.29.0`)
- **Affected Binary:** [ ] Desktop GUI (`huginnsentinel.exe`)  [ ] Console CLI (`huginnsentinel-cli.exe`)  [ ] Both
- **Distribution:** [ ] Official GitHub Release  [ ] Portable ZIP  [ ] Built from Source (`cargo build --release`)
- **Rust Toolchain (if built from source):** Output of `rustc --version` and `cargo --version` (e.g., `stable-x86_64-pc-windows-msvc`)
- **Privilege Level:** [ ] Standard User  [ ] Elevated (Run as Administrator)

## Affected Subsystem
- [ ] Real-Time File Watcher (`ReadDirectoryChangesW` / `notify`)
- [ ] Recursive Directory Scanner (`Scanner`)
- [ ] Event Audit Table / Spreadsheet (`SpreadsheetGrid`, filters, sorting, frozen rows/columns)
- [ ] 58-Column Metadata Extraction / PE Headers / Authenticode (`WinTrust`)
- [ ] Security Engine (`SEC-R01` to `SEC-R06`), SHA-256 Baseline, or Quarantine
- [ ] Local SQLite WAL Persistence (`%LOCALAPPDATA%\HuginnSentinel\monitoramento.db`)
- [ ] Analytics Charts (17 `egui::Painter` charts / PNG export)
- [ ] Multi-Viewport Modeless Dialogs / Runtime Localization (i18n)

## Steps to Reproduce
1. Select or monitor target directory '...'
2. Open menu or run CLI command '...'
3. Perform action '...'
4. See error or unexpected behavior

## Expected Behavior
A clear and concise description of what you expected to happen.

## Logs, Error Output, or Screenshots
If applicable, attach screenshots or relevant log excerpts from `%LOCALAPPDATA%\HuginnSentinel Sentinel` (please scrub any sensitive paths or usernames):
```text
Paste console output, Rust backtrace (RUST_BACKTRACE=1), or relevant logs here
```

## Monitored Filesystem Context (if applicable)
- Filesystem Type: [ ] Local NTFS  [ ] ReFS  [ ] SMB Network Share  [ ] External / Removable Drive
- Approximate file/event volume in session: (e.g., ~25,000 files)

## Additional Context
Add any other context about the problem here.

</details>
