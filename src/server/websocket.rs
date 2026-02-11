use axum::extract::ws::Message;
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use tokio::sync::broadcast;

pub async fn handler(ws: WebSocketUpgrade, tx: broadcast::Sender<()>) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        let mut rx = tx.subscribe();
        while rx.recv().await.is_ok() {
            if socket.send(Message::Text("reload".into())).await.is_err() {
                break;
            }
        }
    })
}
