# Guia do Usuário / Operations User Guide — Huginn Sentinel

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o guia do usuário em português</summary><br>

# Huginn Sentinel — Manual de Operação e Guia do Usuário

**Versão:** `2026.11.9.0`  
**Data:** `9 de novembro de 2026`  
**Plataforma:** Windows 10 / Windows 11 / Windows Server 2022+ (x64)  
**Autor:** Fernando Nillsson Cidade  

---

## 1. Primeiros Passos

### 1.1. Modos de Execução e Inicialização
- **Execução Desktop:** Dê duplo clique em `huginnsentinel.exe` (localizado em `builds\huginnsentinel\release\`). Opcionalmente, passe uma pasta inicial como argumento de linha de comando.
- **Linha de Comando:** Utilize o utilitário autônomo `huginnsentinel-cli.exe` para auditorias diretas via terminal (`huginnsentinel-cli.exe scan "D:\MeusProjetos"`).
- **Interface Gráfica Acelerada:** A interface egui/eframe utiliza renderização acelerada por GPU, garantindo fluidez nativa a 60 FPS com baixíssimo consumo de memória, sem flashes de console na inicialização e com centralização automática da janela no monitor principal.

### 1.2. Fluxo Rápido de Início (Passo a Passo)
1. Inicie o Huginn Sentinel (`huginnsentinel.exe`).
2. Clique no botão **Selecionar Diretório** na barra de ferramentas (`Ctrl+Shift+O`) e escolha a pasta a monitorar.
3. Clique em **Iniciar Varredura** (`Ctrl+R`) ou **Iniciar Monitoramento**:
   - **Encadeamento Inteligente de Ações:** O aplicativo pergunta se deseja iniciar o monitoramento assim que o escaneamento terminar, ou se prefere apenas escanear. Ao clicar em monitoramento, pergunta se deseja realizar uma varredura prévia ou apenas monitorar eventos futuros.
   - **Configurações Padrão:** É possível ativar ou desativar esses diálogos em `Configurações > Escaneamento` e `Configurações > Monitoramento`.
   - **Isolamento Completo:** Durante o escaneamento recursivo, o monitoramento em tempo real permanece pausado para registrar estritamente eventos do tipo "Escaneado", evitando misturas de eventos.
4. O sistema segue monitorando continuamente o sistema de arquivos via Windows `ReadDirectoryChangesW`.

---

## 2. Layout da Interface e Recursos de Produtividade

O Huginn Sentinel organiza suas ferramentas de forma visual e intuitiva:

### 2.1. Menus Superiores com Abertura Persistente por Hover
- Menus raiz: `Arquivo`, `Configurações` e `Opções`.
- **Comportamento Persistente por Hover:** A árvore de menus e submenus **permanece aberta** enquanto o ponteiro do mouse estiver dentro da sua área delimitada. Clicar em opções de configuração ou alternar seleções não fecha o menu; ele fecha automaticamente assim que o ponteiro do mouse sai da área do menu, permitindo múltiplas configurações com grande velocidade.

### 2.2. Barra de Ferramentas Rápida (Toolbar)
- Acesso com um clique: Selecionar Diretório (`Ctrl+Shift+O`), Iniciar/Parar Varredura (`Ctrl+R`), Pausar/Retomar (`Ctrl+P`), Salvar (`Ctrl+S`), Salvar Como (`Ctrl+Shift+S`), Central de Estatísticas (`Ctrl+G`) e Limpar (`Ctrl+L`).
- **Sistema de Limpeza de Dados:** O botão Limpar (`Ctrl+L`) abre uma janela dedicada permitindo a purga seletiva da grade em memória, do banco de dados de eventos (`events\huginn_events.db`) com recriação e VACUUM, ou do banco de segurança (`security\huginn_security.db`), com etapas de confirmação segura, telas de sucesso e ajuste automático de altura no Windows 11.
- **Alinhamento e Harmonia Geométrica:** Todos os botões principais possuem alinhamento vertical rigoroso em relação ao eixo da barra, com moldura contínua e sem linhas divisórias indesejadas.

### 2.3. Painel Lateral de Filtros Rápidos (Esquerda)
- Busca instantânea global em todas as colunas.
- Filtros por tipo de operação (Adicionado, Modificado, Excluído, Renomeado, Movido, Escaneado).
- **Contador Dinâmico de Eventos Totais:** O contador "Total de Eventos:" calcula dinamicamente em tempo real a soma dos eventos correspondentes exclusivamente às caixas de seleção de operação que estiverem marcadas.
- Chips de extensão rápida (ex: `.exe`, `.dll`, `.ps1`, `.docx`, `.pdf`).
- Seletor de nível de risco e contadores de eventos em tempo real.

### 2.4. Rodapé com Indicador Dinâmico de Janelas Modeless
- No canto inferior direito da tela, o rodapé exibe o contador traduzido dinamicamente em tempo de execução: **"1 Janela:"**, **"2 Janelas:"**, **"3 Janelas:"**, etc., seguido pelos botões rápidos para focar ou alternar cada uma das até 16 janelas modeless abertas.

---

## 3. Planilha Virtualizada e Fixação 2D de Linhas e Colunas (Freeze Panes)

A planilha central suporta centenas de milhares de linhas e divide a renderização em **4 quadrantes** para fixação bidimensional arbitrária:

### 3.1. Fixação de Linhas (`Configurações > 📌 Fixar Linhas`)
- **Fixar 1ª Linha (Cabeçalhos):** Mantém os títulos das colunas visíveis no topo durante a rolagem vertical.
- **Fixar Linha Selecionada:** Congela a linha ativa para comparação detalhada.
- **Fixar Linhas por Intervalos:** Permite digitar números e faixas numéricas base 1 (ex: `1, 3, 5-8`).
- **Submenu com Checkboxes:** Lista interativa para marcar e desmarcar linhas individualmente.
- **Desafixar Linhas / Restaurar Padrões:** Remove fixações ou restaura o padrão (cabeçalhos fixos).

### 3.2. Fixação de Colunas (`Configurações > 📌 Fixar Colunas`)
- **Fixar Coluna `#`:** Mantém a coluna de numeração de linhas fixa à esquerda.
- **Fixar Coluna Selecionada:** Congela a coluna em foco.
- **Fixar Todas as Colunas Visíveis:** Congela instantaneamente todas as colunas visíveis na grade.
- **Submenu com Checkboxes:** Permite fixar qualquer conjunto de colunas (Nome, Tipo de Operação, Risco, etc.).
- **Desafixar Colunas / Restaurar Padrões:** Remove fixações ou restaura o padrão (apenas a coluna `#` fixa).

### 3.3. Menus de Contexto
- Clique com o botão direito sobre cabeçalhos ou células para opções imediatas: `📌 Fixar Coluna`, `📌 Fixar Linha`, `Desafixar Coluna` e `Desafixar Linha`.
- Suas preferências de congelamento são automaticamente salvas em `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`.

### 3.4. Seleção Interativa por Clique e Arraste (Células, Colunas e Linhas)
- **Seleção de Faixas de Células:** Clique com o botão esquerdo do mouse sobre qualquer célula e arraste para selecionar um bloco contínuo de células retangulares. A área selecionada é realçada visualmente em tempo real durante o movimento do cursor.
- **Seleção de Faixas de Colunas:** Clique e segure sobre o cabeçalho de uma coluna e arraste horizontalmente para selecionar múltiplas colunas consecutivas. Todas as colunas cobertas são realçadas imediatamente enquanto você arrasta.
- **Seleção de Faixas de Linhas:** Clique e segure sobre o cabeçalho numérico de uma linha (coluna `#`) e arraste verticalmente para selecionar múltiplas linhas com feedback visual contínuo.
- **Preservação de Divisórias no "Selecionar Tudo":** Ao acionar o botão de seleção geral no canto superior esquerdo da grade, todas as células são selecionadas preservando integralmente as divisórias visuais de linhas e colunas.

### 3.5. Comportamento da Seleção (`Configurações > Comportamento da Seleção`)
- **Modo Estático vs. Acompanhar Seleção:** Permite configurar se o cursor deve permanecer fixo na mesma coordenada de célula ao ordenar/atualizar a grade ou se deve acompanhar o registro selecionado.
- **Personalização da Cor de Seleção das Células:** Abra a janela de personalização para escolher entre as 6 cores da paleta recomendada ou definir cores personalizadas via controles RGB/HSV ou digitando diretamente o código hexadecimal HTML (ex: `#00ff00`).
- **Contraste Dinâmico com Tema do Windows 11:** As opções da paleta recomendada alternam dinamicamente a cor da fonte do item selecionado entre branco (tema escuro) e preto (tema claro) ao mudar as configurações do sistema operacional em tempo real.
- **Pré-visualização Comparativa Lado a Lado:** Visualize simultaneamente a cor selecionada em contraste com a cor padrão (Verde Huginn Sentinel).
- **Diálogos de Confirmação:** Janelas de diálogo protegem a aplicação e a restauração de padrões contra alterações acidentais.

### 3.6. Gerenciadores de Colunas e Cores de Operação
- **Gerenciador de Colunas Simplificado:** Apresentação em lista vertical única e limpa com caixas de seleção, eliminando layouts redundantes.
- **Paleta de Cores com HTML Editável:** Na janela de configuração de cores das operações (`Configurações > 🎨 Configurar Cores`), clique em qualquer operação para abrir a paleta de cores equipada com campo de texto HTML editável, botão de fechamento "❌" e botões U8 e 📋 centralizados verticalmente.
- **Botão "🔄 Inverter" de Ordenação:** Inverta com um único clique a direção da ordenação da coluna ativa diretamente na grade.

---

## 4. Segurança, Quarentena e Linha de Base

- **Linha de Base Criptográfica (Baseline SHA-256):** Registra a fotografia de integridade de diretórios críticos e detecta desvios de conformidade (*compliance drift*).
- **Quarentena Segura:** Move arquivos suspeitos para `%LOCALAPPDATA%\HuginnSentinel\quarantine\` com extensão inerte `.quar` e registra a custódia no banco de dados SQLite WAL em `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`.
- **Restauração Auditada:** A devolução do arquivo ao local original é estritamente validada pelo recálculo do hash SHA-256.

---

## 5. 17 Gráficos Analíticos em GPU e Exportação

- Acesso rápido via `Ctrl+G` ou `Opções > Estatísticas`.
- 17 gráficos estatísticos acelerados na GPU via `egui::Painter`: mapas de calor 7x24, diagramas Sankey, histogramas, dispersões de entropia, radar e boxplots.
- **Controles Interativos nos Diagramas Sankey:**
  - *Sankey Operação → Tipo de Arquivo:* Menu suspenso com caixas de seleção (*checkboxes*) para filtrar múltiplos tipos de operações e menu de limite de extensões exibidas (10 a 100 e Todos) com barra de rolagem e fitas Bézier estáveis sem oscilações.
  - *Sankey Operação → Diretório:* Checkboxes para tipos de operação e limitador com rolagem para pastas de destino.
  - *Sankey Tipo → Diretório:* Limitadores com rolagem para extensões e diretórios.
- **Separação Estrita de Renomeado e Movido:** Eventos de arquivos renomeados e movidos são discriminados independentemente na Linha do Tempo, no Radar Multidimensional de Risco e nos diagramas Sankey.
- **Adaptação Dinâmica a Temas do Windows 11:** Todos os 17 gráficos detectam dinamicamente a alternância entre o Tema Claro e Escuro do Windows 11, ajustando fundos, eixos, fontes e contrastes em tempo real sem necessidade de reiniciar.
- **Aprimoramentos Visuais Adicionais:** Pizza de Operações limpa sem divisórias centrais, legendas centralizadas na Evolução de Eventos por Hora e menus suspensos de estatísticas com fechamento automático ao retirar o ponteiro do mouse.
- Exportação em alta resolução: exporte gráficos individuais ou em lote para arquivos PNG.
- Exportação de relatórios em `.csv` (UTF-8 BOM), `.json`, `.html`, `.txt`, `.xml` com prefixo automático estruturado `huginn_eventos_AAAAMMDD_HHMMSS` e pacotes ZIP de evidências selados criptograficamente.

---

## 6. Teclas de Atalho

- `F1`: Ajuda | `Ctrl+M`: Manual | `Ctrl+I`: Sobre | `Ctrl+Shift+O`: Selecionar Diretório.
- `Ctrl+R`: Iniciar/Parar Varredura | `Ctrl+P`: Pausar/Retomar | `Ctrl+S`: Salvar | `Ctrl+Shift+S`: Salvar Como.
- `Ctrl+G`: Gráficos | `Ctrl+L`: Sistema de Limpeza de Dados | `Ctrl+A`: Todas as Colunas | `Ctrl+O`: Alternar Ordenação | `Ctrl+Q`: Sair.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the operations guide in English</summary><br>

# Huginn Sentinel — Operations & User Guide

**Version:** `2026.11.9.0`  
**Date:** `November 9, 2026`  
**Platform:** Windows 10 / Windows 11 / Windows Server 2022+ (x64)  
**Author:** Fernando Nillsson Cidade  

---

## 1. Getting Started

### 1.1. Execution Modes & Startup
- **Desktop Application:** Double-click `huginnsentinel.exe` in `builds\huginnsentinel\release\`. Optionally pass a starting directory as a command-line parameter.
- **Headless CLI:** Use `huginnsentinel-cli.exe` for automation (`huginnsentinel-cli.exe scan "D:\MyData"`).
- **GPU-Accelerated GUI:** Immediate mode egui/eframe engine rendering natively at 60 FPS with minimal memory footprint, zero terminal flash on startup, and automatic centering on the primary display.

### 1.2. Quick-Start Workflow
1. Launch Huginn Sentinel (`huginnsentinel.exe`).
2. Click **Select Directory** on the toolbar (`Ctrl+Shift+O`) to pick the target directory.
3. Click **Start Scan** (`Ctrl+R`) or **Start Monitoring**:
   - **Interactive Workflow Chaining:** Prompts ask whether to start monitoring after scan or scan before monitoring, configurable in `Settings > Scanning` and `Settings > Monitoring`.
   - **Default Behavior:** Notification dialogs can be enabled or disabled according to preference.
   - **Full Scan Isolation:** During recursive scanning, real-time watcher events are paused so only 'Scanned' events are recorded, avoiding event confusion.
4. The system continues continuous file system auditing via Windows `ReadDirectoryChangesW`.

---

## 2. Graphical Workspace & Productivity Features

### 2.1. Persistent Hover-Open Menus
- Top menus: `File`, `Settings`, `Options`.
- **Persistent Hover-Open Behavior:** Menus and submenus **stay open** while the pointer remains inside the menu area. Clicking options or toggling checkboxes does not close the menu; it automatically dismisses when the mouse pointer leaves the menu bounds.

### 2.2. Quick Toolbar
- Instant actions: Select Directory (`Ctrl+Shift+O`), Start/Stop Scan (`Ctrl+R`), Pause/Resume (`Ctrl+P`), Save (`Ctrl+S`), Save As (`Ctrl+Shift+S`), Charts (`Ctrl+G`), and Clear (`Ctrl+L`).
- **Data Cleanup System:** The Clear (`Ctrl+L`) button opens a dedicated dialog allowing selective purging of in-memory grid rows, the events database (`events\huginn_events.db`) with recreation and VACUUM, or the security database (`security\huginn_security.db`), with safety confirmation steps, success feedback, and auto-adapting window heights in Windows 11.
- **Vertical Button Centering:** All primary toolbar buttons feature strict vertical centering relative to the toolbar frame, with seamless frame outlines and no unwanted divider lines.

### 2.3. Left Quick Filter Panel
- Instant search across all 58 columns.
- Operation filters (Added, Modified, Deleted, Renamed, Moved, Scanned).
- **Dynamic Total Events Counter:** The "Total Events:" counter dynamically computes the exact live sum of events corresponding exclusively to checked operation checkboxes.
- Quick extension chips (e.g., `.exe`, `.dll`, `.ps1`, `.docx`, `.pdf`).
- Risk level selector and real-time event counters.

### 2.4. Status Footer with Dynamic Modeless Window Counter
- The footer displays a live localized child window counter: **"1 Window:"**, **"2 Windows:"**, **"3 Windows:"**, etc., followed by quick focus buttons for each open modeless dialog.

---

## 3. 4-Quadrant Virtualized Grid & 2D Pinning (Freeze Panes)

The central virtualized spreadsheet supports arbitrary 2D row and column pinning:

### 3.1. Row Pinning (`Settings > 📌 Pin Rows`)
- **Pin 1st Row (Headers):** Keeps column headers fixed at the top during vertical scrolling.
- **Pin Selected Row:** Freezes the active event row for comparative inspection.
- **Pin Specific Rows by Range:** Accepts 1-based row numbers and ranges (e.g., `1, 3, 5-8`).
- **Checkbox Submenu:** Toggle individual rows interactively.
- **Unpin All Rows / Reset Defaults:** Clears row pins or restores default header freezing.

### 3.2. Column Pinning (`Settings > 📌 Pin Columns`)
- **Pin `#` Column:** Keeps row numbers fixed on the left.
- **Pin Selected Column:** Freezes the currently active column.
- **Pin All Visible Columns:** Instantly freezes every visible column.
- **Checkbox Submenu:** Toggle any column (Name, Operation Type, Risk, etc.).
- **Unpin All Columns / Reset Defaults:** Clears column pins or restores default `#` column freezing.

### 3.3. Context Menus
- Right-click any cell or header for direct pinning actions: `📌 Pin Column`, `📌 Pin Row`, `Unpin Column`, `Unpin Row`.
- Preferences are saved automatically to `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`.

### 3.4. Interactive Click-and-Drag Range Selection (Cells, Columns, and Rows)
- **Cell Block Drag Selection:** Click and drag with the left mouse button across any cells to select a rectangular continuous region. Visual highlights update smoothly in real time during mouse drag.
- **Column Range Drag Selection:** Click and hold on any column header and drag horizontally to select multiple adjacent columns with live visual feedback.
- **Row Range Drag Selection:** Click and hold on any row index header (`#` column) and drag vertically to select multiple consecutive rows.
- **Select All Header Divider Preservation:** Clicking the top-left Select All corner button selects the entire grid while keeping column and row header dividers perfectly intact.

### 3.5. Selection Behavior (`Settings > Selection Behavior`)
- **Static vs. Follow Selection Mode:** Configure whether selection remains pinned to a fixed cell coordinate or tracks the selected record through sorting and updates.
- **Cell Selection Color Customization:** Open the color customization window to choose between 6 recommended palette presets or define custom colors via RGB/HSV controls or direct editable HTML hex entry (`#00ff00`).
- **Dynamic Contrast with Windows 11 Theme:** Selected option text color dynamically adapts between white (dark theme) and black (light theme) when switching OS themes at runtime.
- **Side-by-Side Comparative Preview:** Inspect active custom selection color against default green in real time.
- **Confirmation Dialogs:** Modal confirmation dialogs protect against unintended color overrides or resets.

### 3.6. Column and Operation Colors Managers
- **Streamlined Column Manager:** Unified single-column vertical layout with checkboxes, eliminating empty layouts.
- **Interactive Palette with Editable HTML:** In the Operation Colors dialog (`Settings > 🎨 Configure Colors`), clicking any operation opens a color picker featuring an editable HTML text input with live sync, a dedicated "❌" close button, and vertically centered U8 / 📋 actions.
- **"🔄 Invert" Sort Order Button:** Reverse sort order for the active column directly on the grid with one click.

---

## 4. Security, Quarantine & Baseline

- **Cryptographic Baseline (SHA-256):** Records snapshot integrity and monitors compliance drift.
- **Safe Quarantine:** Isolates suspicious files under `%LOCALAPPDATA%\HuginnSentinel\quarantine\` with inert `.quar` extensions and SQLite custody logs at `%LOCALAPPDATA%\HuginnSentinel\security\huginn_security.db`.
- **Verified Rollback:** Restoration is strictly validated via SHA-256 recalculation.

---

## 5. 17 Native Charts & Export Capabilities

- Access via `Ctrl+G` or `Options > Statistics`.
- 17 native GPU charts: 7x24 heatmaps, Sankey flows, histograms, entropy scatter plots, radar, and boxplots.
- **Interactive Sankey Flow Controls:**
  - *Sankey Operation → File Type:* Checkbox dropdown menu to filter specific operation types, plus item count limiters (10 to 100 and All) with scrollbars and stable Bézier ribbons without flutter.
  - *Sankey Operation → Directory:* Operation checkboxes and scrollable directory limiters.
  - *Sankey Type → Directory:* Scrollable extension and directory limiters.
- **Strict Separation of Renamed and Moved Events:** Disambiguated reporting in Timeline, Multidimensional Risk Radar, and Sankey diagrams.
- **Dynamic Windows 11 Light/Dark Theme Adaptation:** Automatic live theme adjustment across all 17 charts adapting background, text, and axes without requiring application restart.
- **Additional Visual Refinements:** Clean Operations Pie chart without center divider line, centered legends on the hourly evolution chart, and statistical dropdowns that auto-collapse when the mouse moves away.
- High-DPI PNG export for individual charts or full batch.
- Report exports to `.csv` (UTF-8 BOM), `.json`, `.html`, `.txt`, `.xml` with the official structured `huginn_eventos_YYYYMMDD_HHMMSS` prefix and sealed ZIP evidence packages.

---

## 6. Keyboard Shortcuts

- `F1`: Help | `Ctrl+M`: Manual | `Ctrl+I`: About | `Ctrl+Shift+O`: Select Directory.
- `Ctrl+R`: Start/Stop Scan | `Ctrl+P`: Pause/Resume | `Ctrl+S`: Save | `Ctrl+Shift+S`: Save As.
- `Ctrl+G`: Charts | `Ctrl+L`: Data Cleanup System | `Ctrl+A`: Select All Columns | `Ctrl+O`: Toggle Sorting | `Ctrl+Q`: Exit.

</details>
