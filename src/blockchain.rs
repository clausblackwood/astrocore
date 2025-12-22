use crate::block::Block;
use crate::transaction::{Transaction, TxOutput};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct PrioritizedTx {
    pub tx: Transaction,
    pub fee: u64,
}

impl Ord for PrioritizedTx {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fee.cmp(&other.fee)
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
        self.fee == other.fee && self.tx.hash == other.tx.hash
    }
}

pub struct DagBlockchain {
    pub blocks: Vec<Block>,
    pub tips: Vec<String>,
    pub difficulty: u32,
    pub mempool: Arc<Mutex<BinaryHeap<PrioritizedTx>>>,
    pub utxo_set: Arc<Mutex<HashMap<String, TxOutput>>>,
}

impl DagBlockchain {
    pub fn new(difficulty: u32) -> Self {
        let genesis_miner_pk = "0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8".to_string();
        
        let coinbase_tx = Transaction::new(
            vec![],
            vec![TxOutput {
                to_address: genesis_miner_pk.clone(),
                amount: 50_000_000,
            }],
            0,
            "00".repeat(33),
        );

        let mut genesis_block = Block::new(0, vec![coinbase_tx.clone()], vec![]);
        genesis_block.mine(difficulty);

        let mut utxo_set = HashMap::new();
        utxo_set.insert(
            format!("{}:0", coinbase_tx.hash),
            coinbase_tx.outputs[0].clone(),
        );

        DagBlockchain {
            blocks: vec![genesis_block.clone()],
            tips: vec![genesis_block.hash],
            difficulty,
            mempool: Arc::new(Mutex::new(BinaryHeap::new())),
            utxo_set: Arc::new(Mutex::new(utxo_set)),
        }
    }

    pub fn submit_transaction(&self, tx: Transaction) -> Result<(), String> {
        if !tx.verify() {
            return Err("Invalid transaction signature or hash".to_string());
        }

        let mut mempool = self.mempool.lock().unwrap();
        mempool.push(PrioritizedTx {
            fee: tx.fee,
            tx: tx.clone(),
        });
        
        println!("[Mempool] Transaction accepted: {}", tx.hash);
        Ok(())
    }

    pub fn mine_parallel_blocks(&mut self, num_blocks: usize, max_tx_per_block: usize) {
        let current_index = self.blocks.len() as u64;
        let current_tips = self.tips.clone();
        let difficulty = self.difficulty;

        let mut all_selected_txs = Vec::new();
        {
            let mut mempool = self.mempool.lock().unwrap();
            for _ in 0..num_blocks {
                let mut block_txs = Vec::new();
                for _ in 0..max_tx_per_block {
                    if let Some(p_tx) = mempool.pop() {
                        block_txs.push(p_tx.tx);
                    }
                }
                if !block_txs.is_empty() {
                    all_selected_txs.push(block_txs);
                }
            }
        }

        let new_blocks: Vec<Block> = all_selected_txs
            .into_par_iter()
            .enumerate()
            .map(|(i, txs)| {
                let mut block = Block::new(
                    current_index + i as u64,
                    txs,
                    current_tips.clone(),
                );
                block.mine(difficulty);
                block
            })
            .collect();

        for block in new_blocks {
            if self.validate_block(&block) {
                self.add_block(block);
            }
        }
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        let target = "0".repeat(self.difficulty as usize);
        if !block.hash.starts_with(&target) || block.hash != block.calculate_hash() {
            return false;
        }

        let utxos = self.utxo_set.lock().unwrap();
        let mut seen_inputs = HashSet::new();

        for tx in &block.transactions {
            if !tx.verify() { return false; }

            for input in &tx.inputs {
                let utxo_key = format!("{}:{}", input.prev_tx_hash, input.output_index);
                if !utxos.contains_key(&utxo_key) || seen_inputs.contains(&utxo_key) {
                    return false;
                }
                seen_inputs.insert(utxo_key);
            }
        }
        true
    }

    pub fn add_block(&mut self, block: Block) {
        {
            let mut utxos = self.utxo_set.lock().unwrap();
            for tx in &block.transactions {
                for input in &tx.inputs {
                    let key = format!("{}:{}", input.prev_tx_hash, input.output_index);
                    utxos.remove(&key);
                }
                for (i, output) in tx.outputs.iter().enumerate() {
                    let key = format!("{}:{}", tx.hash, i);
                    utxos.insert(key, output.clone());
                }
            }
        }

        self.tips.retain(|h| !block.parents.contains(h));
        self.tips.push(block.hash.clone());
        self.blocks.push(block);
        
        println!("[Chain] Block added to DAG. New Height: {}", self.blocks.len());
    }
}