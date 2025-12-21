use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
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

    pub async fn start(port: u16, _bc: Arc<Mutex<DagBlockchain>>) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        println!("[P2P] Listening on port {}", port);

        let p2p = Arc::new(P2P::new());

        let p2p_clone = p2p.clone();
        tokio::spawn(async move {
            loop {
                let (stream, addr) = listener.accept().await.unwrap();
                println!("[P2P] New peer connected from {}", addr);

                let mut peers = p2p_clone.peers.lock().await;
                peers.push(stream);
            }
        });

        let p2p_clone = p2p.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(stream) = TcpStream::connect("127.0.0.1:4000").await {
                    println!("[P2P] Connected to peer on port 4001");
                    let mut peers = p2p_clone.peers.lock().await;
                    peers.push(stream);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let p2p_clone = p2p.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                let msg = b"Hello from peer! New block mined!\n";

                let mut peers = p2p_clone.peers.lock().await;
                let mut to_remove = Vec::new();
                for (i, peer) in peers.iter_mut().enumerate() {
                    if peer.write_all(msg).await.is_err() {
                        to_remove.push(i);
                    }
                }
                for i in to_remove.into_iter().rev() {
                    peers.remove(i);
                }
            }
        });

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}