mod models;
mod core;
mod cli;
mod block;
mod hash;
mod logger;

fn main() {
    match core::init() {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    };
}
