//! Shared, business-domain-independent primitives for URP.

pub mod clock;
pub mod context;
pub mod errors;
pub mod ids;

pub use clock::{Clock, ClockError, FixedClock, SystemClock};
pub use context::{Actor, ExecutionContext, Locale, TimeZone};
pub use errors::KernelError;
pub use ids::{BranchId, CompanyId, ModuleId, RecordId, ServiceId, TenantId, UserId};
