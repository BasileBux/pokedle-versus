use pokedle_versus::pokeapi::get_generation;

#[tokio::main]
async fn main() {
    let generation = 1;
    let _ = get_generation(generation).await.unwrap();
}
