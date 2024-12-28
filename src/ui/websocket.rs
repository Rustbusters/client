use crate::ui::{CLIENTS_STATE, HTTP_PORT, THREADS};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tungstenite::WebSocket;
use wg_2024::network::NodeId;

const WEBSOCKET_PORT: u16 = HTTP_PORT + 1;

pub(crate) fn run_websocket_server(id: NodeId) {
    let listener = TcpListener::bind(format!("0.0.0.0:{WEBSOCKET_PORT}")).unwrap();
    listener.set_nonblocking(true).ok();
    loop {
        match listener.accept() {
            Ok((tcp_stream, _)) => {
                println!("New WebSocket connection");
                let web_socket_updates = thread::spawn(move || {
                    if let Ok(mut web_socket_stream) = tungstenite::accept(tcp_stream) {
                        web_socket_stream
                            .write(tungstenite::Message::Text(
                                format!("{{\"type\": \"new_thread\", \"thread_id\": {id}}}").into(),
                            ))
                            .unwrap();

                        web_socket_stream.flush().ok();
                        handle_new_connection(web_socket_stream);
                    }
                });
                THREADS.lock().unwrap().push(web_socket_updates);
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                // No new connections, just continue
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }

        let clients = CLIENTS_STATE.lock().unwrap();
        if clients.is_empty() {
            break;
        }
    }

    println!("WebSocket server shutting down");
}

fn handle_new_connection(mut ws_stream: WebSocket<TcpStream>) {
    println!("New WebSocket connection");

    loop {
        match ws_stream.read() {
            Ok(msg) => {
                println!("Received message: {msg:?}");
            }
            Err(_err) => {
                // println!("Error reading message: {err}");
                sleep(Duration::from_millis(10));
            }
        }
    }
}
