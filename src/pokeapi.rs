use std::collections::HashMap;

use crate::pokemon::{Pokemon, PokemonColor, PokemonHabitat, PokemonType};
use reqwest::get;
use serde_json::{Value, from_str};

const POKEAPI_BASE_URL: &str = "https://pokeapi.co/api/v2";
const MIN_GENERATION_POKEMON_COUNT: usize = 100;

fn get_id(url: &str) -> u32 {
    url.trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("")
        .to_string()
        .parse::<u32>()
        .unwrap_or(0)
}

pub async fn build_generation(generation: u8) -> Result<(), reqwest::Error> {
    let url = format!("{}/generation/{}", POKEAPI_BASE_URL, generation);
    let response = get(&url).await?.text().await?;
    let json: Value =
        from_str(response.as_str()).unwrap_or_else(|_| panic!("Failed to parse JSON response"));

    std::fs::create_dir_all(format!("gen_{}_sprites", generation)).unwrap();

    let mut pokemons: HashMap<u32, Pokemon> = HashMap::with_capacity(MIN_GENERATION_POKEMON_COUNT);

    for json_simple_pokemon in json["pokemon_species"].as_array().unwrap_or(&vec![]) {
        let name = json_simple_pokemon["name"]
            .as_str()
            .unwrap_or("")
            .to_string()
            .to_lowercase();
        let species_url = json_simple_pokemon["url"].as_str().unwrap_or("");
        let id = get_id(species_url);

        let response = get(species_url).await?.text().await?;
        let pokemon_species_json: Value =
            from_str(response.as_str()).unwrap_or_else(|_| panic!("Failed to parse JSON response"));

        let national_pokedex = pokemon_species_json["pokedex_numbers"][0]["entry_number"]
            .as_u64()
            .unwrap_or(0) as u32;
        let color = PokemonColor::try_from(get_id(
            pokemon_species_json["color"]["url"].as_str().unwrap_or(""),
        ))
        .unwrap();

        let french_name = pokemon_species_json["names"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .find(|name| name["language"]["name"].as_str().unwrap_or("") == "fr")
            .and_then(|name| name["name"].as_str())
            .unwrap_or("")
            .to_string()
            .to_lowercase();

        let habitat = PokemonHabitat::try_from(get_id(
            pokemon_species_json["habitat"]["url"]
                .as_str()
                .unwrap_or(""),
        ))
        .unwrap();

        let evolution_chain_id = get_id(
            pokemon_species_json["evolution_chain"]["url"]
                .as_str()
                .unwrap_or(""),
        );

        let pokemon_url = pokemon_species_json["varieties"][0]["pokemon"]["url"]
            .as_str()
            .unwrap_or("");
        let response = get(pokemon_url).await?.text().await?;
        let pokemon_json: Value =
            from_str(response.as_str()).unwrap_or_else(|_| panic!("Failed to parse JSON response"));
        let weight = pokemon_json["weight"].as_u64().unwrap_or(0) as u32;
        let height = pokemon_json["height"].as_u64().unwrap_or(0) as u32;
        let type1 = PokemonType::try_from(get_id(
            pokemon_json["types"][0]["type"]["url"]
                .as_str()
                .unwrap_or(""),
        ))
        .unwrap();
        let type2 = if pokemon_json["types"].as_array().unwrap_or(&vec![]).len() > 1 {
            PokemonType::try_from(get_id(
                pokemon_json["types"][1]["type"]["url"]
                    .as_str()
                    .unwrap_or(""),
            ))
            .unwrap()
        } else {
            PokemonType::None
        };

        let sprite_url = pokemon_json["sprites"]["front_default"]
            .as_str()
            .unwrap_or("");
        let sprite = get(sprite_url).await?.bytes().await?;
        let sprite_path = format!("gen_{}_sprites/{}.png", generation, id);
        std::fs::write(sprite_path, sprite).unwrap();

        pokemons.insert(
            id,
            Pokemon {
                name,
                french_name,
                id,
                evolution_chain_id,
                national_pokedex,
                height,
                weight,
                type1,
                type2,
                color,
                evolution_stage: 0,   // /evolution-chain
                fully_evolved: false, // /evolution-chain
                habitat,
                generation,
            },
        );
    }

    // TODO: iterate over the pokemons HashMap and if evolution_stage == 0, fetch
    // the page and also set the evolution_stage for all pokemons in the evolution chain
    for pokemon in pokemons.values() {
        if pokemon.evolution_stage == 0 {
            let url = format!(
                "{}/evolution-chain/{}",
                POKEAPI_BASE_URL, pokemon.evolution_chain_id
            );
            let response = get(&url).await?.text().await?;
            let evolution_chain_json: Value = from_str(response.as_str())
                .unwrap_or_else(|_| panic!("Failed to parse JSON response"));
        }
    }

    Ok(())
}
