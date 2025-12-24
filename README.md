# 🚀 AstroCore

AstroCore is a high-performance, decentralized DAG-based blockchain prototype written in **Rust**. It features parallel mining, a UTXO transaction model, and an asynchronous P2P networking layer.

Built for the next generation of distributed systems, AstroCore utilizes a Directed Acyclic Graph (DAG) structure, allowing multiple blocks to be mined and linked simultaneously, overcoming the bottlenecks of traditional linear chains.

## ✨ Key Features

- **DAG Architecture**: Supports multiple parent blocks, enabling better scalability and network throughput.
- **Parallel Mining**: Leverages multi-core processing using `Rayon` for efficient Proof-of-Work (PoW).
- **Asynchronous Networking**: Powered by `Tokio` for handling multiple P2P connections concurrently with robust TCP framing.
- **UTXO Model**: Bitcoin-style Unspent Transaction Output model for reliable balance tracking and double-spend protection.
- **Strong Cryptography**: Uses `secp256k1` for ECDSA digital signatures and `SHA-256` for deterministic hashing.
- **Prioritized Mempool**: A fee-based priority queue (Max-Heap) to ensure high-value transactions are processed first.

## 🛠 Tech Stack

| Component | Technology |
| :--- | :--- |
| **Runtime** | [Tokio](https://tokio.rs/) (Async I/O) |
| **Parallelism** | [Rayon](https://github.com/rayon-rs/rayon) |
| **Crypto** | [secp256k1](https://github.com/rust-bitcoin/rust-secp256k1) |
| **Serialization** | [Serde](https://serde.rs/) / JSON |
| **CLI** | [Clap](https://docs.rs/clap/latest/clap/) |

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024 recommended)
- Cargo (Rust package manager)

### Installation
```bash
git clone [https://github.com/clausblackwood/astrocore.git](https://github.com/clausblackwood/astrocore.git)
cd astrocore
cargo build --release

💻 Usage
1. Create a Wallet

Generate your cryptographic identity (Public Address and Secret Key).
Bash

cargo run -- create-wallet

2. Start a Node

Launch a node and start participating in the network.
Bash

# Start the seed node
cargo run -- start-node --port 4000 --difficulty 3

3. Connect more nodes

Start another instance on a different port. It will automatically attempt to connect to the seed peer.
Bash

cargo run -- start-node --port 4001 --difficulty 3

🏗 Architecture

AstroCore moves away from the "one block at a time" constraint. Each block in our DAG references multiple previous block hashes (parents), allowing the network to heal and converge even during high latency or parallel mining events.