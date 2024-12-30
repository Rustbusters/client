use crate::ui::CLIENTS_STATE;
use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;
use tiny_http::{Header, Response};
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

pub(crate) fn get_servers(
    query_params: &Option<HashMap<String, String>>,
) -> Response<Cursor<Vec<u8>>> {
    // Ottieni il parametro `id` dalla query string
    let id = query_params
        .as_ref()
        .and_then(|params| params.get("id"))
        .and_then(|id_str| id_str.parse::<NodeId>().ok())
        .unwrap_or(0);

    if id == 0 {
        return Response::from_string("Invalid or missing 'id' query parameter")
            .with_status_code(400);
    }
    let clients_state = CLIENTS_STATE.lock().unwrap();
    let known_nodes = clients_state
        .get(&id)
        .and_then(|client| client.known_nodes.clone());

    let known_nodes = match known_nodes {
        None => vec![],
        Some(node_arc) => {
            let node_map = node_arc.lock().unwrap();
            node_map
                .iter()
                .filter_map(|(node_id, node_type)| {
                    if *node_type == NodeType::Server {
                        Some(*node_id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<NodeId>>()
        }
    };

    Response::from_string(format!("{known_nodes:?}"))
        .with_header(Header::from_str("Content-Type: application/json").unwrap())
}
