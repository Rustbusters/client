pub mod routing_tests;

use std::collections::HashMap;

use crate::RustbustersClient;
use crossbeam_channel::unbounded;
use wg_2024::packet::NodeType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_weighted_path() {
        let mut client = create_test_client();
        
        // Creiamo un percorso valido:
        // Client (1) -> Drone (2) -> Server (3)
        client.known_nodes.lock().unwrap().insert(1, NodeType::Client);
        client.known_nodes.lock().unwrap().insert(2, NodeType::Drone);
        client.known_nodes.lock().unwrap().insert(3, NodeType::Server);
        
        client.topology.add_edge(1, 2, 1.0);
        client.topology.add_edge(2, 3, 1.0);

        let path = client.find_weighted_path(3);
        assert_eq!(path, Some(vec![1, 2, 3]));
    }
}

fn create_test_client() -> RustbustersClient {
    let (tx_ctrl, _) = unbounded();
    let (_, rx_ctrl) = unbounded();
    let (_, rx_packet) = unbounded();
    
    RustbustersClient::new(
        1,
        tx_ctrl,
        rx_ctrl,
        rx_packet,
        HashMap::new(),
        None,
    )
}
