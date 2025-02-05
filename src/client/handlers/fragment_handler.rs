use crate::client::RustbustersClient;
use common_utils::HostEvent::{ControllerShortcut, PacketSent};
use common_utils::HostMessage::FromServer;
use common_utils::{PacketHeader, PacketTypeHeader, ServerToClientMessage};
use crossbeam_channel::Sender;
use log::{info, warn};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Packet, PacketType};

impl RustbustersClient {
    /// Handles incoming message fragments and sends acknowledgments.
    ///
    /// This function processes received message fragments by:
    /// 1. Updating reception statistics
    /// 2. Attempting to reassemble the complete message if all fragments are received
    /// 3. Sending acknowledgment back to the source
    /// 4. Forwarding reassembled messages to the UI and controller if complete
    ///
    /// ### Arguments
    /// * `fragment` - The received message fragment
    /// * `session_id` - The ID of the message session
    /// * `source_routing_header` - Routing information for the response
    /// * `sender` - Channel to send messages to the UI
    pub(crate) fn handle_message_fragment(
        &mut self,
        fragment: &Fragment,
        session_id: u64,
        source_routing_header: &SourceRoutingHeader,
        sender: &Sender<(NodeId, ServerToClientMessage)>,
    ) {
        let source = *source_routing_header.hops.first().unwrap();

        // If after insert all fragments of the session are received, reassemble the message
        if self.set_pending(session_id, fragment.clone()) {
            match self.reassemble_fragments(session_id) {
                Ok(msg) => {
                    info!(
                        "Client {}: Received full message {:?} of session {}",
                        self.id, msg, session_id
                    );

                    if let FromServer(s2c_msg) = &msg {
                        if sender.send((source, s2c_msg.clone())).is_err() {
                            warn!("Client {}: Unable to send message to UI", self.id);
                        }
                    } else {
                        warn!(
                            "Client {}: Received message that is from another client",
                            self.id
                        );
                    }
                }
                Err(err) => {
                    warn!("Client {}: {}", self.id, err);
                }
            }
        }

        let fragment_index = fragment.fragment_index;

        // Send an Acknowledgment
        let ack = Ack { fragment_index };

        let ack_packet = Packet {
            pack_type: PacketType::Ack(ack),
            routing_header: SourceRoutingHeader {
                hop_index: 1,
                hops: source_routing_header
                    .hops
                    .iter()
                    .rev()
                    .copied()
                    .collect::<Vec<NodeId>>(),
            },
            session_id,
        };

        // Send the Acknowledgment back to the sender
        let next_hop = ack_packet.routing_header.hops[1];

        if let Some(sender) = self.packet_send.get(&next_hop) {
            if let Err(err) = sender.send(ack_packet.clone()) {
                warn!(
                    "Client {}: Error sending Ack for fragment {} to {}: {}",
                    self.id, fragment_index, next_hop, err
                );
                self.send_to_sc(ControllerShortcut(ack_packet.clone()));
                info!("Client {}: Sending ack through SC", self.id);
            } else {
                info!(
                    "Client {}: Sent Ack for fragment {} to {}",
                    self.id, fragment_index, next_hop
                );
            }
        } else {
            warn!(
                "Client {}: Cannot send Ack for fragment {} to {}",
                self.id, fragment_index, next_hop
            );
            self.send_to_sc(ControllerShortcut(ack_packet.clone()));
        }
        self.send_to_sc(PacketSent(PacketHeader {
            session_id,
            pack_type: PacketTypeHeader::Ack,
            routing_header: ack_packet.routing_header.clone(),
        }));
    }

    /// Stores a fragment in the pending received collection.
    ///
    /// ### Arguments
    /// * `session_id` - The ID of the message session
    /// * `fragment` - The fragment to store
    ///
    /// ### Returns
    /// `true` if all fragments for the session have been received, `false` otherwise
    fn set_pending(&mut self, session_id: u64, fragment: Fragment) -> bool {
        let fragment_index = fragment.fragment_index;
        let total_n_fragments = fragment.total_n_fragments;
        // insert the fragment in the pending_received map at index fragment_index
        let fragments = vec![None; fragment.total_n_fragments as usize];
        let entry = self
            .pending_received
            .entry(session_id)
            .or_insert((fragments, 0));
        entry.0[fragment_index as usize] = Some(fragment);
        entry.1 += 1;

        entry.1 == total_n_fragments
    }
}
