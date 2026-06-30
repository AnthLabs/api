#[derive(Clone)]
pub struct AppState {
    pub database: mongodb::Database,
    pub secret_store: SecretStore,
    pub started_at: std::time::Instant,
}

#[derive(Clone)]
pub struct SecretStore;

impl SecretStore {
    pub fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
