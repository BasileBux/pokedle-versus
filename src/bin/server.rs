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
use futures::{SinkExt, StreamExt};
use pokedle_versus::game::*;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .route("/new-room", get(new_room_handler))
        .route("/check-room", get(check_room_handler))
        .route("/start-game", get(start_game_handler))
        .fallback_service(get_service(ServeDir::new("static")))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// TODO: Add query params for game options (generations, etc)
async fn new_room_handler(State(state): State<Arc<AppState>>) -> Response {
    let room_id = Uuid::new_v4();
    // TODO: replace with actual game options
    state.rooms.insert(room_id.clone(), Room::new(vec![1]));
    println!("Created new room: {}", room_id);
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"room_id": "{}"}}"#, room_id).into())
        .unwrap()
}

// TODO: implement
// WARNING: this should check if a game is not already started in the room
async fn start_game_handler() -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(r#"{"message": "Not implemented yet"}"#.into())
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

    // Store tx in room
    let user_sprite;
    let client_id;
    if let Some(mut room) = state.rooms.get_mut(&room_id)
        && room.whose_turn.is_nil()
    {
        if let Some(exiting_player_id) = player_id
            && let Some(mut client) = room.clients.get_mut(&exiting_player_id)
            && !client.connected
        {
            client_id = exiting_player_id;
            client.tx = tx.clone();
            client.connected = true;
            user_sprite = client.sprite_user_id;
        } else {
            user_sprite = state.get_next_user_id(&mut room);
            client_id = Uuid::new_v4();
            room.clients
                .insert(client_id, Player::new(tx.clone(), user_sprite));
        }

        let player_list = room
            .clients
            .iter()
            .filter(|entry| entry.value().connected)
            .map(|entry| entry.value().sprite_user_id)
            .collect::<Vec<_>>();

        tx.send(Message::Text(
            serde_json::json!({
                "type": "welcome",
                "player_id": client_id.to_string(),
                "sprite_id": user_sprite,
                "players_sprite_ids": player_list,
            })
            .to_string()
            .into(),
        ))
        .ok();

        room.add_to_player_list(user_sprite);
    } else {
        // Room doesn't exist, reject
        let _ = sender.close().await;
        return;
    }

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Some(room) = state.rooms.get(&room_id) {
            if let Some(client) = room.clients.get(&client_id) {
                client
                    .tx
                    .send(Message::Text(format!("{}", msg.to_text().unwrap()).into()))
                    .ok(); // DEBUG: echoing back messages
            }
        }
    }

    let mut is_room_empty = false;
    if let Some(room) = state.rooms.get(&room_id) {
        room.remove_from_player_list(user_sprite);
        if let Some(mut client) = room.clients.get_mut(&client_id) {
            client.connected = false;
        }
        is_room_empty = room.count_connected_players() == 0;
    }
    if is_room_empty {
        state.rooms.remove(&room_id);
    }
}
