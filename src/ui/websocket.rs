use crate::client::KillCommand;
use crate::ui::{CLIENTS_STATE, HTTP_PORT, THREADS};
use crossbeam_channel::Receiver;
use log::{error, info};
use std::net::{TcpListener, TcpStream};
use std::thread;
use tungstenite::{Error, Message, WebSocket};

const WEBSOCKET_PORT: u16 = HTTP_PORT + 1;

/// Runs the WebSocket server that handles client connections
/// and message distribution
pub(crate) fn run_websocket_server(killer_receiver: Receiver<KillCommand>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{WEBSOCKET_PORT}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    loop {
        // Check for kill command
        if killer_receiver.try_recv().is_ok() {
            info!("[CLIENT-WS] Received kill command, shutting down");
            break;
        }

        match listener.accept() {
            Ok((ws_stream, _)) => {
                let web_socket_updates = thread::spawn(move || {
                    if let Ok(web_socket_stream) = tungstenite::accept(ws_stream) {
                        handle_new_connection(web_socket_stream);
                    }
                });
                THREADS.lock().unwrap().push(web_socket_updates);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No new connections, continue
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                error!("[CLIENT-WS] Failed to accept connection: {}", e);
                break;
            }
        }

        let clients = CLIENTS_STATE.lock().unwrap();
        if clients.is_empty() {
            break;
        }
    }

    info!("[CLIENT-WS] WebSocket server shutting down");
    println!("CLIENTS WS SERVER ENDED!! THREAD EXITED");
}

/// Handles a new WebSocket connection, managing message forwarding
/// between clients
/// 
/// ### Arguments
/// * `ws_stream` - The WebSocket stream for the new connection
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
                    if let Err(e) = ws_stream.send(Message::Text(ws_message.into())) {
                        eprintln!("[CLIENT-WS] Failed to send message: {e:?}");
                    }
                    ws_stream.flush().unwrap();
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
