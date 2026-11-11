# Política de Segurança / Security Policy — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher a política em português</summary><br>

# Política de Segurança — Huginn Sentinel

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  

## 1. Versões com Suporte Oficial

Atualizações e correções de segurança são fornecidas ativamente para as versões estáveis mais recentes do **Huginn Sentinel**:

| Versão | Suportada |
|:---:|:---:|
| `v2026.11.x` | :white_check_mark: |
| `< v2026.11` | :x: |

---

## 2. Reportando uma Vulnerabilidade

Levamos a segurança e a integridade do **Huginn Sentinel** muito a sério. Como uma ferramenta voltada para auditoria forense, monitoramento em tempo real do sistema de arquivos Windows e detecção de ransomware, a confiabilidade de nossas verificações é fundamental.

Se você identificou uma potencial vulnerabilidade de segurança (como falha na detecção de regras canônicas `SEC-R01` a `SEC-R06`, divergência no cálculo de hash criptográfico SHA-256, falha de isolamento em quarentena ou desvio no parser de assinaturas Authenticode):

1. **NÃO divulgue publicamente a vulnerabilidade através de issues ou discussões abertas.**
2. Submeta o reporte de forma privada através do recurso **GitHub Security Advisories** na aba [Security](../../security/advisories/new) do repositório ou contate o mantenedor diretamente.
3. Por favor, inclua em seu relato:
   - Descrição detalhada da vulnerabilidade encontrada;
   - Passo a passo para reprodução, comandos PowerShell ou amostras de eventos simulados;
   - Vetor de ataque e impacto potencial estimado sobre os laudos de auditoria;
   - Sugestões de correção ou mitigação em código Rust, caso possua.

### Prazos e Processo de Resposta
- O mantenedor responderá e confirmará o recebimento do seu reporte no melhor esforço e em tempo hábil, considerando a natureza voluntária e independente do projeto.
- Realizaremos uma avaliação técnica inicial e estimaremos o prazo de correção.
- Após a correção, um patch de segurança será publicado e os créditos devidamente atribuídos em coordenação com o pesquisador.

---

## 3. Compromissos Arquiteturais de Segurança

- **100% Autônomo, Local e Offline:** A aplicação jamais envia caminhos de arquivos, nomes de processos, hashes SHA-256 ou eventos de auditoria através da rede. Não há telemetria, conexões em nuvem ou dependência de IA/LLMs externos.
- **Segurança de Memória Nativa em Rust:** O núcleo do Huginn Sentinel é implementado em Rust (Edição 2021) sem blocos inseguros desnecessários, eliminando categorias inteiras de vulnerabilidades clássicas como estouro de buffer, referências nulas e condições de corrida de dados (*data races*).
- **Isolamento Seguro em Quarentena:** Operações de quarentena movem atomicamente os arquivos suspeitos para `%LOCALAPPDATA%\HuginnSentinel\quarantine` com extensão inerte `.quar` e registram a cadeia de custódia completa (caminho original, timestamp, regra acionada e hash SHA-256) na tabela transacional `quarentena` do SQLite WAL em `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`, condicionando qualquer restauração à revalidação estrita do hash SHA-256 original.
- **Persistência Segura SQLite em Modo WAL:** Transações atômicas com integridade transacional garantida contra corrupção em desligamentos inesperados.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the policy in English</summary><br>

# Security Policy — Huginn Sentinel

**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  

## 1. Supported Versions

Security patches and updates are actively maintained for the latest stable releases of **Huginn Sentinel**:

| Version | Supported |
|:---:|:---:|
| `v2026.11.x` | :white_check_mark: |
| `< v2026.11` | :x: |

---

## 2. Reporting a Vulnerability

We take the security and integrity of **Huginn Sentinel** very seriously. As a forensic auditing, real-time file system monitoring, and ransomware detection platform, the trustworthiness of our security engine is essential.

If you believe you have discovered a security vulnerability in Huginn Sentinel (such as a bypass in security rules `SEC-R01`–`SEC-R06`, SHA-256 baseline calculation flaw, quarantine sandbox escape, or Authenticode verification flaw):

1. **Do NOT report security vulnerabilities through public GitHub issues or discussions.**
2. Report the vulnerability privately via **GitHub Security Advisory** under the repository's [Security tab](../../security/advisories/new) or contact the maintainer directly.
3. Please include:
   - A detailed description of the vulnerability.
   - Exact steps to reproduce, sample events, or PowerShell test scripts.
   - The potential impact and attack vector.
   - Any suggested mitigations or Rust patches.

### Response Time & Process
- The maintainer will acknowledge and review your report on a reasonable best-effort basis in a timely manner.
- We will provide an initial assessment and estimated remediation timeline.
- Once fixed, a security release will be tagged and public disclosure coordinated with the reporter.

---

## 3. Core Architectural Security Commitments

- **100% Local-First & Offline:** The application never transmits audited files, process names, hashes, or events over the network. Zero cloud, zero telemetry, and zero LLM/AI dependencies.
- **Rust Memory Safety:** The core engine is built in Rust (Edition 2021), eliminating buffer overflows, dangling pointers, use-after-free, and concurrency race conditions at compile time.
- **Tamper-Evident Quarantine:** Quarantined items are atomically isolated inside `%LOCALAPPDATA%\HuginnSentinel\quarantine` with inert `.quar` extensions, backed by transactional custody records in the SQLite WAL `quarentena` table at `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db` and verified against SHA-256 checksums before rollback.
- **ACID SQLite Storage:** Database operations utilize SQLite WAL mode with batch commits, ensuring database consistency even during power loss.

</details>
