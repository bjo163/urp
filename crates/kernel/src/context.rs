//! Trusted execution context shared by API, worker and domain boundaries.

use uuid::Uuid;

use crate::{
    errors::KernelError,
    ids::{BranchId, CompanyId, ServiceId, TenantId, UserId},
};

/// The identity under which an operation is executed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Actor {
    /// An authenticated human user.
    User(UserId),
    /// An authenticated integration or background service.
    Service(ServiceId),
    /// A platform-owned system operation.
    System,
}

/// A validated locale identifier used for formatting and localization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Locale(String);

impl Locale {
    /// Creates a locale from a conservative language-tag format.
    pub fn parse(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 35
            && value.split('-').all(|part| {
                !part.is_empty()
                    && part.len() <= 8
                    && part.chars().all(|character| character.is_ascii_alphanumeric())
            });

        if valid {
            Ok(Self(value))
        } else {
            Err(KernelError::Validation {
                field: "locale",
                message: "locale must be a valid language tag".to_owned(),
            })
        }
    }

    /// Returns the locale string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A timezone identifier carried through trusted execution context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeZone(String);

impl TimeZone {
    /// Creates a timezone identifier. Calendar adapters perform authoritative
    /// timezone-database validation when applying calendar rules.
    pub fn parse(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 64
            && value.trim() == value
            && !value.chars().any(char::is_control);

        if valid {
            Ok(Self(value))
        } else {
            Err(KernelError::Validation {
                field: "timezone",
                message: "timezone identifier must be non-empty and contain no control characters"
                    .to_owned(),
            })
        }
    }

    /// Returns the timezone identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated context attached by an authenticated API or worker boundary.
///
/// This type intentionally does not implement deserialization: client request
/// bodies must never be able to deserialize a trusted execution context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionContext {
    actor: Actor,
    tenant_id: TenantId,
    company_id: Option<CompanyId>,
    branch_id: Option<BranchId>,
    locale: Locale,
    timezone: TimeZone,
    request_id: Uuid,
    correlation_id: Uuid,
}

impl ExecutionContext {
    /// Builds a context from identity and scope values already resolved by a
    /// trusted authentication/membership boundary.
    #[allow(clippy::too_many_arguments)]
    pub fn from_trusted_boundary(
        actor: Actor,
        tenant_id: TenantId,
        company_id: Option<CompanyId>,
        branch_id: Option<BranchId>,
        locale: Locale,
        timezone: TimeZone,
        request_id: Uuid,
        correlation_id: Uuid,
    ) -> Result<Self, KernelError> {
        if branch_id.is_some() && company_id.is_none() {
            return Err(KernelError::Validation {
                field: "branch_id",
                message: "a branch context requires a company context".to_owned(),
            });
        }

        Ok(Self {
            actor,
            tenant_id,
            company_id,
            branch_id,
            locale,
            timezone,
            request_id,
            correlation_id,
        })
    }

    /// Returns the actor that initiated the operation.
    pub const fn actor(&self) -> &Actor {
        &self.actor
    }

    /// Returns the tenant isolation boundary.
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Returns the selected company, if one is active.
    pub const fn company_id(&self) -> Option<CompanyId> {
        self.company_id
    }

    /// Returns the selected branch, if one is active.
    pub const fn branch_id(&self) -> Option<BranchId> {
        self.branch_id
    }

    /// Returns the selected locale.
    pub const fn locale(&self) -> &Locale {
        &self.locale
    }

    /// Returns the selected timezone.
    pub const fn timezone(&self) -> &TimeZone {
        &self.timezone
    }

    /// Returns the request identifier.
    pub const fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Returns the distributed correlation identifier.
    pub const fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{Actor, ExecutionContext, Locale, TimeZone};
    use crate::{
        errors::KernelError,
        ids::{BranchId, TenantId, UserId},
    };

    #[test]
    fn locale_accepts_language_and_region_tags() {
        let locale = Locale::parse("id-ID");
        assert!(locale.is_ok_and(|value| value.as_str() == "id-ID"));
    }

    #[test]
    fn locale_rejects_empty_and_malformed_tags() {
        assert!(Locale::parse("").is_err());
        assert!(Locale::parse("id--ID").is_err());
    }

    #[test]
    fn branch_requires_company_context() {
        let result = ExecutionContext::from_trusted_boundary(
            Actor::User(UserId::new()),
            TenantId::new(),
            None,
            Some(BranchId::new()),
            Locale("en-US".to_owned()),
            TimeZone("Asia/Jakarta".to_owned()),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        assert!(matches!(
            result,
            Err(KernelError::Validation { field: "branch_id", .. })
        ));
    }

    #[test]
    fn valid_context_keeps_actor_and_scope() {
        let actor = Actor::User(UserId::new());
        let tenant_id = TenantId::new();
        let request_id = Uuid::new_v4();
        let context = ExecutionContext::from_trusted_boundary(
            actor.clone(),
            tenant_id,
            None,
            None,
            Locale("en-US".to_owned()),
            TimeZone("UTC".to_owned()),
            request_id,
            request_id,
        );

        assert!(context.is_ok_and(|value| {
            value.actor() == &actor
                && value.tenant_id() == tenant_id
                && value.request_id() == request_id
        }));
    }
}
