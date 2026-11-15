use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier}
};
use uuid::Uuid;

pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    pub fn generate() -> (String, String, String) {
        let key = format!("sk_{}", Uuid::new_v4().to_string().replace("-", ""));
        let prefix = key.chars().take(12).collect::<String>();
        let hash = Self::hash_key(&key);
        (key, prefix, hash)
    }

    pub fn hash_key(key: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(key.as_bytes(), &salt)
            .expect("Failed to hash API key")
            .to_string()
    }

    pub fn verify_key(key: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(key.as_bytes(), &parsed_hash)
            .is_ok()
    }
}
