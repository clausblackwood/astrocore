use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "AstroCore", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    CreateWallet,
    StartNode {
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
        #[arg(short, long, default_value_t = 3)]
        difficulty: usize,
    },
    Send {
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: u64,
        #[arg(short, long)]
        secret: String,
    },
}