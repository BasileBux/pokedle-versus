use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{any, get_service},
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ws", any(handler))
        .nest_service("/alice", get_service(ServeDir::new("static/alice")))
        .nest_service("/bob", get_service(ServeDir::new("static/bob")));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket).into_response()
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("recv error: {e}");
                return;
            }
        };

        match msg {
            Message::Text(txt) => {
                println!("got text: {}", txt);
                // echo it back
                if socket.send(Message::Text(txt)).await.is_err() {
                    return;
                }
            }
            Message::Binary(bin) => {
                println!("got {} bytes", bin.len());
                if socket.send(Message::Binary(bin)).await.is_err() {
                    return;
                }
            }
            Message::Close(_) => {
                println!("client initiated close");
                return;
            }
            Message::Ping(_) | Message::Pong(_) => {
                // handled automatically by axum/tokio-tungstenite
            }
        }
    }
}
