use crate::ui::{CLIENTS_STATE, HTTP_PORT, THREADS};
use log::info;
use std::net::{TcpListener, TcpStream};
use std::thread;
use tungstenite::{Error, Message, WebSocket};

const WEBSOCKET_PORT: u16 = HTTP_PORT + 1;

pub(crate) fn run_websocket_server() {
    let listener = TcpListener::bind(format!("0.0.0.0:{WEBSOCKET_PORT}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    for stream in listener.incoming() {
        if let Ok(ws_stream) = stream {
            let web_socket_updates = thread::spawn(move || {
                if let Ok(web_socket_stream) = tungstenite::accept(ws_stream) {
                    handle_new_connection(web_socket_stream);
                }
            });
            THREADS.lock().unwrap().push(web_socket_updates);

            let clients = CLIENTS_STATE.lock().unwrap();
            if clients.is_empty() {
                break;
            }
        } else {
            // DO NOTHING
        }
    }

    info!("[CLIENT-WS] WebSocket server shutting down");
}

fn handle_new_connection(mut ws_stream: WebSocket<TcpStream>) {
    info!("[CLIENT-WS] New WebSocket connection");
    ws_stream.get_ref().set_nonblocking(true).unwrap();

    loop {
        let clients = CLIENTS_STATE.lock().unwrap().clone();
        for (client_id, client_state) in &clients {
            if let Some(receiver) = &client_state.receiver {
                if let Ok(msg) = receiver.try_recv() {
                    let ws_message = format!(
                        "{{\"client_id\":{client_id},\"server_id\": {},\"message\":{}}}",
                        msg.0,
                        serde_json::to_string(&msg.1).expect("Should be serializable")
                    );
                    ws_stream.send(Message::Text(ws_message.into())).ok();
                }
            }
        }
        drop(clients);

        if let Err(Error::ConnectionClosed | Error::AlreadyClosed) = ws_stream.read() {
            break;
        }
    }

    info!("[CLIENT-WS] WebSocket connection closed");
}
