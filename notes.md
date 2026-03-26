# Pokedle versus

https://pokeapi.co/docs/v2

A generation corresponds to the set of pokemon that were introduced in a particular generation.

**We need to generate a list of all pokemon in a generation for each generation:**
https://pokeapi.co/api/v2/generation/<n> -> get the list of all pokemon in that generation
Then for each pokemon: https://pokeapi.co/api/v2/pokemon/<id>
Then get: https://pokeapi.co/api/v2/pokemon-species/<id> to get more info

Once this is done, we need to go back over each pokemon family and get the evolution
chain for each pokemon family: https://pokeapi.co/api/v2/evolution-chain/<id>
