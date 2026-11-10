# Mapeamento de Conformidade / Compliance Mapping — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o guia de conformidade em português</summary><br>

# Huginn Sentinel — Mapeamento de Conformidade Regulatória

**Versão:** `2026.11.9.0`  
**Data:** `9 de novembro de 2026`  
**Frameworks:** ISO/IEC 27001:2022 | NIST SP 800-53 Rev. 5 | CIS Controls v8 | LGPD / GDPR | SOX  

---

## 1. Visão Geral de Conformidade

Normas internacionais de governança da informação, legislações de proteção de dados e frameworks de cibersegurança exigem das organizações o **Monitoramento Contínuo de Integridade de Arquivos (FIM - File Integrity Monitoring)**, a manutenção de trilhas de auditoria imutáveis e o rastreamento rigoroso de qualquer alteração, exclusão ou movimentação não autorizada de dados sensíveis.

O **Huginn Sentinel** cumpre esse papel operacional como uma plataforma soberana **100% Local-First**, assegurando que os dados periciados e os registros de auditoria jamais deixem os limites da estação de trabalho ou do servidor local.

---

## 2. Mapeamento dos Principais Frameworks

### 2.1. ISO/IEC 27001:2022
- **Controle A.8.15 — Registro de Logs (Logging):**
  - Exige a criação e proteção de registros de eventos de usuários, exceções e eventos de segurança.
  - *Atendimento pelo Huginn Sentinel:* Registra o ciclo de vida integral dos arquivos (criação, modificação, renomeação, movimentação e exclusão) com histórico cronológico indelével e **58 colunas de metadados enriquecidos e harmonizados** (`harmonizacao.rs`).
- **Controle A.8.16 — Atividades de Monitoramento:**
  - Determina o monitoramento de sistemas e aplicações contra comportamentos anômalos.
  - *Atendimento pelo Huginn Sentinel:* 17 gráficos estatísticos nativos em GPU (Mapas de Calor 7x24, Horários de Pico, Histogramas e Diagramas Sankey) permitem diagnosticar desvios de volume e acessos anormais fora do horário comercial.
- **Controle A.8.28 — Proteção de Código-Fonte:**
  - Exige a rastreabilidade estrita de diretórios de código e binários em ambientes de desenvolvimento e produção.

### 2.2. NIST SP 800-53 (Revisão 5)
- **SI-7 — Integridade de Software, Firmware e Informação (Software Integrity):**
  - "A organização emprega ferramentas de monitoramento de integridade para detectar alterações não autorizadas em arquivos críticos de sistema e dados organizacionais."
  - *Atendimento pelo Huginn Sentinel:* O motor de linha de base criptográfica calcula hashes SHA-256 de diretórios inteiros, sinalizando imediatamente qualquer adulteração, substituição de DLLs ou ataque de ransomware via regra `SEC-R06`.
- **AU-2 & AU-3 — Conteúdo dos Registros de Auditoria:**
  - Registra a genealogia das operações (caminho anterior vs. caminho atual), tipos MIME reais inspecionados por Magic Numbers binários e carimbos de data/hora (criação, modificação e último acesso) para arquivos e pastas.
- **AU-9 — Proteção das Informações de Auditoria:**
  - A persistência local em SQLite3 com gravação assíncrona WAL em subdiretórios dedicados e isolados (`events/huginn_events.db` e `security/huginn_security.db`) garante que os logs de auditoria fiquem estruturados e protegidos contra interceptação de rede ou falhas em nuvem.

### 2.3. CIS Controls v8
- **Controle 8 — Gerenciamento de Logs de Auditoria:**
  - Coleta, análise e retenção de logs detalhados de atividades em estações e servidores.
- **Controle 10 — Recursos de Recuperação de Dados:**
  - O subsistema de quarentena permite isolar arquivos suspeitos com extensão inerte `.quar` e restaurá-los mediante conferência estrita de hash SHA-256 gravado na tabela `quarentena` do SQLite WAL.

### 2.4. LGPD (Lei nº 13.709/2018) & GDPR (Regulamento UE 2016/679)
- **LGPD Art. 46 / GDPR Artigo 32 — Segurança do Tratamento:**
  - Exige medidas técnicas capazes de proteger dados pessoais contra destruição acidental ou ilícita, perda, alteração ou vazamento.
  - *Atendimento pelo Huginn Sentinel:* Detecta movimentações suspeitas em lote de pastas corporativas contendo dados pessoais (RH, Financeiro, Bases de Clientes), prevenindo e evidenciando incidentes de vazamento interno (*insider threats*).
  - *Privacidade Soberana:* Por operar em modelo 100% Local-First e sem IA em nuvem, a utilização do Huginn Sentinel nunca gera transferência internacional de dados ou exposição a terceiros.

---

## 3. Entregáveis Técnicos para Perícia e Auditoria

O Huginn Sentinel gera relatórios técnicos e pacotes probatórios estruturados:
1. **Planilhas CSV Padronizadas (UTF-8 BOM):** Prontas para abertura direta no Microsoft Excel, LibreOffice Calc e Power BI sem corrupção de caracteres ou acentuação.
2. **Formato JSON / XML Estruturado:** Ideal para ingestão e correlação em plataformas SIEM corporativas (Splunk, Microsoft Sentinel, Elastic, Graylog).
3. **Relatórios em HTML e TXT:** Documentação visual formatada e tabulações legíveis para auditorias executivas imediatas.
4. **Base Relacional SQLite (.db):** Banco de dados local em modo WAL que serve como evidência forense digital custodiada e imutável do período auditado.
5. **Pacotes Selados de Evidência em ZIP:** Arquivo compactado contendo relatório, metadados periciais, manifesto JSON e hashes SHA-256 de custódia.

---

## 4. Isenção de Responsabilidade Regulatória e Legal

> [!NOTE]
> **Aviso Legal:** O Huginn Sentinel é um utilitário técnico de software para monitoramento de integridade de arquivos e apoio à auditoria forense local. A disponibilização ou uso do software não constitui consultoria jurídica ou auditoria contábil/financeira, não confere por si só certificação formal de conformidade perante normas (como ISO/IEC 27001 ou SOC 2) e não substitui os processos formais, controles de acesso e obrigações corporativas de governança das organizações perante órgãos reguladores.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the compliance guide in English</summary><br>

# Huginn Sentinel — Regulatory Compliance Mapping

**Version:** `2026.11.9.0`  
**Date:** `November 9, 2026`  
**Frameworks:** ISO/IEC 27001:2022 | NIST SP 800-53 Rev. 5 | CIS Controls v8 | LGPD / GDPR | SOX  

---

## 1. Compliance Overview

International data governance standards, data protection regulations, and cybersecurity frameworks mandate continuous **File Integrity Monitoring (FIM)**, immutable audit logging, and strict accounting of unauthorized modifications, deletions, or data movements.

**Huginn Sentinel** fulfills these requirements as a **100% Local-First** sovereign platform, ensuring audit records and forensic evidence never leave the local endpoint or server.

---

## 2. Key Framework Mappings

### 2.1. ISO/IEC 27001:2022
- **Control A.8.15 — Logging:**
  - Mandates recording user activities, exceptions, and security events.
  - *Huginn Sentinel Implementation:* Captures the full file lifecycle (creations, modifications, renames, moves, deletions) with an indelible chronological history and **58 harmonized metadata columns** (`harmonizacao.rs`).
- **Control A.8.16 — Monitoring Activities:**
  - Requires monitoring systems and networks for anomalous behavior.
  - *Huginn Sentinel Implementation:* 17 native GPU analytics charts (7x24 heatmaps, peak activity hours, histograms, Sankey flows) identify off-hours spikes or mass alteration patterns.
- **Control A.8.28 — Protection of Source Code:**
  - Strict traceability of source repositories and binaries in dev/production environments.

### 2.2. NIST SP 800-53 (Revision 5)
- **SI-7 — Software, Firmware, and Information Integrity:**
  - "The organization employs integrity monitoring tools to detect unauthorized changes to software and information."
  - *Huginn Sentinel Implementation:* Cryptographic SHA-256 baseline snapshots detect unauthorized binary replacements, DLL injection, and ransomware tampering via rule `SEC-R06`.
- **AU-2 & AU-3 — Audit Record Content:**
  - Retains operational genealogy (previous vs. current path), magic-number MIME classifications, and high-precision timestamps (created, modified, accessed).
- **AU-9 — Protection of Audit Information:**
  - Local SQLite3 persistence with Write-Ahead Logging (WAL) in dedicated isolated subdirectories (`events/huginn_events.db` and `security/huginn_security.db`) ensures event streams remain structured, tamper-resistant, and immune to network eavesdropping.

### 2.3. CIS Controls v8
- **Control 8 — Audit Log Management:**
  - Collection, review, and retention of detailed event logs across endpoints and servers.
- **Control 10 — Data Recovery Capabilities:**
  - Quarantine sandbox isolates suspicious assets under `.quar` extensions, permitting verified rollback only upon SHA-256 confirmation against SQLite custody records.

### 2.4. LGPD (Law 13,709/2018) & GDPR (EU Regulation 2016/679)
- **LGPD Art. 46 / GDPR Article 32 — Security of Processing:**
  - Enforces technical measures to protect personal data from accidental or unlawful destruction, loss, alteration, or disclosure.
  - *Huginn Sentinel Implementation:* Detects abnormal bulk operations across directories holding sensitive personal information (HR, payroll, customer databases), evidencing insider threat incidents.
  - *Sovereign Privacy:* Because operations are 100% Local-First and free from cloud AI, using Huginn Sentinel never triggers cross-border data transfers or third-party disclosures.

---

## 3. Forensic & Audit Deliverables

Huginn Sentinel produces standardized compliance artifacts:
1. **Standardized CSV Spreadsheets (UTF-8 BOM):** Ready for immediate import into Microsoft Excel, LibreOffice Calc, and Power BI without encoding issues.
2. **Structured JSON / XML Schemas:** Directly ingestible into corporate SIEM pipelines (Splunk, Microsoft Sentinel, Elastic, Graylog).
3. **HTML & Plain Text Reports:** Clean, formatted summaries for executive audits.
4. **Relational SQLite Database (.db):** Self-contained local WAL database serving as tamper-resistant forensic digital evidence.
5. **Sealed ZIP Evidence Packages:** Cryptographically auditable ZIP containers combining event logs, forensic metadata, JSON manifest, and SHA-256 custody checksums.

---

## 4. Regulatory & Legal Disclaimer

> [!NOTE]
> **Legal Notice:** Huginn Sentinel is a technical desktop software utility designed for local file integrity monitoring and forensic audit assistance. The deployment or use of this software does not constitute legal or formal compliance auditing advice, does not by itself confer third-party certification (such as ISO/IEC 27001 or SOC 2), and does not replace an organization's formal security policies, identity controls, or statutory data governance obligations.

</details>
