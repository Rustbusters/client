use crate::client::routing::edge_stats::BASE_WEIGHT;
use crate::tests::create_test_client;
use wg_2024::packet::NodeType;

#[test]
fn test_edge_stats_initial_state() {
    let (mut client, _, _, _) = create_test_client();
    let stats = client.get_or_create_edge_stats(1, 2);

    assert_eq!(stats.get_estimated_pdr(), 0.0);
    assert_eq!(stats.get_consecutive_nacks(), 0);
    assert_eq!(stats.get_edge_weight(), BASE_WEIGHT);
}

#[test]
fn test_edge_stats_update() {
    let (mut client, _, _, _) = create_test_client();
    let stats = client.get_or_create_edge_stats(1, 2);

    // Simulate a packet drop
    stats.update(true);
    assert!(stats.get_estimated_pdr() > 0.0);
    assert_eq!(stats.get_consecutive_nacks(), 1);

    // Simulate successful transmission
    stats.update(false);
    assert_eq!(stats.get_consecutive_nacks(), 0);
}

#[test]
fn test_path_finding_simple() {
    let (mut client, _, _, _) = create_test_client();

    // Setup a simple topology:
    // Client (1) -> Drone (2) -> Server (3)
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

    client.topology.add_edge(1, 2, BASE_WEIGHT);
    client.topology.add_edge(2, 3, BASE_WEIGHT);

    let path = client.find_weighted_path(3);
    assert_eq!(path, Some(vec![1, 2, 3]));
}

#[test]
fn test_path_finding_invalid_path() {
    let (mut client, _, _, _) = create_test_client();

    // Setup an invalid topology:
    // Client (1) -> Client (2) -> Server (3)
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(1, NodeType::Client);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(2, NodeType::Client);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(3, NodeType::Server);

    client.topology.add_edge(1, 2, BASE_WEIGHT);
    client.topology.add_edge(2, 3, BASE_WEIGHT);

    let path = client.find_weighted_path(3);
    assert_eq!(path, None);
}

#[test]
fn test_weighted_path_multiple_options() {
    let (mut client, _, _, _) = create_test_client();

    // Setup a topology with multiple paths:
    //                    (3)
    //                   /   \
    // Client(1) -- Drone(2)  Drone(4) -- Server(6)
    //                   \   /
    //                  Drone(5)

    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(1, NodeType::Client);
    for id in [2, 3, 4, 5] {
        client
            .known_nodes
            .lock()
            .unwrap()
            .insert(id, NodeType::Drone);
    }
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(6, NodeType::Server);

    // Add edges with different weights
    client.topology.add_edge(1, 2, BASE_WEIGHT); // Client to first drone
    client.topology.add_edge(2, 3, BASE_WEIGHT * 1.2); // Path through upper drones
    client.topology.add_edge(3, 4, BASE_WEIGHT * 1.1);
    client.topology.add_edge(2, 5, BASE_WEIGHT); // Path through lower drone
    client.topology.add_edge(5, 4, BASE_WEIGHT);
    client.topology.add_edge(4, 6, BASE_WEIGHT); // Final hop to server

    let path = client.find_weighted_path(6);
    // Should choose path [1, 2, 5, 4, 6] as it has lower total weight
    assert_eq!(path, Some(vec![1, 2, 5, 4, 6]));
}

#[test]
fn test_weighted_path_congested_network() {
    let (mut client, _, _, _) = create_test_client();

    // Setup a topology where the shortest path is congested:
    // Client(1) -- Drone(2) ====== Server(4)
    //              |
    //           Drone(3) ------- Server(4)
    // (where === represents a congested/high weight path)

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
        .insert(3, NodeType::Drone);
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(4, NodeType::Server);

    // Add edges - direct path is congested
    client.topology.add_edge(1, 2, BASE_WEIGHT);
    client.topology.add_edge(2, 4, BASE_WEIGHT * 3.0); // Congested direct path
    client.topology.add_edge(2, 3, BASE_WEIGHT);
    client.topology.add_edge(3, 4, BASE_WEIGHT * 1.5); // Better alternative

    let path = client.find_weighted_path(4);
    // Should choose longer but less congested path [1, 2, 3, 4]
    assert_eq!(path, Some(vec![1, 2, 3, 4]));
}

#[test]
fn test_weighted_path_mesh_network() {
    let (mut client, _, _, _) = create_test_client();

    // Setup a complex mesh topology:
    //
    //           Drone(3) ---- Drone(4) ---- Drone(5)
    //          /    |    \     |      \     |     \
    // Client(1)     |     \    |       \    |    Server(8)
    //          \    |      \   |        \   |     /
    //           Drone(2) ---- Drone(6) ---- Drone(7)

    // Setup node types
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(1, NodeType::Client);
    for id in 2..=7 {
        client
            .known_nodes
            .lock()
            .unwrap()
            .insert(id, NodeType::Drone);
    }
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(8, NodeType::Server);

    // Add edges with varying weights to simulate different network conditions
    // Client connections
    client.topology.add_edge(1, 2, BASE_WEIGHT);
    client.topology.add_edge(1, 3, BASE_WEIGHT * 1.1);

    // First level connections
    client.topology.add_edge(2, 3, BASE_WEIGHT);
    client.topology.add_edge(2, 6, BASE_WEIGHT * 1.2);

    // Second level connections
    client.topology.add_edge(3, 4, BASE_WEIGHT);
    client.topology.add_edge(3, 6, BASE_WEIGHT * 1.5);
    client.topology.add_edge(4, 5, BASE_WEIGHT);
    client.topology.add_edge(4, 6, BASE_WEIGHT * 1.3);
    client.topology.add_edge(4, 7, BASE_WEIGHT * 1.1);

    // Third level connections
    client.topology.add_edge(5, 7, BASE_WEIGHT);
    client.topology.add_edge(5, 8, BASE_WEIGHT * 1.4);
    client.topology.add_edge(6, 7, BASE_WEIGHT);

    // Server connections
    client.topology.add_edge(7, 8, BASE_WEIGHT);

    let path = client.find_weighted_path(8);
    // The path with lowest total weight should be [1, 3, 4, 7, 8]
    assert_eq!(path, Some(vec![1, 3, 4, 7, 8]));
}

#[test]
fn test_weighted_path_redundant_layers() {
    let (mut client, _, _, _) = create_test_client();

    // Setup a layered topology with redundant paths:
    //
    //              Drone(2) -------- Drone(5) -------- Server(8)
    //             /          \      /          \
    // Client(1) ----- Drone(3) -- Drone(6) ---- Server(9)
    //             \          /      \          /
    //              Drone(4) -------- Drone(7) -------- Server(10)

    // Setup node types
    client
        .known_nodes
        .lock()
        .unwrap()
        .insert(1, NodeType::Client);
    for id in 2..=7 {
        client
            .known_nodes
            .lock()
            .unwrap()
            .insert(id, NodeType::Drone);
    }
    for id in 8..=10 {
        client
            .known_nodes
            .lock()
            .unwrap()
            .insert(id, NodeType::Server);
    }

    // First layer connections
    client.topology.add_edge(1, 2, BASE_WEIGHT);
    client.topology.add_edge(1, 3, BASE_WEIGHT * 1.1);
    client.topology.add_edge(1, 4, BASE_WEIGHT * 1.2);

    // Second layer mesh
    client.topology.add_edge(2, 5, BASE_WEIGHT);
    client.topology.add_edge(2, 6, BASE_WEIGHT * 2.0); // congested
    client.topology.add_edge(3, 5, BASE_WEIGHT * 1.3);
    client.topology.add_edge(3, 6, BASE_WEIGHT);
    client.topology.add_edge(3, 7, BASE_WEIGHT * 1.4);
    client.topology.add_edge(4, 6, BASE_WEIGHT * 1.2);
    client.topology.add_edge(4, 7, BASE_WEIGHT);

    // Final layer to servers
    client.topology.add_edge(5, 8, BASE_WEIGHT);
    client.topology.add_edge(6, 9, BASE_WEIGHT);
    client.topology.add_edge(7, 10, BASE_WEIGHT * 1.1);

    // Test paths to different servers
    let path_to_8 = client.find_weighted_path(8);
    let path_to_9 = client.find_weighted_path(9);
    let path_to_10 = client.find_weighted_path(10);

    // Verify that we get different optimal paths to each server
    assert_eq!(path_to_8, Some(vec![1, 2, 5, 8]));
    assert_eq!(path_to_9, Some(vec![1, 3, 6, 9]));
    assert_eq!(path_to_10, Some(vec![1, 4, 7, 10]));
}
