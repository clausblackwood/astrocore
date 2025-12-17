use secp256k1::{Secp256k1, SecretKey, Message, Error};
use secp256k1::rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use hex;
use serde_json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxInput {
    pub prev_tx_hash: String,
    pub output_index: usize,
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxOutput {
    pub to_address: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub fee: u64,
    pub hash: String,
}

impl Transaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>, fee: u64) -> Self {
        let mut tx = Transaction {
            inputs,
            outputs,
            fee,
            hash: String::new(),
        };
        tx.hash = tx.calculate_hash();
        tx
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&self.inputs).unwrap_or("[]".to_string()));
        hasher.update(serde_json::to_string(&self.outputs).unwrap_or("[]".to_string()));
        hasher.update(self.fee.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<(), Error> {
        let secp = Secp256k1::new();

        let hash_bytes = hex::decode(&self.hash).expect("Hash should be valid hex");
        let message = Message::from_digest_slice(&hash_bytes)
            .expect("Hash should be exactly 32 bytes");

        let sig = secp.sign_ecdsa(&message, secret_key);

        for input in &mut self.inputs {
            input.signature = sig.serialize_der().to_vec();
        }

        Ok(())
    }
}

pub fn create_key_pair() -> (String, String) {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);
    let secret_hex = hex::encode(secret_key.secret_bytes());
    let pub_hex = hex::encode(public_key.serialize());
    (secret_hex, pub_hex)
}