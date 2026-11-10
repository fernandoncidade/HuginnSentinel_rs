use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::core::event::EventModel;
use crate::utils::logger::get_logger;
use crate::utils::paths::obter_caminho_persistente;

static EVIDENCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct ExportacaoEvidencia;

impl ExportacaoEvidencia {
    /// Gera identificador forense único, hash de custódia e manifesto compacto para o evento
    /// (paridade com `evid_01_exportador_evidencias.cpp`).
    pub fn atribuir_evidencia_evento(evento: &mut EventModel) {
        let seq = EVIDENCE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let compact_date: String = evento
            .timestamp
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(8)
            .collect();
        let date_part = if compact_date.is_empty() {
            chrono::Local::now().format("%Y%m%d").to_string()
        } else {
            compact_date
        };

        evento.evidence_id = format!("EV-{}-{:05}", date_part, seq);

        let mut hasher = Sha256::new();
        hasher.update(evento.evidence_id.as_bytes());
        hasher.update(evento.tipo_operacao.as_str().as_bytes());
        hasher.update(evento.nome.as_bytes());
        hasher.update(evento.dir_atual.as_bytes());
        hasher.update(evento.sha256_atual.as_bytes());
        hasher.update(evento.risk_score.to_le_bytes());

        evento.evidence_hash = hex::encode(hasher.finalize());
        evento.evidence_manifest = format!(
            "id={};op={};sha256={};risk={}",
            evento.evidence_id,
            evento.tipo_operacao.as_str(),
            if evento.sha256_atual.is_empty() {
                "none"
            } else {
                &evento.sha256_atual
            },
            evento.risk_score
        );
    }

    pub fn exportar_pacote(
        eventos: &[EventModel],
        destino_zip_opt: Option<&Path>,
    ) -> Result<PathBuf, String> {
        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let zip_path = if let Some(p) = destino_zip_opt {
            p.to_path_buf()
        } else {
            obter_caminho_persistente().join(format!("evidencia_huginn_{}.zip", ts))
        };

        let file = File::create(&zip_path)
            .map_err(|e| format!("Falha ao criar arquivo de evidência: {}", e))?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let json_data = serde_json::to_string_pretty(eventos)
            .map_err(|e| format!("Falha ao serializar eventos: {}", e))?;
        let _ = zip.start_file("eventos_auditoria.json", options);
        let _ = zip.write_all(json_data.as_bytes());

        let mut manifest_lines = Vec::new();
        manifest_lines
            .push("=================================================================".to_string());
        manifest_lines
            .push("  HUGINN — MANIFESTO CRIPTOGRÁFICO DE CUSTÓDIA FORENSE ".to_string());
        manifest_lines
            .push("=================================================================".to_string());
        manifest_lines.push(format!("Gerado em: {}", chrono::Local::now().to_rfc3339()));
        manifest_lines.push(format!("Total de eventos auditados: {}", eventos.len()));
        manifest_lines
            .push("-----------------------------------------------------------------".to_string());

        let mut hasher_global = Sha256::new();
        hasher_global.update(json_data.as_bytes());

        for (idx, ev) in eventos.iter().enumerate() {
            let line = format!(
                "[{}] [{}] {} (SHA-256: {}) - Risco: {} (Score: {})",
                idx + 1,
                ev.tipo_operacao,
                ev.full_path(),
                if ev.sha256_atual.is_empty() {
                    "N/A"
                } else {
                    &ev.sha256_atual
                },
                ev.risk_level,
                ev.risk_score
            );
            manifest_lines.push(line);
            hasher_global.update(ev.full_path().as_bytes());
            hasher_global.update(ev.sha256_atual.as_bytes());
        }

        let selo_custodia = hex::encode(hasher_global.finalize());
        manifest_lines
            .push("-----------------------------------------------------------------".to_string());
        manifest_lines.push(format!(
            "SELO DE INTEGRIDADE GLOBAL (SHA-256): {}",
            selo_custodia
        ));
        manifest_lines
            .push("=================================================================".to_string());

        let manifest_content = manifest_lines.join("\r\n");
        let _ = zip.start_file("MANIFESTO_CUSTODIA.txt", options);
        let _ = zip.write_all(manifest_content.as_bytes());

        zip.finish()
            .map_err(|e| format!("Falha ao finalizar pacote ZIP de evidência: {}", e))?;

        get_logger().info(&format!(
            "Pacote de evidência gerado com sucesso: {:?}",
            zip_path
        ));

        Ok(zip_path)
    }
}
