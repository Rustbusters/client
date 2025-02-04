mod api;
mod request_handler;
mod utils;
mod websocket;

use crate::RustbustersClient;
use common_utils::{ClientToServerMessage, ServerToClientMessage};
use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Server;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

const HTTP_PORT: u16 = 7373;

type KnownNodes = Option<Arc<Mutex<HashMap<NodeId, NodeType>>>>;

#[derive(Clone, Debug)]
pub(crate) struct ClientState {
    known_nodes: KnownNodes,
    // NodeId is the destination server
    sender: Option<Sender<(NodeId, ClientToServerMessage)>>,
    // NodeId is the id of the destination client
    receiver: Option<Receiver<(NodeId, ServerToClientMessage)>>,
}

lazy_static! {
    pub(crate) static ref THREADS: Mutex<Vec<thread::JoinHandle<()>>> = Mutex::new(Vec::new());
}

lazy_static! {
    pub(crate) static ref CLIENTS_STATE: Mutex<HashMap<NodeId, ClientState>> =
        Mutex::new(HashMap::new());
}

impl RustbustersClient {
    pub(crate) fn run_ui(
        &self,
        sender: Sender<(NodeId, ClientToServerMessage)>,
        receiver: Receiver<(NodeId, ServerToClientMessage)>,
    ) {
        // log the content of Clients
        let mut clients_state = CLIENTS_STATE.lock().unwrap();

        // if it is empty, run the http server
        if clients_state.is_empty() {
            let http_handle = thread::spawn(run_http_server);
            let websocket_handle = thread::spawn(websocket::run_websocket_server);

            let mut threads = THREADS.lock().unwrap();
            threads.push(http_handle);
            threads.push(websocket_handle);
        }

        // add the client to the list
        clients_state.insert(
            self.id,
            ClientState {
                known_nodes: Some(self.known_nodes.clone()),
                sender: Some(sender),
                receiver: Some(receiver),
            },
        );
    }
}

fn run_http_server() {
    println!("[CLIENT-HTTP] Visit http://localhost:{HTTP_PORT} for the client UI");
    let http_server = Server::http(format!("0.0.0.0:{HTTP_PORT}")).unwrap();

    loop {
        match http_server.try_recv() {
            Ok(Some(request)) => {
                if let Err(e) = request_handler::handle_request(request) {
                    eprintln!("[CLIENT-HTTP] Error handling request: {e}");
                }
            }
            Ok(None) => {
                // No request available, sleep a bit
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("[CLIENT-HTTP] Server error: {e}");
                break;
            }
        }

        // Check if we should stop (no more clients)
        let clients = CLIENTS_STATE.lock().unwrap();
        if clients.is_empty() {
            break;
        }
    }

    info!("[CLIENT-HTTP] HTTP server shutting down");
}
