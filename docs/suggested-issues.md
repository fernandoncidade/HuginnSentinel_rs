# Sugestões de Issues Iniciais / Starter Issues — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher as sugestões de issues em português</summary><br>

# Issues Recomendadas para Engajamento da Comunidade

Abaixo estão 7 templates de issues prontas para publicação no GitHub em [Issues -> New Issue](../issues/new), facilitando a colaboração de desenvolvedores Rust, pesquisadores de segurança e administradores de sistemas Windows:

---

### Issue 1: Suporte a Compartilhamentos de Rede SMB / Caminhos UNC com Reconexão Automática
- **Título:** `[FEATURE] Support remote SMB / UNC network share paths with automatic reconnect`
- **Labels:** `enhancement`, `help wanted`, `networking`, `rust`
- **Conteúdo:**
```markdown
### Resumo
Atualmente, o observador em `src/services/observador.rs` opera com alta performance em volumes locais do Windows através da API nativa `ReadDirectoryChangesW` (via crate `notify`).

### Melhoria Proposta
Adicionar suporte aprimorado para caminhos de rede UNC (ex: `\\servidor\compartilhamento`), incluindo detecção de perda temporária de conectividade de rede SMB e retentativas exponenciais de reconexão sem interromper o loop de eventos da interface.

### Critérios de Aceite
- [ ] Monitoramento contínuo de caminhos UNC sem pânico em threads de trabalho.
- [ ] Reconexão automática em caso de oscilação transitória da rede SMB.
- [ ] Notificação clara na barra de status quando o compartilhamento estiver inacessível.
```

---

### Issue 2: Otimização de Lote SQLite WAL para Fluxos Superiores a 5 Milhões de Eventos
- **Título:** `[PERF] Optimize SQLite WAL batching and memory retention for > 5,000,000 events`
- **Labels:** `performance`, `sqlite`, `rust`
- **Conteúdo:**
```markdown
### Resumo
Em ambientes com altíssima taxa de mutação (ex: compilações contínuas, testes massivos), o volume de eventos acumulados pode atingir milhões de registros em poucas horas.

### Melhoria Proposta
- Otimizar os lotes de transação e intervalos de flush em `src/services/db_writer.rs`.
- Avaliar estratégias de compactação e paginação sob demanda na memória para a visualização na grade de 58 colunas.

### Critérios de Aceite
- [ ] Consumo de memória estável e controlado mesmo com 5.000.000 de eventos capturados.
- [ ] Gravação transacional assíncrona fluida sem causar queda de taxa de quadros (60 FPS) no egui.
```

---

### Issue 3: Regras Customizadas com Expressões Regulares no Motor de Segurança
- **Título:** `[FEATURE] Support user-defined regex rules in Security Rules Engine`
- **Labels:** `enhancement`, `security`, `rust`
- **Conteúdo:**
```markdown
### Resumo
O motor de segurança em `src/security/rules.rs` implementa 6 regras canônicas determinísticas (`SEC-R01` a `SEC-R06`).

### Melhoria Proposta
Permitir que os usuários configurem regras adicionais através de expressões regulares (regex) na interface gráfica ou via arquivo JSON, detectando padrões específicos de extensões corporativas, nomes de arquivo confidenciais ou palavras-chave proibidas.

### Critérios de Aceite
- [ ] Diálogo interativo para cadastro de regras customizadas com validação de regex.
- [ ] Atribuição de severidade configurável (Baixo, Médio, Alto, Crítico).
- [ ] Emissão automática de alertas na Central de Alertas e registro com código `SEC-CUSTOM`.
```

---

### Issue 4: Adicionar Traduções para Japonês (ja_JP) e Chinês Simplificado (zh_CN)
- **Título:** `[I18N] Add Japanese (ja_JP) and Simplified Chinese (zh_CN) language support`
- **Labels:** `i18n`, `good first issue`
- **Conteúdo:**
```markdown
### Resumo
O Huginn Sentinel suporta nativamente 6 idiomas em tempo de execução (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `de_DE`). A inclusão de Japonês e Chinês Simplificado ampliará significativamente o alcance internacional da ferramenta.

### Etapas de Implementação
1. Criar os dicionários `src/language/ja_JP.json` e `src/language/zh_CN.json` com todas as chaves de interface.
2. Registrar os novos idiomas em `src/locale/localizer.rs`.
3. Adicionar as seções traduzidas correspondentes no manual interativo em `src/locale/manual.rs`.
```

---

### Issue 5: Exportação Vetorial Direta dos 17 Gráficos Estatísticos em SVG
- **Título:** `[FEATURE] Support direct SVG vector export alongside PNG raster in Charts dialog`
- **Labels:** `enhancement`, `data-viz`, `rust`
- **Conteúdo:**
```markdown
### Resumo
Atualmente, os 17 gráficos estatísticos são renderizados via `egui::Painter` e exportados em alta resolução rasterizada em PNG através de `src/data/render/export.rs`.

### Melhoria Proposta
Adicionar a opção de salvar os gráficos diretamente no formato vetorial SVG, facilitando a inclusão em laudos periciais e publicações acadêmicas em escala infinita sem perda de definição.

### Critérios de Aceite
- [ ] Botão "Exportar SVG" na barra de ações da janela de Estatísticas.
- [ ] Geração limpa de primitivas SVG (linhas, arcos, polígonos, textos).
```

---

### Issue 6: Filtros com Operadores Lógicos Compostos (AND/OR/NOT)
- **Título:** `[FEATURE] Advanced search filters with composite boolean operators (AND/OR/NOT)`
- **Labels:** `enhancement`, `ui`, `rust`
- **Conteúdo:**
```markdown
### Resumo
Os filtros avançados em `src/tools/filters.rs` permitem combinações de operação, data, extensão e risco.

### Melhoria Proposta
Adicionar suporte a operadores booleanos avançados no campo de busca rápida (ex: `ext:exe AND NOT dir:Temp AND risk:>=60`).

### Critérios de Aceite
- [ ] Parser seguro para expressões de filtro com operadores `AND`, `OR`, `NOT`.
- [ ] Filtragem em tempo real com validação sintática visual.
```

---

### Issue 7: Geração Automática de Artefatos em Releases do GitHub Actions
- **Título:** `[CI/CD] Automate release packaging and checksum generation in GitHub Actions`
- **Labels:** `ci`, `devops`, `automation`
- **Conteúdo:**
```markdown
### Resumo
Ao realizar o push de uma nova tag de versão (ex: `v2026.11.11.0`), a esteira do GitHub Actions deve compilar os binários de release MSVC, gerar os pacotes ZIP e calcular os hashes SHA-256 automaticamente.

### Critérios de Aceite
- [ ] Workflow em `.github/workflows/` ativado por tags de versão.
- [ ] Compilação otimizada com perfil Release para `huginnsentinel.exe` e `huginnsentinel-cli.exe`.
- [ ] Anexo automático dos binários e do arquivo `SHA256SUMS.txt` na Release do GitHub.
```

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse suggested starter issues in English</summary><br>

# Recommended Issues for Community Engagement

Below are 7 ready-to-publish issue templates for GitHub at [Issues -> New Issue](../issues/new) to guide open-source contributions:

---

### Issue 1: Support Remote SMB / UNC Network Share Paths with Automatic Reconnect
- **Title:** `[FEATURE] Support remote SMB / UNC network share paths with automatic reconnect`
- **Labels:** `enhancement`, `help wanted`, `networking`, `rust`
- **Content:**
```markdown
### Summary
Currently, `src/services/observador.rs` monitors Windows file systems with high performance using `ReadDirectoryChangesW` via `notify`.

### Proposed Enhancement
Add enhanced support for UNC network paths (e.g., `\\server\share`), including transient connection drop detection and exponential backoff reconnection without freezing the UI event loop.

### Acceptance Criteria
- [ ] Continuous monitoring of UNC paths without worker thread panics.
- [ ] Automatic reconnect on network flakiness.
- [ ] Status bar notifications when remote paths become unreachable.
```

---

### Issue 2: Optimize SQLite WAL Batching for Over 5,000,000 Events
- **Title:** `[PERF] Optimize SQLite WAL batching and memory retention for > 5,000,000 events`
- **Labels:** `performance`, `sqlite`, `rust`
- **Content:**
```markdown
### Summary
In heavy development or rendering environments, accumulated event volumes can reach millions of entries within hours.

### Proposed Enhancement
- Optimize commit batching and checkpoint intervals in `src/services/db_writer.rs`.
- Implement on-demand memory paging for the 58-column virtualized table.

### Acceptance Criteria
- [ ] Controlled memory usage even with 5,000,000 recorded events.
- [ ] Smooth asynchronous commits without dropping below 60 FPS in egui.
```

---

### Issue 3: Custom User-Defined Regex Rules in Security Engine
- **Title:** `[FEATURE] Support user-defined regex rules in Security Rules Engine`
- **Labels:** `enhancement`, `security`, `rust`
- **Content:**
```markdown
### Summary
The security engine in `src/security/rules.rs` enforces 6 deterministic canonical rules (`SEC-R01`–`SEC-R06`).

### Proposed Enhancement
Allow users to define custom regex pattern matching rules via GUI or JSON to flag sensitive extensions or forbidden filename keywords.

### Acceptance Criteria
- [ ] Interactive dialog with real-time regex syntax checking.
- [ ] Configurable threat rating (Low, Medium, High, Critical).
- [ ] Immediate alerts dispatched with code `SEC-CUSTOM`.
```

---

### Issue 4: Add Japanese (ja_JP) and Simplified Chinese (zh_CN) Translations
- **Title:** `[I18N] Add Japanese (ja_JP) and Simplified Chinese (zh_CN) language support`
- **Labels:** `i18n`, `good first issue`
- **Content:**
```markdown
### Summary
Huginn Sentinel supports 6 languages at runtime (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, `de_DE`). Adding Japanese and Simplified Chinese will broaden international adoption.

### Implementation Steps
1. Add `src/language/ja_JP.json` and `src/language/zh_CN.json`.
2. Register the new locales in `src/locale/localizer.rs`.
3. Add corresponding localized sections in `src/locale/manual.rs`.
```

---

### Issue 5: Direct SVG Vector Export for the 17 Charts
- **Title:** `[FEATURE] Support direct SVG vector export alongside PNG raster in Charts dialog`
- **Labels:** `enhancement`, `data-viz`, `rust`
- **Content:**
```markdown
### Summary
Currently, the 17 charts are rendered via `egui::Painter` and exported as high-DPI PNG images via `src/data/render/export.rs`.

### Proposed Enhancement
Add an option to save charts directly as vector SVG files for publication and forensic reporting.

### Acceptance Criteria
- [ ] "Export SVG" option in the Statistics window.
- [ ] Clean vector primitives generation (lines, arcs, polygons, labels).
```

---

### Issue 6: Advanced Filters with Composite Boolean Operators (AND/OR/NOT)
- **Title:** `[FEATURE] Advanced search filters with composite boolean operators (AND/OR/NOT)`
- **Labels:** `enhancement`, `ui`, `rust`
- **Content:**
```markdown
### Summary
Advanced filters in `src/tools/filters.rs` support combined criteria across operations, dates, extensions, and risk.

### Proposed Enhancement
Add support for composite boolean expressions in the quick search box (e.g., `ext:exe AND NOT dir:Temp AND risk:>=60`).

### Acceptance Criteria
- [ ] Safe query parser for `AND`, `OR`, `NOT` operators.
- [ ] Real-time filtering with visual query validation.
```

---

### Issue 7: Automated Release Artifact Packaging in GitHub Actions
- **Title:** `[CI/CD] Automate release packaging and checksum generation in GitHub Actions`
- **Labels:** `ci`, `devops`, `automation`
- **Content:**
```markdown
### Summary
Upon tagging a new release (e.g., `v2026.11.11.0`), the GitHub Actions workflow should compile optimized release binaries, package them into ZIP files, and compute SHA-256 checksums automatically.

### Acceptance Criteria
- [ ] Workflow triggered on version tags.
- [ ] Release profile builds for `huginnsentinel.exe` and `huginnsentinel-cli.exe`.
- [ ] Automated attachment of binary archives and `SHA256SUMS.txt` to GitHub Releases.
```

</details>
