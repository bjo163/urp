//! Authentication identity and credential metadata contracts.
//!
//! This crate does not implement password verification or session issuance.
//! Those behaviors belong to separate application-layer capabilities.

mod identity;

pub use identity::{
    AuthIdentity, CredentialMetadata, IdentityError, IdentityProvider, IdentityStatus,
    PasswordHashMetadata, UserAccount, UserStatus,
};
