use crate::client::RustbustersClient;
use common_utils::{HostEvent, PacketHeader, PacketTypeHeader};
use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NackType::Dropped;
use wg_2024::packet::{NackType, Packet};

impl RustbustersClient {
    /// Handles negative acknowledgments (NACKs) for message fragments.
    ///
    /// This function processes different types of NACKs:
    /// - Dropped: Attempts to resend the fragment with possibly a new route
    /// - ErrorInRouting: Updates topology when a drone is unreachable
    /// - Other types: Handles various routing and destination errors
    ///
    /// ### Arguments
    /// * `session_id` - The ID of the message session
    /// * `fragment_index` - The index of the fragment that failed
    /// * `nack_type` - The type of failure that occurred
    /// * `nack_header` - The source routing header of the NACK
    pub(crate) fn handle_nack(
        &mut self,
        session_id: u64,
        fragment_index: u64,
        nack_type: NackType,
        nack_header: &SourceRoutingHeader,
    ) {
        match self
            .pending_sent
            .get(&(session_id, fragment_index))
            .cloned()
        {
            Some(mut packet) => {
                if let Dropped = nack_type {
                    info!("Client {}: Resending fragment {}", self.id, fragment_index);

                    // Update stats for the dropping edge
                    self.update_edge_stats_on_nack(&nack_header.hops);

                    // Find a better path to reduce the probability of dropping the fragment
                    self.reroute_packet(&mut packet, fragment_index, nack_header);

                    // Resend the fragment
                    if let Some(sender) = self.packet_send.get(&packet.routing_header.hops[1]) {
                        if let Err(err) = sender.send(packet.clone()) {
                            warn!(
                                "Client {}: Unable to resend fragment {}: {}",
                                self.id, fragment_index, err
                            );
                        } else {
                            self.send_to_sc(HostEvent::PacketSent(PacketHeader {
                                session_id,
                                pack_type: PacketTypeHeader::MsgFragment,
                                routing_header: packet.routing_header.clone(),
                            }));
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
                            self.edge_stats
                                .retain(|(from, to), _| *from != drone && *to != drone);
                            self.known_nodes.lock().unwrap().remove(&drone);
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

    /// Attempts to find a better route for a packet that was dropped.
    ///
    /// This function is called when a packet needs to be rerouted due to
    /// repeated failures or poor link quality. It uses edge statistics
    /// to make routing decisions.
    ///
    /// ### Arguments
    /// * `packet` - The packet to be rerouted
    /// * `fragment_index` - The index of the fragment being rerouted
    /// * `nack_header` - The source routing header of the NACK
    fn reroute_packet(
        &mut self,
        packet: &mut Packet,
        fragment_index: u64,
        nack_header: &SourceRoutingHeader,
    ) {
        let drop_from = nack_header.hops[0];
        let drop_to = nack_header.hops[1];

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
