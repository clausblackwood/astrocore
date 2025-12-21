use crate::block::Block;
use crate::transaction::{Transaction, TxOutput};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use chrono::Utc;
use tokio::spawn;
use crate::p2p::P2P;

#[derive(Clone)]
struct PrioritizedTx {
    tx: Transaction,
    fee: u64,
}

impl Ord for PrioritizedTx {
    fn cmp(&self, other: &Self) -> Ordering {
        other.fee.cmp(&self.fee)
    }
}

impl PartialOrd for PrioritizedTx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PrioritizedTx {}

impl PartialEq for PrioritizedTx {
    fn eq(&self, other: &Self) -> bool {
        self.fee == other.fee
    }
}

pub struct DagBlockchain {
    pub blocks: Vec<Block>,
    pub tips: Vec<String>,
    pub difficulty: u32,
    pub mempool: Arc<Mutex<BinaryHeap<PrioritizedTx>>>,
    pub utxo_set: Arc<Mutex<HashMap<String, TxOutput>>>,
    pub p2p: Option<Arc<P2P>>,
}

impl DagBlockchain {
    pub fn new(difficulty: u32) -> Self {
        let coinbase_tx = Transaction::new(
            vec![],
            vec![TxOutput {
                to_address: "genesis_miner".to_string(),
                amount: 50_000_000,
            }],
            0,
        );

        let mut genesis = Block::new(0, vec![coinbase_tx], vec![]);
        genesis.mine(difficulty);

        let genesis_hash = genesis.hash.clone();

        let mut utxo_set = HashMap::new();
        utxo_set.insert(
            format!("{}:0", genesis.transactions[0].hash),
            TxOutput {
                to_address: "genesis_miner".to_string(),
                amount: 50_000_000,
            },
        );

        DagBlockchain {
            blocks: vec![genesis],
            tips: vec![genesis_hash],
            difficulty,
            mempool: Arc::new(Mutex::new(BinaryHeap::new())),
            utxo_set: Arc::new(Mutex::new(utxo_set)),
            p2p: None,
        }
    }

    pub fn submit_transaction(&self, tx: Transaction) {
        let prioritized = PrioritizedTx {
            tx: tx.clone(),
            fee: tx.fee,
        };

        let mut mempool = self.mempool.lock().unwrap();
        mempool.push(prioritized);
        println!("[Mempool] Added tx {} (fee: {})", tx.hash, tx.fee);
    }

    pub fn mine_parallel_blocks(&mut self, num_blocks: usize, max_tx_per_block: usize) {
        let current_index = self.blocks.len() as u64;
        let current_tips = self.tips.clone();
        let difficulty = self.difficulty;

        let new_blocks: Vec<Block> = (0..num_blocks)
            .into_par_iter()
            .filter_map(|offset| {
                let mut mempool_guard = self.mempool.lock().unwrap();

                let mut selected_txs: Vec<Transaction> = Vec::new();
                for _ in 0..max_tx_per_block {
                    if let Some(prioritized) = mempool_guard.pop() {
                        selected_txs.push(prioritized.tx);
                    } else {
                        break;
                    }
                }

                if selected_txs.is_empty() {
                    return None;
                }

                let mut new_block = Block::new(
                    current_index + offset as u64,
                    selected_txs,
                    current_tips.clone(),
                );
                new_block.mine(difficulty);

                println!(
                    "[Mining] Mined block #{} with {} txs",
                    new_block.index,
                    new_block.transactions.len()
                );

                Some(new_block)
            })
            .collect();

        for new_block in new_blocks {
            self.tips.retain(|h| !new_block.parents.contains(h));
            self.tips.push(new_block.hash.clone());
            self.blocks.push(new_block.clone());

            if let Some(p2p) = &self.p2p {
                let p2p_clone = p2p.clone();
                let block_clone = new_block.clone();
                spawn(async move {
                    p2p_clone.broadcast_block(&block_clone).await;
                });
            }
        }

        println!(
            "[Mining] Added {} new blocks. Total blocks: {}",
            self.blocks.len() - current_index as usize,
            self.blocks.len()
        );
    }

    pub fn recommend_fee(&self) -> f64 {
        let mempool = self.mempool.lock().unwrap();
        let mempool_size = mempool.len();

        let avg_fee: f64 = if mempool_size > 0 {
            mempool.iter().map(|p| p.fee as f64).sum::<f64>() / mempool_size as f64
        } else {
            50.0
        };

        let last_block_time = if !self.blocks.is_empty() {
            (Utc::now().timestamp() - self.blocks.last().unwrap().timestamp) as f64
        } else {
            10.0
        };

        let base_fee = avg_fee * 1.2;
        let congestion_factor = mempool_size as f64 * 0.5;
        let time_factor = last_block_time * 0.1;

        (base_fee + congestion_factor + time_factor).max(10.0)
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        if !block.hash.starts_with(&"0".repeat(self.difficulty as usize)) {
            println!("[Validation] Invalid PoW for block #{}", block.index);
            return false;
        }

        if block.hash != block.calculate_hash() {
            println!("[Validation] Invalid hash for block #{}", block.index);
            return false;
        }

        if block.timestamp > Utc::now().timestamp() + 60 {
            println!("[Validation] Block timestamp in future: #{}", block.index);
            return false;
        }

        for tx in &block.transactions {
        }

        if self.blocks.iter().any(|b| b.hash == block.hash) {
            println!("[Validation] Block already exists: #{}", block.index);
            return false;
        }

        true
    }

    pub fn add_block(&mut self, block: Block) {
        self.tips.retain(|h| !block.parents.contains(h));
        self.tips.push(block.hash.clone());
        self.blocks.push(block);

        println!("[Sync] Added block #{}", self.blocks.last().unwrap().index);
    }
}