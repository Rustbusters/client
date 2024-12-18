use crate::client::RustbustersClient;
use crate::commands::HostEvent::ControllerShortcut;
use log::info;
use log::warn;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NodeType::Client;
use wg_2024::packet::{FloodRequest, FloodResponse, Packet, PacketType};

impl RustbustersClient {
    pub(crate) fn handle_flood_response(&mut self, flood_response: &FloodResponse) {
        for window in flood_response.path_trace.windows(2) {
            if let [(from_id, from_type), (to_id, to_type)] = window {
                self.known_nodes.insert(*from_id, *from_type);
                self.known_nodes.insert(*to_id, *to_type);

                // Update topology
                self.topology.add_edge(*from_id, *to_id, 1.0);
            }
        }

        info!("Node {}: Updated topology: {:?}", self.id, self.topology);
        info!("Node {}: Known nodes: {:?}", self.id, self.known_nodes);
    }

    pub(crate) fn handle_flood_request(&mut self, flood_request: FloodRequest, session_id: u64) {
    pub(crate) fn handle_flood_request(&mut self, flood_request: &FloodRequest, session_id: u64) {
        let mut new_path_trace = flood_request.path_trace.clone();
        new_path_trace.push((self.id, Client));

        let flood_response = FloodResponse {
            flood_id: flood_request.flood_id,
            path_trace: new_path_trace.clone(),
        };

        // If the packet was sent by this client, learn the topology without sending a response
        if flood_request.initiator_id == self.id {
            info!(
                "Node {}: Received own FloodRequest with flood_id {}. Learning topology...",
                self.id, flood_request.flood_id
            );
            self.handle_flood_response(&flood_response);
            return;
        }

        // Create the packet
        let response_packet = Packet {
            pack_type: PacketType::FloodResponse(flood_response),
            routing_header: SourceRoutingHeader {
                hop_index: 1,
                hops: new_path_trace.iter().map(|(id, _)| *id).rev().collect(),
            },
            session_id,
        };

        // Send the FloodResponse back to the initiator
        if let Some(sender) = self
            .packet_send
            .get(&response_packet.routing_header.hops[1])
        {
            info!(
                "Node {}: Sending FloodResponse to initiator {}, next hop {}",
                self.id, flood_request.initiator_id, response_packet.routing_header.hops[1]
            );
            if let Err(err) = sender.send(response_packet.clone()) {
                warn!(
                    "Node {}: Error sending FloodResponse to initiator {}: {}",
                    self.id, flood_request.initiator_id, err
                );
                self.send_to_sc(ControllerShortcut(response_packet));
            }
        } else {
            warn!(
                "Node {}: Cannot send FloodResponse to initiator {}",
                self.id, flood_request.initiator_id
            );
            self.send_to_sc(ControllerShortcut(response_packet));
        }
    }
}
