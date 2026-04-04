use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Response},
    routing::any,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use uuid::Uuid;

type ClientTx = mpsc::UnboundedSender<Message>;

#[derive(Debug)]
struct RoomState {
    clients: DashMap<Uuid, ClientTx>,
    whose_turn: Uuid,
    guesses: Vec<u32>,
}

#[derive(Debug)]
struct AppState {
    rooms: DashMap<Uuid, RoomState>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        rooms: DashMap::new(),
    });
    state.rooms.insert(
        Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
        RoomState {
            clients: DashMap::new(),
            whose_turn: Uuid::nil(),
            guesses: Vec::new(),
        },
    );

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .fallback_service(ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    println!("New WebSocket connection: {:?}", params);
    ws.on_upgrade(move |socket| {
        let player_id = params
            .get("player_id")
            .and_then(|id| Uuid::from_str(id).ok());
        let room_id = Uuid::from_str(params.get("room_id").unwrap()).unwrap();
        handle_socket(socket, state, player_id, room_id)
    })
    .into_response()
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    player_id: Option<Uuid>,
    room_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let player_id = player_id.unwrap_or_else(Uuid::new_v4);

    // Store tx in room
    if let Some(room) = state.rooms.get(&room_id) {
        room.clients.insert(player_id, tx.clone());
    } else {
        // Room doesn't exist, reject
        let _ = sender.close().await;
        return;
    }

    tx.send(Message::Text(
        serde_json::json!({
            "type": "welcome",
            "player_id": player_id.to_string(),
        })
        .to_string()
        .into(),
    ))
    .ok();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Some(room) = state.rooms.get(&room_id) {
            if let Some(client_tx) = room.clients.get(&player_id) {
                client_tx
                    .send(Message::Text(
                        format!("Echo: {:?} - uuid: {}", msg, player_id).into(),
                    ))
                    .ok();
            }
        }
    }

    // Cleanup on disconnect
    if let Some(room) = state.rooms.get(&room_id) {
        room.clients.remove(&player_id);
    }
}
