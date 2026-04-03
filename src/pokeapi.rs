use std::collections::HashMap;

use crate::pokemon::{Pokemon, PokemonColor, PokemonHabitat, PokemonType};
use reqwest::get;
use serde_json::{Value, from_str};

const POKEAPI_BASE_URL: &str = "https://pokeapi.co/api/v2";
const MIN_GENERATION_POKEMON_COUNT: usize = 100;
const DATA_DIR: &str = "data";

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

    std::fs::create_dir_all(format!("{}/gen_{}_sprites", DATA_DIR, generation)).unwrap();

    let mut pokemons: HashMap<u32, Pokemon> = HashMap::with_capacity(MIN_GENERATION_POKEMON_COUNT);

    for json_simple_pokemon in json["pokemon_species"].as_array().unwrap_or(&vec![]) {
        let name = json_simple_pokemon["name"]
            .as_str()
            .unwrap_or("")
            .to_string()
            .to_lowercase();
        let species_url = json_simple_pokemon["url"].as_str().unwrap_or("");
        let species_id = get_id(species_url);

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

        let habitat = match pokemon_species_json["habitat"].is_null() {
            true => PokemonHabitat::Other,
            false => PokemonHabitat::try_from(get_id(
                pokemon_species_json["habitat"]["url"]
                    .as_str()
                    .unwrap_or(""),
            ))
            .unwrap(),
        };

        let is_baby = pokemon_species_json["is_baby"].as_bool().unwrap_or(false);

        let evolution_chain_id = get_id(
            pokemon_species_json["evolution_chain"]["url"]
                .as_str()
                .unwrap_or(""),
        );

        let pokemon_url = pokemon_species_json["varieties"][0]["pokemon"]["url"]
            .as_str()
            .unwrap_or("");
        let pokemon_id = get_id(pokemon_url);
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
        let sprite_path = format!("{}/gen_{}_sprites/{}.png", DATA_DIR, generation, pokemon_id);
        std::fs::write(sprite_path, sprite).unwrap();

        pokemons.insert(
            pokemon_id,
            Pokemon {
                name,
                french_name,
                species_id,
                pokemon_id,
                evolution_chain_id,
                national_pokedex,
                height,
                weight,
                type1,
                type2,
                color,
                evolution_stage: 0,   // Temporary, will be updated later
                fully_evolved: false, // Temporary, will be updated later
                is_baby,
                habitat,
                generation,
            },
        );
    }

    process_evolution_chains(&mut pokemons).await?;

    let json_output = serde_json::to_string_pretty(&pokemons)
        .unwrap_or_else(|e| panic!("Failed to serialize pokemons to JSON: {}", e));
    let json_path = format!("{}/gen_{}_pokemon.json", DATA_DIR, generation);
    std::fs::write(&json_path, json_output)
        .unwrap_or_else(|e| panic!("Failed to write JSON file {}: {}", json_path, e));

    Ok(())
}

async fn process_evolution_chains(
    pokemons: &mut std::collections::HashMap<u32, Pokemon>,
) -> Result<(), reqwest::Error> {
    use std::collections::{HashMap, HashSet};

    let mut evolution_info_map: HashMap<u32, (u8, bool)> = HashMap::new();

    let mut processed_chains: HashSet<u32> = HashSet::new();

    for pokemon in pokemons.values() {
        if pokemon.evolution_chain_id > 0 && !processed_chains.contains(&pokemon.evolution_chain_id)
        {
            let url = format!(
                "{}/evolution-chain/{}",
                POKEAPI_BASE_URL, pokemon.evolution_chain_id
            );
            let response = get(&url).await?.text().await?;
            let evolution_chain_json: Value = from_str(response.as_str()).unwrap_or_else(|_| {
                panic!(
                    "Failed to parse evolution chain JSON for ID {}",
                    pokemon.evolution_chain_id
                )
            });

            process_evolution_node(&evolution_chain_json["chain"], 0, &mut evolution_info_map);

            processed_chains.insert(pokemon.evolution_chain_id);
        }
    }

    for pokemon in pokemons.values_mut() {
        if let Some(&(evolution_stage, fully_evolved)) = evolution_info_map.get(&pokemon.species_id)
        {
            pokemon.evolution_stage = evolution_stage;
            pokemon.fully_evolved = fully_evolved;
        } else if pokemon.evolution_chain_id == 0 {
            pokemon.evolution_stage = 0;
            pokemon.fully_evolved = true;
        } else {
            panic!(
                "Pokemon species {} not found in evolution chain data",
                pokemon.species_id
            );
        }
    }

    Ok(())
}

fn process_evolution_node(
    node: &Value,
    stage: u8,
    evolution_info_map: &mut std::collections::HashMap<u32, (u8, bool)>,
) {
    let species_url = node["species"]["url"].as_str().unwrap_or("");
    let species_id = get_id(species_url);

    let empty_vec = vec![];
    let evolves_to = node["evolves_to"].as_array().unwrap_or(&empty_vec);
    let has_evolutions = !evolves_to.is_empty();

    evolution_info_map.insert(species_id, (stage, !has_evolutions));

    for child_node in evolves_to {
        process_evolution_node(child_node, stage + 1, evolution_info_map);
    }
}
