use crate::ui::{CLIENTS_STATE, HTTP_PORT, THREADS};
use crossbeam_channel::TryRecvError;
use log::{error, info, warn};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tungstenite::{Error, Message, WebSocket};

const WEBSOCKET_PORT: u16 = HTTP_PORT + 1;

/// Runs the WebSocket server that handles client connections
/// and message distribution
pub(crate) fn run_websocket_server() {
    let listener = TcpListener::bind(format!("0.0.0.0:{WEBSOCKET_PORT}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    loop {
        match listener.accept() {
            Ok((ws_stream, _)) => {
                let web_socket_updates = thread::spawn(move || {
                    if let Ok(web_socket_stream) = tungstenite::accept(ws_stream) {
                        if let Err(e) = handle_new_connection(web_socket_stream) {
                            error!("[CLIENT-WS] Connection error: {}", e);
                        }
                    }
                });
                THREADS.lock().unwrap().push(web_socket_updates);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No new connections, continue
                std::thread::sleep(std::time::Duration::from_millis(100));
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
}

/// Handles a new WebSocket connection, managing message forwarding
/// between clients
///
/// ### Arguments
/// * `ws_stream` - The WebSocket stream for the new connection
fn handle_new_connection(
    mut ws_stream: WebSocket<TcpStream>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("[CLIENT-WS] New WebSocket connection");
    ws_stream.get_ref().set_nonblocking(true)?;

    loop {
        let clients = CLIENTS_STATE.lock().map_err(|_| "Failed to acquire lock")?;
        for (client_id, client_state) in &*clients {
            if let Some(receiver) = &client_state.receiver {
                match receiver.try_recv() {
                    Ok(msg) => {
                        let ws_message = serde_json::json!({
                            "client_id": client_id,
                            "server_id": msg.0,
                            "message": msg.1
                        })
                        .to_string();

                        ws_stream
                            .send(Message::Text(ws_message.into()))
                            .map_err(|e| {
                                warn!("[CLIENT-WS] Failed to send message: {e:?}");
                                e
                            })?;
                    }
                    Err(TryRecvError::Empty) => { /* CONTINUE */ }
                    Err(TryRecvError::Disconnected) => {
                        warn!("[CLIENT-WS] Channel disconnected for client {}", client_id);
                    }
                }
            }
        }
        drop(clients);

        match ws_stream.read() {
            Err(Error::ConnectionClosed | Error::AlreadyClosed) => break,
            Err(Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                sleep(Duration::from_millis(50));
            }
            Err(e) => {
                warn!("[CLIENT-WS] WebSocket error: {}", e);
            }
            Ok(_) => continue,
        }
    }

    info!("[CLIENT-WS] WebSocket connection closed");
    Ok(())
}
