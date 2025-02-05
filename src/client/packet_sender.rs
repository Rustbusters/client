use crate::client::RustbustersClient;
use common_utils::{HostEvent, HostMessage, PacketHeader, PacketTypeHeader, ServerToClientMessage};
use crossbeam_channel::Sender;
use log::{debug, info, warn};
use std::time::Instant;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Packet, PacketType};

impl RustbustersClient {
    /// Sends a message to a specific destination node using source routing
    ///
    /// ### Arguments
    /// * `destination_id` - The ID of the destination node
    /// * `message` - The message to be sent
    /// * `ws_to_ui_sender` - Channel sender for sending error messages back to the UI
    pub(crate) fn send_message(
        &mut self,
        destination_id: NodeId,
        message: HostMessage,
        ws_to_ui_sender: &Sender<(NodeId, ServerToClientMessage)>,
    ) {
        // Compute the route to the destination
        if let Some(route) = self.find_weighted_path(destination_id) {
            println!("Route: {route:?}",);
            // Increment session_id_counter
            self.session_id_counter += 1;
            let session_id = self.session_id_counter;

            let fragments = self.disassemble_message(&message);

            // Send the fragments along the route
            for fragment in fragments {
                debug!(
                    "Client {}: Sending fragment {:?} of session {} to {}",
                    self.id, fragment, session_id, destination_id
                );
                let fragment_index = fragment.fragment_index;
                let packet = Packet {
                    pack_type: PacketType::MsgFragment(fragment),
                    routing_header: SourceRoutingHeader {
                        hop_index: 1,
                        hops: route.clone(),
                    },
                    session_id,
                };

                // Store the time the message was sent
                self.pending_session_info.insert(
                    session_id,
                    (destination_id, message.clone(), Instant::now()),
                );

                // Send the packet to the first hop
                let next_hop = packet.routing_header.hops[1];
                if let Some(sender) = self.packet_send.get(&next_hop) {
                    if let Err(e) = sender.send(packet.clone()) {
                        warn!(
                            "Client {}: Failed to send packet to {}: {:?}",
                            self.id, next_hop, e
                        );
                        let error_msg = ServerToClientMessage::SendingError {
                            error: "Failed to send message! Retry in a few seconds".to_string(),
                            message: match message.clone() {
                                HostMessage::FromClient(client_msg) => client_msg,
                                _ => unreachable!("Client {}: Invalid message type", self.id),
                            },
                        };

                        // set to 0 because server_id is not relevant
                        if ws_to_ui_sender.send((0, error_msg)).is_err() {
                            warn!("Client {}: Unable to send error message to UI", self.id);
                        }
                    } else {
                        self.send_to_sc(HostEvent::PacketSent(PacketHeader {
                            session_id,
                            pack_type: PacketTypeHeader::MsgFragment,
                            routing_header: packet.routing_header.clone(),
                        }));
                        self.pending_sent
                            .entry((session_id, fragment_index))
                            .or_insert(packet);
                        info!(
                            "Client {}: Sent PacketSent event for session {} fragment {}",
                            self.id, session_id, fragment_index
                        );
                    }
                }
            }

            info!(
                "Client {}: Sent message to {} via route {:?}",
                self.id, destination_id, route
            );
        } else {
            info!("Client {}: No route to {}", self.id, destination_id);
            let error_msg = ServerToClientMessage::SendingError {
                error: "Destination unreachable! Retry in a few seconds".to_string(),
                message: match message {
                    HostMessage::FromClient(client_msg) => client_msg,
                    _ => return,
                },
            };

            // set to 0 because server_id is not relevant
            if ws_to_ui_sender.send((0, error_msg)).is_err() {
                warn!("Client {}: Unable to send error message to UI", self.id);
            }
        }
    }
}
