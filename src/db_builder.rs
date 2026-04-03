use crate::pokemon::Pokemon;
use serde_json;
use sqlx::sqlite::{Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Sql(sqlx::Error),
}

impl From<std::io::Error> for DbError {
    fn from(err: std::io::Error) -> Self {
        DbError::Io(err)
    }
}

impl From<serde_json::Error> for DbError {
    fn from(err: serde_json::Error) -> Self {
        DbError::Json(err)
    }
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        DbError::Sql(err)
    }
}

async fn create_schema(conn: &sqlx::Pool<Sqlite>) -> Result<(), DbError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pokemons (
            pokemon_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            french_name TEXT NOT NULL,
            height INTEGER NOT NULL,
            weight INTEGER NOT NULL,
            type1 INTEGER NOT NULL,
            type2 INTEGER NOT NULL,
            color INTEGER NOT NULL,
            evolution_stage INTEGER NOT NULL,
            fully_evolved BOOLEAN NOT NULL,
            is_baby BOOLEAN NOT NULL,
            habitat INTEGER NOT NULL,
            generation INTEGER NOT NULL,
            species_id INTEGER NOT NULL,
            evolution_chain_id INTEGER NOT NULL,
            national_pokedex INTEGER NOT NULL
        )",
    )
    .execute(conn)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pokemon_name ON pokemons(name)")
        .execute(conn)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pokemon_french_name ON pokemons(french_name)")
        .execute(conn)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pokemon_generation ON pokemons(generation)")
        .execute(conn)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_pokemon_order ON pokemons(generation, national_pokedex)",
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn load_json_files(data_dir: &Path) -> Result<Vec<Pokemon>, DbError> {
    let mut all_pokemons = Vec::new();

    for entry in std::fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            println!("Loading {:?}", path.file_name().unwrap());

            let content = std::fs::read_to_string(&path)?;
            let pokemons: HashMap<u32, Pokemon> = serde_json::from_str(&content)?;

            for pokemon in pokemons.values() {
                all_pokemons.push(pokemon.clone());
            }
        }
    }

    Ok(all_pokemons)
}

async fn insert_pokemons(conn: &sqlx::Pool<Sqlite>, pokemons: &[Pokemon]) -> Result<(), DbError> {
    for pokemon in pokemons {
        sqlx::query(
            "INSERT INTO pokemons (
                pokemon_id, name, french_name, height, weight, type1, type2,
                color, evolution_stage, fully_evolved, is_baby, habitat, generation,
                species_id, evolution_chain_id, national_pokedex
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(pokemon.pokemon_id as i64)
        .bind(&pokemon.name)
        .bind(&pokemon.french_name)
        .bind(pokemon.height as i64)
        .bind(pokemon.weight as i64)
        .bind(pokemon.type1 as i64)
        .bind(pokemon.type2 as i64)
        .bind(pokemon.color as i64)
        .bind(pokemon.evolution_stage as i64)
        .bind(pokemon.fully_evolved)
        .bind(pokemon.is_baby)
        .bind(pokemon.habitat as i64)
        .bind(pokemon.generation as i64)
        .bind(pokemon.species_id as i64)
        .bind(pokemon.evolution_chain_id as i64)
        .bind(pokemon.national_pokedex as i64)
        .execute(conn)
        .await?;
    }

    Ok(())
}

pub async fn build_database(data_dir: &Path, db_path: &Path) -> Result<(), DbError> {
    println!("Creating database at {:?}", db_path);

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::File::create(db_path)?;

    let db_url = format!("sqlite:{}", db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;

    println!("Creating schema...");
    create_schema(&pool).await?;

    println!("Loading JSON files from {:?}", data_dir);
    let pokemons: Vec<Pokemon> = load_json_files(data_dir).await?;

    println!("Loaded {} Pokemon from JSON files", pokemons.len());

    println!("Inserting into database...");
    insert_pokemons(&pool, &pokemons).await?;

    println!("Database created successfully!");

    pool.close().await;
    Ok(())
}
