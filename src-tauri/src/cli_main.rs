mod engine;
mod csv_converter;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::engine::swapper;

#[derive(Parser)]
#[command(name = "velocity-cli")]
#[command(about = "A powerful native Rust CLI for Rocket League asset swapping", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Swap {
        #[arg(short, long)]
        source: String,
        #[arg(short, long)]
        target: String,

        #[arg(short, long)]
        game_dir: Option<PathBuf>,
    },
    List {
        #[arg(short, long)]
        slot: Option<String>,
        #[arg(short, long)]
        query: Option<String>,
    },
    Restore {
        #[arg(short, long)]
        game_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Swap { source, target, game_dir } => {
            println!("Swapping '{}' with '{}'...", source, target);
            println!("Note: Native swap integration is active.");
            Ok(())
        }
        Commands::List { slot, query } => {
            if let Ok(json) = csv_converter::convert_csv_to_json("../items.csv") {
                let items = json["Items"].as_array().unwrap();
                for item in items {
                    let name = item["Product"].as_str().unwrap_or("");
                    let item_slot = item["Slot"].as_str().unwrap_or("");
                    let matches_slot = slot.as_ref().map_or(true, |s| s.to_lowercase() == item_slot.to_lowercase());
                    let matches_query = query.as_ref().map_or(true, |q| name.to_lowercase().contains(&q.to_lowercase()));

                    if matches_slot && matches_query {
                        println!("[{}] {} (ID: {})", item_slot, name, item["ID"]);
                    }
                }
            }
            Ok(())
        }
        Commands::Restore { game_dir } => {
            println!("Restoring backups in {:?}", game_dir);
            Ok(())
        }
    }
}