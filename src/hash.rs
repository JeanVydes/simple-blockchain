use rand::Rng;
use sha2::{Sha256, Digest};

pub fn next_problem_hash(logger: crate::logger::Logger, debug: bool) -> Box<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let nonce = format!("{}.{}", rng.gen::<u64>(), rng.gen::<u64>());
    let mut hasher = Sha256::new();
    hasher.update(nonce.clone());
    let result = hasher.finalize();
    
    if debug {
        let msg = format!("A new hash has been generated to be validated. \nNONCE: {}\nHASH: {}\n", nonce, hex::encode(result.to_vec()));
        logger.log(&msg)
    }

    return Box::new(result.to_vec());
}