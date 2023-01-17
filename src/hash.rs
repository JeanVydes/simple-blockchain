use rand::Rng;
use sha2::{Sha256, Digest};

pub fn next_problem_hash() -> Box<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let hash_data = format!("{}.{}", rng.gen::<u64>(), rng.gen::<u64>());

    let mut hasher = Sha256::new();
    hasher.update(hash_data);
    let result = hasher.finalize();
    
    return Box::new(result.to_vec());
}