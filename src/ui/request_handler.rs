use crate::ui::CLIENTS_STATE;
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Error};
use std::str::FromStr;
use tiny_http::{Header, Method, Request, Response};
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

const STATIC_PATH: &str = "static/client/frontend/dist";

pub(crate) fn handle_request(mut req: Request) -> Result<(), Error> {
    let method = req.method();
    let full_url = req.url(); // Include sia il path che i query parameters
    let path = full_url.split('?').next().unwrap_or("/"); // Ottieni solo il path

    // Parsing della query string (se esiste)
    let query_params: Option<HashMap<String, String>> = full_url.find('?').map(|pos| {
        full_url[pos + 1..]
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?.to_string();
                let value = parts.next()?.to_string();
                Some((key, value))
            })
            .collect()
    });

    println!("Received request: {method} {full_url}");
    let response = match (method, path) {
        // Servire il file index.html sulla root
        (Method::Get, "/") => {
            let file = fs::read_to_string(format!("{STATIC_PATH}/index.html"))
                .unwrap_or("DEFAULT_HTML".to_string());
            Response::from_string(file)
                .with_header(Header::from_str("Content-Type: text/html").unwrap())
        }
        (Method::Get, "/api/clients") => {
            let clients = CLIENTS_STATE.lock().unwrap();
            // respond with the list of active threads
            let clients_list: Vec<NodeId> = clients.keys().copied().collect();

            Response::from_string(format!("{clients_list:?}"))
                .with_header(Header::from_str("Content-Type: application/json").unwrap())
        }
        (Method::Get, "/api/servers") => get_servers(&query_params),
        // Servire contenuti statici
        (Method::Get, path) if path.starts_with('/') => {
            let sanitized_path = &path[1..]; // Rimuove lo slash iniziale
            match fs::read(format!("{STATIC_PATH}/{sanitized_path}")) {
                Ok(content) => {
                    println!("Serving static file: {sanitized_path}");
                    Response::from_data(content).with_header(
                        Header::from_str(&format!(
                            "Content-Type: {}",
                            get_mime_type(sanitized_path)
                        ))
                        .unwrap(),
                    )
                }
                Err(err) => {
                    println!("Error reading file: {err}");
                    Response::from_string("404 Not Found").with_status_code(404)
                }
            }
        }
        // API POST
        (Method::Post, "/api/send-to") => {
            let mut body = String::new();
            req.as_reader()
                .read_to_string(&mut body)
                .unwrap_or_else(|_| {
                    println!("Failed to read request body");
                    0
                });

            println!("POST request body: {}", body);

            Response::from_string("POST request received")
        }
        (Method::Post, "/api/register") => Response::from_string("POST request received"),
        // API PUT
        (Method::Put, "/api") => {
            println!("PUT request received");
            Response::from_string("PUT request received")
        }
        // API DELETE
        (Method::Delete, "/api") => {
            println!("DELETE request received");
            Response::from_string("DELETE request received")
        }
        // Route non trovata
        _ => {
            let response = Response::from_string("404 Not Found");
            response.with_status_code(404)
        }
    };

    req.respond(response)
}

fn get_mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".mp4") {
        "video/mp4"
    } else if path.ends_with(".webm") {
        "video/webm"
    } else if path.ends_with(".ogg") {
        "video/ogg"
    } else if path.ends_with(".avi") {
        "video/x-msvideo"
    } else if path.ends_with(".mpeg") {
        "video/mpeg"
    } else {
        "application/octet-stream"
    }
}

fn get_servers(query_params: &Option<HashMap<String, String>>) -> Response<Cursor<Vec<u8>>> {
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
