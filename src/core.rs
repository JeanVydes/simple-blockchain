use crate::models;
use crate::cli;
use crate::models::Runtime;
use crate::block;
use crate::hash;
use crate::logger::Logger;

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
            Ok(_) => {}
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        self.port = cli_config.port;
        self.host = cli_config.host;
        self.node_identifier = "0x0".to_string();
        self.uncofirmed_transactions = vec![];
        self.current_hash = hash::next_problem_hash(self.logger.clone(), cli_config.log);
        self.workdir = cli_config.workdir.clone();
        self.debug = cli_config.log;

        match block::get_newest_block(workdir_path) {
            Ok(block) => {
                self.last_block = block;
            }
            Err(_) => {
                // create genesis
                let genesis_block = models::Block::default();
                self.last_block = genesis_block;

                match block::register_block_locally(cli_config.workdir.clone(), self.last_block.clone()) {
                    Ok(_) => if self.debug {println!("Genesis block created!")},
                    Err(_) => if self.debug {println!("Error creating genesis block")},
                }
            }
        }

        
        if self.debug {
            let msg = format!("Last block index: {}", self.last_block.index);
            self.logger.log(&msg); 
            self.logger.log("Starting HTTP Gateway...")
        }

        match self.tcp_server() {
            Ok(_) => {},
            Err(_) => if self.debug {self.logger.log("Error starting HTTP Gateway...")},
        };
    }

    fn check_workdir(&mut self, workdir: String) -> std::io::Result<()> {
        let path = std::path::Path::new(&workdir);
        if path.exists() {
            self.current_hash = hash::next_problem_hash(self.logger.clone(), self.debug);
        } else {
            if self.debug {self.logger.log("Creating new work directory...")}
            std::fs::create_dir_all(path)?;
            if self.debug {self.logger.log("Work directory created!")}
        }
        
        Ok(())
    }

    fn tcp_server(&mut self) -> std::io::Result<()> {
        let listener = Server::http(format!("{}:{}", self.host, self.port)).unwrap();

        if self.debug {self.logger.log("HTTP Gateway started!")}
        if self.debug {self.logger.log("Receiving requests!")}
        for request in listener.incoming_requests() {
            self.handle_request(request)?;
        }

        Ok(())
    }

    fn handle_request(&mut self, request: Request) -> std::io::Result<()> {
        let mut request = request;
        if request.body_length() > Some(1000) {
            let response = Response::from_string("400");
            request.respond(response)?;
            return Ok(())
        }
        
        let url_skeleton = request.url().split("?").collect::<Vec<&str>>();
        match (url_skeleton[0].clone(), request.method()) {
            ("/hash", Method::Get) => {
                let current_hash = self.get_current_hash().clone();
                let hash_encoded = hex::encode(&current_hash.as_ref().as_slice());

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
    
                if result.to_vec() == self.get_current_hash().clone().as_ref().as_slice() {
                    self.validate_current("".to_string(), "".to_string())?;
                    let payload = models::NodeServerPayload {
                        message: "validated".to_string(),
                        data: self.last_block.clone(),
                    };

                    let payload_plain = serde_json::to_string(&payload).unwrap();

                    let response = Response::from_string(payload_plain);
                    request.respond(response)?;
                } else {
                    let response = Response::from_string("invalid nonce");
                    request.respond(response)?;
                }
                
            }
            ("/send", Method::Post) => {
                let mut body = String::new();
                let result = request.as_reader().read_to_string(&mut body);
                if result.is_err() {
                    let response = Response::from_string("invalid body");
                    request.respond(response)?;
                    return Ok(())
                }

                let transaction = serde_json::from_str::<models::TransactionClientPayload>(&body);
                match transaction {
                    Ok(transaction) => {
                        let transaction = models::Transaction {
                            sender: transaction.sender,
                            recipient: transaction.recipient,
                            amount: transaction.amount,
                            timestamp: time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs(),
                        };

                        if self.debug {
                            let msg = format!("New transaction received: {:?}", transaction);
                            self.logger.log(&msg)
                        }

                        self.uncofirmed_transactions.push(transaction.clone());
                        let response = Response::from_string("transaction added");
                        request.respond(response)?;
                    }
                    Err(_) => {
                        let response = Response::from_string("invalid body");
                        request.respond(response)?;
                    }
                }
            }
            ("/get/block", Method::Get) => {
                if url_skeleton.len() < 2 {
                    let response = Response::from_string("not query provided");
                    request.respond(response)?;
                    return Ok(())
                }

                let query = url_skeleton[1].split("&").collect::<Vec<&str>>();
                let mut id = String::new();
                for i in 0..query.len() {
                    let query_skeleton = query[i].split("=").collect::<Vec<&str>>();
                    if query_skeleton.len() < 2 {
                        continue
                    }

                    if query_skeleton[0] == "id" {
                        if query_skeleton[1].len() == 0 {
                            let response = Response::from_string("id is not provided");
                            request.respond(response)?;
                            return Ok(())
                        }
                        
                        id = query_skeleton[1].to_string();
                        break;
                    }
                }

                if id == "" {
                    let response = Response::from_string("id is not provided");
                    request.respond(response)?;
                    return Ok(())
                }

                let id = id.parse::<u64>();
                let id = match id {
                    Ok(r) => r,
                    Err(_) => {
                        let response = Response::from_string("id is not a number");
                        request.respond(response)?;
                        return Ok(())
                    }
                };

                let workdir_i = self.workdir.clone();
                let workdir_path = std::path::Path::new(&workdir_i);
                let result = block::get_block_by_index(id, workdir_path);
                let result = match result {
                    Ok(r) => r,
                    Err(_) => {
                        let response = Response::from_string("block not found");
                        request.respond(response)?;
                        return Ok(())
                    }
                };

                let response = Response::from_string(serde_json::to_string(&result).unwrap());
                request.respond(response)?;
            }
            ("/get/lastblock", Method::Get) => {
                let response = Response::from_string(serde_json::to_string(&self.last_block).unwrap());
                request.respond(response)?;
            }
            ("/get/unconfirmedtransactions", Method::Get) => {
                let response = Response::from_string(serde_json::to_string(&self.uncofirmed_transactions).unwrap());
                request.respond(response)?;
            }
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
    
        self.current_hash = hash::next_problem_hash(self.logger.clone(), self.debug);
        self.uncofirmed_transactions = vec![];
        self.last_block = block.clone();

        block::register_block_locally(self.workdir.clone(), block)?;

        if self.debug {
            let msg = format!("New block registered: id:{}", self.last_block.index);
            self.logger.log(&msg)
        }
    
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
        logger: Logger{
            name: "NODE".to_string(),
        },
        debug: false,
    });

    Ok(())
}