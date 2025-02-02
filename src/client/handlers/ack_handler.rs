use crate::client::RustbustersClient;
use log::{info, warn};

impl RustbustersClient {
    /// Processes acknowledgments for sent message fragments.
    ///
    /// This function:
    /// 1. Updates acknowledgment statistics
    /// 2. Removes acknowledged fragments from pending sent collection
    /// 3. Updates routing statistics for successful transmissions
    /// 4. Logs when all fragments of a session are acknowledged
    ///
    /// ### Arguments
    /// * `session_id` - The ID of the message session
    /// * `fragment_index` - The index of the acknowledged fragment
    pub(crate) fn handle_ack(&mut self, session_id: u64, fragment_index: u64) {
        // Update stats
        self.stats.inc_acks_received();

        // Remove the acked fragment from the pending_sent list
        let acked = self.pending_sent.remove(&(session_id, fragment_index));
        if let Some(packet) = acked {
            self.register_successful_transmission(&packet.routing_header.hops);
        } else {
            warn!(
                "Client {}: Ack for unknown fragment with index {} and session_id {}",
                self.id, fragment_index, session_id
            );
        }

        // Check if all fragments with key (session_id, _) have been acked
        if self
            .pending_sent
            .iter()
            .filter(|((key, _), _)| *key == session_id)
            .collect::<Vec<_>>()
            .is_empty()
        {
            info!(
                "Client {}: All fragments of session {} acked",
                self.id, session_id
            );
        }
    }
}
