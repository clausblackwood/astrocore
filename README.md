Astrocore — A Proof-of-Work BlockDAG Blockchain in Rust

A from-scratch blockchain implementation inspired by modern high-throughput designs like Kaspa. Built entirely in Rust for performance, safety, and deep learning.
Features

    BlockDAG structure — Multiple parents per block enabling parallel block production (higher TPS than traditional linear chains)
    Parallel Proof-of-Work mining — Multi-threaded nonce search using rayon
    Mempool with fee prioritization — Transactions are ordered by fee (higher fee = faster inclusion)
    Real transactions — UTXO model with ECDSA signatures using secp256k1 (same curve as Bitcoin)
    Genesis coinbase transaction — 50 million initial supply
    Simple P2P networking — TCP-based peer discovery and message broadcasting between nodes
    Zero external frameworks — Pure Rust implementation

Quick Start

git clone https://github.com/clausblackwood/astrocore.git
cd astrocore
cargo run --release

The node will:

Generate wallet addresses
Create and sign transactions spending the genesis output
Mine blocks in parallel
Start a P2P listener on port 4000

To run a second node and see peer connection:

Change the listening port to 4001 and connection target to 127.0.0.1:4000 in src/p2p.rs
Run another instance in a separate terminal

You will see peer discovery and periodic broadcast messages.
Tech Stack

Rust — Core language
tokio — Async runtime for networking
rayon — Parallel mining
secp256k1 — Cryptographic signatures
sha2 — Hashing
serde — Serialization

Why I Built This
This project was created to deeply understand low-level blockchain mechanics:

Consensus in a DAG
Transaction validation and mempool management
Parallelism in Proof-of-Work
Basic peer-to-peer networking

All code written from scratch in December 2025.
Future Ideas

Full UTXO validation and double-spend protection
Automatic block and transaction broadcasting
Simple CLI wallet
Web-based block explorer

Feel free to star ⭐ or fork!