use urp_auth::{LoginRateLimiter, PasswordService};
use urp_storage::Database;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub password_service: PasswordService,
    pub login_rate_limiter: LoginRateLimiter,
    pub version: &'static str,
}
