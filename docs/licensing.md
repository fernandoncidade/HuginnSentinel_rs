# Licenciamento e Esclarecimento Comercial / Licensing & Commercial Model — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o documento de licenciamento em português</summary><br>

# Huginn Sentinel — Licenciamento, Termos de Código Aberto e Esclarecimento Comercial

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  
**Licença do Código-Fonte:** GNU Affero General Public License v3.0 (`GNU AGPLv3`)  
**Autor e Detentor dos Direitos Autorais:** Fernando Nillsson Cidade  

---

## 1. Visão Geral do Modelo de Licenciamento

O **Huginn Sentinel** é uma plataforma corporativa e ferramenta técnica distribuída com total transparência sob o modelo de software livre copyleft forte através da **GNU Affero General Public License v3 (AGPLv3)**:

| Componente | Licença | Usos Permitidos |
|---|---|---|
| **Código-Fonte (Repositório Integral)** | [GNU AGPLv3](../LICENSE) | 100% Livre: Estudo, execução, modificação, auditoria, distribuição comercial e não comercial, com obrigação de compartilhamento do código-fonte modificado sob os mesmos termos. |
| **Binários Oficiais da Microsoft Store** | Licença Comercial / EULA | Uso comercial em dispositivos vinculados à conta Microsoft, com isenção de copyleft para uso corporativo interno e suporte via e-mail. |
| **Documentação em `src/assets/`** | Autoria do Projeto (Uso Livre para Desenvolvedores) | Documentações criadas para distribuição da aplicação; desenvolvedores possuem liberdade total para adaptar, alterar, excluir ou adicionar conteúdos em seus próprios forks e plataformas. |
| **Binários Compilados por Terceiros** | GNU AGPLv3 | Livres para compilar, empacotar e distribuir sob responsabilidade própria, desde que acompanhados do código-fonte correspondente sob a GNU AGPLv3. |
| **Bibliotecas Rust Upstream (egui, notify, rusqlite, windows-sys)** | MIT / Apache 2.0 / CC0-1.0 / Domínio Público | Atribuições de código aberto mantidas em estrita conformidade com as licenças permissivas originais. |

---

## 2. Direitos de Código Aberto (GNU AGPLv3)

Todo o código-fonte do Huginn Sentinel — abrangendo o motor de monitoramento assíncrono em Rust (`notify` / `ReadDirectoryChangesW`), o subsistema de 58 colunas de metadados enriquecidos e harmonizados, a persistência relacional local SQLite em modo WAL (`DbWriter`), a planilha virtualizada em 4 quadrantes com fixação 2D de linhas e colunas, os menus persistentes por hover, as 17 janelas modeless independentes (incluindo o Sistema de Limpeza de Dados Granular), o motor heurístico de regras canônicas (`SEC-R01` a `SEC-R06`), a central de 17 gráficos estatísticos em GPU, o utilitário autônomo CLI (`huginn-cli`) e o sistema de internacionalização em 6 idiomas — está licenciado sob a **GNU Affero General Public License v3 (AGPLv3)**.

---

## 3. Política de Duplo Licenciamento (Dual-Licensing) e Concessão Comercial

O Huginn Sentinel adota formalmente o modelo de **Duplo Licenciamento** (*Dual-Licensing*):
1. **Modalidade Comunitária (GNU AGPLv3):** Livre e gratuita para qualquer pessoa que deseje utilizar, inspecionar, auditar ou colaborar com o projeto, sujeitando-se às obrigações de reciprocidade e copyleft da AGPLv3;
2. **Modalidade Comercial (Microsoft Store):** Para empresas, organizações e profissionais que desejam usufruir da comodidade de instalação binária oficial no Windows sem as obrigações de divulgação de código-fonte decorrentes de integrações e usos proprietários internos. A aquisição oficial na Microsoft Store confere uma licença binária comercial independente, com atualizações automáticas gerenciadas pela loja e suporte por e-mail no melhor esforço.

> **Declaração Expressa de Fernando Nillsson Cidade:**  
> Eu, **Fernando Nillsson Cidade**, na qualidade de criador original e único detentor dos direitos autorais do projeto **Huginn Sentinel**, declaro e concedo expressamente a qualquer indivíduo, empresa ou organização todas as permissões consagradas na licença **GNU Affero General Public License v3 (GNU AGPLv3)**, incluindo, sem restrição:
> - O direito de usar, compilar, executar e testar o software para qualquer finalidade corporativa, pessoal, governamental ou acadêmica;
> - O direito de modificar, aprimorar, adicionar novas regras heurísticas, novos gráficos ou novos idiomas;
> - O direito de **distribuir comercialmente** ou fornecer serviços baseados no software, desde que quaisquer versões modificadas e seus respectivos códigos-fonte sejam mantidos públicos e acessíveis sob a mesma licença GNU AGPLv3.

---

## 4. Documentação em `src/assets/` e Flexibilidade para Forks

A documentação institucional e regulatória mantida no diretório `src/assets/` (abrangendo as pastas `ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY` e `RELEASE`) possui finalidade de referência e publicação oficial:
- Mantenedores de forks têm total liberdade e autorização expressa para modificar, traduzir, customizar ou excluir esses textos para se adequarem às legislações e exigências específicas de seus países e canais de distribuição.

---

## 5. Licenças de Terceiros e Higiene de Dependências

O Huginn Sentinel é construído com dependências Rust modernas e auditadas:
1. **egui / eframe:** Licenciado sob **MIT** ou **Apache 2.0**.
2. **rusqlite / SQLite3:** O crate `rusqlite` é licenciado sob **MIT**, e a engine relacional SQLite3 é formalmente dedicada ao **Domínio Público**.
3. **notify:** Licenciado sob **CC0-1.0** (Dedicação ao Domínio Público).
4. **windows-sys:** Licenciado sob **MIT** ou **Apache 2.0**.
5. **quick-xml, zip, rfd, crossbeam-channel, sha2, sha1, md-5, parking_lot:** Licenciados sob termos padrão do ecossistema Rust (**MIT** / **Apache 2.0**).

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the licensing document in English</summary><br>

# Huginn Sentinel — Licensing, Open Source Terms & Commercial Clarification

**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  
**Source Code License:** GNU Affero General Public License v3.0 (`GNU AGPLv3`)  
**Author & Copyright Holder:** Fernando Nillsson Cidade  

---

## 1. Licensing Model Overview

**Huginn Sentinel** is an enterprise-grade platform distributed with absolute transparency under the strong copyleft open-source terms of the **GNU Affero General Public License v3 (AGPLv3)**:

| Component | License | Permitted Uses |
|---|---|---|
| **Source Code (Full Repository)** | [GNU AGPLv3](../LICENSE) | 100% Free: Study, execution, modification, auditing, commercial and non-commercial distribution, with mandatory sharing of modified source code under the same terms. |
| **Official Microsoft Store Binaries** | Commercial License / EULA | Commercial use across Windows devices linked to the purchaser's Microsoft account, with copyleft exemption for internal use and email support. |
| **Documentation in `src/assets/`** | Project Authorship (Free Developer Grant) | Created for project distribution; developers and downstream forks have complete freedom to adapt, customize, add, or remove documentation for their own platforms. |
| **Self-Compiled Binaries** | GNU AGPLv3 | Free to compile, package, and distribute under your own responsibility, provided that corresponding source code is made available under the GNU AGPLv3. |
| **Upstream Rust Crates (egui, notify, rusqlite, windows-sys)** | MIT / Apache 2.0 / CC0-1.0 / Public Domain | Full attribution compliance maintained with upstream permissive licenses. |

---

## 2. Open Source Rights (GNU AGPLv3)

The complete codebase of Huginn Sentinel — including the asynchronous Windows kernel monitoring engine (`notify` / `ReadDirectoryChangesW`), the 58-column enriched metadata pipeline, the local SQLite WAL persistence (`DbWriter`), the 4-quadrant virtualized spreadsheet with 2D row/column pinning, persistent hover-open menus, 17 independent Windows 11 modeless dialogs (including the Granular Data Cleanup System), canonical security rules (`SEC-R01`–`SEC-R06`), 17 GPU analytics charts, standalone CLI utility (`huginn-cli`), and 6-language localization — is licensed under the **GNU AGPLv3**.

---

## 3. Dual-Licensing Policy & Commercial Authorization

Huginn Sentinel formally adopts a **Dual-Licensing** framework:
1. **Community Edition (GNU AGPLv3):** Free and fully accessible for developers, auditors, and researchers under strong copyleft and reciprocal open-source terms;
2. **Commercial Edition (Microsoft Store):** For organizations and users seeking seamless installation on Windows devices without source-code disclosure obligations for their proprietary workflows. Official purchase via Microsoft Store grants an independent commercial binary license, automatic store updates, and best-effort email support.

> **Rights Holder Statement by Fernando Nillsson Cidade:**  
> I, **Fernando Nillsson Cidade**, as the sole creator and copyright owner of the **Huginn Sentinel** project, hereby expressly grant to any person or organization all permissions established in the **GNU Affero General Public License v3 (AGPLv3)**, including without limitation:
> - The right to use, run, compile, and deploy the software for any commercial, internal, personal, or educational purpose;
> - The right to modify, extend, enhance, and create derivative works;
> - The right to **commercially distribute** or offer services powered by the software, provided that any modified versions and their complete source code are made publicly accessible under the same GNU AGPLv3 terms.

---

## 4. Documentation in `src/assets/` and Downstream Flexibility

The institutional and regulatory documentation maintained under `src/assets/` (`ABOUT`, `CLC`, `COPYRIGHT`, `EULA`, `LICENSES`, `NOTICES`, `PRIVACY_POLICY`, `RELEASE`) is provided for official publication and reference:
- Downstream maintainers and developers have full authorization to modify, adapt, translate, or remove these texts to comply with their local market regulations and packaging requirements.

---

## 5. Third-Party Licenses & Dependency Hygiene

Huginn Sentinel is built strictly with audited, memory-safe Rust dependencies:
1. **egui / eframe:** Dual-licensed under **MIT** or **Apache 2.0**.
2. **rusqlite / SQLite3:** `rusqlite` is licensed under **MIT**, and SQLite3 is dedicated to the **Public Domain**.
3. **notify:** Licensed under **CC0-1.0** (Public Domain Dedication).
4. **windows-sys:** Dual-licensed under **MIT** or **Apache 2.0**.
5. **quick-xml, zip, rfd, crossbeam-channel, sha2, sha1, md-5, parking_lot:** Standard open-source Rust crates (**MIT** / **Apache 2.0**).

</details>
