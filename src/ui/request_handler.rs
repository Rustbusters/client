use crate::ui::CLIENTS;
use std::fs;
use std::io::Error;
use std::str::FromStr;
use tiny_http::{Header, Method, Request, Response};

pub(crate) fn handle_request(mut req: Request) -> Result<(), Error> {
    let method = req.method();
    let url = req.url();
    println!("Received request: {method} {url}");
    let response = match (method, url) {
        // Servire il file index.html sulla root
        (Method::Get, "/") => {
            let file = fs::read_to_string("static/client/tonini/index.html")
                .unwrap_or("DEFAULT_HTML".to_string());
            Response::from_string(file)
                .with_header(Header::from_str("Content-Type: text/html").unwrap())
        }
        (Method::Get, "/api/threads") => {
            let threads = CLIENTS.lock().unwrap();
            // respond with the list of active threads
            Response::from_string(format!("{threads:?}"))
        }
        // Servire contenuti statici
        (Method::Get, path) if path.starts_with('/') => {
            let sanitized_path = &path[1..]; // Rimuove lo slash iniziale
            match fs::read(format!("static/client/tonini/{sanitized_path}")) {
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
