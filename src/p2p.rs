use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::blockchain::DagBlockchain;

pub struct P2P {
    pub peers: Arc<Mutex<Vec<TcpStream>>>,
}

impl P2P {
    pub fn new() -> Self {
        P2P { peers: Arc::new(Mutex::new(Vec::new())) }
    }

    pub async fn start(port: u16, _bc: Arc<Mutex<DagBlockchain>>) {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await.unwrap();
        println!("[P2P] Listening on {}", addr);

        let p2p = Arc::new(P2P::new());

        // Inbound connection handler
        let p2p_clone = p2p.clone();
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                println!("[P2P] New peer connected: {}", addr);
                p2p_clone.peers.lock().await.push(stream);
            }
        });

        // Basic broadcast loop
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let mut peers = p2p.peers.lock().await;
            for peer in peers.iter_mut() {
                let _ = peer.write_all(b"Keepalive: DAG Node Active\n").await;
            }
        }
    }
}