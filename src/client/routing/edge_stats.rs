use crate::RustbustersClient;
use petgraph::data::Build;
use wg_2024::network::NodeId;

pub(crate) const BASE_WEIGHT: f32 = 1.0;

#[derive(Debug)]
pub(crate) struct EdgeStats {
    packets_sent: u64,
    current_pdr: f32,
    confidence: f32,
    alpha: f32,
    consecutive_nacks: u32,
    consecutive_acks: u32,
    last_was_nack: bool,
}

impl EdgeStats {
    fn new(alpha: f32) -> Self {
        Self {
            packets_sent: 0,
            current_pdr: 0.0,
            confidence: 0.0,
            alpha,
            consecutive_nacks: 0,
            consecutive_acks: 0,
            last_was_nack: false,
        }
    }

    fn update(&mut self, dropped: bool) {
        self.packets_sent += 1;

        if dropped {
            if self.last_was_nack {
                self.consecutive_nacks += 1;
            } else {
                self.consecutive_nacks = 1;
                self.consecutive_acks = 0;
            }
            self.last_was_nack = true;
        } else {
            if self.last_was_nack {
                self.consecutive_acks = 1;
                self.consecutive_nacks = 0;
            } else {
                self.consecutive_acks += 1;
            }
            self.last_was_nack = false;
        }

        // Aggiorna alpha basandosi sui NACK consecutivi
        if self.consecutive_nacks >= 3 {
            // Aumenta alpha per reagire più velocemente
            self.alpha = (self.alpha + 0.1).min(0.8);
        } else if self.consecutive_acks >= 5 {
            // Diminuisci alpha per stabilizzare
            self.alpha = (self.alpha - 0.05).max(0.2);
        }

        let new_value = if dropped { 1.0 } else { 0.0 };
        // EMA = α * current_value + (1 - α) * old_EMA
        self.current_pdr = self.alpha * new_value + (1.0 - self.alpha) * self.current_pdr;
        self.confidence = 1.0 / (1.0 + (-0.1 * self.packets_sent as f32).exp());
    }

    fn get_edge_weight(&self) -> f32 {
        if self.packets_sent == 0 {
            return BASE_WEIGHT;
        }

        let consecutive_penalty = if self.consecutive_nacks > 2 {
            0.5 * (self.consecutive_nacks as f32 - 2.0)
        } else {
            0.0
        };

        BASE_WEIGHT * (1.0 + self.current_pdr * self.confidence + consecutive_penalty)
    }

    pub(crate) fn get_estimated_pdr(&self) -> f32 {
        self.current_pdr
    }

    pub(crate) fn get_consecutive_nacks(&self) -> u32 {
        self.consecutive_nacks
    }
}

impl RustbustersClient {
    fn get_or_create_edge_stats(&mut self, from: NodeId, to: NodeId) -> &mut EdgeStats {
        self.edge_stats
            .entry((from, to))
            .or_insert_with(|| EdgeStats::new(0.2)) // Alpha = 0.2 come valore di default
    }

    pub(crate) fn update_edge_stats_on_nack(&mut self, nack_path: &[NodeId]) {
        if nack_path.len() < 2 {
            return;
        }

        // Il primo arco nel path è quello dove è avvenuto il drop
        // nack_path[0] è il drone che ha droppato
        // nack_path[1] è il nodo precedente
        let dropped_from = nack_path[0];
        let dropped_to = nack_path[1];

        // Penalizza l'arco dove è avvenuto il drop
        let stats = self.get_or_create_edge_stats(dropped_from, dropped_to);
        stats.update(true);
        let weight = stats.get_edge_weight();
        self.topology.update_edge(dropped_from, dropped_to, weight);

        self.register_successful_transmission(&nack_path[1..]);
    }

    pub(crate) fn register_successful_transmission(&mut self, path: &[NodeId]) {
        // Aggiorna le statistiche per tutti gli archi usati con successo
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];

            let stats = self.get_or_create_edge_stats(from, to);
            stats.update(false);

            // Aggiorna il peso nel grafo
            let weight = stats.get_edge_weight();
            self.topology.update_edge(from, to, weight);
        }
    }
}
