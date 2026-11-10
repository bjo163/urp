use urp_storage::Database;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub version: &'static str,
}
