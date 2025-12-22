use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use std::fmt;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub parents: Vec<String>,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, parents: Vec<String>) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            parents,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.index.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());

        let mut sorted_parents = self.parents.clone();
        sorted_parents.sort();
        for parent_hash in sorted_parents {
            hasher.update(hex::decode(parent_hash).unwrap_or_default());
        }

        for tx in &self.transactions {
            hasher.update(hex::decode(&tx.hash).unwrap_or_default());
        }

        hex::encode(hasher.finalize())
    }

    pub fn mine(&mut self, difficulty: u32) {
        let target = "0".repeat(difficulty as usize);

        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        
        println!("[Miner] Block mined! Hash: {}", self.hash);
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let short_hash = if self.hash.len() > 8 { &self.hash[..8] } else { "n/a" };
        write!(
            f,
            "Block #{} [Hash: {}... Nonce: {} Parents: {} Txs: {}]",
            self.index,
            short_hash,
            self.nonce,
            self.parents.len(),
            self.transactions.len()
        )
    }
}