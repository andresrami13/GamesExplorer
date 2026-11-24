use mis_juegos_core::game::Game;
use mis_juegos_core::images::ImageCache;
use mis_juegos_core::metadata::StoreClient;
use mis_juegos_core::{paths, steam};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

pub struct AppState {
    pub games: RwLock<Vec<Game>>,
    pub images: Arc<ImageCache>,
    pub store: Arc<StoreClient>,
    /// Evita lanzar dos enriquecedores a la vez si se refresca varias veces.
    pub enriching: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        let cache = paths::cache_dir();
        AppState {
            games: RwLock::new(Vec::new()),
            images: Arc::new(ImageCache::new(cache.join("images"), steam::steam_root())),
            store: Arc::new(StoreClient::new(cache.join("store"), "spanish")),
            enriching: AtomicBool::new(false),
        }
    }

    pub fn find(&self, id: &str) -> Option<Game> {
        self.games.read().unwrap().iter().find(|g| g.id == id).cloned()
    }
}
