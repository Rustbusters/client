use crate::RustbustersClient;
use petgraph::data::Build;
use wg_2024::network::NodeId;

/// Base weight for edges when no statistics are available
pub(crate) const BASE_WEIGHT: f32 = 1.0;

/// Statistics tracked for each edge in the network topology.
/// Used to compute dynamic edge weights based on network performance.
#[derive(Debug)]
pub(crate) struct EdgeStats {
    /// Total number of packets sent through this edge
    packets_sent: u64,
    /// Current Packet Drop Rate (PDR), updated using Exponential Moving Average
    current_pdr: f32,
    /// Confidence in the PDR measurement, increases with more packets sent
    confidence: f32,
    /// Learning rate for the EMA calculation, adapts based on network conditions
    alpha: f32,
    /// Number of consecutive NACK packets received
    consecutive_nacks: u32,
    /// Number of consecutive ACK packets received
    consecutive_acks: u32,
    /// Tracks if the last packet was a NACK
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

    /// Updates edge statistics based on packet transmission result.
    /// 
    /// # Arguments
    /// * `dropped` - Whether the packet was dropped (true) or successfully transmitted (false)
    ///
    /// Updates both instantaneous metrics (consecutive ACKs/NACKs) and long-term statistics (PDR).
    /// Adjusts the learning rate (alpha) based on network stability.
    pub(crate) fn update(&mut self, dropped: bool) {
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

        // Update alpha based on consecutive NACKs
        if self.consecutive_nacks >= 3 {
            // Increase alpha to react faster
            self.alpha = (self.alpha + 0.1).min(0.8);
        } else if self.consecutive_acks >= 5 {
            // Decrease alpha to stabilize
            self.alpha = (self.alpha - 0.05).max(0.2);
        }

        let new_value = if dropped { 1.0 } else { 0.0 };
        // EMA = α * current_value + (1 - α) * old_EMA
        self.current_pdr = self.alpha * new_value + (1.0 - self.alpha) * self.current_pdr;
        self.confidence = 1.0 / (1.0 + (-0.1 * self.packets_sent as f32).exp());
    }

    /// Calculates the edge weight based on current statistics.
    /// 
    /// Returns a weight value that reflects:
    /// - Base weight for the edge
    /// - Current PDR weighted by confidence
    /// - Additional penalty for consecutive failures
    /// 
    /// Higher weights indicate worse performance/reliability.
    pub(crate) fn get_edge_weight(&self) -> f32 {
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
    /// Retrieves or creates edge statistics for a given network edge.
    /// 
    /// # Arguments
    /// * `from` - Source node ID
    /// * `to` - Destination node ID
    ///
    /// Creates new statistics with default alpha if none exist.
    pub(crate) fn get_or_create_edge_stats(&mut self, from: NodeId, to: NodeId) -> &mut EdgeStats {
        self.edge_stats
            .entry((from, to))
            .or_insert_with(|| EdgeStats::new(0.2)) // Default alpha = 0.2
    }

    /// Updates edge statistics when a NACK is received.
    /// 
    /// # Arguments
    /// * `nack_path` - Path of nodes from where the packet was dropped back to source
    ///
    /// Penalizes the edge where the drop occurred and registers successful transmission
    /// for the rest of the path.
    pub(crate) fn update_edge_stats_on_nack(&mut self, nack_path: &[NodeId]) {
        if nack_path.len() < 2 {
            return;
        }

        // First edge in path is where the drop occurred
        // nack_path[0] is the drone that dropped
        // nack_path[1] is the previous node
        let dropped_from = nack_path[0];
        let dropped_to = nack_path[1];

        // Penalize the edge where the drop occurred
        let stats = self.get_or_create_edge_stats(dropped_from, dropped_to);
        stats.update(true);
        let weight = stats.get_edge_weight();
        self.topology.update_edge(dropped_from, dropped_to, weight);

        self.register_successful_transmission(&nack_path[1..]);
    }

    pub(crate) fn register_successful_transmission(&mut self, path: &[NodeId]) {
        // Update statistics for all edges used successfully
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];

            let stats = self.get_or_create_edge_stats(from, to);
            stats.update(false);

            // Update weight in graph
            let weight = stats.get_edge_weight();
            self.topology.update_edge(from, to, weight);
        }
    }
}
