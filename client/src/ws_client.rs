use iced::{
    futures::{FutureExt, SinkExt, StreamExt},
    subscription, Subscription,
};
use structs::requests::Session;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, PartialEq)]
pub enum WsEvent {
    Ready,
    Message(String),
}

enum State {
    Starting,
    Ready(WebSocketStream<MaybeTlsStream<TcpStream>>),
}

pub fn connect(session: String) -> Subscription<WsEvent> {
    struct WsClient;
    subscription::channel(
        std::any::TypeId::of::<WsClient>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;

            loop {
                match &mut state {
                    State::Starting => {
                        const WS: &str = "ws://127.0.0.1:3000/ws";
                        let session = format!("Bearer {session}");
                        let mut headers = [httparse::Header {
                            name: "Authorization",
                            value: session.as_bytes(),
                        }];
                        let mut req = httparse::Request {
                            method: Some("GET"),
                            path: Some(WS),
                            version: Some(1),
                            headers: &mut headers,
                        };
                        let (stream, _) = tokio_tungstenite::connect_async(req).await.unwrap();
                        let _ = output.send(WsEvent::Ready).await;
                        state = State::Ready(stream);
                    }
                    State::Ready(stream) => {
                        let mut ws = stream.by_ref().fuse();

                        while let Some(Some(Ok(msg))) = ws.next().now_or_never() {
                            let _ = output
                                .send(WsEvent::Message(msg.to_text().unwrap().into()))
                                .await;
                        }
                    }
                }
            }
        },
    )
}
