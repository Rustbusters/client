use serde_json::Value;
use serde_json::Value::Number;

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

pub(crate) fn get_number_from_json(json: &Value, field: &str) -> Option<u8> {
    match &json[field] {
        Number(num) => num.as_u64().map(|value| value as u8),
        _ => None,
    }
}

pub(crate) fn get_string_from_json(json: &Value, field: &str) -> Option<String> {
    match &json[field] {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

pub(crate) enum Content {
    String(String),
    Bytes(Vec<u8>),
}

pub(crate) fn get_content_from_json(json: &Value) -> Option<Content> {
    match &json["content"] {
        Value::String(s) => Some(Content::String(s.clone())),
        Value::Array(bytes) => {
            let mut vec = Vec::new();
            for byte in bytes {
                if let Number(num) = byte {
                    vec.push(num.as_u64()? as u8);
                } else {
                    return None;
                }
            }
            Some(Content::Bytes(vec))
        }
        _ => None,
    }
}
