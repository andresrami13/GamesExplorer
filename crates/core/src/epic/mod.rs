//! Juegos instalados de Epic Games, leídos de los manifiestos `.item` del launcher.
//! Epic no expone en local los juegos comprados pero no instalados.

use crate::game::{Game, LaunchTarget, Source};
use crate::library::LibraryProvider;
use crate::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const EPIC_MANIFESTS_ENV: &str = "MIS_JUEGOS_EPIC_MANIFESTS";

pub struct EpicProvider {
    manifests_dir: PathBuf,
}

impl EpicProvider {
    pub fn detect() -> Option<Self> {
        let default = std::env::var_os("ProgramData")
            .map(|p| PathBuf::from(p).join(r"Epic\EpicGamesLauncher\Data\Manifests"));
        std::env::var_os(EPIC_MANIFESTS_ENV)
            .map(PathBuf::from)
            .or(default)
            .filter(|p| p.is_dir())
            .map(EpicProvider::at)
    }

    pub fn at(manifests_dir: PathBuf) -> Self {
        EpicProvider { manifests_dir }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Item {
    display_name: String,
    app_name: String,
    catalog_namespace: String,
    catalog_item_id: String,
    install_location: Option<PathBuf>,
    #[serde(default)]
    app_categories: Vec<String>,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete: bool,
}

impl LibraryProvider for EpicProvider {
    fn name(&self) -> &'static str {
        "Epic Games"
    }

    fn load(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        for entry in std::fs::read_dir(&self.manifests_dir)?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("item")) {
                if let Some(game) = read_item(&path) {
                    games.push(game);
                }
            }
        }
        Ok(games)
    }
}

/// Un manifiesto ilegible, un DLC (`addons`) o una instalación a medias se ignoran.
fn read_item(path: &Path) -> Option<Game> {
    let item: Item = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    if item.incomplete || item.app_categories.iter().any(|c| c == "addons") {
        return None;
    }
    let mut game = Game::new(
        format!("epic:{}", item.app_name),
        Source::Epic,
        item.display_name,
        LaunchTarget::Epic {
            namespace: item.catalog_namespace,
            catalog_item_id: item.catalog_item_id,
            app_name: item.app_name,
        },
    );
    game.install_dir = item.install_location;
    Some(game)
}
