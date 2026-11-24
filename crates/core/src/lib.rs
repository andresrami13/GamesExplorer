//! Núcleo de "Mis juegos": todo lo que no depende de la ventana ni de WebView2.
//! Se compila y se prueba en cualquier sistema; lo específico de Windows va tras `cfg(windows)`.

pub mod epic;
pub mod game;
pub mod http;
pub mod images;
pub mod library;
pub mod metadata;
pub mod paths;
pub mod persistence;
pub mod snapshot;
pub mod steam;
pub mod vdf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("formato VDF: {0}")]
    Parse(String),
    #[error("red: {0}")]
    Http(String),
    #[error("imagen: {0}")]
    Image(#[from] image::ImageError),
    #[error("no encontrado: {0}")]
    NotFound(String),
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        Error::Http(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
