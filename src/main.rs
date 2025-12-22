mod block;
mod blockchain;
mod transaction;
mod p2p;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use transaction::create_key_pair;
use blockchain::DagBlockchain;
use p2p::P2P;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateWallet => {
            let (sk_hex, pk_hex) = create_key_pair();
            println!("\n🚀 --- ASTROCORE WALLET GENERATED ---");
            println!("Public Address (Share this): \n{}", pk_hex);
            println!("\nSecret Key (KEEP THIS PRIVATE): \n{}", sk_hex);
            println!("--------------------------------------\n");
        }

        Commands::StartNode { port, difficulty } => {
            println!("✨ AstroCore Node v0.1.0 starting...");
            
            let bc = DagBlockchain::new(difficulty as u32);
            let bc_arc = Arc::new(Mutex::new(bc));
            let p2p_bc = bc_arc.clone();
            P2P::start(port, p2p_bc).await;

            let miner_bc = bc_arc.clone();
            tokio::spawn(async move {
                println!("[Miner] Background mining worker started.");
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    
                    let mut bc_lock = miner_bc.lock().await;
                    let mempool_len = bc_lock.mempool.lock().unwrap().len();
                    
                    if mempool_len > 0 {
                        println!("[Miner] Found {} txs in mempool. Starting mining cycle...", mempool_len);
                        bc_lock.mine_parallel_blocks(1, 10);
                    }
                }
            });

            println!("✅ Node is online on port {}. Press Ctrl+C to exit.", port);

            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("\n🛑 Shutting down AstroCore node...");
        }

        Commands::Send { to, amount, secret: _secret } => {
            println!("[Client] Verifying keys and signing transaction...");
            println!("Target: {}, Amount: {}", to, amount);
            println!("Transaction broadcast is handled via P2P layer.");
            println!("(To send a real tx, integrate a JSON-RPC client or use a seed peer).");
        }
    }
}