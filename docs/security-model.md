# Modelo de Segurança / Security Model & Threat Analysis — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o modelo de segurança em português</summary><br>

# Huginn Sentinel — Modelo de Segurança e Análise de Ameaças

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  
**Nível de Segurança:** 100% Local-First | Zero Telemetria | Auditoria Determinística de Integridade (Sem IA Externa)  
**Contexto Operacional:** Windows 10 / 11 / Windows Server 2022+  

---

## 1. Modelo de Ameaças e Garantias de Design

O **Huginn Sentinel** foi concebido para atender às mais rigorosas políticas de segurança corporativa, governança de TI e perícia forense digital, operando com total isolamento do ambiente externo.

### 1.1. Garantias Centrais de Segurança
1. **Privacidade Soberana e Zero Comunicação Externa:**
   - A aplicação não efetua conexões de rede de entrada ou saída (*inbound* ou *outbound*).
   - Não há código de telemetria, rastreamento analítico, *beacons* publicitários ou envio automatizado de relatórios de falhas.
   - Todo o processamento, persistência SQLite e geração de gráficos ocorrem exclusivamente na máquina local (`%LOCALAPPDATA%\HuginnSentinel Sentinel`).
2. **Determinismo sem IA Externa ou Modelos de Linguagem:**
   - O sistema não utiliza LLMs (Large Language Models), RAG (Retrieval-Augmented Generation) ou APIs de IA na nuvem.
   - Todas as decisões de classificação de eventos, cálculo de risco (0–100) e alertas derivam de lógica algorítmica e matemática determinística, eliminando riscos de injeção de prompt, vazamento de contexto ou alucinações.
3. **Quarentena Não-Destrutiva e Restauração Criptograficamente Segura:**
   - Ao enviar um arquivo suspeito para quarentena, seus bytes são isolados atomicamente na pasta `%LOCALAPPDATA%\HuginnSentinel\quarantine` com extensão inerte `.quar`.
   - Uma cadeia de custódia imutável (caminho original, timestamp, regra acionada e hash SHA-256) é registrada na tabela transacional `quarentena` do SQLite WAL em `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`.
   - Qualquer rotina de restauração exige recálculo estrito e conferência do hash SHA-256 contra o registro da custódia antes de gravar no disco, impossibilitando restaurações de dados adulterados.
4. **Inspeção Segura do Sistema de Arquivos:**
   - A observação de eventos utiliza notificações assíncronas nativas do Windows (`ReadDirectoryChangesW` via `notify`) desacopladas com canais `crossbeam-channel`.
   - Handles de arquivos para leitura de metadados são abertos exclusivamente em modo somente leitura (`GENERIC_READ`, `FILE_SHARE_READ | FILE_SHARE_WRITE`), sem locks exclusivos que interfiram no funcionamento do sistema operacional.

---

## 2. Fluxo de Captura, Correlação e Detecção de Ameaças

```text
           [Evento do Sistema de Arquivos no Windows]
                               │
                               ▼
        [API Assíncrona ReadDirectoryChangesW (Buffer Ring)]
                               │
                               ▼
          [Fila Thread-Safe de Normalização e Correlação]
                               │
                               ▼
            [Desambiguação Semântica: Move vs. Rename]
            (Elimina pares falsos de criação e exclusão)
                               │
                               ▼
     [Enriquecimento e Harmonização de 58 Colunas de Metadados]
       - Tipo MIME e Magic Numbers reais no cabeçalho
       - Timestamps UTC e locais (criação, modificação, acesso)
       - Hashes criptográficos SHA-256, SHA-1, MD5
       - Verificação de assinatura digital Authenticode
                               │
                               ▼
            [Motor Heurístico de Regras e Risco (0-100)]
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
     (Dentro do Padrão)                    (Risco Identificado)
            │                                     │
     [Log Histórico SQLite]               [Alerta em Tempo Real]
                                          [Isolamento / Quarentena]
```

---

## 3. Matriz de Detecção Heurística de Riscos

O Huginn Sentinel avalia cada arquivo e evento através de 6 regras canônicas determinísticas (`src/security/rules.rs`):

| Código | Nome da Regra | Condição de Acionamento | Severidade | Ação Recomendada |
|---|---|---|:---:|---|
| **`SEC-R01`** | Heurística de Ransomware | Rajadas de escrita/modificação com salto de entropia de Shannon e extensões conhecidas de ransomware (`.locked`, `.crypto`). | **Crítico** | Isolamento imediato em quarentena e contenção do processo. |
| **`SEC-R02`** | Extensão Dupla Dissimulada | Arquivo mascarado com múltiplas extensões terminando em executável ou script (ex: `fatura.pdf.exe`). | **Alto** | Quarentena preventiva e análise pericial. |
| **`SEC-R03`** | Executável Não Assinado em Temp/AppData | Binário PE sem assinatura digital Authenticode válida detectado em pastas temporárias do usuário. | **Alto** | Validação de proveniência e bloqueio de execução. |
| **`SEC-R04`** | Script de Automação de Alto Risco | Criação ou modificação de scripts (`.ps1`, `.bat`, `.cmd`, `.vbs`, `.js`) em diretórios críticos. | **Médio** | Auditoria de conteúdo e verificação de autorização. |
| **`SEC-R05`** | Executável Oculto no Sistema | Arquivo executável criado com atributo de arquivo Oculto (`Hidden`) ou de Sistema (`System`). | **Médio** | Inspeção forense de persistência. |
| **`SEC-R06`** | Deriva de Linha de Base (Baseline Drift) | Divergência de hash SHA-256 em relação ao snapshot criptográfico armazenado na base de integridade. | **Alto** | Avaliação de adulteração não autorizada. |

---

## 4. Inteligência PE e Verificação Authenticode

Para arquivos executáveis do Windows (`.exe`, `.dll`, `.sys`), o subsistema `src/security/executable_intel.rs` inspeciona:
- Cabeçalhos PE: Machine Type (x86, x64, ARM64), Subsystem (GUI vs. Console), características e seções.
- Assinatura Authenticode: Consulta via Win32 `WinTrust` (`WinVerifyTrust`) para atestar se o binário foi assinado digitalmente por autoridade certificadora válida ou se está adulterado.
- Entropia de Shannon: Medição matemática de aleatoriedade dos bytes (0.00 a 8.00 bits/byte), permitindo distinguir arquivos comuns de códigos ofuscados, compactados ou empacotados (*packers*).

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the security model in English</summary><br>

# Huginn Sentinel — Security Model & Threat Analysis

**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  
**Security Level:** 100% Local-First | Zero Telemetry | Deterministic Integrity Auditing (Zero External AI)  
**Operating Environment:** Windows 10 / 11 / Windows Server 2022+  

---

## 1. Threat Model & Architectural Guarantees

**Huginn Sentinel** is engineered to satisfy the strictest corporate security policies, IT governance mandates, and digital forensics standards, operating completely isolated from external networks.

### 1.1. Core Security Commitments
1. **Sovereign Privacy & Zero External Communication:**
   - Zero inbound or outbound network connections.
   - Zero telemetry, analytics tracking, advertising beacons, or automated crash reporting.
   - All processing, SQLite persistence, and graphics rendering take place locally at `%LOCALAPPDATA%\HuginnSentinel Sentinel`.
2. **Deterministic Governance Without External AI/LLMs:**
   - No large language models (LLMs), RAG pipelines, or cloud AI APIs.
   - All event classifications, quantitative risk scores (0–100), and alerts stem from pure algorithmic and deterministic logic, eliminating prompt injection risks, context leakage, and hallucinations.
3. **Non-Destructive Quarantine & Verified Restoration:**
   - Quarantining an asset moves it atomically to `%LOCALAPPDATA%\HuginnSentinel\quarantine` under an inert `.quar` extension.
   - An immutable custody record (source path, timestamp, triggered rule, and SHA-256 hash) is committed to the SQLite WAL `quarentena` table at `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`.
   - Rollback strictly mandates SHA-256 re-computation and verification against the custody record before writing back to disk.
4. **Safe File System Inspection:**
   - Monitored asynchronously via Windows `ReadDirectoryChangesW` (`notify`) decoupled with `crossbeam-channel`.
   - File metadata handles are opened strictly in read-only shared mode (`GENERIC_READ`, `FILE_SHARE_READ | FILE_SHARE_WRITE`), avoiding disruptive exclusive locks.

---

## 2. Threat Capture & Correlation Pipeline

```text
            [Windows File System Kernel Event]
                            │
                            ▼
      [Asynchronous ReadDirectoryChangesW (Ring Buffer)]
                            │
                            ▼
         [Thread-Safe Normalization & Correlation]
                            │
                            ▼
          [Semantic Disambiguation: Move vs. Rename]
                            │
                            ▼
     [58-Column Metadata Enrichment & Harmonization]
       - Magic number header inspection for real MIME
       - High-precision UTC/local timestamps
       - Cryptographic hashes: SHA-256, SHA-1, MD5
       - Authenticode signature validation via WinTrust
                            │
                            ▼
          [Heuristic Security Engine (Risk Score 0-100)]
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
      (Within Norm)                  (Threat Flagged)
            │                               │
     [SQLite WAL Log]             [Real-Time Alert]
                                  [Quarantine Sandbox]
```

---

## 3. Canonical Heuristic Rule Matrix

Huginn Sentinel evaluates each event against 6 deterministic canonical rules (`src/security/rules.rs`):

| Code | Rule Name | Trigger Condition | Severity | Recommended Action |
|---|---|---|:---:|---|
| **`SEC-R01`** | Ransomware Heuristics | High-frequency write bursts coupled with Shannon entropy spikes and known ransomware extensions (`.locked`, `.crypto`). | **Critical** | Immediate quarantine isolation and process containment. |
| **`SEC-R02`** | Disguised Double Extension | Masked executable extensions hiding malicious payloads (e.g., `invoice.pdf.exe`). | **High** | Preemptive quarantine and binary inspection. |
| **`SEC-R03`** | Unsigned Binary in Temp/AppData | PE executable lacking valid Authenticode signature found in user temporary directories. | **High** | Provenance verification and execution prevention. |
| **`SEC-R04`** | High-Risk Automation Script | Creation or modification of scripting assets (`.ps1`, `.bat`, `.cmd`, `.vbs`, `.js`) in protected paths. | **Medium** | Content review and change authorization audit. |
| **`SEC-R05`** | Hidden System Executable | Executable file created with Hidden or System attributes active. | **Medium** | Forensic persistence inspection. |
| **`SEC-R06`** | Baseline Compliance Drift | SHA-256 checksum discrepancy compared against the stored cryptographic baseline snapshot. | **High** | Investigate unauthorized file tampering. |

---

## 4. PE Structure & Authenticode Intelligence

For Windows binaries (`.exe`, `.dll`, `.sys`), `src/security/executable_intel.rs` inspects:
- PE Headers: Architecture (x86, x64, ARM64), Subsystem, compile timestamps, and sections.
- Authenticode Signatures: Validated via Win32 `WinTrust` (`WinVerifyTrust`) to verify certificate trustworthiness and detect tampering.
- Shannon Entropy: Mathematical randomness calculation (0.00 to 8.00 bits/byte), separating plain code from encrypted or packed payloads.

</details>
