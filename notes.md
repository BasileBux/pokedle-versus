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

## Game building logic

Two screens: 
- menu to create game but also lets the user select some settings for the game
- actual game screen

You should only be able to join a game from a link. Each game will have a unique id that is the thing shared in the link and allows to know in which game to put you.

A game has an id and each player also has a uid. These are used for reconnecting to the game if the page refreshes or anything. So we will use websockets to send messages to specific games and specific players in those games. And identify connections with the uid so that each websocket connection is associated with a specific player in a specific game.

When you join a game either through creation or link, it checks if you already have a uid for this game. So it will check local storage and if it finds one, check if it's in the game and if successful, associate this uid to the new connection. If the server doesn't find it in the specific game, it will create a new player on the server and associate the new uid to the game and the connection.

Also, when we disconnect, it should not remove the players uid but mark it as disconnected. If we don't do that and associate new connections constantly to the same uid, we may have multiple clients who believe that they are the same player when only one is connected.

Now, the websocket connections are set and identified correctly. We will use web sockets from now on with each time the game id and the uid as part of the message so that the server can know who does what when. 

## Game flow

Player switch on either timeout or guess:
A player switch sends a message to all clients in the game with the new player which turn it is and also maybe the guess.
Communications during the game are rather trivial as there are two things that trigger a turn switch and then it just restarts by sending a message to everyone so that they all keep synced and know the same things.
