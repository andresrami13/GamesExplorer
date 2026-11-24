//! Une las bibliotecas de todas las plataformas.

use crate::game::Game;
use crate::{epic, steam, Result};

pub trait LibraryProvider {
    fn name(&self) -> &'static str;
    fn load(&self) -> Result<Vec<Game>>;
}

/// Proveedores disponibles en esta máquina. Una plataforma no instalada simplemente no aparece.
pub fn detect_providers() -> Vec<Box<dyn LibraryProvider + Send + Sync>> {
    let mut providers: Vec<Box<dyn LibraryProvider + Send + Sync>> = Vec::new();
    if let Some(p) = steam::SteamProvider::detect() {
        providers.push(Box::new(p));
    }
    if let Some(p) = epic::EpicProvider::detect() {
        providers.push(Box::new(p));
    }
    providers
}

/// Carga todas las bibliotecas. Un proveedor que falla no tumba a los demás:
/// su error se devuelve aparte para mostrarlo en la interfaz.
pub fn load_all(providers: &[Box<dyn LibraryProvider + Send + Sync>]) -> (Vec<Game>, Vec<String>) {
    let mut games = Vec::new();
    let mut errors = Vec::new();
    for provider in providers {
        match provider.load() {
            Ok(mut found) => games.append(&mut found),
            Err(e) => errors.push(format!("{}: {e}", provider.name())),
        }
    }
    games.sort_by(|a, b| a.id.cmp(&b.id));
    games.dedup_by(|a, b| a.id == b.id);
    (games, errors)
}
