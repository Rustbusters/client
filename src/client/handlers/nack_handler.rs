use crate::client::RustbustersClient;
use log::{info, warn};
use wg_2024::packet::NackType;
use wg_2024::packet::NackType::Dropped;

impl RustbustersClient {
    pub(crate) fn handle_nack(
        &mut self,
        session_id: u64,
        fragment_index: u64,
        nack_type: NackType,
    ) {
        // Update stats
        self.stats.inc_nacks_received();

        match self
            .pending_sent
            .get(&(session_id, fragment_index))
            .cloned()
        {
            Some(packet) => {
                if let Dropped = nack_type {
                    info!("Client {}: Resending fragment {}", self.id, fragment_index);
                    self.update_edge_stats(&packet.routing_header.hops.clone());

                    if let Some(sender) = self.packet_send.get(&packet.routing_header.hops[1]) {
                        if let Err(err) = sender.send(packet.clone()) {
                            warn!(
                                "Client {}: Unable to resend fragment {}: {}",
                                self.id, fragment_index, err
                            );
                        } else {
                            self.stats.inc_fragments_sent();
                        }
                    }
                } else {
                    match nack_type {
                        NackType::ErrorInRouting(drone) => {
                            warn!(
                                "Client {}: Nack for fragment {} with type {:?}",
                                self.id, fragment_index, nack_type
                            );
                            self.topology.remove_node(drone);
                            self.discover_network();
                        }
                        NackType::DestinationIsDrone | NackType::UnexpectedRecipient(_) => {
                            warn!(
                                "Client {}: Nack for fragment {} with type {:?}",
                                self.id, fragment_index, nack_type
                            );
                        }
                        _ => {
                            unreachable!("Unexpected nack type");
                        }
                    }
                }
            }
            None => {
                warn!("Client {}: Nack for unknown fragment", self.id);
            }
        }
    }
}
