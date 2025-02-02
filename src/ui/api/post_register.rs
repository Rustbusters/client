use crate::ui::utils::{get_number_from_json, get_request_body, get_string_from_json};
use crate::ui::CLIENTS_STATE;
use common_utils::ClientToServerMessage;
use serde_json::Value;
use std::io::Cursor;
use tiny_http::{Request, Response};

/// Processes a registration request from a new user
/// 
/// ### Arguments
/// * `req` - The HTTP request containing the registration details
/// 
/// Returns an HTTP response indicating the result of the operation
pub(crate) fn post_register(req: &mut Request) -> Response<Cursor<Vec<u8>>> {
    let json_body: Value = get_request_body(req);

    let client_id = get_number_from_json(&json_body, "client_id");
    let server_id = get_number_from_json(&json_body, "server_id");
    let username = get_string_from_json(&json_body, "username");

    if client_id.is_none() || server_id.is_none() || username.is_none() {
        return Response::from_string("Invalid request body").with_status_code(400);
    }

    let client_sender = CLIENTS_STATE
        .lock()
        .unwrap()
        .get(&client_id.unwrap())
        .and_then(|client| client.sender.clone());

    if let Some(sender) = client_sender {
        // build the message
        let message = ClientToServerMessage::RegisterUser {
            name: username.unwrap(),
        };

        // send the message to the client node
        sender.send((server_id.unwrap(), message)).ok();
    }

    Response::from_string("Register request received").with_status_code(200)
}
