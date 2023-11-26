use rand::{rngs::OsRng, RngCore};
use base64::Engine as _;

pub fn generate_random_key() -> String {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    base64::engine::general_purpose::STANDARD.encode(&key)    
}