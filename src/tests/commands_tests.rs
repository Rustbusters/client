use crate::client::routing::edge_stats::BASE_WEIGHT;

use super::create_test_client;
use common_utils::HostCommand;
use crossbeam_channel::unbounded;
use wg_2024::packet::{NodeType, PacketType};

#[test]
fn test_send_random_message() {
    let (mut client, _, _, _) = create_test_client();
    let (packet_2_tx, packet_2_rx) = unbounded();
    let (tx, _) = unbounded();
    let dest = 3;

    // setup neighbors
    client.topology.add_node(2);
    client.topology.add_node(3);
    client.topology.add_edge(1, 2, BASE_WEIGHT);
    client.topology.add_edge(2, 3, BASE_WEIGHT);

    // setup known nodes
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(1, NodeType::Client);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(2, NodeType::Drone);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(3, NodeType::Server);

    // setup packet senders
    client.packet_send.insert(2, packet_2_tx.clone());

    client.handle_command(HostCommand::SendRandomMessage(dest), &tx);

    if let Ok(packet) = packet_2_rx.try_recv() {
        assert_eq!(packet.routing_header.hops, vec![1, 2, 3]);
    } else {
        panic!("No message was sent");
    }
}

#[test]
fn test_discover_network() {
    let (mut client, _, _, _) = create_test_client();
    let (tx, _) = unbounded();
    let (packet_2_tx, packet_2_rx) = unbounded();

    // Setup topology
    client.topology.add_node(2);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(2, NodeType::Drone);
    client.packet_send.insert(2, packet_2_tx);

    client.handle_command(HostCommand::DiscoverNetwork, &tx);

    // Verify that discovery packets are sent to neighbors
    if let Ok(packet) = packet_2_rx.try_recv() {
        if let PacketType::FloodRequest(flood_request) = packet.pack_type {
            assert_eq!(flood_request.initiator_id, 1);
            assert_eq!(flood_request.path_trace, vec![(1, NodeType::Client)]);
        } else {
            panic!("Unexpected packet type received");
        }
    } else {
        panic!("No discovery packet was sent");
    }
}

#[test]
fn test_add_remove_sender() {
    let (mut client, _, _, _) = create_test_client();
    let (tx, _) = unbounded();
    let (sender, receiver) = unbounded();
    let sender_id = 3;

    // Setup topology for the new node
    client.topology.add_node(sender_id);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(sender_id, NodeType::Drone);

    // Test Add Sender
    client.handle_command(HostCommand::AddSender(sender_id, sender.clone()), &tx);
    assert!(client.packet_send.contains_key(&sender_id));

    // Verify that discovery packet was sent to the new sender
    if let Ok(packet) = receiver.try_recv() {
        if let PacketType::FloodRequest(flood_request) = packet.pack_type {
            assert_eq!(flood_request.initiator_id, 1);
            assert_eq!(flood_request.path_trace, vec![(1, NodeType::Client)]);
        } else {
            panic!("Unexpected packet type received after add");
        }
    } else {
        panic!("No discovery packet was sent after add");
    }

    // Test Remove Sender and verify discovery is triggered
    let (new_sender, new_receiver) = unbounded();
    client.packet_send.insert(2, new_sender); // Add another node to receive discovery after remove
    client.topology.add_node(2);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(2, NodeType::Drone);

    client.handle_command(HostCommand::RemoveSender(sender_id), &tx);
    assert!(!client.packet_send.contains_key(&sender_id));

    // Verify that discovery packet was sent to remaining nodes
    if let Ok(packet) = new_receiver.try_recv() {
        if let PacketType::FloodRequest(flood_request) = packet.pack_type {
            assert_eq!(flood_request.initiator_id, 1);
            assert_eq!(flood_request.path_trace, vec![(1, NodeType::Client)]);
        } else {
            panic!("Unexpected packet type received after remove");
        }
    } else {
        panic!("No discovery packet was sent after remove");
    }
}
