#[repr(u8)]
pub enum PokemonType {
    Normal = 1,
    Fighting = 2,
    Flying = 3,
    Poison = 4,
    Ground = 5,
    Rock = 6,
    Bug = 7,
    Ghost = 8,
    Steel = 9,
    Fire = 10,
    Water = 11,
    Grass = 12,
    Electric = 13,
    Psychic = 14,
    Ice = 15,
    Dragon = 16,
    Dark = 17,
    Fairy = 18,
    Stellar = 19,
    None = 20,
}

impl PokemonType {
    pub fn to_string(&self) -> String {
        match self {
            PokemonType::Normal => "normal".to_string(),
            PokemonType::Fire => "fire".to_string(),
            PokemonType::Water => "water".to_string(),
            PokemonType::Grass => "grass".to_string(),
            PokemonType::Electric => "electric".to_string(),
            PokemonType::Ice => "ice".to_string(),
            PokemonType::Fighting => "fighting".to_string(),
            PokemonType::Poison => "poison".to_string(),
            PokemonType::Ground => "ground".to_string(),
            PokemonType::Flying => "flying".to_string(),
            PokemonType::Psychic => "psychic".to_string(),
            PokemonType::Bug => "bug".to_string(),
            PokemonType::Rock => "rock".to_string(),
            PokemonType::Ghost => "ghost".to_string(),
            PokemonType::Dragon => "dragon".to_string(),
            PokemonType::Dark => "dark".to_string(),
            PokemonType::Steel => "steel".to_string(),
            PokemonType::Fairy => "fairy".to_string(),
            PokemonType::Stellar => "stellar".to_string(),
            PokemonType::None => "none".to_string(),
        }
    }
}

impl TryFrom<u32> for PokemonType {
    type Error = UnknownValue;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(PokemonType::Normal),
            2 => Ok(PokemonType::Fighting),
            3 => Ok(PokemonType::Flying),
            4 => Ok(PokemonType::Poison),
            5 => Ok(PokemonType::Ground),
            6 => Ok(PokemonType::Rock),
            7 => Ok(PokemonType::Bug),
            8 => Ok(PokemonType::Ghost),
            9 => Ok(PokemonType::Steel),
            10 => Ok(PokemonType::Fire),
            11 => Ok(PokemonType::Water),
            12 => Ok(PokemonType::Grass),
            13 => Ok(PokemonType::Electric),
            14 => Ok(PokemonType::Psychic),
            15 => Ok(PokemonType::Ice),
            16 => Ok(PokemonType::Dragon),
            17 => Ok(PokemonType::Dark),
            18 => Ok(PokemonType::Fairy),
            19 => Ok(PokemonType::Stellar),
            20 => Ok(PokemonType::None),
            _ => Err(UnknownValue(v)),
        }
    }
}

// Hard coded values for Pokemon colors based on PokeAPI data
#[repr(u8)]
pub enum PokemonColor {
    Black = 1,
    Blue = 2,
    Brown = 3,
    Gray = 4,
    Green = 5,
    Pink = 6,
    Purple = 7,
    Red = 8,
    White = 9,
    Yellow = 10,
}

#[derive(Debug)]
pub struct UnknownValue(u32);

impl TryFrom<u32> for PokemonColor {
    type Error = UnknownValue;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(PokemonColor::Black),
            2 => Ok(PokemonColor::Blue),
            3 => Ok(PokemonColor::Brown),
            4 => Ok(PokemonColor::Gray),
            5 => Ok(PokemonColor::Green),
            6 => Ok(PokemonColor::Pink),
            7 => Ok(PokemonColor::Purple),
            8 => Ok(PokemonColor::Red),
            9 => Ok(PokemonColor::White),
            10 => Ok(PokemonColor::Yellow),
            _ => Err(UnknownValue(v)),
        }
    }
}

#[repr(u8)]
pub enum PokemonHabitat {
    Cave = 1,
    Forest = 2,
    Grassland = 3,
    Mountain = 4,
    Rare = 5,
    RoughTerrain = 6,
    Sea = 7,
    Urban = 8,
    WatersEdge = 9,
}

impl TryFrom<u32> for PokemonHabitat {
    type Error = UnknownValue;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(PokemonHabitat::Cave),
            2 => Ok(PokemonHabitat::Forest),
            3 => Ok(PokemonHabitat::Grassland),
            4 => Ok(PokemonHabitat::Mountain),
            5 => Ok(PokemonHabitat::Rare),
            6 => Ok(PokemonHabitat::RoughTerrain),
            7 => Ok(PokemonHabitat::Sea),
            8 => Ok(PokemonHabitat::Urban),
            9 => Ok(PokemonHabitat::WatersEdge),
            _ => Err(UnknownValue(v)),
        }
    }
}

impl PokemonColor {
    pub fn to_string(&self) -> String {
        match self {
            PokemonColor::Red => "red".to_string(),
            PokemonColor::Blue => "blue".to_string(),
            PokemonColor::Yellow => "yellow".to_string(),
            PokemonColor::Green => "green".to_string(),
            PokemonColor::Black => "black".to_string(),
            PokemonColor::White => "white".to_string(),
            PokemonColor::Brown => "brown".to_string(),
            PokemonColor::Purple => "purple".to_string(),
            PokemonColor::Pink => "pink".to_string(),
            PokemonColor::Gray => "gray".to_string(),
        }
    }
}

// TODO: align better
pub struct Pokemon {
    pub name: String,
    pub french_name: String,
    pub id: u32,
    pub evolution_chain_id: u32,
    pub national_pokedex: u32,
    pub height: u32,
    pub weight: u32,
    pub type1: PokemonType,
    pub type2: PokemonType,
    pub color: PokemonColor,
    pub evolution_stage: u8,
    pub fully_evolved: bool,
    pub habitat: PokemonHabitat,
    pub generation: u8,
}
