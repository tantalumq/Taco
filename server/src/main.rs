use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::broadcast;

#[allow(warnings, unused)]
mod prisma;

mod auth;
pub(crate) use auth::Session;

mod user;

macro_rules! option_vec {
    [$($x: expr),+ $(,)?] => {
        {
            let mut items = Vec::new();
            $(
                if let Some(x) = $x { items.push(x) }
            )*
            items
        }
    };
}
pub(crate) use option_vec;

#[derive(Clone, Debug)]
pub(crate) struct WsMessage {
    recipient_ids: HashSet<String>,
    data: Value,
}

#[derive(Clone)]
pub(crate) struct AppState {
    client: Arc<prisma::PrismaClient>,
    message_sender: broadcast::Sender<WsMessage>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    const MAX_MESSAGES: usize = 100;

    let (tx, _rx) = broadcast::channel(MAX_MESSAGES);

    let app = Router::new()
        .nest("/", auth::router())
        .nest("/", user::router())
        .route("/ws", get(ws_handler))
        .with_state(AppState {
            client: Arc::new(prisma::new_client().await.unwrap()),
            message_sender: tx,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    State(state): State<AppState>,
    session: Session,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_client(state, session.user_id, socket))
}

async fn handle_client(state: AppState, user_id: String, ws: WebSocket) {
    let (mut sender, mut _receiver) = ws.split();

    let mut message_receiver = state.message_sender.subscribe();

    while let Ok(msg) = message_receiver.recv().await {
        if msg.recipient_ids.contains(&user_id) {
            if sender
                .send(Message::Text(msg.data.to_string()))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}
