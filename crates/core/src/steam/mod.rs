//! Juegos de Steam instalados, leídos de disco sin credenciales.

mod library_folders;
mod localconfig;
mod locator;
mod manifest;

use crate::game::{Game, LaunchTarget, Source};
use crate::library::LibraryProvider;
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub use locator::steam_root;

/// Paquetes que Steam instala como apps pero no son juegos.
const NON_GAME_APP_IDS: &[u32] = &[
    228980, // Steamworks Common Redistributables
];

pub fn game_id(app_id: u32) -> String {
    format!("steam:{app_id}")
}

pub struct SteamProvider {
    root: PathBuf,
}

impl SteamProvider {
    pub fn detect() -> Option<Self> {
        steam_root().map(|root| SteamProvider { root })
    }

    pub fn at(root: PathBuf) -> Self {
        SteamProvider { root }
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }
}

impl LibraryProvider for SteamProvider {
    fn name(&self) -> &'static str {
        "Steam"
    }

    fn load(&self) -> Result<Vec<Game>> {
        let mut games: HashMap<u32, Game> = HashMap::new();

        for library in library_folders::read(&self.root)? {
            for m in manifest::read_all(&library)? {
                // Una descarga a medias todavía no se puede jugar.
                if !m.fully_installed || NON_GAME_APP_IDS.contains(&m.app_id) {
                    continue;
                }
                let mut game = Game::new(game_id(m.app_id), Source::Steam, m.name, LaunchTarget::Steam { app_id: m.app_id });
                game.install_dir = Some(library.join("steamapps").join("common").join(m.install_dir));
                game.last_played = m.last_played;
                games.insert(m.app_id, game);
            }
        }

        // Horas y última sesión del usuario que inició sesión más recientemente.
        for (app_id, usage) in localconfig::read_usage(&self.root)? {
            let Some(game) = games.get_mut(&app_id) else { continue };
            game.playtime_minutes = usage.playtime_minutes.or(game.playtime_minutes);
            game.last_played = usage.last_played.max(game.last_played);
        }

        Ok(games.into_values().collect())
    }
}
