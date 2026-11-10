//! Authentication identity, password and login rate-limit contracts.

mod identity;
mod password;
mod rate_limit;

pub use identity::{
    AuthIdentity, CredentialMetadata, IdentityError, IdentityProvider, IdentityStatus,
    PasswordHashMetadata, UserAccount, UserStatus,
};
pub use password::{
    EncodedPasswordHash, PasswordError, PasswordHashConfig, PasswordService,
};
pub use rate_limit::LoginRateLimiter;
