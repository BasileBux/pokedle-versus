use pokedle_versus::db_builder::build_database;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <data_directory> <output_database>", args[0]);
        eprintln!("Example: {} ./data pokedle.db", args[0]);
        std::process::exit(1);
    }

    let data_dir = Path::new(&args[1]);
    let db_path = Path::new(&args[2]);

    if !data_dir.exists() {
        eprintln!(
            "Error: Data directory '{}' does not exist",
            data_dir.display()
        );
        std::process::exit(1);
    }

    if !data_dir.is_dir() {
        eprintln!("Error: '{}' is not a directory", data_dir.display());
        std::process::exit(1);
    }

    match build_database(data_dir, db_path).await {
        Ok(_) => println!("Database built successfully!"),
        Err(e) => {
            eprintln!("Error building database: {:?}", e);
            std::process::exit(1);
        }
    }
}
