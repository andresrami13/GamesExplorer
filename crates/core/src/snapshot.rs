//! Instantánea de la biblioteca ya enriquecida (nombres, colores, metadatos).
//! Se pinta al arrancar sin tocar Steam ni la red; así el arranque con caché es inmediato.

use crate::game::{Game, GameId};
use crate::{paths, persistence, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Sube al cambiar el formato de `Game`: una instantánea vieja se descarta y se relee.
const FORMAT_VERSION: u32 = 2;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    version: u32,
    pub games: Vec<Game>,
}

pub fn path() -> PathBuf {
    paths::cache_dir().join("library.json")
}

pub fn load() -> Option<Snapshot> {
    persistence::read_json::<Snapshot>(&path())
        .ok()
        .flatten()
        .filter(|s| s.version == FORMAT_VERSION)
}

pub fn save(games: &[Game]) -> Result<()> {
    persistence::write_json(&path(), &Snapshot { version: FORMAT_VERSION, games: games.to_vec() })
}

/// Combina una lectura nueva de disco con lo ya enriquecido: conserva color y metadatos
/// obtenidos por red, y descarta los juegos que ya no están instalados.
pub fn merge(fresh: Vec<Game>, previous: &[Game]) -> Vec<Game> {
    let previous: HashMap<&GameId, &Game> = previous.iter().map(|g| (&g.id, g)).collect();
    fresh
        .into_iter()
        .map(|mut game| {
            if let Some(old) = previous.get(&game.id) {
                game.color = game.color.or_else(|| old.color.clone());
                game.metadata = game.metadata.or_else(|| old.metadata.clone());
            }
            game
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{LaunchTarget, Source};

    #[test]
    fn merge_keeps_enrichment_and_drops_removed_games() {
        let game = |id: u32| Game::new(format!("steam:{id}"), Source::Steam, "Juego".into(), LaunchTarget::Steam { app_id: id });
        let mut old = game(1);
        old.color = Some("#112233".into());
        let gone = game(2);
        let fresh = game(1);

        let merged = merge(vec![fresh], &[old, gone]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].color.as_deref(), Some("#112233"));
    }
}
