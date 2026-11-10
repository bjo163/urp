//! Strongly typed identifiers used at URP boundaries.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[doc = concat!("A strongly typed ", stringify!($name), " value.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new random identifier.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wraps an existing UUID.
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            /// Returns the underlying UUID.
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, formatter)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self)
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }
    };
}

define_id!(TenantId);
define_id!(UserId);
define_id!(CompanyId);
define_id!(BranchId);
define_id!(ModuleId);
define_id!(ServiceId);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{CompanyId, TenantId};

    #[test]
    fn identifiers_round_trip_through_strings() {
        let id = TenantId::new();
        let parsed = TenantId::from_str(&id.to_string());
        assert_eq!(parsed.ok(), Some(id));
    }

    #[test]
    fn identifiers_of_different_types_are_distinct_types() {
        let tenant = TenantId::new();
        let company = CompanyId::from_uuid(*tenant.as_uuid());
        assert_eq!(tenant.to_string(), company.to_string());
    }
}
