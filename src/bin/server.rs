use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Response},
    routing::{any, get, get_service},
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use pokedle_versus::game::*;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        rooms: DashMap::new(),
    });

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .route("/new-room", get(new_room_handler))
        .route("/check-room", get(check_room_handler))
        .nest_service("/room", get_service(ServeDir::new("static/game")))
        .fallback_service(get_service(ServeDir::new("static/menu")))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// TODO: Add query params for game options (generations, etc)
async fn new_room_handler(State(state): State<Arc<AppState>>) -> Response {
    let room_id = Uuid::new_v4();
    state
        .rooms
        .insert(room_id.clone(), Room::new(vec![1, 2, 3, 4, 5]));
    println!("Created new room: {}", room_id);
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"room_id": "{}"}}"#, room_id).into())
        .unwrap()
}

async fn check_room_handler(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if let Some(room_id_str) = params.get("room_id") {
        if let Ok(room_id) = Uuid::from_str(room_id_str) {
            let exists = state.rooms.contains_key(&room_id);
            return Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(format!(r#"{{"exists": {}}}"#, exists).into())
                .unwrap();
        }
    }
    Response::builder()
        .status(400)
        .header("Content-Type", "application/json")
        .body(r#"{"error": "Invalid room_id"}"#.into())
        .unwrap()
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
                    .send(Message::Text(format!("{}", msg.to_text().unwrap()).into()))
                    .ok();
            }
        }
    }

    // Cleanup on disconnect
    // TODO: maybe not remove client immediately, but mark as disconnected and allow them to
    // reconnect for a short time?
    if let Some(room) = state.rooms.get(&room_id) {
        room.clients.remove(&player_id);
    }

    // TODO: remove room if all connected clients are gone
}
