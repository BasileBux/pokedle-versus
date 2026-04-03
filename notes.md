# Pokedle versus

https://pokeapi.co/docs/v2

A generation corresponds to the set of pokemon that were introduced in a particular generation.

**We need to generate a list of all pokemon in a generation for each generation:**
https://pokeapi.co/api/v2/generation/<n> -> get the list of all pokemon in that generation
Then for each pokemon: https://pokeapi.co/api/v2/pokemon/<id>
Then get: https://pokeapi.co/api/v2/pokemon-species/<id> to get more info

Once this is done, we need to go back over each pokemon family and get the evolution
chain for each pokemon family: https://pokeapi.co/api/v2/evolution-chain/<id>

```rust
pub struct Pokemon {
    pub name: String, -> /generation
    pub id: u32, -> /generation
    pub national_pokedex: u32, -> /pokemon-species
    pub height: u32, -> /pokemon
    pub weight: u32, -> /pokemon
    pub type1: PokemonType, -> /pokemon
    pub type2: PokemonType, -> /pokemon
    pub color: PokemonColor, -> /pokemon-species
    pub evolution_stage: u8, -> /evolution-chain
    pub fully_evolved: bool, -> /evolution-chain
    pub habitat: PokemonHabitat, -> /pokemon-species
}
sprites -> /pokemon
```
