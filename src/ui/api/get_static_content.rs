use crate::ui::utils::get_mime_type;
use log::{info, warn};
use std::fs;
use std::io::Cursor;
use std::str::FromStr;
use tiny_http::{Header, Response};

const STATIC_PATH: &str = "static/client/frontend/dist";

pub(crate) fn provide_static_file(path: &str) -> Response<Cursor<Vec<u8>>> {
    let sanitized_path = &path[1..]; // Rimuove lo slash iniziale
    match fs::read(format!("{STATIC_PATH}/{sanitized_path}")) {
        Ok(content) => {
            info!("[CLIENT-HTTP] Serving static file: {sanitized_path}");
            Response::from_data(content).with_header(
                Header::from_str(&format!("Content-Type: {}", get_mime_type(sanitized_path)))
                    .unwrap(),
            )
        }
        Err(err) => {
            warn!("[CLIENT-HTTP] Error reading file: {err}");
            Response::from_string("404 Not Found").with_status_code(404)
        }
    }
}
