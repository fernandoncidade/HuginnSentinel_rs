use egui::{RichText, Slider};
use serde::{Deserialize, Serialize};

use crate::locale::Localizer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPopupKind {
    Alerts,
    Sessions,
    Quarantine,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPopupConfig {
    pub enabled: bool,
    pub threshold_count: i32,
    pub max_lines: i32,
    pub cooldown_seconds: i32,
}

impl Default for SecurityPopupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_count: 5,
            max_lines: 4,
            cooldown_seconds: 10,
        }
    }
}

pub struct NotificationPopupDialog {
    pub kind: NotificationPopupKind,
    pub aberto: bool,
    pub elevar_z_order: bool,
    pub maximizado: bool,
    pub minimizado: bool,
    pub config: SecurityPopupConfig,
}

impl NotificationPopupDialog {
    pub fn new(kind: NotificationPopupKind) -> Self {
        Self {
            kind,
            aberto: false,
            elevar_z_order: false,
            maximizado: false,
            minimizado: false,
            config: SecurityPopupConfig::default(),
        }
    }

    pub fn abrir(&mut self) {
        self.aberto = true;
        self.minimizado = false;
        self.elevar_z_order = true;
    }
}

pub struct NotificationConfigDialog {
    // Paridade C++: Pop-ups de segurança granulares
    pub alerts_popup: NotificationPopupDialog,
    pub sessions_popup: NotificationPopupDialog,
    pub quarantine_popup: NotificationPopupDialog,
    pub summary_popup: NotificationPopupDialog,

    // Compatibilidade com configurações legadas
    pub threshold_score: i32,
    pub cooldown_segundos: u32,
    pub emitir_som: bool,
}

impl Default for NotificationConfigDialog {
    fn default() -> Self {
        let mut d = Self {
            alerts_popup: NotificationPopupDialog::new(NotificationPopupKind::Alerts),
            sessions_popup: NotificationPopupDialog::new(NotificationPopupKind::Sessions),
            quarantine_popup: NotificationPopupDialog::new(NotificationPopupKind::Quarantine),
            summary_popup: NotificationPopupDialog::new(NotificationPopupKind::Summary),
            threshold_score: 60,
            cooldown_segundos: 10,
            emitir_som: true,
        };
        d.carregar();
        d
    }
}

impl NotificationConfigDialog {
    pub fn abrir(&mut self) {
        self.abrir_alerts();
    }

    pub fn abrir_alerts(&mut self) {
        self.alerts_popup.abrir();
    }

    pub fn abrir_sessions(&mut self) {
        self.sessions_popup.abrir();
    }

    pub fn abrir_quarantine(&mut self) {
        self.quarantine_popup.abrir();
    }

    pub fn abrir_summary(&mut self) {
        self.summary_popup.abrir();
    }

    pub fn salvar(&self) {
        let base = crate::utils::obter_caminho_persistente();
        let payload = serde_json::json!({
            "alerts": self.alerts_popup.config,
            "sessions": self.sessions_popup.config,
            "quarantine": self.quarantine_popup.config,
            "summary": self.summary_popup.config,
            "threshold_score": self.threshold_score,
            "cooldown_segundos": self.cooldown_segundos,
            "emitir_som": self.emitir_som,
        });
        if let Ok(s) = serde_json::to_string_pretty(&payload) {
            let _ = std::fs::write(base.join("notifications_config.json"), s);
        }
    }

    pub fn carregar(&mut self) {
        let base = crate::utils::obter_caminho_persistente();
        let path = base.join("notifications_config.json");
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if let Some(c) = val.get("alerts").and_then(|v| serde_json::from_value::<SecurityPopupConfig>(v.clone()).ok()) {
                    self.alerts_popup.config = c;
                }
                if let Some(c) = val.get("sessions").and_then(|v| serde_json::from_value::<SecurityPopupConfig>(v.clone()).ok()) {
                    self.sessions_popup.config = c;
                }
                if let Some(c) = val.get("quarantine").and_then(|v| serde_json::from_value::<SecurityPopupConfig>(v.clone()).ok()) {
                    self.quarantine_popup.config = c;
                }
                if let Some(c) = val.get("summary").and_then(|v| serde_json::from_value::<SecurityPopupConfig>(v.clone()).ok()) {
                    self.summary_popup.config = c;
                }
                if let Some(s) = val.get("threshold_score").and_then(|v| v.as_i64()) {
                    self.threshold_score = s as i32;
                }
                if let Some(s) = val.get("cooldown_segundos").and_then(|v| v.as_u64()) {
                    self.cooldown_segundos = s as u32;
                }
                if let Some(b) = val.get("emitir_som").and_then(|v| v.as_bool()) {
                    self.emitir_som = b;
                }
            }
        }
    }

    pub fn mostrar(&mut self, ctx: &egui::Context, loc: &Localizer) {
        let (e1, t1, m1, c1) = (
            self.alerts_popup.config.enabled,
            self.alerts_popup.config.threshold_count,
            self.alerts_popup.config.max_lines,
            self.alerts_popup.config.cooldown_seconds,
        );
        let (e2, t2, m2, c2) = (
            self.sessions_popup.config.enabled,
            self.sessions_popup.config.threshold_count,
            self.sessions_popup.config.max_lines,
            self.sessions_popup.config.cooldown_seconds,
        );
        let (e3, t3, m3, c3) = (
            self.quarantine_popup.config.enabled,
            self.quarantine_popup.config.threshold_count,
            self.quarantine_popup.config.max_lines,
            self.quarantine_popup.config.cooldown_seconds,
        );
        let e4 = self.summary_popup.config.enabled;

        // 1. Pop-up de Alertas (Alerts Popup)
        Self::mostrar_popup_dialog(
            ctx,
            loc,
            &mut self.alerts_popup,
            "dialog_notify_alerts_popup",
            "security_alert_popup_notifications",
            "security_popup_unit_alerts",
        );

        // 2. Pop-up de Sessões (Sessions Popup)
        Self::mostrar_popup_dialog(
            ctx,
            loc,
            &mut self.sessions_popup,
            "dialog_notify_sessions_popup",
            "security_session_popup_notifications",
            "security_popup_unit_sessions",
        );

        // 3. Pop-up de Quarentena (Quarantine Popup)
        Self::mostrar_popup_dialog(
            ctx,
            loc,
            &mut self.quarantine_popup,
            "dialog_notify_quarantine_popup",
            "security_quarantine_popup_notifications",
            "security_popup_unit_quarantine",
        );

        // 4. Notificações de Resumo (Summary Notifications)
        if self.summary_popup.aberto {
            let mut state = super::ModelessState {
                aberto: self.summary_popup.aberto,
                elevar_z_order: self.summary_popup.elevar_z_order,
                maximizado: self.summary_popup.maximizado,
                minimizado: self.summary_popup.minimizado,
            };

            let title = format!("📢 {}", loc.t("security_footer_notifications_notice_title"));
            super::show_modeless_dialog(
                ctx,
                "dialog_notify_summary",
                title,
                [430.0, 145.0],
                [360.0, 140.0],
                &mut state,
                |ui, fechar| {
                    ui.label(RichText::new(loc.t("security_footer_notifications_notice_text")).size(12.0));
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);

                    ui.checkbox(
                        &mut self.summary_popup.config.enabled,
                        RichText::new(loc.t("security_popup_config_enabled")).size(12.5),
                    );
                    ui.add_space(6.0);

                    ui.separator();
                    ui.add_space(6.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(2.0);
                        if ui
                            .add_sized(
                                [84.0, 26.0],
                                egui::Button::new(RichText::new(loc.t("ok")).size(12.0)),
                            )
                            .clicked()
                        {
                            *fechar = true;
                        }
                    });
                    ui.add_space(6.0);
                },
            );

            self.summary_popup.aberto = state.aberto;
            self.summary_popup.elevar_z_order = state.elevar_z_order;
            self.summary_popup.maximizado = state.maximizado;
            self.summary_popup.minimizado = state.minimizado;
        }

        if (e1, t1, m1, c1) != (
            self.alerts_popup.config.enabled,
            self.alerts_popup.config.threshold_count,
            self.alerts_popup.config.max_lines,
            self.alerts_popup.config.cooldown_seconds,
        ) || (e2, t2, m2, c2) != (
            self.sessions_popup.config.enabled,
            self.sessions_popup.config.threshold_count,
            self.sessions_popup.config.max_lines,
            self.sessions_popup.config.cooldown_seconds,
        ) || (e3, t3, m3, c3) != (
            self.quarantine_popup.config.enabled,
            self.quarantine_popup.config.threshold_count,
            self.quarantine_popup.config.max_lines,
            self.quarantine_popup.config.cooldown_seconds,
        ) || e4 != self.summary_popup.config.enabled
        {
            self.salvar();
        }
    }

    fn mostrar_popup_dialog(
        ctx: &egui::Context,
        loc: &Localizer,
        dlg: &mut NotificationPopupDialog,
        dialog_id: &'static str,
        label_key: &'static str,
        unit_key: &'static str,
    ) {
        if !dlg.aberto {
            return;
        }

        let mut state = super::ModelessState {
            aberto: dlg.aberto,
            elevar_z_order: dlg.elevar_z_order,
            maximizado: dlg.maximizado,
            minimizado: dlg.minimizado,
        };

        let label = loc.t(label_key);
        let title = format!(
            "🔔 {}",
            loc.t_args("security_popup_config_dialog_title", &[&label])
        );
        let unit_label = loc.t(unit_key);
        let cooldown_suffix = loc.t("security_popup_config_cooldown_suffix");

        super::show_modeless_dialog(
            ctx,
            dialog_id,
            title,
            [460.0, 245.0],
            [380.0, 235.0],
            &mut state,
            |ui, fechar| {
                ui.label(RichText::new(loc.t("security_popup_config_hint")).size(12.0));
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                ui.checkbox(
                    &mut dlg.config.enabled,
                    RichText::new(loc.t("security_popup_config_enabled")).size(12.5),
                );
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new(loc.t("security_popup_config_threshold")).size(12.0));
                    ui.add(
                        Slider::new(&mut dlg.config.threshold_count, 1..=1000)
                            .suffix(format!(" {}", unit_label)),
                    );
                });
                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new(loc.t("security_popup_config_max_lines")).size(12.0));
                    ui.add(Slider::new(&mut dlg.config.max_lines, 1..=20));
                });
                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new(loc.t("security_popup_config_cooldown")).size(12.0));
                    ui.add(
                        Slider::new(&mut dlg.config.cooldown_seconds, 0..=3600)
                            .suffix(format!(" {}", cooldown_suffix)),
                    );
                });
                ui.add_space(6.0);

                ui.separator();
                ui.add_space(6.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(2.0);
                    if ui
                        .add_sized(
                            [84.0, 26.0],
                            egui::Button::new(RichText::new(loc.t("ok")).size(12.0)),
                        )
                        .clicked()
                    {
                        *fechar = true;
                    }
                });
                ui.add_space(6.0);
            },
        );

        dlg.aberto = state.aberto;
        dlg.elevar_z_order = state.elevar_z_order;
        dlg.maximizado = state.maximizado;
        dlg.minimizado = state.minimizado;
    }
}
