use crate::client::RustbustersClient;
use common_utils::{HostEvent, PacketHeader, PacketTypeHeader};
use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
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
        match self.pending_sent.get(&(session_id, fragment_index)).cloned() {
            Some(mut packet) => {
                match nack_type {
                    NackType::Dropped => {
                        info!("Client {}: Resending fragment {}", self.id, fragment_index);
                        self.update_edge_stats_on_nack(&nack_header.hops);
                        
                        // Check conditions for dropped packets
                        let drop_from = nack_header.hops[0];
                        let drop_to = nack_header.hops[1];
                        let should_reroute = if let Some(stats) = self.edge_stats.get(&(drop_from, drop_to)) {
                            stats.get_estimated_pdr() > 0.3 || stats.get_consecutive_nacks() >= 3
                        } else {
                            false
                        };
                        
                        self.reroute_and_resend(&mut packet, fragment_index, should_reroute);
                    }
                    NackType::ErrorInRouting(drone) => {
                        warn!(
                            "Client {}: Nack for fragment {} with type {:?}",
                            self.id, fragment_index, nack_type
                        );
                        self.topology.remove_node(drone);
                        self.edge_stats
                            .retain(|(from, to), _| *from != drone && *to != drone);
                        self.known_nodes.lock().unwrap().remove(&drone);
                        self.reroute_and_resend(&mut packet, fragment_index, true);
                    }
                    NackType::DestinationIsDrone | NackType::UnexpectedRecipient(_) => {
                        warn!(
                            "Client {}: Nack for fragment {} with type {:?}",
                            self.id, fragment_index, nack_type
                        );
                    }
                }
            }
            None => {
                warn!("Client {}: Nack for unknown fragment", self.id);
            }
        }
    }

    /// Attempts to reroute and resend a packet after a failure.
    ///
    /// This function handles both dropped packets and routing errors by:
    /// 1. Finding a new path to the destination
    /// 2. Updating the packet's routing header
    /// 3. Resending the packet through the new route
    ///
    /// ### Arguments
    /// * `packet` - The packet to be rerouted and resent
    /// * `fragment_index` - The index of the fragment
    /// * `force_reroute` - Whether to force rerouting
    fn reroute_and_resend(
        &mut self,
        packet: &mut Packet,
        fragment_index: u64,
        force_reroute: bool,
    ) {
        let destination = *packet.routing_header.hops.last().unwrap();
        
        if force_reroute {
            if let Some(new_path) = self.find_weighted_path(destination) {
                if new_path != packet.routing_header.hops {
                    info!(
                        "Client {}: Rerouting packet for session {} fragment {} from path {:?} to {:?}",
                        self.id, packet.session_id, fragment_index, packet.routing_header.hops, new_path
                    );

                    packet.routing_header.hops = new_path;
                    packet.routing_header.hop_index = 1;
                }
            }
        }

        // Attempt to resend the packet
        if let Some(sender) = self.packet_send.get(&packet.routing_header.hops[1]) {
            if let Err(err) = sender.send(packet.clone()) {
                warn!(
                    "Client {}: Unable to resend fragment {}: {}",
                    self.id, fragment_index, err
                );
            } else {
                self.send_to_sc(HostEvent::PacketSent(PacketHeader {
                    session_id: packet.session_id,
                    pack_type: PacketTypeHeader::MsgFragment,
                    routing_header: packet.routing_header.clone(),
                }));
            }
        }
    }
}
