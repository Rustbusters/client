use crate::ui::api::get_clients::get_clients;
use crate::ui::api::get_servers::get_servers;
use crate::ui::api::get_static_content::provide_static_file;
use crate::ui::api::post_send_message::post_send_message;
use std::collections::HashMap;
use std::io::Error;
use tiny_http::{Method, Request, Response};

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
        // API GET
        (Method::Get, "/") => provide_static_file("/index.html"),
        (Method::Get, "/api/clients") => get_clients(),
        (Method::Get, "/api/servers") => get_servers(&query_params),
        (Method::Get, path) if path.starts_with('/') => provide_static_file(path),
        // API POST
        (Method::Post, "/api/send-to") => post_send_message(&mut req),
        (Method::Post, "/api/register") => Response::from_string("POST request received"),
        _ => {
            let response = Response::from_string("404 Not Found");
            response.with_status_code(404)
        }
    };

    req.respond(response)
}
