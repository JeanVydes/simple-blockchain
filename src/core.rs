use crate::models;
use crate::cli;
use crate::models::Runtime;
use crate::block;
use crate::hash;
use std::time;
use tiny_http::{Server, Request, Response, Method};
use sha2::{Sha256, Digest};

trait Service {
    fn new(&mut self);
    fn check_workdir(&mut self, workdir: String) -> std::io::Result<()>;
    fn tcp_server(&mut self) -> std::io::Result<()>;
    fn handle_request(&mut self, request: Request) -> std::io::Result<()>;
    fn validate_current(&mut self, validated_by: String, nonce: String) -> std::io::Result<()>;
    fn get_current_hash(&mut self) -> &mut Box<Vec<u8>>;
}

impl Service for models::Runtime {
    fn new(&mut self) {
        let cli_config = cli::init_cli_processor();
        let path_str = cli_config.workdir.clone();
        let workdir_path = std::path::Path::new(&path_str);
        match self.check_workdir(workdir_path.clone().to_string_lossy().to_string()) {
            Ok(_) => {
                println!("Workdir checked");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        self.port = cli_config.port;
        self.host = cli_config.host;
        self.node_identifier = "0x0".to_string();
        self.uncofirmed_transactions = vec![];
        self.current_hash = hash::next_problem_hash();
        self.workdir = cli_config.workdir.clone();

        self.last_block = block::get_newest_block(workdir_path).unwrap();

        println!("{:?}", self.last_block.clone());

        match self.tcp_server() {
            Ok(_) => println!("TCP Server UP"),
            Err(_) => println!("ERROR TCP SERVER"),
        };
    }

    fn check_workdir(&mut self, workdir: String) -> std::io::Result<()> {
        let path = std::path::Path::new(&workdir);
        if path.exists() {
            self.current_hash = hash::next_problem_hash();
        } else {
            std::fs::create_dir(path)?;

            let genesis_block = models::Block {
                index: 0,
                hash: vec![0; 24],
                nonce: "".to_string(),
                previous_hash: vec![],
                timestamp: time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs(),
                transactions: vec![],
                validated_by: "".to_string(),
                minted: models::BLOCK_REWARD,
            };

            block::register_block_locally(workdir, genesis_block.clone())?;
        }

        Ok(())
    }

    fn tcp_server(&mut self) -> std::io::Result<()> {
        let listener = Server::http(format!("{}:{}", self.host, self.port)).unwrap();

        for request in listener.incoming_requests() {
            self.handle_request(request)?;
        }

        Ok(())
    }

    fn handle_request(&mut self, request: Request) -> std::io::Result<()> {
        let current_hash = self.get_current_hash();
        let ch_2 = current_hash.clone();
        let hash_encoded = hex::encode(&current_hash.as_ref().as_slice());

        let mut request = request;
        if request.body_length() > Some(1000) {
            let response = Response::from_string("400");
            request.respond(response)?;
            return Ok(())
        }
        
        match (request.url(), request.method()) {
            ("/hash", Method::Get) => {
                let response = Response::from_string(hash_encoded);
                let request = request;
                request.respond(response)?;
            }
            ("/validate", Method::Post) => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;
                let body = body.trim();
                let mut hasher = Sha256::new();
                hasher.update(body);
                let result = hasher.finalize();
    
                if result.to_vec() == *ch_2.as_ref().as_slice() {
                    self.validate_current("".to_string(), "".to_string())?;
                    let payload = models::NodeServerPayload {
                        message: "validated".to_string(),
                        data: "".to_string(),
                    };

                    let payload_plain = serde_json::to_string(&payload).unwrap();

                    let response = Response::from_string(payload_plain);
                    request.respond(response)?;
                } else {
                    let response = Response::from_string("400");
                    request.respond(response)?;
                }
                
            },
            ("/send", Method::Post) => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;
                let transaction: models::TransactionClientPayload = serde_json::from_str(&body).unwrap();

                self.uncofirmed_transactions.push(models::Transaction {
                    sender: transaction.sender,
                    recipient: transaction.recipient,
                    amount: transaction.amount,
                    timestamp: time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs(),
                });
            },
            _ => {
                let response = Response::from_string("404");
                request.respond(response)?;
            }
        }

        Ok(())
    }

    fn validate_current(&mut self, validated_by: String, nonce: String) -> std::io::Result<()> {
        let block: models::Block = models::Block {
            index: self.last_block.index + 1,
            hash: *self.current_hash.clone(),
            nonce,
            previous_hash: self.last_block.hash.clone(),
            timestamp: time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs(),
            transactions: self.uncofirmed_transactions.clone(),
            validated_by,
            minted: models::BLOCK_REWARD,
        };
    
        self.current_hash = hash::next_problem_hash();
        self.uncofirmed_transactions = vec![];
        self.last_block = block.clone();

        block::register_block_locally(self.workdir.clone(), block)?;
    
        Ok(())
    }

    fn get_current_hash(&mut self) -> &mut Box<Vec<u8>> {
        &mut self.current_hash
    }
}

pub fn init() -> std::io::Result<()> {
    models::Runtime::new(&mut Runtime {
        port: 0,
        host: "".to_string(),
        node_identifier: "".to_string(),
        workdir: "".to_string(),
        uncofirmed_transactions: vec![],
        current_hash: Box::new(vec![]),
        last_block: models::Block {
            index: 0,
            hash: vec![0; 24],
            nonce: "".to_string(),
            previous_hash: vec![],
            timestamp: 0,
            transactions: vec![],
            validated_by: "0".to_string(),
            minted: 0,
        },
    });

    Ok(())
}