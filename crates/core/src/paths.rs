//! Carpetas propias de la app. Datos del usuario en Roaming (viajan con el perfil);
//! caché en Local (se puede borrar sin perder nada).

use std::path::PathBuf;

const APP_DIR: &str = "MisJuegos";

pub fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(std::env::temp_dir).join(APP_DIR)
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join(APP_DIR)
}
