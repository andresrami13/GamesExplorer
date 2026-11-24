use crate::{vdf, Result};
use std::path::Path;

/// Bit `StateFlags` de "completamente instalado". Con actualización pendiente sigue activo.
const STATE_FULLY_INSTALLED: u32 = 4;

pub struct Manifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: String,
    pub fully_installed: bool,
    pub last_played: Option<i64>,
}

/// Lee todos los `steamapps/appmanifest_*.acf` de una carpeta de biblioteca.
/// Un manifiesto corrupto se salta: no debe impedir ver el resto de la biblioteca.
pub fn read_all(library: &Path) -> Result<Vec<Manifest>> {
    let dir = library.join("steamapps");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(Vec::new()) };
    let mut manifests = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("appmanifest_") && name.ends_with(".acf") {
            if let Some(m) = std::fs::read_to_string(entry.path()).ok().and_then(|s| parse(&s)) {
                manifests.push(m);
            }
        }
    }
    Ok(manifests)
}

fn parse(text: &str) -> Option<Manifest> {
    let doc = vdf::parse(text).ok()?;
    let state = doc.get("AppState")?;
    Some(Manifest {
        app_id: state.num("appid")?,
        name: state.str("name")?.to_owned(),
        install_dir: state.str("installdir").unwrap_or_default().to_owned(),
        fully_installed: state.num::<u32>("StateFlags").unwrap_or(0) & STATE_FULLY_INSTALLED != 0,
        last_played: state.num::<i64>("LastPlayed").filter(|t| *t > 0),
    })
}
