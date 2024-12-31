use crate::ui::utils::{
    get_content_from_json, get_number_from_json, get_string_from_json, Content,
};
use crate::ui::CLIENTS_STATE;
use common_utils::{ClientToServerMessage, MessageBody, MessageContent};
use serde_json::Value;
use std::io::Cursor;
use tiny_http::{Request, Response};

pub(crate) fn post_send_message(req: &mut Request) -> Response<Cursor<Vec<u8>>> {
    // read the body of the request
    let mut body = String::new();
    req.as_reader()
        .read_to_string(&mut body)
        .unwrap_or_else(|_| {
            println!("Failed to read request body");
            0
        });
    println!("POST request body: {body}",);

    // parse the body as JSON
    let json_body: Value = serde_json::from_str(&body).unwrap_or_else(|_| {
        println!("Failed to parse request body");
        Value::Null
    });

    // extract sender_id, receiver_id, timestamp (it's a string), content (string or Vec<u8>)
    let sender_id = get_number_from_json(&json_body, "sender_id");
    let receiver_id = get_number_from_json(&json_body, "receiver_id");
    let server_id = get_number_from_json(&json_body, "server_id");
    let timestamp = get_string_from_json(&json_body, "timestamp");
    let content = get_content_from_json(&json_body);

    // check validity of fields
    if sender_id.is_none()
        || receiver_id.is_none()
        || server_id.is_none()
        || timestamp.is_none()
        || content.is_none()
    {
        Response::from_string("Invalid request body").with_status_code(400)
    } else {
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
                    content: match content.unwrap() {
                        Content::String(text) => MessageContent::Text(text),
                        Content::Bytes(vec) => MessageContent::Image(vec),
                    },
                    timestamp: timestamp.unwrap(),
                },
            };

            // send the message to the client node
            sender.send((server_id.unwrap(), message)).ok();
        }

        Response::from_string("Message sent").with_status_code(200)
    }
}
