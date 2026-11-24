//! Portadas y artes: caché propia → caché local de Steam → CDN público de Steam.

mod color;

pub use color::dominant_color;

use crate::http::Http;
use crate::{persistence, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Tras un 404 no se vuelve a pedir la misma imagen hasta pasado este tiempo.
const MISSING_RETRY_AFTER: Duration = Duration::from_secs(7 * 24 * 3600);

const STEAM_CDN_BASES: &[&str] = &[
    "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps",
    "https://cdn.cloudflare.steamstatic.com/steam/apps",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtKind {
    /// Portada vertical 600×900.
    Capsule,
    /// Fondo ancho sin logo.
    Hero,
    /// Logo con transparencia.
    Logo,
    /// Cabecera horizontal 460×215.
    Header,
}

impl ArtKind {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "capsule" => ArtKind::Capsule,
            "hero" => ArtKind::Hero,
            "logo" => ArtKind::Logo,
            "header" => ArtKind::Header,
            _ => return None,
        })
    }

    fn steam_file(self) -> &'static str {
        match self {
            ArtKind::Capsule => "library_600x900.jpg",
            ArtKind::Hero => "library_hero.jpg",
            ArtKind::Logo => "logo.png",
            ArtKind::Header => "header.jpg",
        }
    }

    pub fn mime(self) -> &'static str {
        match self {
            ArtKind::Logo => "image/png",
            _ => "image/jpeg",
        }
    }
}

pub struct ImageCache {
    cache_root: PathBuf,
    steam_root: Option<PathBuf>,
    http: Http,
}

impl ImageCache {
    pub fn new(cache_root: PathBuf, steam_root: Option<PathBuf>) -> Self {
        ImageCache { cache_root, steam_root, http: Http::new() }
    }

    /// Ruta local del arte de un juego de Steam, descargándolo si hace falta.
    /// `Ok(None)` significa que no existe en ningún sitio (o se comprobó hace poco).
    pub fn steam_art(&self, app_id: u32, kind: ArtKind) -> Result<Option<PathBuf>> {
        let own = self.cache_root.join("steam").join(app_id.to_string()).join(kind.steam_file());
        if own.is_file() {
            return Ok(Some(own));
        }
        if let Some(local) = self.steam_root.as_deref().and_then(|r| find_in_librarycache(r, app_id, kind)) {
            return Ok(Some(local));
        }
        let missing = own.with_extension("missing");
        if is_recent(&missing) {
            return Ok(None);
        }
        for base in STEAM_CDN_BASES {
            let response = self.http.get(&format!("{base}/{app_id}/{}", kind.steam_file()))?;
            if response.status == 200 && !response.body.is_empty() {
                persistence::write_atomic(&own, &response.body)?;
                return Ok(Some(own));
            }
        }
        persistence::write_atomic(&missing, &[])?;
        Ok(None)
    }
}

/// Steam guardaba `librarycache/<appid>_library_600x900.jpg`; desde 2024 usa
/// `librarycache/<appid>/library_600x900.jpg`, a veces dentro de una subcarpeta con hash.
fn find_in_librarycache(steam_root: &Path, app_id: u32, kind: ArtKind) -> Option<PathBuf> {
    let cache = steam_root.join("appcache").join("librarycache");
    let file = kind.steam_file();
    let app_dir = cache.join(app_id.to_string());

    let flat = cache.join(format!("{app_id}_{file}"));
    let direct = app_dir.join(file);
    if let Some(found) = [flat, direct].into_iter().find(|p| p.is_file()) {
        return Some(found);
    }
    std::fs::read_dir(&app_dir)
        .ok()?
        .flatten()
        .map(|e| e.path().join(file))
        .find(|p| p.is_file())
}

fn is_recent(marker: &Path) -> bool {
    marker
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < MISSING_RETRY_AFTER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_all_librarycache_layouts() {
        let root = tempfile::tempdir().unwrap();
        let cache = root.path().join("appcache").join("librarycache");
        std::fs::create_dir_all(cache.join("20").join("abc123")).unwrap();
        std::fs::create_dir_all(cache.join("30")).unwrap();
        std::fs::write(cache.join("10_library_600x900.jpg"), b"x").unwrap();
        std::fs::write(cache.join("20").join("abc123").join("logo.png"), b"x").unwrap();
        std::fs::write(cache.join("30").join("library_hero.jpg"), b"x").unwrap();

        assert!(find_in_librarycache(root.path(), 10, ArtKind::Capsule).is_some());
        assert!(find_in_librarycache(root.path(), 20, ArtKind::Logo).is_some());
        assert!(find_in_librarycache(root.path(), 30, ArtKind::Hero).is_some());
        assert!(find_in_librarycache(root.path(), 30, ArtKind::Capsule).is_none());
    }

    #[test]
    fn own_cache_wins_without_network() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("steam").join("570").join("header.jpg");
        persistence::write_atomic(&file, b"x").unwrap();
        let cache = ImageCache::new(dir.path().to_path_buf(), None);
        assert_eq!(cache.steam_art(570, ArtKind::Header).unwrap(), Some(file));
    }

    #[test]
    fn recent_missing_marker_skips_network() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("steam").join("570").join("logo.missing");
        persistence::write_atomic(&marker, &[]).unwrap();
        let cache = ImageCache::new(dir.path().to_path_buf(), None);
        assert_eq!(cache.steam_art(570, ArtKind::Logo).unwrap(), None);
    }
}
