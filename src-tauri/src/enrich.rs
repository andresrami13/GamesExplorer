//! Completa en segundo plano lo que no está en disco de Steam: color dominante de la
//! portada y metadatos de la tienda.
//! Cada juego terminado se emite al frontend con el evento `game-updated`.

use crate::state::AppState;
use mis_juegos_core::game::Game;
use mis_juegos_core::images::{dominant_color, ArtKind};
use mis_juegos_core::snapshot;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

pub const GAME_UPDATED: &str = "game-updated";
pub const GAME_REMOVED: &str = "game-removed";

/// Guardar la instantánea cada tantos juegos, para no perder el trabajo si se cierra la app.
const SAVE_EVERY: usize = 20;

pub fn start(app: AppHandle) {
    let state = app.state::<AppState>();
    if state.enriching.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        run(&app);
        app.state::<AppState>().enriching.store(false, Ordering::SeqCst);
    });
}

fn run(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut pending: Vec<Game> = state
        .games
        .read()
        .unwrap()
        .iter()
        .filter(|g| g.steam_app_id().is_some() && (g.color.is_none() || g.metadata.is_none()))
        .cloned()
        .collect();
    // Primero lo jugado hace poco: es lo que el usuario más probablemente mira.
    pending.sort_by_key(|g| std::cmp::Reverse(g.last_played));

    for (done, mut game) in pending.into_iter().enumerate() {
        let keep = enrich_one(&state, &mut game);
        {
            let mut games = state.games.write().unwrap();
            if keep {
                if let Some(slot) = games.iter_mut().find(|g| g.id == game.id) {
                    *slot = game.clone();
                }
            } else {
                games.retain(|g| g.id != game.id);
            }
        }
        let _ = if keep { app.emit(GAME_UPDATED, &game) } else { app.emit(GAME_REMOVED, &game.id) };
        if (done + 1) % SAVE_EVERY == 0 {
            save(&state);
        }
    }
    save(&state);
}

/// Devuelve `false` si el juego debe desaparecer: la tienda dice que no es un juego
/// (herramienta, banda sonora, SDK...).
fn enrich_one(state: &AppState, game: &mut Game) -> bool {
    let Some(app_id) = game.steam_app_id() else { return true };

    if game.color.is_none() {
        if let Ok(Some(cover)) = state.images.steam_art(app_id, ArtKind::Capsule) {
            game.color = dominant_color(&cover).ok();
        }
    }
    if game.metadata.is_none() {
        match state.store.fetch(app_id) {
            Ok(Some(info)) => {
                if !info.is_playable() {
                    return false;
                }
                game.metadata = Some(info.metadata);
            }
            // Sin ficha en la tienda (app retirada): se queda con lo que hay.
            Ok(None) => {}
            // Sin red o con límite: se reintenta en el próximo arranque.
            Err(e) => eprintln!("metadatos de {app_id}: {e}"),
        }
    }
    true
}

fn save(state: &AppState) {
    if let Err(e) = snapshot::save(&state.games.read().unwrap()) {
        eprintln!("no se pudo guardar la instantánea: {e}");
    }
}
