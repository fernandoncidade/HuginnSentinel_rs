# MANUAL DO USUÁRIO E GUIA DE AUDITORIA E SEGURANÇA — HUGINN SENTINEL
# USER MANUAL AND SECURITY AUDIT GUIDE — HUGINN SENTINEL

**Versão:** `2026.11.11.0`  
**Data:** `11 de novembro de 2026`  
**Version:** `2026.11.11.0`  
**Date:** `November 11, 2026`  
**Ambiente Tecnológico / Tech Stack:** Rust (Edição 2021) | egui 0.37 / eframe | Win32 API (`windows-sys`, `ReadDirectoryChangesW`, `WinTrust`) | SQLite3 (`rusqlite` WAL)  
**Plataforma / Platform:** Windows 11 / Windows 10 / Windows Server 2022+ (x64)  
**Binários / Binaries:** `huginnsentinel.exe` (GUI Desktop) | `huginnsentinel-cli.exe` (Console CLI)  
**Licença / License:** GNU AGPLv3  
**Autor / Author:** Fernando Nillsson Cidade  

---

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o Manual do Usuário em Português</summary><br>

# Manual do Usuário — Huginn Sentinel

Versão documentada: `2026.11.11.0`  
Data: `11 de novembro de 2026`  

## 1. Introdução & Filosofia Local-First

O **Huginn Sentinel** é uma plataforma corporativa desktop de alta performance para auditoria forense de integridade de arquivos, monitoramento contínuo em tempo real do sistema de arquivos Windows e detecção proativa e autônoma de ameaças cibernéticas locais (ransomware, binários dissimulados, scripts desprotegidos e desvios de conformidade).

A aplicação foi integralmente construída em **Rust (Edição 2021)** com interface em modo imediato acelerada por hardware em **`egui 0.37` / `eframe`** e banco de dados relacional embutido SQLite em modo Write-Ahead Logging (WAL). O sistema opera sob o compromisso inegociável de **Privacidade Soberana (Local-First)**:
- **100% Offline:** Zero tráfego de rede, zero telemetria, zero telemetria oculta e zero conexões com servidores externos.
- **Sem IA ou Modelos em Nuvem:** Todos os laudos periciais, pontuações quantitativas de risco (0–100) e as 58 colunas de metadados enriquecidos são gerados e harmonizados através de lógica algorítmica e matemática determinística pura.
- **Armazenamento Seguro Local:** Persistência transacional em pastas próprias e isoladas em `%LOCALAPPDATA%\HuginnSentinel Sentinel` (`events\huginn_events.db` para eventos de auditoria e `security\huginn_security.db` para linha de base, incidentes e quarentena), garantindo integridade ACID mesmo em desligamentos inesperados.
- **Inicialização Ergonômica:** A janela principal do aplicativo é inicializada automaticamente centralizada no monitor principal do usuário.

---

## 2. Layout da Interface Gráfica e Menus Persistentes por Hover

A interface desktop do Huginn Sentinel é estruturada de forma ergonômica em componentes de alta produtividade:

1. **Barra de Menus Superior com Abertura Persistente por Hover:**
   - Menus raiz: `Arquivo`, `Configurações` e `Opções`.
   - **Comportamento Persistente por Hover:** Ao abrir qualquer menu ou submenu (inclusive menus de contexto acionados com botão direito e listas suspensas ComboBox), a árvore do menu **permanece aberta** enquanto o ponteiro do mouse estiver sobre a sua área. Clicar em opções (como ativar/desativar colunas, fixar linhas ou alternar modos) **não fecha o menu automaticamente**, permitindo múltiplas seleções consecutivas com agilidade. O menu é recolhido automaticamente assim que o usuário retira o ponteiro do mouse da região do menu.
2. **Barra de Ferramentas Rápida (Toolbar):**
   - Botões de acesso imediato: Selecionar Diretório (`Ctrl+Shift+O`), Iniciar/Parar Varredura (`Ctrl+R`), Pausar/Retomar Monitoramento (`Ctrl+P`), Salvar Relatório (`Ctrl+S`), Salvar Como (`Ctrl+Shift+S`), Estatísticas/Gráficos (`Ctrl+G`) e Limpar (`Ctrl+L`). Todos os botões contam com alinhamento vertical rigoroso e moldura integrada e justificada.
   - **Fluxo Interativo de Escaneamento e Monitoramento:** Ao clicar em "🔍 Escanear", o sistema pergunta se deseja monitorar após a varredura; ao clicar em "▶️ Iniciar Monitoramento", pergunta se deseja escanear antes de monitorar. Os submenus `Configurações > Escaneamento` e `Configurações > Monitoramento` permitem configurar o comportamento padrão e ativar/desativar os avisos de confirmação. Durante o escaneamento, o observador é isolado para reportar estritamente eventos do tipo "Escaneado".

### 2.1. Sistema de Limpeza de Dados Individual e Granular (`🗑️ Sistema de Limpeza de Dados`)
Ao acionar o botão `Limpar` na barra de ferramentas, o atalho `Ctrl+L` ou os menus `Arquivo > 🗑️ Limpar...` e `Segurança > 🛡️ Limpar Banco de Segurança...`, o Huginn Sentinel abre uma janela modeless nativa com suporte a Z-order dinâmico que oferece controle pontual sobre o que deve ser purgado:
- **🧹 Limpar Tabela, Banco de Segurança e de Dados:** Efetua uma limpeza integral, expurgando simultaneamente os eventos em memória da planilha, o banco de dados de eventos gravados e os registros de segurança.
- **🗄️ Limpar banco de dados:** Purga exclusivamente o banco relacional de eventos (`events\huginn_events.db`), recriando sua estrutura relacional, eliminando arquivos temporários WAL/SHM e executando compactação atômica (`VACUUM`).
- **📋 Limpar tabela da UI:** Limpa unicamente as linhas visíveis na grade da interface em memória, mantendo intactos todos os eventos gravados no banco de dados para consultas futuras.
- **🛡️ Limpar banco de segurança:** Purga atomicamente a base de dados de segurança (`security\huginn_security.db`), reinicializando o estado de linhas de base e incidentes.
- **❌ Cancelar:** Fecha o diálogo sem realizar qualquer alteração nos dados.

**Fluxo de Segurança e Confirmação:**
- Qualquer seleção de limpeza encaminha o usuário para uma etapa de confirmação (`⚠️ Confirmar`), detalhando o alvo escolhido e emitindo alertas claros sobre a irreversibilidade da operação, com botão de confirmação em destaque visual de perigo e botão `↩️ Voltar`.
- Telas informativas de sucesso (`✅ Sucesso`) confirmam a conclusão exata da operação, e janelas de alerta notificam eventuais falhas.
- O diálogo conta com botões proporcionais e alinhados entre si com base na maior legenda nos 6 idiomas, centralização óptica nos eixos vertical e horizontal, espaçamento inferior de respiro e ajuste automático de altura para cada etapa no Windows 11.
3. **Painel Lateral de Filtros Rápidos (Esquerda):**
   - Campo de busca instantânea global, seleção rápida de tipos de operação, chips de extensões, filtro por severidade de risco e contadores de eventos em tempo real.
   - **Contador Dinâmico de Eventos:** O mostrador "Total de Eventos:" calcula dinamicamente em tempo de execução a soma exata apenas dos tipos de operações cujas caixas de seleção estiverem marcadas, permitindo a apuração precisa de subconjuntos de eventos.
4. **Planilha Central Virtualizada de 58 Colunas em 4 Quadrantes:**
   - Área central de visualização de alta performance com suporte a centenas de milhares de linhas e fixação 2D arbitrária de linhas e colunas.
5. **Rodapé de Status com Contador Dinâmico de Janelas Modeless:**
   - Exibe o diretório sob monitoramento, status do observador de disco, barra de progresso de varredura e o indicador de janelas filhas ativas traduzido dinamicamente em tempo de execução: **"1 Janela:"**, **"2 Janelas:"**, **"3 Janelas:"**, etc., seguido pelos botões de alternância e foco de cada diálogo aberto.

---

## 3. Planilha Virtualizada e Fixação 2D de Linhas e Colunas (Freeze Panes)

A planilha do Huginn Sentinel implementa um mecanismo avançado de layout em **4 quadrantes** (`Superior Esquerdo`, `Superior Direito`, `Inferior Esquerdo` e `Inferior Direito`), permitindo ao usuário manter visíveis referências críticas de auditoria enquanto rola horizontal e verticalmente por grandes volumes de dados:

### 3.1. Funcionalidades de Fixação de Linhas (`Configurações > 📌 Fixar Linhas`)
- **Fixar 1ª Linha (Cabeçalhos das Colunas):** Mantém os títulos das colunas permanentemente visíveis no topo da tabela durante a rolagem vertical.
- **Fixar Linha Selecionada:** Permite congelar a linha atualmente em foco no quadrante superior para comparação lado a lado.
- **Fixar Linha Específica por Entrada Numérica / Intervalos:** Permite digitar números de linhas (base 1) e faixas separadas por vírgula e hífen (ex: `1, 3, 5-8`).
- **Submenu com Checkboxes:** Lista as linhas da tabela permitindo marcar e desmarcar individualmente quais linhas permanecem fixas.
- **Desafixar Todas as Linhas / Restaurar Padrões:** Remove todas as fixações de linhas ou redefine para o padrão recomendado (1ª linha de cabeçalhos fixa).

### 3.2. Funcionalidades de Fixação de Colunas (`Configurações > 📌 Fixar Colunas`)
- **Fixar Coluna `#` (Numeração):** Mantém a coluna índice de linhas visível no quadrante esquerdo durante a rolagem horizontal.
- **Fixar Coluna Selecionada:** Fixa a coluna atualmente selecionada.
- **Submenu com Checkboxes de Todas as Colunas:** Permite fixar qualquer conjunto de colunas (ex: `Nome`, `Tipo de Operação`, `Nível de Risco`, `Hash SHA-256`).
- **Fixar Todas as Colunas Visíveis:** Congela instantaneamente todas as colunas que estão visíveis na tela.
- **Desafixar Todas as Colunas / Restaurar Padrões:** Remove as fixações ou redefine para o padrão (apenas a coluna `#` fixa).

### 3.3. Menus de Contexto e Persistência
- Clicar com o botão direito sobre qualquer cabeçalho de coluna ou célula exibe opções diretas: `📌 Fixar Coluna`, `📌 Fixar Linha`, `Desafixar Coluna` e `Desafixar Linha`.
- Todas as preferências de fixação bidimensional são automaticamente salvas e restauradas entre sessões no arquivo `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`.

### 3.4. Seleção Interativa por Clique e Arraste (Células, Colunas e Linhas)
- **Seleção Dinâmica de Intervalo de Células:** Clique e segure com o botão esquerdo sobre qualquer célula e arraste o mouse pela tabela. A seleção retangular de células é realçada visualmente em tempo real durante todo o arraste do ponteiro, confirmando a área selecionada ao soltar o clique.
- **Seleção de Múltiplos Cabeçalhos de Coluna:** Clique e segure sobre o cabeçalho de uma coluna e arraste horizontalmente para selecionar um conjunto contínuo de colunas com realce imediato.
- **Seleção de Múltiplos Cabeçalhos de Linha:** Clique e segure sobre o cabeçalho de numeração de uma linha (`#`) e arraste verticalmente para selecionar um bloco contínuo de linhas com realce imediato.
- **Botão Selecionar Tudo:** O botão localizado no canto superior esquerdo (interseção entre cabeçalhos de linhas e colunas) seleciona instantaneamente todas as células da planilha, preservando com nitidez as linhas divisórias e bordas dos cabeçalhos.

### 3.5. Comportamento da Seleção e Personalização de Cores (`Configurações > Comportamento da Seleção`)
- **Modo Estático vs. Acompanhar Seleção:** Permite ao auditor definir se a seleção de uma célula, conjunto de células ou linhas deve permanecer fixada na posição da grade ou se deve acompanhar ativamente a ordenação e inserção de novos dados.
- **Opções Granulares:** Configuração independente para acompanhar células, acompanhar linhas e ativar rolagem automática para a seleção em foco.
- **Personalizar Cor de Seleção das Células (`🎨 Personalizar Cor de Seleção das Células`):**
  - **Paleta de Cores Recomendadas (Presets Rápidos):** Oferece opções temáticas prontas (Verde Excel, Azul Corporativo, Ciano/Teal, Roxo, Laranja, Vermelho, Grafite) com contraste tipográfico adaptativo: a cor da fonte da opção selecionada migra automaticamente para branco no tema escuro e para preto no tema claro do Windows 11.
  - **Seletor Customizado:** Ajuste livre através de gradiente 2D de saturação/brilho, barra de matiz 1D e canais numéricos R, G, B.
  - **Código HTML Editável (`HTML: #00ff00`):** Permite inserir, digitar ou colar diretamente qualquer código hexadecimal de cor, sincronizando a paleta em tempo real.
  - **Visualização Prévia de Comparação na Planilha:** Exibe lado a lado uma célula não selecionada e uma célula selecionada com a cor em teste, facilitando a decisão antes de aplicar.
  - **Confirmações de Segurança:** Diálogos dedicados para aplicar e salvar ou restaurar a cor padrão recomendada (Verde).

### 3.6. Gerenciador de Colunas e Gerenciador de Cores
- **Gerenciador de Colunas (`Configurações > Configurar Colunas > Abrir Gerenciador de Colunas...`):** Janela otimizada com lista unificada de colunas disponíveis, botões em lote (`Selecionar Todas`, `Desmarcar Todas`) e restauração de larguras e ordens padrão.
- **Gerenciador de Cores das Operações (`Configurações > Configurar Cores`):** Permite configurar as cores associadas às 6 operações auditadas. A paleta de cores conta com saída e entrada HTML editável, botão de fechamento rápido "❌", botões "U8" e "📋" com alinhamento vertical e descrições localizadas dinamicamente em tempo de execução, além de centralização vertical do botão "Restaurar Padrões".
- **Botão Inverter Ordenação:** O botão "🔄 Inverter" na tabela permite alternar instantaneamente a ordenação entre ascendente e descendente para toda a coluna selecionada através do seu cabeçalho.

---

## 4. Tabela de 58 Colunas e Harmonização Determinística de Metadados

A tabela central oferece até **58 colunas** ricas de metadados com extração e harmonização determinística (`harmonizacao.rs`), garantindo integridade e consistência estrita em todas as 6 operações auditadas (`Adicionado`, `Modificado`, `Excluído`, `Renomeado`, `Movido` e `Escaneado`):

- **Identificação e Caminhos:** ID, Tipo de Operação, Nome do Arquivo, Extensão, Diretório Atual, Diretório Anterior, Caminho Completo Atual, Caminho Completo Anterior.
- **Temporalidade Precisa:** Data/Hora Criação (UTC/Local), Data/Hora Modificação (UTC/Local), Data/Hora Último Acesso (UTC/Local).
- **Métricas Físicas:** Tamanho Formatado (B, KB, MB, GB), Tamanho Exato em Bytes, Variação de Tamanho (Diff Bytes).
- **Atributos de Arquivo Windows:** Somente Leitura, Oculto, Sistema, Arquivo Normal, Compactado, Criptografado, Esparso, Temporário, Ponto de Reparse (Reparse Point / Junction).
- **Identificação Forense & Magic Numbers:** Tipo de Arquivo Determinado por Magic Numbers, Tipo MIME Canônico, Entropia de Shannon (0.00 a 8.00 bits/byte).
- **Criptografia & Integridade:** Hashes criptográficos SHA-256, SHA-1, MD5 e status de conferência de Baseline.
- **Segurança & Executáveis PE:** Nível de Risco (Crítico, Alto, Médio, Baixo), Pontuação Numérica de Risco (0–100), Regra de Alerta Acionada, Status de Assinatura Digital Authenticode, Arquitetura PE (x86, x64, ARM64) e Subsistema Windows.

---

## 5. Motor de Regras Canônicas de Segurança

O sistema avalia cada evento contra 6 regras canônicas determinísticas (`rules.rs`):
- **`SEC-R01` (Ransomware Heuristics):** Identifica rajadas rápidas de escrita/modificação associadas a picos de entropia de Shannon (dados cifrados) e renomeação em massa com extensões suspeitas (`.locked`, `.crypto`, etc.).
- **`SEC-R02` (Extensão Dupla Mascarada):** Detecta tentativas de mascarar executáveis e scripts sob aparências inofensivas (ex: `relatorio.pdf.exe`, `fatura.docx.vbs`).
- **`SEC-R03` (Executável ou Binário Não Assinado em Temp/AppData):** Sinaliza a introdução de binários não assinados ou sem assinatura Authenticode válida em diretórios transitórios do usuário.
- **`SEC-R04` (Criação de Scripts de Alto Risco):** Monitora a criação ou edição de scripts de automação (`.ps1`, `.bat`, `.cmd`, `.vbs`, `.js`) em diretórios críticos.
- **`SEC-R05` (Executável Oculto no Sistema):** Detecta executáveis com atributos de arquivo oculto ou de sistema ativados.
- **`SEC-R06` (Deriva Crítica de Linha de Base / Discrepância SHA-256):** Discrepância imediata entre o hash SHA-256 calculado em tempo real e o snapshot pré-gravado na linha de base.

---

## 6. Sistema de Quarentena Segura e Custódia Forense

O subsistema de quarentena permite isolar arquivos perigosos sem risco de execução acidental:
- **Isolamento Atômico:** O arquivo é renomeado e movido para a pasta protegida `%LOCALAPPDATA%\HuginnSentinel\quarantine\` com extensão neutra `.quar`.
- **Cadeia de Custódia em SQLite WAL:** Todos os parâmetros de auditoria (caminho original, timestamp do evento, identificador do incidente, motivo da regra e hash SHA-256 original) são persistidos na tabela transacional `quarentena`.
- **Restauração Segura Verificada:** A devolução do arquivo para o local de origem só é autorizada após recálculo estrito do hash SHA-256, impedindo a restauração de artefatos adulterados.

---

## 7. 17 Gráficos Analíticos Acelerados na GPU

Acessível por `Ctrl+G` ou pelo menu `Opções > Estatísticas`, a central de gráficos oferece 17 visualizações estatísticas em modo imediato geradas via GPU com renderização suave e adaptativa:
1. Distribuição de Eventos por Tipo de Operação (Pizza e Barras — visual limpo sem linhas divisórias indesejadas).
2. Histograma de Distribuição de Tamanhos de Arquivo (KB / MB).
3. Linha Temporal de Atividade por Hora do Dia e Dia da Semana (Mapa de Calor 7x24 com dias e escalas traduzidos).
4. Top Extensões com Maior Volume de Alterações (com seletor de quantidade máxima).
5. Curva de Pareto 80/20 de Extensões e Boxplot Tukey de Tamanhos de Arquivo.
6. Linha do Tempo de Operações e Diagramas de Fluxo Relacional Sankey (`Operação → Tipo`, `Operação → Diretório` e `Tipo → Diretório`).
7. Dispersão Logarítmica de Tamanho vs. Hora e Dotplot de Distribuição Volumétrica.
8. Radar Operacional e Radar Multidimensional de Risco (com reporte separado e exato de `Adicionado`, `Modificado`, `Excluído`, `Renomeado`, `Movido` e `Escaneado`).
9. Árvore Hierárquica de Diretórios e Séries Temporais de Evolução de Eventos por Hora (com legendas perfeitamente centralizadas).

### 7.1. Controles Interativos dos Diagramas Sankey
- **Filtro de Tipos de Operação com Checkboxes:** Nos diagramas `Sankey: Operação → Tipo de Arquivo` e `Sankey: Operação → Diretório`, a lista suspensa de operações disponibiliza caixas de seleção interativas para cada evento (`Adicionado`, `Modificado`, `Excluído`, `Renomeado`, `Movido`, `Escaneado`), permitindo ao usuário isolar ou combinar fluxos com precisão.
- **Configuração de Quantidade de Itens Plotados:** Permite definir o limite de nós exibidos (extensões ou diretórios) através de lista suspensa com opções de 10 em 10 até 100 e a opção "Todos", contando com barra de rolagem integrada (*scrollbar*) para acomodar grandes volumes sem poluição visual.
- **Estabilidade Geométrica de Fitas Bézier:** As conexões de fluxo utilizam ordenação determinística com desempate alfabético estrito, eliminando oscilações, tremores ou flickering nas fitas relacionais.

### 7.2. Adaptação Automática aos Temas Claro e Escuro do Windows 11
- Todos os 17 gráficos ajustam dinamicamente em tempo de execução suas cores de fundo, bordas de cartões, contraste tipográfico, grades e eixos ao alternar o tema do Windows 11 entre Modo Claro e Modo Escuro, sem necessidade de reiniciar o aplicativo.
- As listas suspensas da barra de controle estatístico contam com recolhimento automático ao retirar o ponteiro do mouse da sua área (*hover-exit*).

- **Exportação Gráfica:** Todos os 17 gráficos podem ser exportados individualmente ou em lote para arquivos PNG de alta resolução.

---

## 8. Exportação de Relatórios e Evidências Digitais

- **Formatos de Relatório Disponíveis:** `.csv` (com UTF-8 BOM para Excel e Power BI), `.json` (estruturado), `.html` (estilizado), `.txt` (tabular) e `.xml` (padronizado).
- **Nomenclatura Padrão de Arquivos Exportados:** Ao salvar ou exportar dados escaneados e monitorados, os arquivos gerados recebem automaticamente o prefixo oficial estruturado no padrão `huginn_eventos_AAAAMMDD_HHMMSS` (por exemplo, `huginn_eventos_20261104_113331.csv`), garantindo uniformidade e fácil organização em auditorias e cópias de segurança.
- **Exportação de Evidência Selada em ZIP:** Gera um pacote ZIP criptograficamente auditável contendo os eventos exportados, metadados periciais, hashes SHA-256 de custódia e um manifesto de integridade.

---

## 9. Utilitário de Linha de Comando (CLI Headless)

O executável autônomo `huginnsentinel-cli.exe` fornece suporte completo para scripts PowerShell e esteiras de automação:

```powershell
# Exibir versão oficial e sintaxe de ajuda
.\builds\huginnsentinel\release\huginnsentinel-cli.exe --help
.\builds\huginnsentinel\release\huginnsentinel-cli.exe --version

# Executar varredura recursiva de diretório com relatório JSON
.\builds\huginnsentinel\release\huginnsentinel-cli.exe scan "C:\MeusDados"

# Extrair metadados forenses profundos, atributos e hashes de um arquivo específico
.\builds\huginnsentinel\release\huginnsentinel-cli.exe metadata "C:\MeusDados\binario.exe"

# Comparar ou atualizar a linha de base criptográfica SHA-256
.\builds\huginnsentinel\release\huginnsentinel-cli.exe baseline "C:\MeusDados"
.\builds\huginnsentinel\release\huginnsentinel-cli.exe baseline "C:\MeusDados" --update

# Iniciar monitoramento contínuo em tempo real (opcionalmente com duração em segundos)
.\builds\huginnsentinel\release\huginnsentinel-cli.exe monitor "C:\MeusDados" --duration 120

# Gerenciar o cofre de quarentena
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine list
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine restore 1 "C:\Restaurados"
```

---

## 10. Atalhos de Teclado & Teclas Rápidas

- `F1`: Abrir janela de Ajuda e atalhos.
- `Ctrl+M`: Abrir Manual do Usuário interativo.
- `Ctrl+I`: Abrir diálogo Sobre o Huginn Sentinel.
- `Ctrl+Shift+O`: Selecionar diretório base de trabalho.
- `Ctrl+R`: Iniciar ou parar varredura recursiva.
- `Ctrl+P`: Pausar ou retomar observação em tempo real.
- `Ctrl+S`: Salvar relatório rápido.
- `Ctrl+Shift+S`: Salvar Como (escolher nome, pasta e formato).
- `Ctrl+G`: Abrir central de gráficos estatísticos (17 gráficos).
- `Ctrl+L`: Abrir o Sistema de Limpeza de Dados (limpeza granular de tabela, banco de eventos e banco de segurança).
- `Ctrl+A`: Selecionar e exibir todas as 58 colunas.
- `Ctrl+Shift+A`: Selecionar todas as cores de colunas para realce.
- `Ctrl+O`: Alternar ordenação de linhas ao clicar nos cabeçalhos.
- `Ctrl+Q`: Encerrar a aplicação com segurança.

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse User Operations Manual in English</summary><br>

# User Operations Manual — Huginn Sentinel

Documented version: `2026.11.11.0`  
Date: `November 11, 2026`  

## 1. Overview & Sovereign Local-First Philosophy

**Huginn Sentinel** is an enterprise-grade Windows desktop platform and CLI suite for forensic file system integrity auditing, real-time file system monitoring, and autonomous threat mitigation (ransomware detection, disguised binaries, risky scripts, and compliance drift).

The application is completely engineered in **Rust (Edition 2021)** featuring an immediate mode graphical user interface powered by **`egui 0.37` / `eframe`** with GPU hardware acceleration, paired with local ACID SQLite storage in Write-Ahead Logging (WAL) mode. The platform strictly enforces **Sovereign Privacy (Local-First)**:
- **100% Offline:** Zero network sockets, zero telemetry, zero background reporting, and zero external cloud connections.
- **Zero Cloud / AI Dependencies:** All forensic insights, quantitative risk scores (0–100), and 58 enriched metadata attributes are derived exclusively through deterministic mathematical algorithms.
- **Secure Local Storage:** Transactional persistence across isolated subdirectories under `%LOCALAPPDATA%\HuginnSentinel Sentinel` (`events\huginn_events.db` for audit events and `security\huginn_security.db` for baseline, incidents, and quarantine), guaranteeing ACID integrity even during unexpected power loss.
- **Ergonomic Startup:** The main window automatically launches centered on the user's primary monitor.

---

## 2. Graphical Workspace & Persistent Hover-Open Menus

The desktop interface of Huginn Sentinel is cleanly organized into high-efficiency components:

1. **Top Menu Bar with Persistent Hover-Open Behavior:**
   - Root menus: `File`, `Settings`, and `Options`.
   - **Persistent Hover-Open Behavior:** When opening any menu, cascaded submenu, context menu (right-click), or ComboBox dropdown, the menu tree **remains open** as long as the mouse pointer stays inside the menu area. Clicking options (such as toggling columns, pinning rows, or selecting modes) **does not close the menu automatically**, allowing rapid consecutive adjustments. The menu automatically closes the instant the mouse pointer leaves the menu tree boundary.
2. **Quick Toolbar:**
   - Single-click action buttons: Select Directory (`Ctrl+Shift+O`), Start/Stop Scan (`Ctrl+R`), Pause/Resume Monitoring (`Ctrl+P`), Save (`Ctrl+S`), Save As (`Ctrl+Shift+S`), Charts (`Ctrl+G`), and Clear (`Ctrl+L`). All buttons feature strict vertical alignment within a unified, justified border layout.
   - **Interactive Scanning & Monitoring Workflow:** Clicking "🔍 Scan" intelligently prompts whether to automatically start monitoring after scanning completes; clicking "▶️ Start Monitoring" asks if a preliminary scan should run first. Dedicated menus under `Settings > Scanning` and `Settings > Monitoring` allow configuring default behaviors and toggling confirmation prompts. The watcher is isolated during recursive scanning to strictly report "Scanned" events without noise.

### 2.1. Granular Data Cleanup System (`🗑️ Data Cleanup System`)
Activating the `Clear` toolbar button, shortcut `Ctrl+L`, or menus `File > 🗑️ Clear...` and `Security > 🛡️ Clear Security Database...` opens a native modeless dialog featuring dynamic Z-order that provides granular purging control:
- **🧹 Clear Table, Security & Events DB:** Full purge that simultaneously wipes in-memory grid events, the persisted events database, and security logs.
- **🗄️ Clear Database:** Exclusively purges the events relational database (`events\huginn_events.db`), recreating its schema, removing temporary WAL/SHM files, and issuing an atomic `VACUUM`.
- **📋 Clear UI Table:** Wipes only in-memory rows currently visible on the grid, keeping all recorded database events intact for future review.
- **🛡️ Clear Security Database:** Atomically resets the security database (`security\huginn_security.db`), resetting baselines and incidents.
- **❌ Cancel:** Closes the dialog with zero data modifications.

**Safety & Confirmation Workflow:**
- Selecting any cleanup target advances to a confirmation step (`⚠️ Confirm`) detailing the chosen target with explicit irreversibility warnings, a highlighted danger button, and a `↩️ Back` button.
- Clean success feedback views (`✅ Success`) verify task completion, and dedicated alert dialogs report any potential issues.
- The dialog features balanced symmetrical buttons sized to the longest localized label across all 6 languages, optical vertical and horizontal centering, lower padding, and auto-adapting window heights in Windows 11.
3. **Left Quick Filter Panel:**
   - Global instant search bar, operation filter checkboxes, extension quick-chips, threat risk rating selector, and live event counters.
   - **Dynamic Event Counter:** The "Total Events:" counter dynamically computes the exact sum of events for currently checked operation filters in real time, enabling precise sub-total analysis.
4. **4-Quadrant Virtualized 58-Column Audit Grid:**
   - High-performance central workspace capable of smoothly handling hundreds of thousands of rows with arbitrary 2D row/column pinning.
5. **Status Footer with Live Localized Modeless Window Counter:**
   - Displays monitored path, watcher status, scan progress bar, and dynamic open child window counters localized at runtime: **"1 Window:"**, **"2 Windows:"**, **"3 Windows:"**, etc., followed by quick focus/toggle buttons for each open modeless dialog.

---

## 3. Virtualized Spreadsheet & Arbitrary 2D Row/Column Pinning (Freeze Panes)

Huginn Sentinel features an advanced **4-quadrant** spreadsheet layout engine (`Top-Left`, `Top-Right`, `Bottom-Left`, and `Bottom-Right`), allowing users to freeze critical audit reference rows and columns while scrolling freely across massive datasets:

### 3.1. Row Pinning Capabilities (`Settings > 📌 Pin Rows`)
- **Pin 1st Row (Column Headers):** Freezes the header row at the top during vertical scrolling.
- **Pin Selected Row:** Freezes the currently focused event row in the top quadrant for side-by-side inspection.
- **Pin Specific Rows via Range Input:** Accepts row numbers (1-based) and ranges separated by commas and hyphens (e.g., `1, 3, 5-8`).
- **Checkbox Submenu:** Interactive list of rows allowing fine-grained toggling of pinned rows.
- **Unpin All Rows / Reset to Defaults:** Clears row pins or restores the recommended default (1st header row pinned).

### 3.2. Column Pinning Capabilities (`Settings > 📌 Pin Columns`)
- **Pin `#` Column (Row Numbers):** Keeps the row index column fixed on the left during horizontal scrolling.
- **Pin Selected Column:** Freezes the currently active column.
- **Checkbox Submenu for All Columns:** Pin any combination of forensic fields (e.g., `File Name`, `Operation Type`, `Risk Level`, `SHA-256`).
- **Pin All Visible Columns:** Instantly freezes every column currently visible on screen.
- **Unpin All Columns / Reset to Defaults:** Clears column pins or restores defaults (only `#` column pinned).

### 3.3. Context Menus & Preference Persistence
- Right-clicking any column header or cell presents instant pinning options: `📌 Pin Column`, `📌 Pin Row`, `Unpin Column`, and `Unpin Row`.
- All 2D freezing configurations are automatically saved and restored across sessions in `%LOCALAPPDATA%\HuginnSentinel\painel_fixo_config.json`.

### 3.4. Interactive Drag Selection (Cells, Columns, and Rows)
- **Continuous Cell Range Drag Selection:** Click and hold the left mouse button on any cell and drag across the spreadsheet. The rectangular cell selection highlights continuously in real time as the mouse pointer moves, finalizing the selected region upon release.
- **Multiple Column Header Drag Selection:** Click and hold on any column header and drag horizontally to select contiguous groups of columns with instant visual feedback.
- **Multiple Row Header Drag Selection:** Click and hold on any row index header (`#`) and drag vertically to select contiguous groups of rows with instant visual feedback.
- **Select All Button:** The corner button at the intersection of column and row headers instantly selects the entire spreadsheet while cleanly preserving all row and column divider borders.

### 3.5. Selection Behavior & Color Customization (`Settings > Selection Behavior`)
- **Static vs. Follow Selection Mode:** Allows the auditor to decide whether a selection of cells or rows stays anchored to its physical grid position or actively tracks data reordering and incoming records.
- **Granular Targets:** Independent switches to follow cell selections, follow row selections, and enable automatic scrolling into view.
- **Cell Selection Color Customization (`🎨 Customize Cell Selection Color`):**
  - **Recommended Palette Presets:** Instant presets (Excel Green, Corporate Blue, Cyan/Teal, Purple, Orange, Red, Slate Gray) featuring adaptive typographic contrast: selected button text dynamically flips to white in Windows 11 Dark Mode and black in Light Mode.
  - **Custom Color Picker:** Fine-tuning via 2D saturation/value gradient, 1D hue bar, and numeric R, G, B channels.
  - **Direct Editable HTML Code (`HTML: #00ff00`):** Type or paste any 3, 6, or 8-digit hexadecimal color string with immediate bi-directional synchronization.
  - **Comparison Spreadsheet Preview:** Side-by-side comparative preview of unselected vs. selected cells before applying changes.
  - **Safe Confirmation Dialogs:** Confirmation prompts when applying changes or resetting to the recommended default green.

### 3.6. Column Manager and Operation Colors Manager
- **Column Manager (`Settings > Configure Columns > Open Column Manager...`):** Streamlined single-column responsive list with batch actions (`Select All`, `Deselect All`) and default restoration.
- **Operation Colors Manager (`Settings > Configure Colors`):** Configure distinct colors for all 6 operations. The color palette dialog features direct editable HTML hex input, a quick close "❌" button, vertically centered "U8" and "📋" buttons with dynamic runtime-localized tooltips, and a centered "Reset Defaults" action.
- **Invert Sort Order Button:** The "🔄 Invert" toolbar button instantly toggles between ascending and descending sort order for the column selected by its header.

---

## 4. 58-Column Audit Grid & Deterministic Metadata Harmonization

The audit grid displays up to **58 metadata columns** backed by deterministic metadata extraction and harmonization (`harmonizacao.rs`), ensuring strict schema parity across all 6 tracked operations (`Added`, `Modified`, `Deleted`, `Renamed`, `Moved`, and `Scanned`):

- **Identification & Paths:** ID, Operation Type, File Name, Extension, Current Directory, Previous Directory, Full Current Path, Full Previous Path.
- **Exact Timestamps:** Created (UTC/Local), Modified (UTC/Local), Last Accessed (UTC/Local).
- **Physical Metrics:** Formatted Size (B, KB, MB, GB), Exact Byte Size, Size Difference (Diff Bytes).
- **Windows File Attributes:** Read-Only, Hidden, System, Archive, Compressed, Encrypted, Sparse, Temporary, Reparse Point.
- **Forensic Content & Magic Numbers:** Magic Numbers File Type, Canonical MIME Type, Shannon Entropy (0.00–8.00 bits/byte).
- **Integrity & Hashing:** SHA-256, SHA-1, MD5 checksums, and baseline comparison status.
- **Security & PE Headers:** Threat Risk Level (Critical, High, Medium, Low), Risk Score (0–100), Triggered Security Rule, Authenticode Digital Signature Status, PE Architecture (x86, x64, ARM64), and Windows Subsystem.

---

## 5. Canonical Security Rules Engine

Huginn Sentinel continuously evaluates events against 6 deterministic canonical rules (`rules.rs`):
- **`SEC-R01` (Ransomware Heuristics):** Detects high-frequency write bursts coupled with Shannon entropy spikes (encrypted content) and mass suspicious extension renames.
- **`SEC-R02` (Dangerous Double Extension):** Flags disguised executable extensions (e.g., `document.pdf.exe`, `invoice.docx.vbs`).
- **`SEC-R03` (Unsigned Executable in Temp/AppData):** Alerts when unsigned binaries or invalid Authenticode executables are dropped into user temporary folders.
- **`SEC-R04` (High-Risk Script Drop):** Monitors creation or editing of scripting assets (`.ps1`, `.bat`, `.cmd`, `.vbs`, `.js`) in protected paths.
- **`SEC-R05` (Hidden System Executable):** Detects binaries configured with hidden or system attributes.
- **`SEC-R06` (Critical Baseline Drift / SHA-256 Mismatch):** Instant alert when a live file's SHA-256 hash deviates from the saved baseline snapshot.

---

## 6. Safe Quarantine & Forensic Custody Sandbox

The quarantine subsystem safely neutralizes suspicious files:
- **Atomic Quarantine Isolation:** Files are renamed and moved to `%LOCALAPPDATA%\HuginnSentinel\quarantine\` with an inert `.quar` extension.
- **SQLite Custody Log:** Crucial forensic parameters (original path, event timestamp, incident ID, rule reason, original SHA-256) are preserved in the `quarentena` SQLite table.
- **Audited Restoration:** Restoring a file back to disk is strictly conditioned upon verifying the file's current SHA-256 hash against its recorded baseline hash.

---

## 7. 17 Native GPU-Accelerated Analytics Charts

Accessible via `Ctrl+G` or `Options > Statistics`, the analytics dialog delivers 17 native charts rendered on the GPU with smooth, responsive vector drawing:
1. Event Distribution by Operation Type (Pie and Bar — clean display without center dividers).
2. File Size Distribution Histograms (KB / MB).
3. 7x24 Temporal Activity Heatmap (Hourly vs. Day of Week with localized weekdays and scales).
4. Most Active File Extensions (with configurable quantity limiters).
5. Pareto 80/20 Extension Curve & Tukey File Size Boxplots.
6. Operations Timeline & Sankey Relational Flow Diagrams (`Operation → Type`, `Operation → Directory`, and `Type → Directory`).
7. Shannon Entropy vs. File Size Scatter Plots and Volumetric Dotplots.
8. Operational Radar and Multidimensional Risk Radar (with exact, independent reporting of `Added`, `Modified`, `Deleted`, `Renamed`, `Moved`, and `Scanned`).
9. Directory Tree Hierarchy and Hourly Event Time Series (with centered plot legends).

### 7.1. Sankey Diagram Interactive Controls
- **Operation Filter with Checkboxes:** In `Sankey: Operation → File Type` and `Sankey: Operation → Directory`, the operation dropdown offers interactive checkboxes for each event type (`Added`, `Modified`, `Deleted`, `Renamed`, `Moved`, `Scanned`), allowing users to isolate or combine specific flows.
- **Configurable Plotted Item Limits:** Define how many destination items (extensions or directories) are rendered (options: 10 to 100 in increments of 10, or All) with integrated scrollbars for dense datasets.
- **Deterministic Bézier Ribbon Stability:** Relationship ribbons employ strict alphabetical tie-breaking, eliminating jitter, trembling, or flickering during updates.

### 7.2. Windows 11 Light and Dark Theme Dynamic Adaptation
- All 17 charts dynamically adapt their background cards, borders, text contrast, grids, and axes in real time when toggling Windows 11 Light Mode and Dark Mode, with zero restart required.
- Statistics control bar dropdowns feature smooth hover-exit automatic collapsing.

- **High-DPI Export:** All 17 charts can be exported individually or in batch to crisp PNG files.

---

## 8. Audit Reports & Sealed Evidence Bundles

- **Available Report Formats:** `.csv` (UTF-8 BOM), `.json` (structured), `.html` (styled), `.txt` (plain table), and `.xml` (standardized schema).
- **Standardized Export File Naming:** Exported or saved reports automatically adopt the structured official prefix pattern `huginn_eventos_YYYYMMDD_HHMMSS` (e.g., `huginn_eventos_20261104_113331.csv`), ensuring clean identification and consistency across audit records and backups.
- **Cryptographic Sealed ZIP Evidence Export:** Packages audit event data, forensic metadata, SHA-256 custody hashes, and an integrity manifest into a single sealed ZIP container.

---

## 9. Headless CLI Utility (`huginnsentinel-cli.exe`)

The standalone command-line binary `huginnsentinel-cli.exe` powers automation scripts and CI/CD pipelines:

```powershell
# Help and version information
.\builds\huginnsentinel\release\huginnsentinel-cli.exe --help
.\builds\huginnsentinel\release\huginnsentinel-cli.exe --version

# Recursive directory security scan with JSON output
.\builds\huginnsentinel\release\huginnsentinel-cli.exe scan "C:\TargetData"

# Deep forensic metadata and hash extraction for a specific file
.\builds\huginnsentinel\release\huginnsentinel-cli.exe metadata "C:\TargetData\sample.exe"

# Compare or update cryptographic SHA-256 baseline
.\builds\huginnsentinel\release\huginnsentinel-cli.exe baseline "C:\TargetData"
.\builds\huginnsentinel\release\huginnsentinel-cli.exe baseline "C:\TargetData" --update

# Real-time console monitoring (optional --duration in seconds)
.\builds\huginnsentinel\release\huginnsentinel-cli.exe monitor "C:\TargetData" --duration 120

# Quarantine sandbox management
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine list
.\builds\huginnsentinel\release\huginnsentinel-cli.exe quarantine restore 1 "C:\Restored"
```

---

## 10. Keyboard Shortcuts & Productivity

- `F1`: Open Help dialog with all keyboard shortcuts.
- `Ctrl+M`: Open User Manual.
- `Ctrl+I`: Open About dialog.
- `Ctrl+Shift+O`: Browse and select root directory.
- `Ctrl+R`: Start/Stop recursive scanning.
- `Ctrl+P`: Pause/Resume real-time monitoring.
- `Ctrl+S`: Quick save report.
- `Ctrl+Shift+S`: Save As (choose name, folder, and format).
- `Ctrl+G`: Open Statistics & Charts (17 native charts).
- `Ctrl+L`: Open Data Cleanup System (granular purging of UI table, events database, or security database).
- `Ctrl+A`: Select and show all 58 columns.
- `Ctrl+Shift+A`: Colorize all columns.
- `Ctrl+O`: Toggle column header row sorting.
- `Ctrl+Q`: Safely exit the application.

</details>
