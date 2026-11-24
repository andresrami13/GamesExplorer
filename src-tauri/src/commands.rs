use crate::{enrich, state::AppState};
use mis_juegos_core::game::Game;
use mis_juegos_core::{library, snapshot};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize)]
pub struct Library {
    games: Vec<Game>,
    /// Errores por plataforma ("Epic Games: ..."). La app sigue con lo que sí pudo leer.
    errors: Vec<String>,
}

/// Instantánea de la última sesión, sin tocar Steam ni la red: el arranque rápido.
#[tauri::command]
pub fn get_cached_library(state: State<'_, AppState>) -> Vec<Game> {
    let games = snapshot::load().map(|s| s.games).unwrap_or_default();
    *state.games.write().unwrap() = games.clone();
    games
}

/// Relee las bibliotecas de disco y arranca el enriquecimiento en segundo plano.
#[tauri::command]
pub async fn refresh_library(app: AppHandle, state: State<'_, AppState>) -> Result<Library, String> {
    let (fresh, errors) = tauri::async_runtime::spawn_blocking(|| library::load_all(&library::detect_providers()))
        .await
        .map_err(|e| e.to_string())?;

    let games = {
        let mut current = state.games.write().unwrap();
        *current = snapshot::merge(fresh, &current);
        current.clone()
    };
    if let Err(e) = snapshot::save(&games) {
        eprintln!("no se pudo guardar la instantánea: {e}");
    }
    enrich::start(app);
    Ok(Library { games, errors })
}

#[tauri::command]
pub fn launch_game(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    open(&app, &state, &id, |g| g.launch.launch_uri())
}

#[tauri::command]
pub fn open_store_page(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    open(&app, &state, &id, |g| g.launch.store_page_uri())
}

/// El URI se construye siempre a partir del juego conocido, nunca del texto que llega
/// del frontend: así la interfaz no puede abrir URIs arbitrarios.
fn open(app: &AppHandle, state: &AppState, id: &str, uri: impl Fn(&Game) -> String) -> Result<(), String> {
    let game = state.find(id).ok_or_else(|| format!("juego desconocido: {id}"))?;
    app.opener().open_url(uri(&game), None::<&str>).map_err(|e| e.to_string())
}
