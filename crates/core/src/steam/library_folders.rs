use crate::{vdf, Result};
use std::path::{Path, PathBuf};

/// Carpetas de biblioteca: la raíz de Steam más las de `steamapps/libraryfolders.vdf`.
/// Soporta el formato actual (`"0" { "path" "..." }`) y el antiguo (`"1" "D:\\Juegos"`).
pub fn read(root: &Path) -> Result<Vec<PathBuf>> {
    let mut folders = vec![root.to_path_buf()];
    let file = root.join("steamapps").join("libraryfolders.vdf");
    if !file.is_file() {
        return Ok(folders);
    }
    let doc = vdf::parse(&std::fs::read_to_string(file)?)?;
    let Some(list) = doc.get("libraryfolders") else { return Ok(folders) };

    for (key, value) in list.entries() {
        if key.parse::<u32>().is_err() {
            continue; // "contentstatsid", "TimeNextStatsReport"...
        }
        let path = match value {
            vdf::Vdf::Str(p) => Some(p.as_str()),
            obj => obj.str("path"),
        };
        if let Some(path) = path.map(PathBuf::from) {
            if !folders.iter().any(|f| same_path(f, &path)) {
                folders.push(path);
            }
        }
    }
    Ok(folders)
}

/// Windows no distingue mayúsculas ni el tipo de barra.
fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_lowercase();
    norm(a) == norm(b)
}
