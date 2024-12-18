use lazy_static::lazy_static;
use rocket::fs::FileServer;
use rocket::response::stream::{Event, EventStream};
use rocket::{get, post, routes, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use wg_2024::network::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MessageSSE {
    sent_by: NodeId,
    content: String,
    timestamp: String,
}

impl MessageSSE {
    pub(crate) fn new(sent_by: NodeId, content: String) -> Self {
        Self {
            sent_by,
            content,
            timestamp: "ACASO".to_string(),
        }
    }
}

// Global channel for SSE messages
lazy_static! {
    pub(crate) static ref MESSAGE_CHANNEL: Sender<MessageSSE> = {
        let (tx, _) = channel(100);
        tx
    };
}

#[rocket::main]
pub(crate) async fn setup_ui() -> Result<(), rocket::Error> {
    rocket::build()
        .manage(Arc::new(MESSAGE_CHANNEL.clone()))
        .mount("/", routes![send_message_to, stream])
        .mount("/", FileServer::from("static"))
        .launch()
        .await?;

    Ok(())
}

// post -> body of request { message: String, dst: NodeId }
#[post("/send-message-to", data = "<message>")]
async fn send_message_to(message: String) -> &'static str {
    MESSAGE_CHANNEL.send(MessageSSE::new(0, message)).unwrap();
    "bella"
}

#[get("/stream")]
fn stream(rx: &State<Arc<Sender<MessageSSE>>>) -> EventStream![] {
    let mut rx1 = rx.subscribe();

    EventStream! {
        while let Ok(msg) = rx1.recv().await {
            yield Event::json(&msg);
        }
    }
}
