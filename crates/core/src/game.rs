use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Steam,
    Epic,
}

/// Identificador estable y único entre plataformas: `steam:570`, `epic:Fortnite`.
pub type GameId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LaunchTarget {
    Steam { app_id: u32 },
    Epic { namespace: String, catalog_item_id: String, app_name: String },
}

impl LaunchTarget {
    /// URI que arranca el juego a través de su launcher (con DRM, overlay y horas).
    pub fn launch_uri(&self) -> String {
        match self {
            LaunchTarget::Steam { app_id } => format!("steam://rungameid/{app_id}"),
            LaunchTarget::Epic { namespace, catalog_item_id, app_name } => format!(
                "com.epicgames.launcher://apps/{namespace}%3A{catalog_item_id}%3A{app_name}?action=launch&silent=true"
            ),
        }
    }

    /// URI que abre la ficha del juego en su launcher.
    pub fn store_page_uri(&self) -> String {
        match self {
            LaunchTarget::Steam { app_id } => format!("steam://nav/games/details/{app_id}"),
            // Epic no expone un URI documentado para la ficha de la biblioteca: se abre el launcher.
            LaunchTarget::Epic { .. } => "com.epicgames.launcher://store".into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub year: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: GameId,
    pub source: Source,
    /// `None` cuando la fuente local no trae nombre (juegos de Steam no instalados);
    /// lo rellena la Store API.
    pub title: Option<String>,
    pub installed: bool,
    pub install_dir: Option<PathBuf>,
    pub playtime_minutes: Option<u32>,
    /// Segundos Unix.
    pub last_played: Option<i64>,
    pub launch: LaunchTarget,
    /// Color dominante de la portada, `#rrggbb`.
    pub color: Option<String>,
    pub metadata: Option<Metadata>,
}

impl Game {
    pub fn new(id: GameId, source: Source, launch: LaunchTarget) -> Self {
        Game {
            id,
            source,
            title: None,
            installed: false,
            install_dir: None,
            playtime_minutes: None,
            last_played: None,
            launch,
            color: None,
            metadata: None,
        }
    }

    pub fn steam_app_id(&self) -> Option<u32> {
        match self.launch {
            LaunchTarget::Steam { app_id } => Some(app_id),
            LaunchTarget::Epic { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_launch_uris() {
        assert_eq!(LaunchTarget::Steam { app_id: 570 }.launch_uri(), "steam://rungameid/570");
        let epic = LaunchTarget::Epic {
            namespace: "ns".into(),
            catalog_item_id: "cat".into(),
            app_name: "App".into(),
        };
        assert_eq!(
            epic.launch_uri(),
            "com.epicgames.launcher://apps/ns%3Acat%3AApp?action=launch&silent=true"
        );
    }
}
