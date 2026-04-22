use clap::{Parser, Subcommand};
use atlas_registry::{AtlasRegistry, EntityType};
use atlas_trade::TradeEscrow;

#[derive(Parser)]
#[command(name = "atlas")]
#[command(about = "Atlas-OS Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Registry operations (.usdc domains)
    Registry {
        #[command(subcommand)]
        action: RegistryAction,
    },
    /// Trade operations (Escrows and Triggers)
    Trade {
        #[command(subcommand)]
        action: TradeAction,
    },
}

#[derive(Subcommand)]
enum RegistryAction {
    /// Register a new .usdc domain
    Register {
        domain: String,
        owner: String,
        target: String,
        #[arg(short, long)]
        entity: String, // government, corporation, vessel, individual
    },
    /// Resolve a .usdc domain to an address
    Resolve {
        domain: String,
    },
}

#[derive(Subcommand)]
enum TradeAction {
    /// Trigger an escrow release based on GPS coordinates
    Trigger {
        trade_id: String,
        lat: f32,
        lon: f32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Registry { action } => match action {
            RegistryAction::Register { domain, owner, target, entity } => {
                let mut registry = AtlasRegistry::new();
                let entity_type = match entity.to_lowercase().as_str() {
                    "government" => EntityType::Government,
                    "corporation" => EntityType::Corporation,
                    "vessel" => EntityType::Vessel,
                    _ => EntityType::Individual,
                };

                match registry.register(&domain, &owner, &target, entity_type) {
                    Ok(_) => println!("SUCCESS: Registered {} to {}", domain, target),
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            }
            RegistryAction::Resolve { domain } => {
                // In a real scenario, this would call a persistent registry or gRPC server.
                // For the SDK demo, we'll simulate the lookup.
                println!("RESOLVING: {}...", domain);
                if domain.ends_with(".usdc") {
                    println!("FOUND: 0x71C7656EC7ab88b098defB751B7401B5f6d8976F (Simulated)");
                } else {
                    println!("NOT_FOUND: Domain must end with .usdc");
                }
            }
        },
        Commands::Trade { action } => match action {
            TradeAction::Trigger { trade_id, lat, lon } => {
                // Instantiate a mock escrow for the demo
                let mut escrow = TradeEscrow::new(
                    &trade_id, 
                    1_000_000.0, 
                    "vessel-alpha.usdc", 
                    51.9225, 4.4791 // Port of Rotterdam
                );

                println!("TRIGGERING: Release for trade {} at ({}, {})...", trade_id, lat, lon);
                match escrow.release_on_arrival(lat, lon) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("FAILED: {}", e),
                }
            }
        },
    }
}
