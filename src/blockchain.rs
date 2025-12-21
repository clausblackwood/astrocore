use crate::block::Block;
use crate::transaction::{Transaction, TxOutput};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct DagBlockchain {
    pub blocks: Vec<Block>,
    pub tips: Vec<String>,
    pub difficulty: u32,
    mempool: Arc<Mutex<BinaryHeap<PrioritizedTx>>>,
    #[allow(dead_code)]
    pub utxo_set: Arc<Mutex<HashMap<String, TxOutput>>>,
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
        }
    }
    #[allow(dead_code)]
    pub fn submit_transaction(&self, tx: Transaction) {
        let prioritized = PrioritizedTx {
            tx: tx.clone(),
            fee: tx.fee,
        };

        let mut mempool = self.mempool.lock().unwrap();
        mempool.push(prioritized);
        println!("[Mempool] Added tx {} (fee: {})", tx.hash, tx.fee);
    }
    #[allow(dead_code)]
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
            self.blocks.push(new_block);
        }

        println!(
            "[Mining] Added {} new blocks. Total blocks: {}",
            self.blocks.len() - current_index as usize,
            self.blocks.len()
        );
    }
}