use crate::tests::create_test_client;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{Fragment, NackType, Packet, PacketType};

// Local helper function for creating test packets
fn create_test_packet(session_id: u64, fragment_index: u64, path: &[u8]) -> Packet {
    Packet {
        session_id,
        routing_header: SourceRoutingHeader {
            hops: path.to_vec(),
            hop_index: 1,
        },
        pack_type: PacketType::MsgFragment(Fragment {
            fragment_index,
            total_n_fragments: 1,
            data: {
                let mut data = [0; 128];
                data[0] = 1;
                data[1] = 2;
                data[2] = 3;
                data
            },
            length: 3,
        }),
    }
}

#[test]
fn test_edge_stats_ack_handling() {
    let (mut client, _, _, _) = create_test_client();

    // Setup initial path
    let path = vec![1, 2, 3, 4];
    let session_id = 1;
    let fragment_index = 0;

    // Create a dummy packet and add it to pending_sent
    let packet = create_test_packet(session_id, fragment_index, &path);
    client
        .pending_sent
        .insert((session_id, fragment_index), packet);

    // Handle ACK
    client.handle_ack(session_id, fragment_index);

    // Verify edge stats were updated correctly
    for window in path.windows(2) {
        let stats = client.get_or_create_edge_stats(window[0], window[1]);
        assert_eq!(stats.get_consecutive_nacks(), 0);
        assert_eq!(stats.get_estimated_pdr(), 0.0); // Perfect transmission
    }
}

#[test]
fn test_edge_stats_nack_handling() {
    let (mut client, _, _, _) = create_test_client();

    // Setup path where drop occurred
    let path = vec![3, 2, 1]; // NACK path is reversed
    let session_id = 1;
    let fragment_index = 0;

    // Create a dummy packet and add it to pending_sent
    let original_path = vec![1, 2, 3, 4];
    let packet = create_test_packet(session_id, fragment_index, &original_path);
    client
        .pending_sent
        .insert((session_id, fragment_index), packet);

    // Create NACK header
    let nack_header = SourceRoutingHeader {
        hops: path.clone(),
        hop_index: 0,
    };

    // Handle NACK
    client.handle_nack(session_id, fragment_index, NackType::Dropped, nack_header);

    // Verify edge stats for the dropping edge
    let stats = client.get_or_create_edge_stats(path[0], path[1]);
    assert!(stats.get_estimated_pdr() > 0.0); // Should indicate some packet loss
    assert_eq!(stats.get_consecutive_nacks(), 1);
}

#[test]
fn test_edge_stats_multiple_nacks() {
    let (mut client, _, _, _) = create_test_client();

    let path = vec![3, 2, 1];
    let session_id = 1;
    let fragment_index = 0;

    // Setup original packet
    let original_path = vec![1, 2, 3, 4];
    let packet = create_test_packet(session_id, fragment_index, &original_path);
    client
        .pending_sent
        .insert((session_id, fragment_index), packet);

    let nack_header = SourceRoutingHeader {
        hops: path.clone(),
        hop_index: 0,
    };

    // Send multiple NACKs
    for _ in 0..3 {
        client.handle_nack(
            session_id,
            fragment_index,
            NackType::Dropped,
            nack_header.clone(),
        );
    }

    // Verify edge stats show deteriorating condition
    let stats = client.get_or_create_edge_stats(path[0], path[1]);
    assert!(stats.get_estimated_pdr() > 0.5); // Should indicate significant packet loss
    assert_eq!(stats.get_consecutive_nacks(), 3);
}