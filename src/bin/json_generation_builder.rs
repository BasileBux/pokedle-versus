use pokedle_versus::pokeapi::build_generation;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <generation>", args[0]);
        eprintln!("Example: {} 1", args[0]);
        std::process::exit(1);
    }

    let generation = args[1].parse::<u8>().unwrap_or_else(|_| {
        eprintln!("Error: Generation must be a number between 1 and 9");
        std::process::exit(1);
    });
    build_generation(generation).await.unwrap();
}
