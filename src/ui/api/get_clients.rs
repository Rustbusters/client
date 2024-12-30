use crate::ui::CLIENTS_STATE;
use std::io::Cursor;
use std::str::FromStr;
use tiny_http::{Header, Response};
use wg_2024::network::NodeId;

pub(crate) fn get_clients() -> Response<Cursor<Vec<u8>>> {
    let clients = CLIENTS_STATE.lock().unwrap();
    // respond with the list of active threads
    let clients_list: Vec<NodeId> = clients.keys().copied().collect();

    Response::from_string(format!("{clients_list:?}"))
        .with_header(Header::from_str("Content-Type: application/json").unwrap())
}
