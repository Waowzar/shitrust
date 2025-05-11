use std::collections::HashMap;
use std::fmt::Write;
use sha2::{Sha256, Sha512, Digest};
use hmac::{Hmac, Mac};
use aes_gcm::{
    aead::{Aead, NewAead, generic_array::GenericArray},
    Aes256Gcm, Key, Nonce
};
use rand::{Rng, rngs::OsRng};
use base64::{Engine as _, engine::general_purpose};

use crate::error::{ShitRustError, Result};
use crate::interpreter::Value;

/// Convert a byte slice to a hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

/// Convert a hex string to bytes
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err(ShitRustError::RuntimeError(
            "Hex string must have an even number of characters".to_string()
        ));
    }
    
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i+2];
        let byte = u8::from_str_radix(byte_str, 16).map_err(|_| 
            ShitRustError::RuntimeError(format!("Invalid hex character: {}", byte_str))
        )?;
        bytes.push(byte);
    }
    
    Ok(bytes)
}

/// Calculate SHA-256 hash of data
fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Calculate SHA-512 hash of data
fn sha512(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Calculate HMAC with SHA-256
fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|_| ShitRustError::RuntimeError("Invalid key length".to_string()))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Generate a random key for AES-256-GCM
fn generate_aes_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill(&mut key);
    key
}

/// Generate a random nonce for AES-256-GCM
fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill(&mut nonce);
    nonce
}

/// Encrypt data with AES-256-GCM
fn encrypt_aes_gcm(key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(ShitRustError::RuntimeError(
            format!("AES-256-GCM requires a 32-byte key, got {}", key.len())
        ));
    }
    
    if nonce.len() != 12 {
        return Err(ShitRustError::RuntimeError(
            format!("AES-256-GCM requires a 12-byte nonce, got {}", nonce.len())
        ));
    }
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.encrypt(nonce, plaintext)
        .map_err(|_| ShitRustError::RuntimeError("Encryption failed".to_string()))
}

/// Decrypt data with AES-256-GCM
fn decrypt_aes_gcm(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(ShitRustError::RuntimeError(
            format!("AES-256-GCM requires a 32-byte key, got {}", key.len())
        ));
    }
    
    if nonce.len() != 12 {
        return Err(ShitRustError::RuntimeError(
            format!("AES-256-GCM requires a 12-byte nonce, got {}", nonce.len())
        ));
    }
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|_| ShitRustError::RuntimeError("Decryption failed: invalid ciphertext or authentication tag".to_string()))
}

/// Initialize the crypto module
pub fn init_crypto_module() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    
    // SHA-256 function
    exports.insert("sha256".to_string(), Value::NativeFunction {
        name: "sha256".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("sha256() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let data = match &args[0] {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "sha256() takes a string or byte array argument".to_string()
                )),
            };
            
            let hash = sha256(&data);
            Ok(Value::String(bytes_to_hex(&hash)))
        }),
    });
    
    // SHA-512 function
    exports.insert("sha512".to_string(), Value::NativeFunction {
        name: "sha512".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("sha512() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let data = match &args[0] {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "sha512() takes a string or byte array argument".to_string()
                )),
            };
            
            let hash = sha512(&data);
            Ok(Value::String(bytes_to_hex(&hash)))
        }),
    });
    
    // HMAC function
    exports.insert("hmac_sha256".to_string(), Value::NativeFunction {
        name: "hmac_sha256".to_string(),
        arity: 2,
        function: Box::new(|args, _| {
            if args.len() != 2 {
                return Err(ShitRustError::RuntimeError(
                    format!("hmac_sha256() takes 2 arguments, but {} were given", args.len())
                ));
            }
            
            let key = match &args[0] {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "hmac_sha256() key must be a string or byte array".to_string()
                )),
            };
            
            let data = match &args[1] {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "hmac_sha256() data must be a string or byte array".to_string()
                )),
            };
            
            let hmac = hmac_sha256(&key, &data)?;
            Ok(Value::String(bytes_to_hex(&hmac)))
        }),
    });
    
    // Generate AES key
    exports.insert("generate_aes_key".to_string(), Value::NativeFunction {
        name: "generate_aes_key".to_string(),
        arity: 0,
        function: Box::new(|args, _| {
            if !args.is_empty() {
                return Err(ShitRustError::RuntimeError(
                    format!("generate_aes_key() takes 0 arguments, but {} were given", args.len())
                ));
            }
            
            let key = generate_aes_key();
            Ok(Value::String(bytes_to_hex(&key)))
        }),
    });
    
    // Generate nonce
    exports.insert("generate_nonce".to_string(), Value::NativeFunction {
        name: "generate_nonce".to_string(),
        arity: 0,
        function: Box::new(|args, _| {
            if !args.is_empty() {
                return Err(ShitRustError::RuntimeError(
                    format!("generate_nonce() takes 0 arguments, but {} were given", args.len())
                ));
            }
            
            let nonce = generate_nonce();
            Ok(Value::String(bytes_to_hex(&nonce)))
        }),
    });
    
    // Encrypt with AES-GCM
    exports.insert("encrypt".to_string(), Value::NativeFunction {
        name: "encrypt".to_string(),
        arity: 3,
        function: Box::new(|args, _| {
            if args.len() != 3 {
                return Err(ShitRustError::RuntimeError(
                    format!("encrypt() takes 3 arguments, but {} were given", args.len())
                ));
            }
            
            let key = match &args[0] {
                Value::String(s) => hex_to_bytes(s)?,
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "encrypt() key must be a hex string or byte array".to_string()
                )),
            };
            
            let nonce = match &args[1] {
                Value::String(s) => hex_to_bytes(s)?,
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "encrypt() nonce must be a hex string or byte array".to_string()
                )),
            };
            
            let plaintext = match &args[2] {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "encrypt() plaintext must be a string or byte array".to_string()
                )),
            };
            
            let ciphertext = encrypt_aes_gcm(&key, &nonce, &plaintext)?;
            Ok(Value::String(general_purpose::STANDARD.encode(&ciphertext)))
        }),
    });
    
    // Decrypt with AES-GCM
    exports.insert("decrypt".to_string(), Value::NativeFunction {
        name: "decrypt".to_string(),
        arity: 3,
        function: Box::new(|args, _| {
            if args.len() != 3 {
                return Err(ShitRustError::RuntimeError(
                    format!("decrypt() takes 3 arguments, but {} were given", args.len())
                ));
            }
            
            let key = match &args[0] {
                Value::String(s) => hex_to_bytes(s)?,
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "decrypt() key must be a hex string or byte array".to_string()
                )),
            };
            
            let nonce = match &args[1] {
                Value::String(s) => hex_to_bytes(s)?,
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "decrypt() nonce must be a hex string or byte array".to_string()
                )),
            };
            
            let ciphertext = match &args[2] {
                Value::String(s) => general_purpose::STANDARD.decode(s).map_err(|_| 
                    ShitRustError::RuntimeError("Invalid base64 string".to_string())
                )?,
                Value::Bytes(b) => b.clone(),
                _ => return Err(ShitRustError::RuntimeError(
                    "decrypt() ciphertext must be a base64 string or byte array".to_string()
                )),
            };
            
            let plaintext = decrypt_aes_gcm(&key, &nonce, &ciphertext)?;
            
            // Try to convert to string if possible
            match String::from_utf8(plaintext.clone()) {
                Ok(s) => Ok(Value::String(s)),
                Err(_) => Ok(Value::Bytes(plaintext)),
            }
        }),
    });
    
    // Random bytes generator
    exports.insert("random_bytes".to_string(), Value::NativeFunction {
        name: "random_bytes".to_string(),
        arity: 1,
        function: Box::new(|args, _| {
            if args.len() != 1 {
                return Err(ShitRustError::RuntimeError(
                    format!("random_bytes() takes 1 argument, but {} were given", args.len())
                ));
            }
            
            let length = match &args[0] {
                Value::Int(len) if *len > 0 => *len as usize,
                _ => return Err(ShitRustError::RuntimeError(
                    "random_bytes() length must be a positive integer".to_string()
                )),
            };
            
            let mut bytes = vec![0u8; length];
            OsRng.fill(&mut bytes[..]);
            
            Ok(Value::Bytes(bytes))
        }),
    });
    
    exports
} 
