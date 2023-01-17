use std::default::Default;
use serde::{Serialize, Deserialize};

pub const NODE_DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const NODE_DEFAULT_PORT: u16 = 5954;
pub const NODE_DEFAULT_DIR_DATA: &str = "/tmp/blockchain";
pub const BLOCK_REWARD: u64 = 100;

pub type INDEX = u64;
pub type ADDRESS = String;
pub type HASH = Vec<u8>;
pub type NONCE = String;
pub type TIMESTAMP = u64;

#[derive(Debug)]
pub struct CLIConfiguration {
    pub port: u16,
    pub host: String,
    pub workdir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: INDEX,
    pub hash: HASH,
    pub nonce: NONCE,
    pub previous_hash: HASH,
    pub timestamp: TIMESTAMP,
    pub transactions: Vec<Transaction>,
    pub validated_by: ADDRESS,
    pub minted: u64,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            index: 0,
            hash: vec![],
            nonce: "".to_string(),
            previous_hash: vec![],
            timestamp: 0,
            transactions: vec![],
            validated_by: "".to_string(),
            minted: 0,
        }
    }
}

// This node information
#[derive(Debug, Clone)]
pub struct Runtime {
    pub port: u16, // implement copy trait
    pub host: String,
    pub node_identifier: String,
    pub workdir: String,
    pub uncofirmed_transactions: Vec<Transaction>,
    pub current_hash: Box<HASH>,
    pub last_block: Block,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub sender: ADDRESS,
    pub recipient: ADDRESS,
    pub amount: u64,
    pub timestamp: TIMESTAMP,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionClientPayload {
    pub sender: ADDRESS,
    pub recipient: ADDRESS,
    pub amount: u64,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeServerPayload {
    pub message: String,
    pub data: String,
}