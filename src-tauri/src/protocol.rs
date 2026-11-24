//! Protocolo `img`: la interfaz pide `img://localhost/steam/<appid>/<tipo>`
//! (`http://img.localhost/...` en Windows) y aquí se resuelve la imagen a disco.
//! Así las imágenes nunca viajan por IPC en base64.

use crate::state::AppState;
use mis_juegos_core::images::{ArtKind, ImageCache};
use std::sync::Arc;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

pub const SCHEME: &str = "img";

pub fn handle<R: Runtime>(ctx: UriSchemeContext<'_, R>, request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let images = Arc::clone(&ctx.app_handle().state::<AppState>().images);
    let path = request.uri().path().to_owned();
    tauri::async_runtime::spawn_blocking(move || responder.respond(serve(&images, &path)));
}

fn serve(images: &ImageCache, path: &str) -> Response<Vec<u8>> {
    let mut parts = path.trim_matches('/').split('/');
    let (Some("steam"), Some(app_id), Some(kind), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return status(StatusCode::BAD_REQUEST);
    };
    let (Ok(app_id), Some(kind)) = (app_id.parse::<u32>(), ArtKind::parse(kind)) else {
        return status(StatusCode::BAD_REQUEST);
    };
    match images.steam_art(app_id, kind).map(|p| p.and_then(|p| std::fs::read(p).ok())) {
        Ok(Some(bytes)) => Response::builder()
            .header(header::CONTENT_TYPE, kind.mime())
            .header(header::CACHE_CONTROL, "max-age=604800, immutable")
            .body(bytes)
            .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR)),
        Ok(None) => status(StatusCode::NOT_FOUND),
        Err(_) => status(StatusCode::BAD_GATEWAY),
    }
}

fn status(code: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = code;
    response
}
