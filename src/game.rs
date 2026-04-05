use crate::pokemon::Pokemon;
use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use axum::extract::ws::Message;

type ClientTx = mpsc::UnboundedSender<Message>;

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
    pub clients: DashMap<Uuid, ClientTx>,
    pub whose_turn: Uuid,
    pub guesses: Vec<Guess>,
    pub secret_pokemon: u32,
    pub generations: Vec<u8>,
}

impl Room {
    pub fn new(generations: Vec<u8>) -> Self {
        Self {
            clients: DashMap::new(),
            whose_turn: Uuid::nil(),
            guesses: Vec::new(),
            secret_pokemon: 0, // TODO: generate random pokemon
            generations,
        }
    }

    pub fn reset(&mut self, generations: Vec<u8>) {
        self.whose_turn = Uuid::nil();
        self.guesses.clear();
        self.secret_pokemon = 0; // TODO: generate random pokemon
        self.generations = generations;
    }
}

pub struct AppState {
    pub rooms: DashMap<Uuid, Room>,
}
