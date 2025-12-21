use secp256k1::{Secp256k1, SecretKey, Message, Error};
use secp256k1::rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

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
        let mut tx = Transaction { inputs, outputs, fee, hash: String::new() };
        tx.hash = tx.calculate_hash();
        tx
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&self.inputs).unwrap_or_default());
        hasher.update(serde_json::to_string(&self.outputs).unwrap_or_default());
        hasher.update(self.fee.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<(), Error> {
        let secp = Secp256k1::new();
        let hash_bytes = hex::decode(&self.hash).map_err(|_| Error::InvalidMessage)?;
        let message = Message::from_digest_slice(&hash_bytes).map_err(|_| Error::InvalidMessage)?;
        let sig = secp.sign_ecdsa(&message, secret_key);

        for input in &mut self.inputs {
            input.signature = sig.serialize_der().to_vec();
        }
        Ok(())
    }
}

pub fn create_key_pair() -> (String, String) {
    let secp = Secp256k1::new();
    let (sk, pk) = secp.generate_keypair(&mut OsRng);
    (hex::encode(sk.secret_bytes()), hex::encode(pk.serialize()))
}