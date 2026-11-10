# Modelo de Pull Request / Pull Request Template

<p align="center">
  <b>Selecione o idioma / Select language:</b><br>
  <a href="#ptbr">🇧🇷 Português (BR)</a> | <a href="#enus">🇺🇸 English (US)</a>
</p>

> [!NOTE]
> **Aviso ao Colaborador / Contributor Notice:**  
> 🇧🇷 *Este repositório é um projeto pessoal e educacional mantido pelo autor em ritmo próprio de aprendizado. Pull Requests são bem-vindos, mas a análise técnica e incorporação de código ocorrem de forma voluntária e sem compromisso de prazo. Caso não haja manifestação ou merge imediato, sinta-se plenamente livre para manter, evoluir e distribuir suas melhorias em seu próprio fork sob os termos da licença **GNU AGPLv3**.*  
> 🇺🇸 *This repository is a personal and educational project maintained by the author at their own learning pace. Pull Requests are welcome, but technical review and merging are carried out voluntarily without any guaranteed timeframe. If there is no immediate response or merge, please feel free to maintain, evolve, and distribute your improvements in your own fork under the **GNU AGPLv3** license.*

---

## <a id="ptbr"></a>🇧🇷 Português (BR)

<details>
<summary>Clique para expandir / recolher o template em português</summary><br>

## Descrição
Descreva brevemente a finalidade deste Pull Request no **Huginn Sentinel** e qual problema ou melhoria ele entrega.

## Tipo de Alteração
- [ ] Correção de bug (alteração compatível que corrige uma falha)
- [ ] Nova funcionalidade na GUI (`egui` / `SpreadsheetGrid` / Diálogos Modeless / Gráficos)
- [ ] Melhoria no Motor de Monitoramento, Scanner ou Metadados (58 colunas)
- [ ] Melhoria no Motor de Segurança (`SEC-R01..R06`), Baseline SHA-256 ou Quarentena
- [ ] Melhoria no Utilitário CLI (`huginnsentinel-cli.exe`)
- [ ] Otimização de desempenho / Refatoração em Rust
- [ ] Atualização de documentação (`README.md` / `src/assets/`)
- [ ] Atualização de internacionalização / tradução (`src/language/*.json` nos 6 idiomas)

## Testes e Validação Realizados
Descreva as verificações executadas localmente antes de submeter o PR:
- [ ] Verificação estática sem erros (`cargo check --all-targets`)
- [ ] Suíte de testes automatizados 100% aprovada (`cargo test --no-fail-fast`)
- [ ] Binários de produção compilados em MSVC x64 (`cargo build --release --bin huginn --bin huginnsentinel-cli`)
- [ ] Testes manuais realizados no Windows 10 / Windows 11 / Windows Server:
  - Detalhes: 

## Conformidade Arquitetural, Internacionalização e Segurança
- [ ] Os módulos de núcleo (`src/core/`, `src/services/`, `src/security/`) permanecem desacoplados da camada visual (`egui`) para suportar tanto a GUI quanto o binário headless `huginnsentinel-cli.exe`.
- [ ] Todas as novas strings de interface foram adicionadas aos 6 dicionários (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT` e `de_DE` em `src/language/`) para tradução dinâmica em tempo de execução.
- [ ] Nenhuma conexão externa de rede, telemetria ou dependência de modelos de IA/LLM externos foi introduzida (100% Local-First / Offline).
- [ ] Operações de quarentena e restauração preservam a integridade de hashes SHA-256 e metadados `.meta` para reversão segura (rollback).
- [ ] As alterações respeitam integralmente os termos da licença **GNU Affero General Public License v3 (GNU AGPLv3)**.

## Issues Relacionadas
Corrige #(número da issue)

</details>

---

## <a id="enus"></a>🇺🇸 English (US)

<details>
<summary>Click to expand / collapse the template in English</summary><br>

## Description
Briefly describe the purpose of this Pull Request in **Huginn Sentinel** and what bug fix or enhancement it delivers.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New GUI feature (`egui` / `SpreadsheetGrid` / Modeless Dialogs / Charts)
- [ ] File Watcher, Recursive Scanner, or 58-Column Metadata Engine improvement
- [ ] Security Engine (`SEC-R01..R06`), SHA-256 Baseline, or Quarantine enhancement
- [ ] Headless CLI (`huginnsentinel-cli.exe`) / Automation improvement
- [ ] Rust performance optimization / Refactoring
- [ ] Documentation update (`README.md` / `src/assets/`)
- [ ] Localization / Translation update (`src/language/*.json` across all 6 languages)

## Testing & Verification Performed
Describe the checks you ran locally to verify your changes:
- [ ] Type & target check passed (`cargo check --all-targets`)
- [ ] Automated test suite passed (`cargo test --no-fail-fast`)
- [ ] Production binaries built on MSVC x64 (`cargo build --release --bin huginn --bin huginnsentinel-cli`)
- [ ] Manual testing on Windows 10 / Windows 11 / Windows Server:
  - Details: 

## Architectural, Localization & Security Compliance
- [ ] Core engine modules (`src/core/`, `src/services/`, `src/security/`) remain cleanly decoupled from GUI widgets so both `huginnsentinel.exe` and `huginnsentinel-cli.exe` build cleanly.
- [ ] Every new UI string includes translation keys in all 6 language files (`pt_BR`, `en_US`, `es_ES`, `fr_FR`, `it_IT`, and `de_DE` under `src/language/`) for dynamic runtime switching.
- [ ] Zero outbound network connections, telemetry, or external AI/LLM dependencies were introduced (100% Local-First / Offline).
- [ ] Quarantine and recovery operations preserve SHA-256 verification and `.meta` rollback integrity.
- [ ] All contributions comply with the **GNU Affero General Public License v3 (GNU AGPLv3)**.

## Related Issues
Fixes #(issue number)

</details>
