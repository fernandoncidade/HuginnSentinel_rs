use crate::core::event::EventModel;
use crate::core::types::OperationType;

#[derive(Debug, Clone)]
pub struct FiltroEstado {
    pub op_adicionado: bool,
    pub op_modificado: bool,
    pub op_excluido: bool,
    pub op_renomeado: bool,
    pub op_movido: bool,
    pub op_escaneado: bool,

    // Flags para ignorar filtro de data por operação (paridade total com C++ fil_01_Filtros / gfil_04_setup_ui)
    pub ignorar_data_mover: bool,
    pub ignorar_data_renomeados: bool,
    pub ignorar_data_adicionados: bool,
    pub ignorar_data_excluidos: bool,
    pub ignorar_data_modificados: bool,
    pub ignorar_data_escaneados: bool,

    pub texto_busca: String,
    pub filtro_extensao: String,
    pub filtro_risco: String, // "Todos", "Baixo", "Médio", "Alto", "Crítico"
    pub data_inicio: String,
    pub data_fim: String,
}

impl Default for FiltroEstado {
    fn default() -> Self {
        Self {
            op_adicionado: true,
            op_modificado: true,
            op_excluido: true,
            op_renomeado: true,
            op_movido: true,
            op_escaneado: true,
            ignorar_data_mover: true,
            ignorar_data_renomeados: true,
            ignorar_data_adicionados: true,
            ignorar_data_excluidos: true,
            ignorar_data_modificados: true,
            ignorar_data_escaneados: true,
            texto_busca: String::new(),
            filtro_extensao: String::new(),
            filtro_risco: "Todos".to_string(),
            data_inicio: String::new(),
            data_fim: String::new(),
        }
    }
}

impl FiltroEstado {
    pub fn resetar(&mut self) {
        *self = Self::default();
    }

    pub fn corresponde(&self, ev: &EventModel) -> bool {
        // 1. Filtro por operação
        let op_ok = match ev.op_type() {
            OperationType::Adicionado => self.op_adicionado,
            OperationType::Modificado => self.op_modificado,
            OperationType::Excluido => self.op_excluido,
            OperationType::Renomeado => self.op_renomeado,
            OperationType::Movido => self.op_movido,
            OperationType::Escaneado => self.op_escaneado,
        };
        if !op_ok {
            return false;
        }

        // 2. Filtro de texto de busca (nome, diretórios, tipo, sha256)
        if !self.texto_busca.is_empty() {
            let q = self.texto_busca.to_lowercase();
            let matches_name = ev.nome.to_lowercase().contains(&q);
            let matches_dir = ev.dir_atual.to_lowercase().contains(&q);
            let matches_prev = ev.dir_anterior.to_lowercase().contains(&q);
            let matches_type = ev.tipo.to_lowercase().contains(&q);
            let matches_hash = ev.sha256_atual.to_lowercase().contains(&q);

            if !matches_name && !matches_dir && !matches_prev && !matches_type && !matches_hash {
                return false;
            }
        }

        // 3. Filtro por extensão
        if !self.filtro_extensao.is_empty() {
            let exts: Vec<String> = self
                .filtro_extensao
                .split(',')
                .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            if !exts.is_empty() {
                let ev_ext = ev.nome.rsplit('.').next().unwrap_or("").to_lowercase();
                if !exts.contains(&ev_ext) {
                    return false;
                }
            }
        }

        // 4. Filtro por nível de risco
        if self.filtro_risco != "Todos" && !self.filtro_risco.is_empty() {
            if !ev.risk_level.eq_ignore_ascii_case(&self.filtro_risco) {
                return false;
            }
        }

        // 5. Filtro por data/hora (respeitando flags de ignorar por operação)
        let ignorar_data = match ev.op_type() {
            OperationType::Movido => self.ignorar_data_mover,
            OperationType::Renomeado => self.ignorar_data_renomeados,
            OperationType::Adicionado => self.ignorar_data_adicionados,
            OperationType::Excluido => self.ignorar_data_excluidos,
            OperationType::Modificado => self.ignorar_data_modificados,
            OperationType::Escaneado => self.ignorar_data_escaneados,
        };

        if !ignorar_data {
            if !self.data_inicio.is_empty() && ev.timestamp < self.data_inicio {
                return false;
            }
            if !self.data_fim.is_empty() && ev.timestamp > self.data_fim {
                return false;
            }
        }

        true
    }

    pub fn calcular_contadores(&self, eventos: &[EventModel]) -> ContadoresEventos {
        let mut c = ContadoresEventos::default();
        for ev in eventos {
            c.total += 1;
            match ev.op_type() {
                OperationType::Adicionado => c.adicionados += 1,
                OperationType::Modificado => c.modificados += 1,
                OperationType::Excluido => c.excluidos += 1,
                OperationType::Renomeado => c.renomeados += 1,
                OperationType::Movido => c.movidos += 1,
                OperationType::Escaneado => c.escaneados += 1,
            }
            if ev.risk_score >= 85 {
                c.criticos += 1;
            } else if ev.risk_score >= 60 {
                c.altos += 1;
            } else if ev.risk_score >= 30 {
                c.medios += 1;
            }
        }
        c
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContadoresEventos {
    pub total: usize,
    pub adicionados: usize,
    pub modificados: usize,
    pub excluidos: usize,
    pub renomeados: usize,
    pub movidos: usize,
    pub escaneados: usize,
    pub criticos: usize,
    pub altos: usize,
    pub medios: usize,
}

impl ContadoresEventos {
    /// Calcula a quantidade de eventos correspondentes às operações ativamente marcadas nas checkboxes.
    pub fn total_operacoes_ativas(&self, filtros: &FiltroEstado) -> usize {
        let mut total = 0;
        if filtros.op_adicionado {
            total += self.adicionados;
        }
        if filtros.op_modificado {
            total += self.modificados;
        }
        if filtros.op_excluido {
            total += self.excluidos;
        }
        if filtros.op_renomeado {
            total += self.renomeados;
        }
        if filtros.op_movido {
            total += self.movidos;
        }
        if filtros.op_escaneado {
            total += self.escaneados;
        }
        total
    }
}
