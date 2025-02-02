use crate::ui::utils::{
    get_content_from_msg, get_number_from_json, get_request_body, get_string_from_json,
};
use crate::ui::CLIENTS_STATE;
use common_utils::{ClientToServerMessage, MessageBody};
use serde_json::Value;
use std::io::Cursor;
use tiny_http::{Request, Response};

/// Processes a message sending request between users
/// 
/// ### Arguments
/// * `req` - The HTTP request containing the message details
/// 
/// Returns an HTTP response indicating the result of the operation
pub(crate) fn post_send_message(req: &mut Request) -> Response<Cursor<Vec<u8>>> {
    // get the body of the request
    let json_body: Value = get_request_body(req);

    // extract sender_id, receiver_id, timestamp (it's a string), content (string or Vec<u8>)
    let sender_id = get_number_from_json(&json_body, "sender_id");
    let receiver_id = get_number_from_json(&json_body, "receiver_id");
    let server_id = get_number_from_json(&json_body, "server_id");
    let timestamp = get_string_from_json(&json_body, "timestamp");
    let content = get_content_from_msg(&json_body);

    // check validity of fields
    if sender_id.is_none()
        || receiver_id.is_none()
        || server_id.is_none()
        || timestamp.is_none()
        || content.is_none()
    {
        return Response::from_string("Invalid request body").with_status_code(400);
    }

    // get the sender of the client node
    let client_sender = CLIENTS_STATE
        .lock()
        .unwrap()
        .get(&sender_id.unwrap())
        .and_then(|client| client.sender.clone());

    if let Some(sender) = client_sender {
        // build the message
        let message = ClientToServerMessage::SendPrivateMessage {
            recipient_id: receiver_id.unwrap(),
            message: MessageBody {
                sender_id: sender_id.unwrap(),
                content: content.unwrap(),
                timestamp: timestamp.unwrap(),
            },
        };

        // send the message to the client node
        sender.send((server_id.unwrap(), message)).ok();
    }

    Response::from_string("Message sent").with_status_code(200)
}
