use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use crate::block::Block;
use crate::transaction::Transaction;
use crate::blockchain::DagBlockchain;

pub struct P2P {
    pub peers: Arc<Mutex<Vec<TcpStream>>>,
}

impl P2P {
    pub fn new() -> Self {
        P2P {
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(port: u16, bc: Arc<Mutex<DagBlockchain>>) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        println!("[P2P] Listening on port {}", port);

        let p2p = Arc::new(P2P::new());

        let p2p_clone = p2p.clone();
        let bc_clone = bc.clone();
        tokio::spawn(async move {
            loop {
                let (mut stream, addr) = listener.accept().await.unwrap();
                println!("[P2P] New peer connected: {}", addr);

                P2P::request_headers(&mut stream).await;

                tokio::spawn(handle_peer(stream, bc_clone.clone(), p2p_clone.clone()));
            }
        });

        let p2p_clone = p2p.clone();
        let bc_clone = bc.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(mut stream) = TcpStream::connect("127.0.0.1:4001").await {
                    println!("[P2P] Connected to peer on port 4001");

                    P2P::request_headers(&mut stream).await;

                    tokio::spawn(handle_peer(stream, bc_clone.clone(), p2p_clone.clone()));

                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    }

    async fn request_headers(stream: &mut TcpStream) {
        let msg = json!({"type": "getheaders"}).to_string();
        let _ = stream.write_all(msg.as_bytes()).await;
    }

    pub async fn broadcast_block(&self, block: &Block) {
        let msg = json!({"type": "block", "data": block}).to_string();
        let msg_bytes = msg.as_bytes();
        let mut peers = self.peers.lock().await;
        let mut failed = Vec::new();
        for (i, peer) in peers.iter_mut().enumerate() {
            if peer.write_all(msg_bytes).await.is_err() {
                failed.push(i);
            }
        }
        for i in failed.into_iter().rev() {
            peers.remove(i);
        }
    }

    pub async fn broadcast_tx(&self, tx: &Transaction) {
        let msg = json!({"type": "tx", "data": tx}).to_string();
        let msg_bytes = msg.as_bytes();
        let mut peers = self.peers.lock().await;
        let mut failed = Vec::new();
        for (i, peer) in peers.iter_mut().enumerate() {
            if peer.write_all(msg_bytes).await.is_err() {
                failed.push(i);
            }
        }
        for i in failed.into_iter().rev() {
            peers.remove(i);
        }
    }
}

async fn handle_peer(mut stream: TcpStream, bc: Arc<Mutex<DagBlockchain>>, p2p: Arc<P2P>) {
    let mut buffer = [0; 4096];
    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => break,
            Ok(n) => n,
            Err(_) => break,
        };

        let msg = String::from_utf8_lossy(&buffer[..n]);
        let json: Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => continue,
        };

        match json["type"].as_str() {
            Some("getheaders") => {
                let bc_guard = bc.lock().await;
                let headers: Vec<String> = bc_guard.blocks.iter().map(|b| b.hash.clone()).collect();
                let response = json!({"type": "headers", "data": headers}).to_string();
                let _ = stream.write_all(response.as_bytes()).await;
            }
            Some("headers") => {
                println!("[P2P] Received headers: {:?}", json["data"]);
            }
            Some("block") => {
                let block_data = json["data"].clone();
                println!("[P2P] Received block: {:?}", block_data);

                let received_block: Block = match serde_json::from_value(block_data) {
                    Ok(b) => b,
                    Err(e) => {
                        println!("[P2P] Invalid block format: {}", e);
                        continue;
                    }
                };

                let mut bc_guard = bc.lock().await;
                if bc_guard.validate_block(&received_block) {
                    println!("[P2P] Valid block received: #{}", received_block.index);
                    bc_guard.add_block(received_block.clone());
                    p2p.broadcast_block(&received_block).await;
                } else {
                    println!("[P2P] Invalid block rejected");
                }
            }
            Some("tx") => {
                println!("[P2P] Received tx: {:?}", json["data"]);
            }
            _ => {}
        }
    }
}