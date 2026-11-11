# Notas de Lançamento / Release Notes — v2026.11.11.0

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher as notas de versão em português</summary><br>

# Huginn Sentinel — Release v2026.11.11.0

**Versão Oficial:** `2026.11.11.0`  
**Tag de Versão:** `v2026.11.11.0`  
**Data de Publicação:** 11 de novembro de 2026  
**Autor:** Fernando Nillsson Cidade  
**Repositório Oficial:** [GitHub HuginnSentinel_rs](../)  

---

## 1. Artefatos de Instalação e Hashes Criptográficos

Valide a integridade do pacote baixado antes da execução utilizando o cmdlet nativo do PowerShell:

```powershell
Get-FileHash -Path ".\builds\huginnsentinel\release\huginnsentinel.exe" -Algorithm SHA256
Get-FileHash -Path ".\builds\huginnsentinel\release\huginnsentinel-cli.exe" -Algorithm SHA256
```

| Nome do Artefato | Formato | Arquitetura Alvo | Descrição |
|---|---|:---:|---|
| `huginnsentinel.exe` | Executável Nativo Windows (GUI) | x64 (MSVC, CRT estática) | Aplicação desktop em modo imediato acelerada na GPU com interface em egui 0.37, sem dependências externas de runtime. |
| `huginnsentinel-cli.exe` | Utilitário Headless de Console | x64 (MSVC, CRT estática) | Executável autônomo para automações PowerShell, agendamentos e CI/CD, sem dependências externas de runtime. |
| `Source code (zip / tar.gz)` | Arquivos de Código-Fonte | Qualquer | Código-fonte completo em Rust (Edição 2021) sob licença GNU AGPLv3. |

---

## 2. Destaques da Versão 2026.11.11.0

Esta versão consolida melhorias estruturais de portabilidade, conformidade com a Microsoft Store e higienização dos recursos visuais, mantendo toda a base de governança legal e suporte consolidada anteriormente:

- **Binários 100% Autônomos com Runtime C++ Estática (`+crt-static`):**
  - Linkagem estática da runtime MSVC em `.cargo/config.toml`, eliminando totalmente a dependência externa de `VCRUNTIME140.dll` e do pacote `Microsoft Visual C++ Redistributable` em `huginnsentinel.exe` e `huginnsentinel-cli.exe`.
  - Atendimento direto à exigência de certificação `10.2.4.1 Security - Software Dependencies` da Microsoft Store sem necessidade de instaladores redistribuíveis adicionais.
- **Adequação do Manifesto MSIX e Scripts Inno Setup:**
  - Remoção de dependências não utilizadas no manifesto MSIX (`repo/AppxManifest.xml`, eliminando `Microsoft.WindowsAppRuntime.1.4`) e limpeza de referências a redistribuíveis nos scripts `repo/Huginn*.iss`.
- **Limpeza e Consolidação de Ícones e Imagens (`src/assets/icons/`):**
  - Exclusão de 38 arquivos legados de ícones e imagens não utilizados pelo aplicativo (`19 .ico` + `19 .png`) e centralização de todos os 20 ícones e imagens oficiais da marca em `src/assets/icons/`.
  - Atualização da resolução de recursos em `src/build.rs`, `src/utils/icons.rs`, `src/utils/paths.rs`, `tests/locale_tests.rs`, `version/src/main.rs` e nos scripts `.iss`.
- **Manutenção da Governança Legal, Suporte e Privacidade Local-First:**
  - Continuidade do modelo de Duplo Licenciamento (GNU AGPLv3 e edição comercial na Microsoft Store), canal unificado de suporte (`linceu_lighthouse@outlook.com`), paridade nos 6 idiomas e operação 100% offline sem telemetria.

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

# Huginn Sentinel — Release v2026.11.11.0

**Official Version:** `2026.11.11.0`  
**Version Tag:** `v2026.11.11.0`  
**Publication Date:** November 11, 2026  
**Author:** Fernando Nillsson Cidade  
**Official Repository:** [GitHub HuginnSentinel_rs](../)  

---

## 1. Installation Packages & Cryptographic Checksums

Validate package integrity prior to launch using native Windows PowerShell:

```powershell
Get-FileHash -Path ".\builds\huginnsentinel\release\huginnsentinel.exe" -Algorithm SHA256
Get-FileHash -Path ".\builds\huginnsentinel\release\huginnsentinel-cli.exe" -Algorithm SHA256
```

| Package Name | Format | Target Arch | Description |
|---|---|:---:|---|
| `huginnsentinel.exe` | Native Windows GUI Executable | x64 (MSVC, static CRT) | GPU immediate-mode desktop application powered by egui 0.37, with zero external runtime dependencies. |
| `huginnsentinel-cli.exe` | Headless Console Executable | x64 (MSVC, static CRT) | Autonomous CLI utility for PowerShell scripts, task scheduling, and CI/CD pipelines, with zero external runtime dependencies. |
| `Source code (zip / tar.gz)` | Source Code Bundle | Any | Full Rust source code (Edition 2021) licensed under GNU AGPLv3. |

---

## 2. Release Highlights (v2026.11.11.0)

This version consolidates structural improvements in portability, Microsoft Store compliance, and visual asset hygiene while preserving the previously established legal and support governance:

- **100% Self-Contained Binaries with Static C++ Runtime (`+crt-static`):**
  - Static MSVC runtime linking configured in `.cargo/config.toml`, completely eliminating external dependencies on `VCRUNTIME140.dll` and `Microsoft Visual C++ Redistributable` for both `huginnsentinel.exe` and `huginnsentinel-cli.exe`.
  - Directly satisfies Microsoft Store Policy `10.2.4.1 Security - Software Dependencies` without requiring additional redistributable packages.
- **MSIX Manifest & Inno Setup Script Alignment:**
  - Removal of unused framework dependencies from the MSIX manifest (`repo/AppxManifest.xml`, removing `Microsoft.WindowsAppRuntime.1.4`) and cleanup of redistributable handling in `repo/Huginn*.iss`.
- **Visual Asset Cleanup & Consolidation (`src/assets/icons/`):**
  - Removal of 38 unused legacy icon and image files (`19 .ico` + `19 .png`) and consolidation of all 20 official brand icons and images into `src/assets/icons/`.
  - Updated asset resolution across `src/build.rs`, `src/utils/icons.rs`, `src/utils/paths.rs`, `tests/locale_tests.rs`, `version/src/main.rs`, and the `.iss` scripts.
- **Continued Legal Governance, Support & Local-First Privacy:**
  - Retains the Dual-Licensing model (GNU AGPLv3 and Microsoft Store commercial edition), unified support channel (`linceu_lighthouse@outlook.com`), 6-language parity, and 100% offline zero-telemetry operation.

---

## 3. System Requirements

- **Operating System:** Windows 11, Windows 10, Windows Server 2022 / 2019 (64-bit x64).
- **Processor:** x86-64 architecture (Intel Core i3/i5/i7/i9 or AMD Ryzen).
- **RAM:** Minimum 256 MB (typical consumption < 120 MB).
- **Disk Space:** ~80 MB free disk space for executable and local SQLite database.

</details>

