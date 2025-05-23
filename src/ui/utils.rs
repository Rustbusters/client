use common_utils::MessageContent;
use log::{debug, error, warn};
use serde_json::Value;
use serde_json::Value::Number;
use tiny_http::Request;

/// Determines the MIME type based on file extension
/// 
/// ### Arguments
/// * `path` - The file path to analyze
/// 
/// Returns the corresponding MIME type as a string
pub(crate) fn get_mime_type(path: &str) -> &'static str {
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

/// Extracts and parses the body of an HTTP request into JSON
/// 
/// ### Arguments
/// * `req` - The HTTP request to process
/// 
/// Returns the parsed JSON Value
pub(crate) fn get_request_body(req: &mut Request) -> Value {
    // read the body of the request
    let mut body = String::new();
    req.as_reader()
        .read_to_string(&mut body)
        .unwrap_or_else(|_| {
            warn!("[CLIENT-HTTP] Failed to read request body");
            0
        });
    debug!("[CLIENT-HTTP] POST request body: {body}",);

    // parse the body as JSON
    serde_json::from_str(&body).unwrap_or_else(|_| {
        error!("[CLIENT-HTTP] Failed to parse request body");
        Value::Null
    })
}

/// Extracts a number from a JSON Value
/// 
/// ### Arguments
/// * `json` - The JSON object to parse
/// * `field` - The field name to extract
/// 
/// Returns an Option containing the extracted number as u8
pub(crate) fn get_number_from_json(json: &Value, field: &str) -> Option<u8> {
    match &json[field] {
        Number(num) => num.as_u64().map(|value| value as u8),
        _ => None,
    }
}

/// Extracts a string from a JSON Value
/// 
/// ### Arguments
/// * `json` - The JSON object to parse
/// * `field` - The field name to extract
/// 
/// Returns an Option containing the extracted string
pub(crate) fn get_string_from_json(json: &Value, field: &str) -> Option<String> {
    match &json[field] {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// Extracts message content from a JSON Value
/// 
/// ### Arguments
/// * `json` - The JSON object containing the message content
/// 
/// Returns an Option containing the parsed MessageContent
pub(crate) fn get_content_from_msg(json: &Value) -> Option<MessageContent> {
    serde_json::from_value(json["content"].clone()).ok()
}
