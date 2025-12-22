use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use crate::block::Block;
use crate::transaction::Transaction;
use crate::blockchain::DagBlockchain;

pub struct P2P {
    pub peers: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>>,
}

impl P2P {
    pub fn new() -> Self {
        P2P {
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(port: u16, bc: Arc<Mutex<DagBlockchain>>) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        println!("[P2P] Server started on port {}", port);

        let p2p = Arc::new(P2P::new());

        let p2p_inbound = p2p.clone();
        let bc_inbound = bc.clone();
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                println!("[P2P] New connection from: {}", addr);
                let shared_stream = Arc::new(Mutex::new(stream));
                p2p_inbound.peers.lock().await.push(shared_stream.clone());
                
                tokio::spawn(handle_peer(shared_stream, bc_inbound.clone(), p2p_inbound.clone()));
            }
        });

        let p2p_outbound = p2p.clone();
        let bc_outbound = bc.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(stream) = TcpStream::connect("127.0.0.1:4001").await {
                    println!("[P2P] Successfully connected to seed peer 4001");
                    let shared_stream = Arc::new(Mutex::new(stream));
                    p2p_outbound.peers.lock().await.push(shared_stream.clone());
                    
                    tokio::spawn(handle_peer(shared_stream, bc_outbound.clone(), p2p_outbound.clone()));
                    break; 
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    }

    pub async fn broadcast_block(&self, block: &Block) {
        let msg = json!({"type": "block", "data": block}).to_string() + "\n";
        self.send_to_all(&msg).await;
    }

    pub async fn broadcast_tx(&self, tx: &Transaction) {
        let msg = json!({"type": "tx", "data": tx}).to_string() + "\n";
        self.send_to_all(&msg).await;
    }

    async fn send_to_all(&self, msg: &str) {
        let mut peers = self.peers.lock().await;
        let mut disconnected = Vec::new();

        for (i, peer) in peers.iter().enumerate() {
            let mut stream = peer.lock().await;
            if stream.write_all(msg.as_bytes()).await.is_err() {
                disconnected.push(i);
            }
        }

        for i in disconnected.into_iter().rev() {
            peers.remove(i);
        }
    }
}

async fn handle_peer(stream_arc: Arc<Mutex<TcpStream>>, bc: Arc<Mutex<DagBlockchain>>, p2p: Arc<P2P>) {
    let stream_guard = stream_arc.lock().await;
    drop(stream_guard); 

    loop {
        let mut line = String::new();
        {
            let mut stream = stream_arc.lock().await;
            let mut reader = BufReader::new(&mut *stream);
            if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                break; 
            }
        }

        let v: Value = match serde_json::from_str(&line) {
            Ok(json) => json,
            Err(_) => continue,
        };

        match v["type"].as_str() {
            Some("tx") => {
                if let Ok(tx) = serde_json::from_value::<Transaction>(v["data"].clone()) {
                    let bc_lock = bc.lock().await;
                    if bc_lock.submit_transaction(tx.clone()).is_ok() {
                        drop(bc_lock); 
                        p2p.broadcast_tx(&tx).await;
                    }
                }
            }
            Some("block") => {
                if let Ok(block) = serde_json::from_value::<Block>(v["data"].clone()) {
                    let mut bc_lock = bc.lock().await;
                    if bc_lock.validate_block(&block) {
                        println!("[P2P] Valid block received: {}", block.hash);
                        bc_lock.add_block(block.clone());
                        drop(bc_lock);
                        p2p.broadcast_block(&block).await;
                    }
                }
            }
            _ => {}
        }
    }
}