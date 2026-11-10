# Notas de Lançamento / Release Notes — v2026.11.9.0

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher as notas de versão em português</summary><br>

# Huginn Sentinel — Release v2026.11.6.0

**Versão Oficial:** `2026.11.6.0`  
**Tag de Versão:** `v2026.11.9.0`  
**Data de Publicação:** 9 de novembro de 2026
**Autor:** Fernando Nillsson Cidade  
**Repositório Oficial:** [GitHub HuginnSentinel_rs](../)  

---

## 1. Artefatos de Instalação e Hashes Criptográficos

Valide a integridade do pacote baixado antes da execução utilizando o cmdlet nativo do PowerShell:

```powershell
Get-FileHash -Path ".\builds\huginn\release\huginn.exe" -Algorithm SHA256
Get-FileHash -Path ".\builds\huginn\release\huginn-cli.exe" -Algorithm SHA256
```

| Nome do Artefato | Formato | Arquitetura Alvo | Descrição |
|---|---|:---:|---|
| `huginn.exe` | Executável Nativo Windows (GUI) | x64 (MSVC) | Aplicação desktop em modo imediato acelerada na GPU com interface em egui 0.37. |
| `huginn-cli.exe` | Utilitário Headless de Console | x64 (MSVC) | Executável autônomo para automações PowerShell, agendamentos e CI/CD. |
| `Source code (zip / tar.gz)` | Arquivos de Código-Fonte | Qualquer | Código-fonte completo em Rust (Edição 2021) sob licença GNU AGPLv3. |

---

## 2. Destaques da Versão 2026.11.6.0

Esta versão consolida aprimoramentos fundamentais na governança legal, suporte oficial, transparência de distribuição e harmonização internacional:

- **Formalização do Modelo de Duplo Licenciamento (Dual-Licensing):**
  - Definição clara da coexistência entre a edição comunitária em código aberto (sob licença GNU AGPLv3) e a edição comercial oficial distribuída através da Microsoft Store.
  - Isenção explícita de obrigações copyleft para uso corporativo interno do binário comercial adquirido na loja.
- **Canal Oficial Centralizado de Atendimento e Suporte Técnico:**
  - Atualização unificada em todas as telas, janelas informativas e termos do endereço de suporte oficial: `linceu_lighthouse@outlook.com`.
  - Atendimento direto e ágil via e-mail em regime de melhor esforço (best-effort), sem necessidade de contato telefônico ou acesso remoto aos dispositivos.
- **Harmonização e Paridade Estrutural nos 6 Idiomas Suportados:**
  - Atualização completa de todos os textos informativos, contratos EULA, termos comerciais (CLC), políticas de privacidade e manuais de uso nos 6 idiomas: Português, Inglês, Espanhol, Francês, Italiano e Alemão.
  - Paridade exata linha por linha em todos os arquivos de termos e documentação oficial.
- **Clareza de Direitos Autorais e Créditos a Terceiros:**
  - Discriminação transparente de componentes e bibliotecas de terceiros nos avisos de copyright, garantindo plena segurança jurídica para uso pessoal e corporativo.
- **Privacidade Soberana Local-First e Conformidade com a LGPD:**
  - Reafirmação da operação 100% offline, local e isolada no computador do usuário, sem qualquer coleta de dados em nuvem ou telemetria invasiva.
- **Correção e Confiabilidade de Links na Documentação:**
  - Revisão e correção de todos os links de navegação interna nos guias de arquitetura, manuais do usuário e referências de conformidade.
- **Sistema Granular de Limpeza de Dados e Organização Local:**
  - Manutenção do sistema individual de purga de tabelas e bancos relacionais isolados em subdiretórios dedicados sob `%LOCALAPPDATA%\Huginn\`.

---

## 3. Requisitos de Sistema

- **Sistema Operacional:** Windows 11, Windows 10, Windows Server 2022 / 2019 (64-bit x64).
- **Processador:** Arquitetura x86-64 (Intel Core i3/i5/i7/i9 ou AMD Ryzen).
- **Memória RAM:** Mínimo de 256 MB (consumo típico em operação contínua < 120 MB).
- **Espaço em Disco:** ~80 MB livres para instalação e banco SQLite local.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the release notes in English</summary><br>

# Huginn Sentinel — Release v2026.11.6.0

**Official Version:** `2026.11.6.0`  
**Version Tag:** `v2026.11.9.0`  
**Publication Date:** November 9, 2026
**Author:** Fernando Nillsson Cidade  
**Official Repository:** [GitHub HuginnSentinel_rs](../)  

---

## 1. Installation Packages & Cryptographic Checksums

Validate package integrity prior to launch using native Windows PowerShell:

```powershell
Get-FileHash -Path ".\builds\huginn\release\huginn.exe" -Algorithm SHA256
Get-FileHash -Path ".\builds\huginn\release\huginn-cli.exe" -Algorithm SHA256
```

| Package Name | Format | Target Arch | Description |
|---|---|:---:|---|
| `huginn.exe` | Native Windows GUI Executable | x64 (MSVC) | GPU immediate-mode desktop application powered by egui 0.37. |
| `huginn-cli.exe` | Headless Console Executable | x64 (MSVC) | Autonomous CLI utility for PowerShell scripts, task scheduling, and CI/CD pipelines. |
| `Source code (zip / tar.gz)` | Source Code Bundle | Any | Full Rust source code (Edition 2021) licensed under GNU AGPLv3. |

---

## 2. Release Highlights (v2026.11.6.0)

This version consolidates essential improvements in legal governance, official support channels, distribution clarity, and international harmonization:

- **Dual-Licensing Framework Formalization:**
  - Clear establishment of the coexistence between the community open-source edition (licensed under GNU AGPLv3) and the official commercial edition distributed through the Microsoft Store.
  - Explicit copyleft exemption for internal corporate use of official commercial binaries.
- **Centralized Official Support Channel:**
  - Unified update of the official technical support contact across all application windows, dialogs, and legal terms: `linceu_lighthouse@outlook.com`.
  - Direct, reliable best-effort email support without requiring telephone calls or intrusive remote desktop sessions.
- **Full Harmonization & Exact Structural Parity across 6 Supported Languages:**
  - Complete updates for informative dialogs, EULA agreements, commercial terms (CLC), privacy policies, and user documentation across English, Portuguese, Spanish, French, Italian, and German.
  - Exact 1:1 line-by-line parity across all legal terms and documentation files.
- **Third-Party Attribution & Intellectual Property Clarity:**
  - Clear and explicit recognition of open-source third-party dependencies in copyright notices, ensuring full legal clarity for personal and enterprise deployment.
- **Local-First Sovereign Privacy & LGPD Compliance:**
  - Reaffirmation of 100% offline, local data processing on the user's workstation, with zero cloud dependency and zero telemetry collection.
- **Documentation Link Integrity & Verification:**
  - Comprehensive inspection and correction of internal navigation links across user manuals, architecture documentation, and compliance guides.
- **Granular Data Cleanup System & Local Data Isolation:**
  - Continuous support for granular purging across in-memory table rows and isolated relational databases under dedicated `%LOCALAPPDATA%\Huginn\` subdirectories.

---

## 3. System Requirements

- **Operating System:** Windows 11, Windows 10, Windows Server 2022 / 2019 (64-bit x64).
- **Processor:** x86-64 architecture (Intel Core i3/i5/i7/i9 or AMD Ryzen).
- **RAM:** Minimum 256 MB (typical consumption < 120 MB).
- **Disk Space:** ~80 MB free disk space for executable and local SQLite database.

</details>

