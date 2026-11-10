use egui::{Color32, RichText, TextEdit, Ui, Vec2};

use super::app::HuginnApp;
use crate::core::types::OperationType;

pub fn renderizar_painel_filtros(app: &mut HuginnApp, ui: &mut Ui) {
    ui.heading(app.loc.t("event_filters"));
    ui.add_space(6.0);

    let mut filtros_alterados = false;

    // Campo de busca
    ui.label(RichText::new(app.loc.t("quick_search")).strong());
    let hint_busca = app.loc.t("search_placeholder");
    if ui
        .add(TextEdit::singleline(&mut app.filtros.texto_busca).hint_text(hint_busca))
        .changed()
    {
        filtros_alterados = true;
    }
    ui.add_space(8.0);

    // Contadores em cache (não recalcula a cada quadro)
    let contadores = &app.contadores_cache;

    ui.separator();
    let total_operacoes = contadores.total_operacoes_ativas(&app.filtros);
    let total_str = total_operacoes.to_string();
    ui.label(RichText::new(app.loc.t_args("total_events", &[&total_str])).strong());
    ui.add_space(4.0);

    // Checkboxes das Operações com chips coloridos
    ui.label(RichText::new(app.loc.t("operations")).strong());

    let mut ops = [
        (
            OperationType::Adicionado,
            app.filtros.op_adicionado,
            contadores.adicionados,
        ),
        (
            OperationType::Modificado,
            app.filtros.op_modificado,
            contadores.modificados,
        ),
        (
            OperationType::Excluido,
            app.filtros.op_excluido,
            contadores.excluidos,
        ),
        (
            OperationType::Renomeado,
            app.filtros.op_renomeado,
            contadores.renomeados,
        ),
        (
            OperationType::Movido,
            app.filtros.op_movido,
            contadores.movidos,
        ),
        (
            OperationType::Escaneado,
            app.filtros.op_escaneado,
            contadores.escaneados,
        ),
    ];

    for (op, checked, count) in &mut ops {
        let (bg, _fg) = app.gerenciador_tabela.obter_cores_operacao(*op);
        let nome_op = app.loc.traduzir_tipo_operacao(op.as_str());

        ui.horizontal(|ui| {
            if ui.checkbox(checked, "").changed() {
                filtros_alterados = true;
            }
            let (rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 3_u8, bg);

            ui.label(format!("{} ({})", nome_op, count));
        });
    }

    if filtros_alterados {
        app.filtros.op_adicionado = ops[0].1;
        app.filtros.op_modificado = ops[1].1;
        app.filtros.op_excluido = ops[2].1;
        app.filtros.op_renomeado = ops[3].1;
        app.filtros.op_movido = ops[4].1;
        app.filtros.op_escaneado = ops[5].1;
    }

    ui.add_space(8.0);
    ui.separator();

    // Filtro de Nível de Risco
    ui.label(RichText::new(app.loc.t("risk_level")).strong());
    let niveis = ["Todos", "Crítico", "Alto", "Médio", "Baixo"];
    let selected_risk_display = app.loc.traduzir_risco(&app.filtros.filtro_risco);
    let combo = egui::ComboBox::from_id_salt("filtro_risco_combo")
        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
        .selected_text(selected_risk_display)
        .show_ui(ui, |ui| {
            let mut mudou = false;
            for &n in &niveis {
                let rotulo = app.loc.traduzir_risco(n);
                if ui
                    .selectable_value(&mut app.filtros.filtro_risco, n.to_string(), rotulo)
                    .changed()
                {
                    mudou = true;
                }
            }
            mudou
        });
    crate::ui::widgets::combo_persistente_registrar(ui.ctx(), &combo.response);

    if combo.inner == Some(true) {
        filtros_alterados = true;
    }

    if contadores.criticos > 0 {
        let txt = format!("⚠️ {} {}", contadores.criticos, app.loc.t("risk_critical"));
        ui.colored_label(Color32::from_rgb(244, 67, 54), txt);
    }
    if contadores.altos > 0 {
        let txt = format!("🔸 {} {}", contadores.altos, app.loc.t("risk_high"));
        ui.colored_label(Color32::from_rgb(255, 152, 0), txt);
    }

    ui.add_space(8.0);
    ui.separator();

    // Filtro por Extensão
    ui.label(RichText::new(app.loc.t("extensions")).strong());
    let hint_ext = app.loc.t("extensions_hint");
    if ui
        .add(TextEdit::singleline(&mut app.filtros.filtro_extensao).hint_text(hint_ext))
        .changed()
    {
        filtros_alterados = true;
    }

    ui.add_space(8.0);
    ui.separator();

    // Filtro por Data Inicial / Final
    ui.label(RichText::new(app.loc.t("start_date")).strong());
    let hint_data = app.loc.t("date_format_hint");
    if ui
        .add(TextEdit::singleline(&mut app.filtros.data_inicio).hint_text(&hint_data))
        .changed()
    {
        filtros_alterados = true;
    }

    ui.label(RichText::new(app.loc.t("end_date")).strong());
    if ui
        .add(TextEdit::singleline(&mut app.filtros.data_fim).hint_text(&hint_data))
        .changed()
    {
        filtros_alterados = true;
    }

    ui.add_space(8.0);
    if ui
        .button(format!("🔍 {}", app.loc.t("advanced_filters")))
        .clicked()
    {
        app.abrir_dialogo_filtros_avancados();
    }

    ui.add_space(8.0);

    if ui.button(app.loc.t("reset_filters")).clicked() {
        app.filtros.resetar();
        filtros_alterados = true;
    }

    if filtros_alterados {
        app.filtros_sujos = true;
    }
}
