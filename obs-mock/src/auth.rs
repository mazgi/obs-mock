use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

pub struct AuthConfig {
    pub password: Option<String>,
    pub salt: String,
    pub challenge: String,
}

impl AuthConfig {
    pub fn new(password: Option<String>) -> Self {
        let salt = generate_random_string(32);
        let challenge = generate_random_string(32);
        Self {
            password,
            salt,
            challenge,
        }
    }

    pub fn verify(&self, auth_string: &str) -> bool {
        let password = match &self.password {
            Some(p) => p,
            None => return true,
        };

        let expected = compute_auth_response(password, &self.salt, &self.challenge);
        auth_string == expected
    }

    pub fn requires_auth(&self) -> bool {
        self.password.is_some()
    }
}

fn compute_auth_response(password: &str, salt: &str, challenge: &str) -> String {
    let b64 = base64::engine::general_purpose::STANDARD;

    // Step 1: SHA256(password + salt) -> base64
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let secret = b64.encode(hasher.finalize());

    // Step 2: SHA256(secret + challenge) -> base64
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(challenge.as_bytes());
    b64.encode(hasher.finalize())
}

fn generate_random_string(len: usize) -> String {
    let b64 = base64::engine::general_purpose::STANDARD;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
    b64.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_roundtrip() {
        let config = AuthConfig::new(Some("testpassword".to_string()));
        let response = compute_auth_response("testpassword", &config.salt, &config.challenge);
        assert!(config.verify(&response));
    }

    #[test]
    fn test_auth_wrong_password() {
        let config = AuthConfig::new(Some("testpassword".to_string()));
        let response = compute_auth_response("wrongpassword", &config.salt, &config.challenge);
        assert!(!config.verify(&response));
    }

    #[test]
    fn test_no_auth() {
        let config = AuthConfig::new(None);
        assert!(!config.requires_auth());
        assert!(config.verify("anything"));
    }
}
