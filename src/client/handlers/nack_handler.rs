use crate::client::RustbustersClient;
use log::{info, warn};
use wg_2024::packet::NackType::Dropped;
use wg_2024::packet::{NackType, Packet};

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
            Some(mut packet) => {
                if let Dropped = nack_type {
                    info!("Client {}: Resending fragment {}", self.id, fragment_index);

                    // Update stats for the dropping edge
                    self.update_edge_stats_on_nack(&packet.routing_header.hops.clone());

                    // Find a better path to reduce the probability of dropping the fragment
                    self.reroute_packet(&mut packet, fragment_index);

                    // Resend the fragment
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

    fn reroute_packet(&mut self, packet: &mut Packet, fragment_index: u64) {
        let drop_from = packet.routing_header.hops[0];
        let drop_to = packet.routing_header.hops[1];

        if let Some(stats) = self.edge_stats.get_mut(&(drop_from, drop_to)) {
            // recompute path if the estimated PDR is above 0.3
            if stats.get_estimated_pdr() > 0.3 || stats.get_consecutive_nacks() >= 3 {
                if let Some(new_path) =
                    self.find_weighted_path(*packet.routing_header.hops.last().unwrap())
                {
                    // If there is a better path, reroute the packet
                    if new_path != packet.routing_header.hops {
                        info!(
                            "Client {}: Rerouting packet for session {} fragment {} from path {:?} to {:?}",
                            self.id, packet.session_id, fragment_index, packet.routing_header.hops, new_path
                        );

                        // Update the packet's path
                        packet.routing_header.hops = new_path;
                        packet.routing_header.hop_index = 1;
                    }
                }
            }
        }
    }
}
