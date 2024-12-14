use rocket::fs::FileServer;
use rocket::response::stream::{Event, EventStream};
use rocket::{get, post, routes, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use wg_2024::network::NodeId;

#[derive(Clone, Serialize, Deserialize)]
struct Message {
    sent_by: NodeId,
    content: String,
    timestamp: String,
}

#[rocket::main]
pub(crate) async fn setup_ui() {
    // let (tx, _) = tokio::sync::broadcast::channel::<String>(100);
    let _ = rocket::build()
        // .manage(Arc::new(tx))
        .mount("/", routes![send_message_to])
        .mount("/", FileServer::from("static"));
}

// post -> body of request { message: String, dst: NodeId }
#[post("/send_message_to", data = "<message>")]
async fn send_message_to(message: String) -> &'static str {
    "bella"
}

#[get("/notify")]
fn stream(rx: &State<Arc<Sender<Message>>>) -> EventStream![] {
    let mut rx1 = rx.subscribe();

    EventStream! {
        while let Ok(msg) = rx1.recv().await {
            yield Event::json(&msg);
        }
    }
}
