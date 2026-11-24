//! Biblioteca de Steam leída de disco, sin credenciales.

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
                if NON_GAME_APP_IDS.contains(&m.app_id) {
                    continue;
                }
                let mut game = new_game(m.app_id);
                game.title = Some(m.name);
                game.installed = m.fully_installed;
                game.install_dir = Some(library.join("steamapps").join("common").join(m.install_dir));
                game.last_played = m.last_played;
                games.insert(m.app_id, game);
            }
        }

        // Horas y última sesión del usuario más reciente. Los juegos jugados pero no
        // instalados entran aquí sin nombre; la Store API lo completa después.
        for (app_id, usage) in localconfig::read_usage(&self.root)? {
            if NON_GAME_APP_IDS.contains(&app_id) {
                continue;
            }
            let known = games.contains_key(&app_id);
            if !known && usage.playtime_minutes.unwrap_or(0) == 0 {
                continue;
            }
            let game = games.entry(app_id).or_insert_with(|| new_game(app_id));
            game.playtime_minutes = usage.playtime_minutes.or(game.playtime_minutes);
            game.last_played = usage.last_played.max(game.last_played);
        }

        Ok(games.into_values().collect())
    }
}

fn new_game(app_id: u32) -> Game {
    Game::new(game_id(app_id), Source::Steam, LaunchTarget::Steam { app_id })
}
