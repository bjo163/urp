use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use urp_kernel::ids::UserId;
use uuid::Uuid;

/// Stable status of a user account in the identity registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    /// Account exists but has not completed activation.
    Invited,
    /// Account may authenticate, subject to membership and policy checks.
    Active,
    /// Account is temporarily prevented from authenticating.
    Suspended,
    /// Account is disabled; historical references remain valid.
    Disabled,
}

/// Identity provider family recorded for an authentication identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct IdentityProvider(String);

impl IdentityProvider {
    /// Creates a provider key after applying the technical-name grammar.
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 64
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
            || value
                .chars()
                .next()
                .is_some_and(|character| matches!(character, '-' | '_'))
            || value
                .chars()
                .last()
                .is_some_and(|character| matches!(character, '-' | '_'))
        {
            return Err(IdentityError::InvalidProvider);
        }
        Ok(Self(value))
    }

    /// Returns the stable provider key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Lifecycle status of an external or local login identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityStatus {
    /// Identity is eligible when its linked user is active.
    Active,
    /// Identity is retained but cannot be used to authenticate.
    Disabled,
}

/// A normalized login identity linked to a platform user.
#[derive(Clone, Eq, PartialEq)]
pub struct AuthIdentity {
    /// Stable authentication identity ID.
    pub identity_id: Uuid,
    /// Canonical URP user ID.
    pub user_id: UserId,
    /// Provider key such as local or oidc.
    pub provider: IdentityProvider,
    /// Provider-issued subject; absent for local credentials.
    pub provider_subject: Option<String>,
    /// Normalized unique login handle.
    pub login_normalized: String,
    /// Normalized optional email address.
    pub email_normalized: Option<String>,
    /// Whether the email address has been verified by the provider.
    pub email_verified: bool,
    /// Identity lifecycle status.
    pub status: IdentityStatus,
}

impl fmt::Debug for AuthIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthIdentity")
            .field("identity_id", &self.identity_id)
            .field("user_id", &self.user_id)
            .field("provider", &self.provider)
            .field(
                "provider_subject",
                &self.provider_subject.as_ref().map(|_| "[REDACTED]"),
            )
            .field("login_normalized", &"[REDACTED]")
            .field(
                "email_normalized",
                &self.email_normalized.as_ref().map(|_| "[REDACTED]"),
            )
            .field("email_verified", &self.email_verified)
            .field("status", &self.status)
            .finish()
    }
}

/// Canonical user account metadata, independent of tenant memberships.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAccount {
    /// Stable globally scoped user ID.
    pub user_id: UserId,
    /// Display label shown in the workspace.
    pub display_name: String,
    /// Authentication lifecycle state.
    pub status: UserStatus,
    /// Whether the account has a soft-disable timestamp.
    pub disabled_at: Option<String>,
}

impl UserAccount {
    /// Returns whether the account is eligible to authenticate.
    ///
    /// Membership and authorization checks remain mandatory after this check.
    pub fn can_authenticate(&self) -> bool {
        self.status == UserStatus::Active && self.disabled_at.is_none()
    }
}

/// Approved password hash algorithm metadata. This is not a password verifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasswordHashMetadata {
    /// Argon2id encoded PHC hash.
    Argon2id,
}

/// Stored credential metadata. The encoded hash is intentionally excluded
/// from Debug and serialization.
#[derive(Clone, Eq, PartialEq)]
pub struct CredentialMetadata {
    /// User that owns this credential.
    pub user_id: UserId,
    /// Approved password hash scheme.
    pub scheme: PasswordHashMetadata,
    /// Monotonic credential format version.
    pub hash_version: u16,
    encoded_hash: String,
    /// UTC timestamp at which the password was last changed.
    pub password_changed_at: String,
    /// Credential revocation timestamp, when revoked.
    pub revoked_at: Option<String>,
}

impl CredentialMetadata {
    /// Creates credential metadata from an encoded password hash.
    ///
    /// This constructor validates storage shape only. It does not claim that
    /// the supplied value is a cryptographically valid Argon2id hash.
    pub fn new(
        user_id: UserId,
        scheme: PasswordHashMetadata,
        hash_version: u16,
        encoded_hash: String,
        password_changed_at: String,
        revoked_at: Option<String>,
    ) -> Result<Self, IdentityError> {
        if hash_version == 0 {
            return Err(IdentityError::InvalidHashVersion);
        }
        if encoded_hash.trim().is_empty() || encoded_hash.len() > 1024 {
            return Err(IdentityError::InvalidEncodedHash);
        }
        if password_changed_at.trim().is_empty() {
            return Err(IdentityError::InvalidTimestamp);
        }
        Ok(Self {
            user_id,
            scheme,
            hash_version,
            encoded_hash,
            password_changed_at,
            revoked_at,
        })
    }

    /// Borrows the encoded hash for a trusted persistence adapter.
    pub fn encoded_hash(&self) -> &str {
        &self.encoded_hash
    }

    /// Returns the password-change timestamp.
    pub fn password_changed_at(&self) -> &str {
        &self.password_changed_at
    }

    /// Returns whether the credential has been revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }
}

impl fmt::Debug for CredentialMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CredentialMetadata")
            .field("user_id", &self.user_id)
            .field("scheme", &self.scheme)
            .field("hash_version", &self.hash_version)
            .field("encoded_hash", &"[REDACTED]")
            .field("password_changed_at", &self.password_changed_at)
            .field("revoked_at", &self.revoked_at)
            .finish()
    }
}

/// Validation errors for identity and credential contracts.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IdentityError {
    /// Provider key violates the technical identifier grammar.
    #[error("identity provider key is invalid")]
    InvalidProvider,
    /// Password hash version must be positive.
    #[error("password hash version must be positive")]
    InvalidHashVersion,
    /// Encoded password hash is empty or exceeds the storage limit.
    #[error("encoded password hash is invalid")]
    InvalidEncodedHash,
    /// Credential timestamp is empty.
    #[error("credential timestamp is invalid")]
    InvalidTimestamp,
}

#[cfg(test)]
mod tests {
    use urp_kernel::ids::UserId;

    use super::{
        AuthIdentity, CredentialMetadata, IdentityError, IdentityProvider, IdentityStatus,
        PasswordHashMetadata, UserAccount, UserStatus,
    };

    #[test]
    fn provider_key_rejects_non_technical_values() {
        assert!(IdentityProvider::parse("local").is_ok());
        assert!(IdentityProvider::parse("oidc-google").is_ok());
        assert!(IdentityProvider::parse("OIDC").is_err());
        assert!(IdentityProvider::parse("oidc.google").is_err());
        assert!(IdentityProvider::parse("../oidc").is_err());
    }

    #[test]
    fn credential_debug_never_discloses_encoded_hash() {
        let credential = CredentialMetadata::new(
            UserId::new(),
            PasswordHashMetadata::Argon2id,
            1,
            "$argon2id$v=19$m=65536,t=3,p=1$secret-salt$secret-hash".to_owned(),
            "2026-11-11T00:00:00Z".to_owned(),
            None,
        );
        let debug = format!("{:?}", credential);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("secret-hash"));
        assert!(credential.is_ok());
    }

    #[test]
    fn credential_metadata_rejects_invalid_hash_version_and_empty_hash() {
        let user_id = UserId::new();
        let zero_version = CredentialMetadata::new(
            user_id,
            PasswordHashMetadata::Argon2id,
            0,
            "hash".to_owned(),
            "2026-11-11T00:00:00Z".to_owned(),
            None,
        );
        assert_eq!(zero_version.err(), Some(IdentityError::InvalidHashVersion));

        let empty_hash = CredentialMetadata::new(
            user_id,
            PasswordHashMetadata::Argon2id,
            1,
            " ".to_owned(),
            "2026-11-11T00:00:00Z".to_owned(),
            None,
        );
        assert_eq!(empty_hash.err(), Some(IdentityError::InvalidEncodedHash));
    }

    #[test]
    fn disabled_and_suspended_users_cannot_authenticate() {
        let disabled = UserAccount {
            user_id: UserId::new(),
            display_name: "Disabled User".to_owned(),
            status: UserStatus::Disabled,
            disabled_at: Some("2026-11-11T00:00:00Z".to_owned()),
        };
        let suspended = UserAccount {
            user_id: UserId::new(),
            display_name: "Suspended User".to_owned(),
            status: UserStatus::Suspended,
            disabled_at: None,
        };
        let active = UserAccount {
            user_id: UserId::new(),
            display_name: "Active User".to_owned(),
            status: UserStatus::Active,
            disabled_at: None,
        };

        assert!(!disabled.can_authenticate());
        assert!(!suspended.can_authenticate());
        assert!(active.can_authenticate());
    }

    #[test]
    fn identity_debug_redacts_login_and_email() {
        let provider = IdentityProvider::parse("local");
        assert!(provider.is_ok());
        let Ok(provider) = provider else {
            return;
        };

        let identity = AuthIdentity {
            identity_id: uuid::Uuid::new_v4(),
            user_id: UserId::new(),
            provider,
            provider_subject: None,
            login_normalized: "person@example.test".to_owned(),
            email_normalized: Some("person@example.test".to_owned()),
            email_verified: false,
            status: IdentityStatus::Active,
        };
        let debug = format!("{identity:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("person@example.test"));
    }
}
