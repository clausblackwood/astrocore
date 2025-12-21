mod block;
mod blockchain;
mod transaction;
mod p2p;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use transaction::create_key_pair;
use blockchain::DagBlockchain;
use p2p::P2P; // Import the P2P struct
use std::sync::Arc;
use tokio::sync::Mutex;
use secp256k1::SecretKey;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateWallet => {
            let (sk_hex, pk_hex) = create_key_pair();
            println!("--- NEW WALLET GENERATED ---");
            println!("Public Address: {}", pk_hex);
            println!("Secret Key:     {}", sk_hex);
            println!("----------------------------");
        }

        Commands::StartNode { port, difficulty } => {
            println!("[Node] Initializing DAG Blockchain...");
            // Fix: convert difficulty usize to u32
            let bc = DagBlockchain::new(difficulty as u32);
            let bc_arc = Arc::new(Mutex::new(bc));

            println!("[Node] Starting P2P server on port {}...", port);
            // Fix: call P2P::start (it's an associated function)
            P2P::start(port, bc_arc).await;
        }

        Commands::Send { to, amount, secret } => {
            match SecretKey::from_str(&secret) {
                Ok(_) => {
                    println!("[Client] Preparing to send {} to {}", amount, to);
                    println!("[Client] Success! Transaction signed and broadcasted (simulated).");
                }
                Err(_) => println!("[Error] Invalid Secret Key! Please provide a 64-character hex string."),
            }
        }
    }
}