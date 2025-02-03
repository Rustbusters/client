use crate::tests::create_test_client;
use common_utils::{ClientToServerMessage, HostMessage, MessageBody, MessageContent};
use wg_2024::packet::FRAGMENT_DSIZE;

#[test]
fn test_small_message_fragmentation() {
    let (client, _, _, _) = create_test_client();
    let message = HostMessage::FromClient(ClientToServerMessage::RegisterUser {
        name: "Alice".to_string(),
    });

    let fragments = client.disassemble_message(&message);
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].fragment_index, 0);
    assert_eq!(fragments[0].total_n_fragments, 1);
    assert!(fragments[0].length > 0);
    assert!(fragments[0].length as usize <= FRAGMENT_DSIZE);
}

#[test]
fn test_large_message_fragmentation() {
    let (client, _, _, _) = create_test_client();
    let large_content = "A".repeat(FRAGMENT_DSIZE * 3);
    let message = HostMessage::FromClient(ClientToServerMessage::SendPrivateMessage {
        recipient_id: 2,
        message: MessageBody {
            sender_id: 1,
            content: MessageContent::Text(large_content),
            timestamp: "12:00".to_string(),
        },
    });

    let fragments = client.disassemble_message(&message);
    assert!(fragments.len() > 1);

    // Check fragment properties
    for (i, fragment) in fragments.iter().enumerate() {
        assert_eq!(fragment.fragment_index, i as u64);
        assert_eq!(fragment.total_n_fragments, fragments.len() as u64);
        assert!(fragment.length > 0);
        assert!(fragment.length as usize <= FRAGMENT_DSIZE);
    }
}

#[test]
fn test_fragment_reassembly() {
    let (mut client, _, _, _) = create_test_client();
    let original_message = HostMessage::FromClient(ClientToServerMessage::RegisterUser {
		name: "Alice".to_string(),
	});

    // Fragment the message
    let fragments = client.disassemble_message(&original_message);
    let session_id = 1234;

    // Simulate receiving fragments
    client.pending_received.insert(
        session_id,
        (
            fragments.iter().map(|f| Some(f.clone())).collect(),
            fragments.len() as u64,
        ),
    );

    // Attempt reassembly
    let result = client.reassemble_fragments(session_id);
    assert!(result.is_ok());

    // Compare with original
    match result.unwrap() {
		HostMessage::FromClient(ClientToServerMessage::RegisterUser { name }) => {
			assert_eq!(name, "Alice");
		}
		_ => panic!("Unexpected message type"),
	}
}

#[test]
fn test_incomplete_reassembly() {
    let (mut client, _, _, _) = create_test_client();
    let message = HostMessage::FromClient(ClientToServerMessage::RegisterUser {
		name: "Alice".to_string(),
	});

    // Fragment the message
    let fragments = client.disassemble_message(&message);
    let session_id = 5678;

    // Simulate receiving fragments but skip one
    client.pending_received.insert(
		session_id,
		(
			fragments.iter().take(fragments.len() - 1).map(|f| Some(f.clone())).collect(),
			fragments.len() as u64,
		),
	);

    // Attempt reassembly should fail
    let result = client.reassemble_fragments(session_id);
    assert!(result.is_err());
}