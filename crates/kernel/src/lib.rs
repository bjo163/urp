//! Shared, business-domain-independent primitives for URP.

pub mod clock;
pub mod errors;
pub mod ids;

pub use clock::{Clock, SystemClock};
pub use errors::KernelError;
pub use ids::{BranchId, CompanyId, ModuleId, TenantId, UserId};
