use crate::client::RustbustersClient;
use log::{info, warn};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::NodeType::Client;
use wg_2024::packet::{FloodRequest, Packet, PacketType};

impl RustbustersClient {
    /// Initiates network discovery by flooding FloodRequest packets.
    /// 
    /// This function:
    /// 1. Generates a unique flood ID
    /// 2. Creates a FloodRequest packet with the client's path trace
    /// 3. Broadcasts the request to all known neighbors
    ///
    /// The discovery process helps build the network topology and
    /// identify available paths to servers.
    pub(crate) fn discover_network(&mut self) {
        // Generate a unique flood_id
        self.flood_id_counter += 1;
        let flood_id = self.flood_id_counter;

        // Initialize the FloodRequest
        let flood_request = FloodRequest {
            flood_id,
            initiator_id: self.id,
            path_trace: vec![(self.id, Client)],
        };

        // Create the packet without routing header (it's ignored for FloodRequest)
        let packet = Packet {
            pack_type: PacketType::FloodRequest(flood_request),
            routing_header: SourceRoutingHeader {
                hop_index: 0,
                hops: vec![],
            },
            session_id: 0,
        };

        for (&neighbor_id, neighbor_sender) in &self.packet_send {
            info!(
                "Client {}: Sending FloodRequest to {} with flood_id {}",
                self.id, neighbor_id, flood_id
            );
            if let Err(err) = neighbor_sender.send(packet.clone()) {
                warn!(
                    "Client {}: Unable to send FloodRequest to {}: {}",
                    self.id, neighbor_id, err
                );
            }
        }
    }
}
