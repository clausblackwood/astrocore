use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, Error};
use secp256k1::ecdsa::Signature;
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
    pub sender_public_key: String,
    pub hash: String,
}

impl Transaction {
    pub fn new(
        inputs: Vec<TxInput>, 
        outputs: Vec<TxOutput>, 
        fee: u64, 
        sender_public_key: String
    ) -> Self {
        let mut tx = Transaction {
            inputs,
            outputs,
            fee,
            sender_public_key,
            hash: String::new(),
        };
        tx.hash = tx.calculate_hash();
        tx
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();

        for input in &self.inputs {
            hasher.update(hex::decode(&input.prev_tx_hash).unwrap_or_default());
            hasher.update(input.output_index.to_le_bytes());
        }

        for output in &self.outputs {
            hasher.update(hex::decode(&output.to_address).unwrap_or_default());
            hasher.update(output.amount.to_le_bytes());
        }

        hasher.update(self.fee.to_le_bytes());
        hasher.update(hex::decode(&self.sender_public_key).unwrap_or_default());

        hex::encode(hasher.finalize())
    }
    #[allow(dead_code)]
    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<(), Error> {
        let secp = Secp256k1::new();
        let hash_bytes = hex::decode(&self.hash).map_err(|_| Error::InvalidMessage)?;
        let message = Message::from_digest_slice(&hash_bytes).map_err(|_| Error::InvalidMessage)?;
        
        let sig = secp.sign_ecdsa(&message, secret_key);
        let serialized_sig = sig.serialize_der().to_vec();

        for input in &mut self.inputs {
            input.signature = serialized_sig.clone();
        }
        Ok(())
    }

    pub fn verify(&self) -> bool {
        if self.hash != self.calculate_hash() {
            return false;
        }

        if !self.inputs.is_empty() {
            let secp = Secp256k1::new();
            
            let pub_key_bytes = match hex::decode(&self.sender_public_key) {
                Ok(b) => b,
                Err(_) => return false,
            };
            let public_key = match PublicKey::from_slice(&pub_key_bytes) {
                Ok(pk) => pk,
                Err(_) => return false,
            };

            let hash_bytes = match hex::decode(&self.hash) {
                Ok(b) => b,
                Err(_) => return false,
            };
            let message = match Message::from_digest_slice(&hash_bytes) {
                Ok(m) => m,
                Err(_) => return false,
            };

            for input in &self.inputs {
                let sig = match Signature::from_der(&input.signature) {
                    Ok(s) => s,
                    Err(_) => return false,
                };
                
                if secp.verify_ecdsa(&message, &sig, &public_key).is_err() {
                    return false;
                }
            }
        }

        true
    }
}

pub fn create_key_pair() -> (String, String) {
    let secp = Secp256k1::new();
    let (sk, pk) = secp.generate_keypair(&mut OsRng);
    (
        hex::encode(sk.secret_bytes()), 
        hex::encode(pk.serialize())
    )
}