use crate::client::RustbustersClient;
use common_utils::ServerToClientMessage;
use crossbeam_channel::Sender;
use log::info;
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

impl RustbustersClient {
    pub(crate) fn handle_packet(
        &mut self,
        packet: Packet,
        ui_sender: &Sender<(NodeId, ServerToClientMessage)>,
    ) {
        match packet.pack_type {
            PacketType::FloodRequest(flood_request) => {
                info!(
                    "Client {}: Received FloodRequest with flood_id {}",
                    self.id, flood_request.flood_id
                );
                self.handle_flood_request(&flood_request, packet.session_id);
            }
            PacketType::FloodResponse(flood_response) => {
                info!(
                    "Client {}: Received FloodResponse with flood_id {}",
                    self.id, flood_response.flood_id
                );
                self.handle_flood_response(&flood_response);
            }
            PacketType::MsgFragment(fragment) => {
                // Handle incoming message fragments
                info!(
                    "Client {}: Received fragment {} of session {}",
                    self.id, fragment.fragment_index, packet.session_id
                );
                self.handle_message_fragment(
                    &fragment,
                    packet.session_id,
                    &packet.routing_header,
                    ui_sender,
                );
            }
            PacketType::Ack(ack) => {
                // Handle Acknowledgments
                info!(
                    "Client {}: Received Ack for fragment {}",
                    self.id, ack.fragment_index
                );
                self.handle_ack(packet.session_id, ack.fragment_index);
            }
            PacketType::Nack(nack) => {
                // Handle Negative Acknowledgments
                info!("Client {}: Received Nack {nack:?}", self.id);
                self.handle_nack(packet.session_id, nack.fragment_index, nack.nack_type);
            }
        }
    }
}
