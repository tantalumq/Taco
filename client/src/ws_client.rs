use std::time::Duration;

use iced::{
    futures::{FutureExt, SinkExt, StreamExt},
    subscription, Subscription,
};
use structs::requests::WsMessageData;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
    MaybeTlsStream, WebSocketStream,
};

#[derive(Debug, Clone, PartialEq)]
pub enum WsEvent {
    Ready,
    Message(WsMessageData),
}

enum State {
    Starting,
    Ready(WebSocketStream<MaybeTlsStream<TcpStream>>),
}

pub fn connect(session: String) -> Subscription<WsEvent> {
    subscription::unfold(
        "get websocket messages",
        State::Starting,
        move |mut s| async move {
            match &mut s {
                State::Starting => {
                    const WS: &str = "ws://127.0.0.1:3000/ws";
                    let session = format!("Bearer {session}");
                    let mut request = WS.into_client_request().unwrap();
                    request
                        .headers_mut()
                        .insert("Authorization", HeaderValue::from_str(&session).unwrap());
                    let (stream, _) = tokio_tungstenite::connect_async(request).await.unwrap();
                    (WsEvent::Ready, State::Ready(stream))
                }
                State::Ready(stream) => {
                    let mut ws = stream.by_ref().fuse();

                    let msg = ws.select_next_some().await;
                    if let Ok(msg) = msg {
                        (
                            WsEvent::Message(serde_json::from_str(msg.to_text().unwrap()).unwrap()),
                            s,
                        )
                    } else {
                        (WsEvent::Ready, s)
                    }
                }
            }
        },
    )
}
