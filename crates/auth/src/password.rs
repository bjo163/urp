use std::fmt;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use thiserror::Error;

/// Password hashing policy used for new local credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PasswordHashConfig {
    /// Memory cost in KiB.
    pub memory_kib: u32,
    /// Number of Argon2 iterations.
    pub iterations: u32,
    /// Number of parallel lanes.
    pub parallelism: u32,
    /// URP credential format version.
    pub hash_version: u16,
}

impl Default for PasswordHashConfig {
    fn default() -> Self {
        Self {
            memory_kib: 19 * 1024,
            iterations: 2,
            parallelism: 1,
            hash_version: 1,
        }
    }
}

/// Password policy and PHC validation errors.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PasswordError {
    /// Hashing parameters are outside URP's permitted range.
    #[error("password hashing configuration is invalid")]
    InvalidConfig,
    /// New password is empty or too long.
    #[error("password length is invalid")]
    InvalidPasswordLength,
    /// Stored PHC hash is malformed, unsafe, or unsupported.
    #[error("stored password hash is invalid")]
    InvalidStoredHash,
    /// Hash generation failed.
    #[error("password hashing failed")]
    HashingFailed,
}

/// PHC-encoded Argon2id hash. Debug output never exposes the encoded value.
#[derive(Clone, Eq, PartialEq)]
pub struct EncodedPasswordHash {
    encoded: String,
    /// URP credential format version.
    pub hash_version: u16,
}

impl EncodedPasswordHash {
    /// Borrows the encoded hash for a trusted storage adapter.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }
}

impl fmt::Debug for EncodedPasswordHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EncodedPasswordHash")
            .field("encoded", &"[REDACTED]")
            .field("hash_version", &self.hash_version)
            .finish()
    }
}

/// Local password hashing and verification.
///
/// New credentials use Argon2id v19 with a random salt. PHC parameters are
/// bounded before verification to prevent malicious resource requests.
#[derive(Clone)]
pub struct PasswordService {
    config: PasswordHashConfig,
    dummy_hash: String,
}

impl PasswordService {
    /// Creates a password service and a process-local dummy hash for unknown logins.
    pub fn new(config: PasswordHashConfig) -> Result<Self, PasswordError> {
        validate_config(config)?;
        let service = Self {
            config,
            dummy_hash: String::new(),
        };
        let dummy_hash = service.hash_raw("urp-dummy-login-verification-value")?;
        Ok(Self { dummy_hash, ..service })
    }

    /// Returns the policy used for newly generated hashes.
    pub const fn config(&self) -> PasswordHashConfig {
        self.config
    }

    /// Hashes a password with a unique cryptographically random salt.
    pub fn hash_password(&self, password: &str) -> Result<EncodedPasswordHash, PasswordError> {
        validate_password_length(password)?;
        Ok(EncodedPasswordHash {
            encoded: self.hash_raw(password)?,
            hash_version: self.config.hash_version,
        })
    }

    /// Verifies a password against a stored Argon2id PHC string.
    pub fn verify_password(
        &self,
        password: &str,
        encoded_hash: &str,
    ) -> Result<bool, PasswordError> {
        validate_password_length(password)?;
        self.verify_phc(password, encoded_hash)
    }

    /// Performs a dummy verification for an unknown login.
    pub fn verify_dummy(&self, password: &str) -> Result<(), PasswordError> {
        validate_password_length(password)?;
        let _ = self.verify_phc(password, &self.dummy_hash)?;
        Ok(())
    }

    /// Verifies real or dummy credentials and returns false for absent/corrupt hashes.
    ///
    /// The dummy path prevents unknown users and malformed stored hashes from
    /// skipping the expensive password verification step.
    pub fn verify_or_dummy(
        &self,
        password: &str,
        encoded_hash: Option<&str>,
    ) -> Result<bool, PasswordError> {
        validate_password_length(password)?;
        match encoded_hash {
            Some(hash) => match self.verify_phc(password, hash) {
                Ok(valid) => Ok(valid),
                Err(_) => {
                    self.verify_dummy(password)?;
                    Ok(false)
                }
            },
            None => {
                self.verify_dummy(password)?;
                Ok(false)
            }
        }
    }

    fn hash_raw(&self, password: &str) -> Result<String, PasswordError> {
        let params = Params::new(
            self.config.memory_kib,
            self.config.iterations,
            self.config.parallelism,
            Some(32),
        )
        .map_err(|_| PasswordError::InvalidConfig)?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| PasswordError::HashingFailed)?;
        Ok(hash.to_string())
    }

    fn verify_phc(&self, password: &str, encoded_hash: &str) -> Result<bool, PasswordError> {
        if encoded_hash.is_empty() || encoded_hash.len() > 1024 {
            return Err(PasswordError::InvalidStoredHash);
        }
        let parsed = PasswordHash::new(encoded_hash).map_err(|_| PasswordError::InvalidStoredHash)?;
        if parsed.algorithm.as_str() != "argon2id" {
            return Err(PasswordError::InvalidStoredHash);
        }
        let params = Params::try_from(&parsed).map_err(|_| PasswordError::InvalidStoredHash)?;
        if params.m_cost() > 131_072
            || params.t_cost() > 10
            || params.p_cost() > 8
            || params.m_cost() < 8 * params.p_cost()
        {
            return Err(PasswordError::InvalidStoredHash);
        }
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        match argon2.verify_password(password.as_bytes(), &parsed) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(_) => Err(PasswordError::InvalidStoredHash),
        }
    }
}

fn validate_config(config: PasswordHashConfig) -> Result<(), PasswordError> {
    if config.hash_version == 0
        || !(19 * 1024..=131_072).contains(&config.memory_kib)
        || !(2..=10).contains(&config.iterations)
        || !(1..=8).contains(&config.parallelism)
        || config.memory_kib < 8 * config.parallelism
    {
        return Err(PasswordError::InvalidConfig);
    }
    Ok(())
}

fn validate_password_length(password: &str) -> Result<(), PasswordError> {
    if password.is_empty() || password.len() > 1024 {
        return Err(PasswordError::InvalidPasswordLength);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{EncodedPasswordHash, PasswordError, PasswordHashConfig, PasswordService};

    fn service() -> Result<PasswordService, PasswordError> {
        PasswordService::new(PasswordHashConfig::default())
    }

    #[test]
    fn hashes_and_verifies_passwords_with_random_salts() -> Result<(), PasswordError> {
        let service = service()?;
        let first = service.hash_password("a-long-enough-password")?;
        let second = service.hash_password("a-long-enough-password")?;
        assert_ne!(first.as_str(), second.as_str());
        assert!(service.verify_password("a-long-enough-password", first.as_str())?);
        assert!(!service.verify_password("a-different-password", first.as_str())?);
        assert_eq!(first.hash_version, 1);
        Ok(())
    }

    #[test]
    fn verifier_rejects_malformed_and_non_argon_hashes() -> Result<(), PasswordError> {
        let service = service()?;
        assert_eq!(
            service.verify_password("password", "not-a-phc-hash"),
            Err(PasswordError::InvalidStoredHash)
        );
        assert_eq!(
            service.verify_password("password", "$pbkdf2-sha256$i=1$salt$hash"),
            Err(PasswordError::InvalidStoredHash)
        );
        Ok(())
    }

    #[test]
    fn verifier_rejects_phc_costs_outside_resource_limits() -> Result<(), PasswordError> {
        let service = service()?;
        let unsafe_hash = "$argon2id$v=19$m=999999999,t=999999,p=1$c2FsdA$aGFzaA";
        assert_eq!(
            service.verify_password("password", unsafe_hash),
            Err(PasswordError::InvalidStoredHash)
        );
        Ok(())
    }

    #[test]
    fn missing_or_corrupt_credential_uses_dummy_verification() -> Result<(), PasswordError> {
        let service = service()?;
        assert!(!service.verify_or_dummy("password", None)?);
        assert!(!service.verify_or_dummy("password", Some("corrupt"))?);
        Ok(())
    }

    #[test]
    fn new_password_and_policy_bounds_are_enforced() -> Result<(), PasswordError> {
        let service = service()?;
        assert_eq!(
            service.hash_password(""),
            Err(PasswordError::InvalidPasswordLength)
        );
        assert_eq!(
            service.hash_password(&"x".repeat(1025)),
            Err(PasswordError::InvalidPasswordLength)
        );
        assert_eq!(
            PasswordService::new(PasswordHashConfig {
                memory_kib: 1024,
                ..PasswordHashConfig::default()
            })
            .err(),
            Some(PasswordError::InvalidConfig)
        );
        Ok(())
    }

    #[test]
    fn encoded_hash_debug_is_redacted() {
        let hash = EncodedPasswordHash {
            encoded: "secret-hash-content".to_owned(),
            hash_version: 1,
        };
        let debug = format!("{hash:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("secret-hash-content"));
    }
}
