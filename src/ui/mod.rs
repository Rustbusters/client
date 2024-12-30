mod api;
mod request_handler;
mod utils;
mod websocket;

use crate::RustbustersClient;
use common_utils::{ClientToServerMessage, ServerToClientMessage};
use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Server;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

const HTTP_PORT: u16 = 7373;

type KnownNodes = Option<Arc<Mutex<HashMap<NodeId, NodeType>>>>;
pub(crate) struct ClientState {
    known_nodes: KnownNodes,
    sender: Option<Sender<(NodeId, NodeId, ClientToServerMessage)>>,
    receiver: Option<Receiver<(NodeId, NodeId, ServerToClientMessage)>>,
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
        sender: Sender<(NodeId, NodeId, ClientToServerMessage)>,
        receiver: Receiver<(NodeId, NodeId, ServerToClientMessage)>,
    ) {
        // log the content of Clients
        let mut clients_state = CLIENTS_STATE.lock().unwrap();

        // if it is empty, run the http server
        if clients_state.is_empty() {
            let http_handle = thread::spawn(run_http_server);
            let websocket_handle = thread::spawn(websocket::run_websocket_server);

            THREADS.lock().unwrap().push(http_handle);
            THREADS.lock().unwrap().push(websocket_handle);
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
    println!("Visit http://localhost:{HTTP_PORT}");
    let http_server = Server::http(format!("0.0.0.0:{HTTP_PORT}")).unwrap();
    loop {
        if let Ok(Some(request)) = http_server.try_recv() {
            match request_handler::handle_request(request) {
                Ok(()) => {}
                Err(e) => eprintln!("Error handling request: {e}"),
            }
        }
    }
}
