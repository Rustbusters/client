use crate::ui::CLIENTS_STATE;
use common_utils::ClientToServerMessage;
use std::collections::HashMap;
use std::io::Cursor;
use tiny_http::Response;
use wg_2024::network::NodeId;

/// Requests the list of registered users from a specific server
///
/// ### Arguments
/// * `query_params` - HashMap containing query parameters, must include both 'client_id' and 'server_id'
pub(crate) fn get_registered_users(
    query_params: &Option<HashMap<String, String>>,
) -> Response<Cursor<Vec<u8>>> {
    // get id from query string
    let client_id = query_params
        .as_ref()
        .and_then(|params| params.get("client_id"))
        .and_then(|id_str| id_str.parse::<NodeId>().ok());
    let server_id = query_params
        .as_ref()
        .and_then(|params| params.get("server_id"))
        .and_then(|id_str| id_str.parse::<NodeId>().ok());

    if client_id.is_none() || server_id.is_none() {
        return Response::from_string(
            "Invalid or missing 'client_id' or 'server_id' query parameter",
        )
        .with_status_code(400);
    }

    let client_id = client_id.unwrap();
    let server_id = server_id.unwrap();

    // get the sender of the client node
    let client_sender = CLIENTS_STATE
        .lock()
        .unwrap()
        .get(&client_id)
        .and_then(|client| client.sender.clone());

    if let Some(sender) = client_sender {
        // build the message
        let message = ClientToServerMessage::RequestActiveUsers;

        // send the message to the client node
        sender.send((server_id, message)).ok();
    }

    // Response OK 200 with sample text
    Response::from_string("Request for active users sent").with_status_code(200)
}
