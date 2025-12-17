mod block;
mod blockchain;
mod transaction;
mod p2p;

use blockchain::DagBlockchain;
use p2p::P2P;
use transaction::{create_key_pair, Transaction, TxInput, TxOutput};
use secp256k1::SecretKey;
use env_logger;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut bc = DagBlockchain::new(3);

    let (_sk1, pk1) = create_key_pair();
    let (_sk2, pk2) = create_key_pair();

    println!("[Wallet] Generated addresses:");
    println!("  Wallet 1: {}", pk1);
    println!("  Wallet 2: {}", pk2);

    let coinbase_hash = bc.blocks[0].transactions[0].hash.clone();

    let mut tx1 = Transaction::new(
        vec![TxInput {
            prev_tx_hash: coinbase_hash.clone(),
            output_index: 0,
            signature: vec![],
        }],
        vec![TxOutput {
            to_address: pk2.clone(),
            amount: 1000,
        }],
        50,
    );

    let mut tx2 = Transaction::new(
        vec![TxInput {
            prev_tx_hash: coinbase_hash.clone(),
            output_index: 0,
            signature: vec![],
        }],
        vec![TxOutput {
            to_address: pk1.clone(),
            amount: 2000,
        }],
        30,
    );

    let mut tx3 = Transaction::new(
        vec![TxInput {
            prev_tx_hash: coinbase_hash.clone(),
            output_index: 0,
            signature: vec![],
        }],
        vec![TxOutput {
            to_address: pk2,
            amount: 500,
        }],
        100,
    );

    let dummy_key = SecretKey::from_slice(&[0xcd; 32]).expect("32 bytes");
    let _ = tx1.sign(&dummy_key);
    let _ = tx2.sign(&dummy_key);
    let _ = tx3.sign(&dummy_key);

    bc.submit_transaction(tx1);
    bc.submit_transaction(tx2);
    bc.submit_transaction(tx3);

    println!("\n[Node] 3 transactions submitted to mempool. Starting mining...\n");

    bc.mine_parallel_blocks(6, 3);

    println!("\n=== Node ready ===");
    println!("Blocks in DAG: {}", bc.blocks.len());
    println!(
        "Confirmed transactions (including coinbase): {}",
        bc.blocks.iter().map(|b| b.transactions.len()).sum::<usize>()
    );
    println!("Current DAG tips: {}", bc.tips.len());

    let bc_arc = Arc::new(Mutex::new(bc));
    P2P::start(4000, bc_arc).await;

    println!("\nP2P node running on port 4000. Run another instance on port 4001 to connect.");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}