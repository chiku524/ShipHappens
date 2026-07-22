//! Custodial Boing Network wallets (Ed25519 AccountId = 0x + 64 hex pubkey).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::SigningKey;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

/// Create a new Boing AccountId + secret key hex (0x-prefixed).
pub fn generate_boing_wallet() -> (String, String) {
    let signing = SigningKey::generate(&mut OsRng);
    let secret = format!("0x{}", hex::encode(signing.to_bytes()));
    let account = format!("0x{}", hex::encode(signing.verifying_key().to_bytes()));
    (account, secret)
}

pub fn is_valid_boing_account(wallet: &str) -> bool {
    let w = wallet.trim();
    if !w.starts_with("0x") || w.len() != 66 {
        return false;
    }
    hex::decode(&w[2..]).ok().is_some_and(|b| b.len() == 32)
}

/// Derive a 256-bit AES key from the server master secret.
pub fn wallet_aes_key(master: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"pudgymon-boing-wallet-v1:");
    hasher.update(master.as_bytes());
    hasher.finalize().into()
}

/// Encrypt a 0x… secret for DB storage (base64: nonce || ciphertext).
pub fn encrypt_wallet_secret(master: &str, secret_hex: &str) -> Result<String, String> {
    let key = wallet_aes_key(master);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, secret_hex.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut packed = Vec::with_capacity(12 + ct.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ct);
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        packed,
    ))
}

/// Decrypt a stored wallet secret.
pub fn decrypt_wallet_secret(master: &str, enc: &str) -> Result<String, String> {
    let key = wallet_aes_key(master);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let packed = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, enc)
        .map_err(|e| e.to_string())?;
    if packed.len() < 13 {
        return Err("corrupt wallet secret blob".into());
    }
    let (nonce_bytes, ct) = packed.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let pt = cipher
        .decrypt(nonce, ct)
        .map_err(|_| "wallet decrypt failed".to_string())?;
    String::from_utf8(pt).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_wallet() {
        let (account, secret) = generate_boing_wallet();
        assert!(is_valid_boing_account(&account));
        assert!(secret.starts_with("0x") && secret.len() == 66);
        let enc = encrypt_wallet_secret("test-master", &secret).unwrap();
        let dec = decrypt_wallet_secret("test-master", &enc).unwrap();
        assert_eq!(dec, secret);
    }
}
