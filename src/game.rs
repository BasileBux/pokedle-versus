use crate::pokemon::Pokemon;
use dashmap::DashMap;
use rand::seq::SliceRandom;
use tokio::sync::mpsc;
use uuid::Uuid;

use axum::extract::ws::Message;

const POKEMON_NUMBER: u16 = 1025;

type ClientTx = mpsc::UnboundedSender<Message>;

pub struct Player {
    pub sprite_user_id: u16,
    pub tx: ClientTx,
    pub connected: bool,
}

impl Player {
    pub fn new(tx: ClientTx, sprite_user_id: u16) -> Self {
        Self {
            tx,
            sprite_user_id,
            connected: true,
        }
    }
}

pub enum NumberComparison {
    Higher,
    Lower,
    Equal,
}

pub enum TypeComparison {
    Equal,
    Partial,
    NotEqual,
}

pub enum MultiComparison {
    Equal,
    Partial,
    NotEqual,
}

// TODO: Also add the actual stats of the guessed pokemon
pub struct Guess {
    height: NumberComparison,
    weight: NumberComparison,
    type1: TypeComparison,
    type2: TypeComparison,
    color: MultiComparison,
    evolution_stage: NumberComparison,
    fully_evolved: bool,
    habitat: MultiComparison,
    generation: NumberComparison,
}

impl Guess {
    pub fn build(guess: &Pokemon, secret: &Pokemon) -> Self {
        todo!()
    }
}

pub struct Room {
    pub clients: DashMap<Uuid, Player>,
    pub whose_turn: Uuid,
    pub guesses: Vec<Guess>,
    pub secret_pokemon: u32,
    pub generations: Vec<u8>,
    pub user_id_idx: u16, // Index for assigning user IDs, not the actual user ID
}

impl Room {
    pub fn new(generations: Vec<u8>) -> Self {
        Self {
            clients: DashMap::new(),
            whose_turn: Uuid::nil(),
            guesses: Vec::new(),
            secret_pokemon: 0, // TODO: generate random pokemon
            generations,
            user_id_idx: 0,
        }
    }

    pub fn reset(&mut self, generations: Vec<u8>) {
        self.whose_turn = Uuid::nil();
        self.guesses.clear();
        self.secret_pokemon = 0; // TODO: generate random pokemon
        self.generations = generations;
    }

    pub fn broadcast(&self, message: Message) {
        for client in self.clients.iter() {
            let _ = client.value().tx.send(message.clone());
        }
    }

    pub fn add_to_player_list(&self, player_sprite_id: u16) {
        self.broadcast(Message::Text(
            serde_json::json!({
                "type": "player_joined",
                "sprite_id": player_sprite_id,
            })
            .to_string()
            .into(),
        ));
    }

    pub fn remove_from_player_list(&self, player_sprite_id: u16) {
        self.broadcast(Message::Text(
            serde_json::json!({
                "type": "player_left",
                "sprite_id": player_sprite_id,
            })
            .to_string()
            .into(),
        ));
    }

    pub fn count_connected_players(&self) -> usize {
        self.clients
            .iter()
            .filter(|entry| entry.value().connected)
            .count()
    }
}

pub struct AppState {
    pub rooms: DashMap<Uuid, Room>,
    pub user_id_pool: Vec<u16>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            user_id_pool: {
                let mut pool: Vec<u16> = (1..=POKEMON_NUMBER).collect();
                pool.shuffle(&mut rand::rng());
                pool
            },
        }
    }

    pub fn get_next_user_id(&self, room: &mut Room) -> u16 {
        room.user_id_idx += 1;
        room.user_id_idx %= self.user_id_pool.len() as u16;
        self.user_id_pool[room.user_id_idx as usize]
    }

    pub fn randomize_user_id_pool(&mut self) {
        self.user_id_pool.shuffle(&mut rand::rng());
    }
}
