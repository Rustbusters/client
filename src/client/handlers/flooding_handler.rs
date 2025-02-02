use crate::client::routing::edge_stats::BASE_WEIGHT;
use crate::client::RustbustersClient;
use common_utils::HostEvent::ControllerShortcut;
use log::info;
use log::warn;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NodeType::Client;
use wg_2024::packet::{FloodRequest, FloodResponse, Packet, PacketType};

impl RustbustersClient {
    /// Processes flood responses to build network topology.
    ///
    /// Updates the client's knowledge of:
    /// - Network topology
    /// - Known nodes and their types
    /// - Edge weights for routing decisions
    ///
    /// ### Arguments
    /// * `flood_response` - Contains the path trace of the flood through the network
    pub(crate) fn handle_flood_response(&mut self, flood_response: &FloodResponse) {
        for window in flood_response.path_trace.windows(2) {
            if let [(from_id, from_type), (to_id, to_type)] = window {
                // Update known nodes
                self.known_nodes
                    .lock()
                    .unwrap()
                    .insert(*from_id, *from_type);
                self.known_nodes.lock().unwrap().insert(*to_id, *to_type);

                // Update topology
                self.topology.add_edge(*from_id, *to_id, BASE_WEIGHT);
            }
        }

        info!("Client {}: Updated topology: {:?}", self.id, self.topology);
        info!(
            "Client {}: Known nodes: {:?}",
            self.id,
            self.known_nodes.lock().unwrap()
        );
    }

    /// Handles incoming flood requests and generates responses.
    ///
    /// This function:
    /// 1. Adds this client to the path trace
    /// 2. Creates a flood response
    /// 3. If not the initiator, routes the response back to the initiator
    ///    else uses the response to learn the network topology
    ///
    /// ### Arguments
    /// * `flood_request` - The received flood request
    /// * `session_id` - The ID of the flooding session
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
                "Client {}: Received own FloodRequest with flood_id {}. Learning topology...",
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
                "Client {}: Sending FloodResponse to initiator {}, next hop {}",
                self.id, flood_request.initiator_id, response_packet.routing_header.hops[1]
            );
            if let Err(err) = sender.send(response_packet.clone()) {
                warn!(
                    "Client {}: Error sending FloodResponse to initiator {}: {}",
                    self.id, flood_request.initiator_id, err
                );
                self.send_to_sc(ControllerShortcut(response_packet));
            }
        } else {
            warn!(
                "Client {}: Cannot send FloodResponse to initiator {}",
                self.id, flood_request.initiator_id
            );
            self.send_to_sc(ControllerShortcut(response_packet));
        }
    }
}
