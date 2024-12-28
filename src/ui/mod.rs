mod request_handler;
mod websocket;
use crate::RustbustersClient;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Server;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

const HTTP_PORT: u16 = 7373;

lazy_static! {
    pub(crate) static ref THREADS: Mutex<Vec<thread::JoinHandle<()>>> = Mutex::new(Vec::new());
}

lazy_static! {
    pub(crate) static ref CLIENTS: Mutex<Vec<NodeId>> = Mutex::new(Vec::new());
}

lazy_static! {
    pub(crate) static ref KNOWN_NODES: Mutex<Option<Arc<Mutex<HashMap<NodeId, NodeType>>>>> =
        Mutex::new(None);
}

impl RustbustersClient {
    pub(crate) fn run_ui(&self) {
        // log the content of Clients
        let mut clients = CLIENTS.lock().unwrap();

        // if it is empty, run the http server
        if clients.is_empty() {
            let http_handle = thread::spawn(run_http_server);
            let client_id = self.id;
            let websocket_handle =
                thread::spawn(move || websocket::run_websocket_server(client_id));

            THREADS.lock().unwrap().push(http_handle);
            THREADS.lock().unwrap().push(websocket_handle);
        }

        // add the client to the list
        clients.push(self.id);

        // Share the topology
        let mut known_nodes = KNOWN_NODES.lock().unwrap();
        *known_nodes = Some(self.known_nodes.clone());
        println!("Known nodes: {known_nodes:?}",);
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
