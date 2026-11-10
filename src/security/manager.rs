use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::attribution::AtribuicaoForense;
use super::baseline::BaselineCriptografico;
use super::diff::DiffConteudo;
use super::evidence::ExportacaoEvidencia;
use super::executable_intel::analisar_executavel;
use super::multi_directory::MultiDiretorio;
use super::quarantine::Quarentena;
use super::risk_score::ScoreRisco;
use super::sessions::CorrelacaoSessoes;
use crate::core::event::EventModel;
use crate::utils::paths::obter_caminho_db_seguranca;

pub struct SecurityManager {
    db_path: PathBuf,
    baseline: Arc<BaselineCriptografico>,
    quarentena: Arc<Quarentena>,
    multi_dir: Arc<MultiDiretorio>,
}

impl SecurityManager {
    pub fn new(db_path_opt: Option<PathBuf>) -> Self {
        let db_path = db_path_opt.unwrap_or_else(obter_caminho_db_seguranca);

        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        Self {
            db_path: db_path.clone(),
            baseline: Arc::new(BaselineCriptografico::new(db_path.clone())),
            quarentena: Arc::new(Quarentena::new(db_path.clone())),
            multi_dir: Arc::new(MultiDiretorio::new(db_path)),
        }
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn baseline(&self) -> Arc<BaselineCriptografico> {
        self.baseline.clone()
    }

    pub fn quarentena(&self) -> Arc<Quarentena> {
        self.quarentena.clone()
    }

    pub fn multi_dir(&self) -> Arc<MultiDiretorio> {
        self.multi_dir.clone()
    }

    pub fn configurar_diretorio_primario(&self, diretorio: &Path) {
        self.multi_dir.configurar_diretorio_primario(diretorio);
    }

    pub fn enriquecer_evento(&self, evento: &mut EventModel) {
        self.executar_pipeline_seguranca(evento, true);
    }

    pub fn enriquecer_evento_scan(&self, evento: &mut EventModel) {
        self.executar_pipeline_seguranca(evento, true);
    }

    fn executar_pipeline_seguranca(&self, evento: &mut EventModel, atualizar_baseline: bool) {
        let full_path = evento.full_path();
        let path = Path::new(&full_path);

        // 1. Perfil de Monitoramento Multi-Diretório (`security_monitoring_profile`)
        let (prof_id, prof_name, prof_dirs) =
            self.multi_dir.resolver_perfil_evento(&evento.dir_atual);
        evento.monitor_profile_id = prof_id;
        evento.monitor_profile_name = prof_name;
        evento.monitor_diretorios = prof_dirs;

        // 2. Inteligência de Executáveis, Scripts e Editor (`security_publisher_name`)
        let intel = analisar_executavel(path, &evento.autor);
        if evento.publisher.is_empty() {
            evento.publisher = intel.publisher;
        }
        evento.signature_state = intel.signature_state;
        evento.script_risk = intel.script_risk;

        // 3. Baseline Criptográfico e Captura de Snapshot (`security_baseline_status`)
        let hash_ref = if evento.sha256_atual.is_empty() {
            None
        } else {
            Some(evento.sha256_atual.as_str())
        };
        let (status, hash_anterior, texto_anterior, texto_atual) =
            self.baseline.verificar_com_snapshot(
                path,
                hash_ref,
                atualizar_baseline,
                &evento.publisher,
                &evento.signature_state,
            );
        evento.baseline_status = status.as_str().to_string();
        if let Some(prev) = hash_anterior {
            evento.sha256_baseline = prev;
        }

        // 4. Correlação de Sessão (`security_session_label`)
        CorrelacaoSessoes::atribuir_sessao_evento(evento);

        // 5. Resumo de Diff Textual (`security_diff_summary`)
        evento.diff_resumo = DiffConteudo::calcular_diff_evento(
            evento,
            texto_anterior.as_deref(),
            texto_atual.as_deref(),
        );

        // 6. Atribuição Forense (`security_actor_user`, `security_actor_process`)
        let (sys_user, process, pid) = AtribuicaoForense::capturar_contexto();
        evento.actor_user = if !evento.autor.is_empty() {
            evento.autor.clone()
        } else {
            sys_user
        };
        evento.actor_process = process;
        evento.actor_pid = pid;

        // 7. Motor de Regras e Score de Risco (`security_alert_summary`, `security_risk_score`, `security_risk_level`)
        ScoreRisco::avaliar_evento(evento);

        // 8. Status de Quarentena (`security_quarantine_status`)
        if evento.quarantine_status.is_empty() {
            evento.quarantine_status = "not_applicable".to_string();
        }

        // 9. Identificador e Manifesto de Evidência Forense (`security_evidence_id`)
        ExportacaoEvidencia::atribuir_evidencia_evento(evento);
    }

    pub fn exportar_evidencia(
        &self,
        eventos: &[EventModel],
        destino: Option<&Path>,
    ) -> Result<PathBuf, String> {
        ExportacaoEvidencia::exportar_pacote(eventos, destino)
    }

    pub fn limpar_banco_seguranca(&self) -> Result<(), String> {
        self.baseline.limpar_conexao_e_cache();
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Falha ao conectar com o banco de segurança: {}", e))?;

        conn.execute_batch(
            "PRAGMA busy_timeout = 5000;
             DELETE FROM baseline;
             DELETE FROM quarentena;
             DELETE FROM perfis_monitoramento;
             DELETE FROM sqlite_sequence WHERE name IN ('baseline', 'quarentena', 'perfis_monitoramento');
             PRAGMA wal_checkpoint(TRUNCATE);
             VACUUM;",
        )
        .map_err(|e| format!("Falha ao executar limpeza do banco de segurança: {}", e))?;

        drop(conn);

        let wal = self.db_path.with_extension("db-wal");
        let shm = self.db_path.with_extension("db-shm");
        let _ = std::fs::remove_file(wal);
        let _ = std::fs::remove_file(shm);

        Ok(())
    }
}
