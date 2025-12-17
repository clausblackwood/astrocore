use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use std::fmt;
use hex;
use serde_json;
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

        let parents_str = serde_json::to_string(&self.parents).unwrap_or("[]".to_string());
        let transactions_str = serde_json::to_string(&self.transactions).unwrap_or("[]".to_string());

        hasher.update(format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp,
            transactions_str,
            parents_str,
            self.nonce
        ));

        hex::encode(hasher.finalize())
    }

    pub fn mine(&mut self, difficulty: u32) {
        let target = "0".repeat(difficulty as usize);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block {} [hash: {}... nonce: {} parents: {} txs: {}]",
            self.index,
            &self.hash[..8],
            self.nonce,
            self.parents.len(),
            self.transactions.len()
        )
    }
}