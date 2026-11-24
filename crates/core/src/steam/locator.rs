use std::path::PathBuf;

/// Variable de entorno para forzar la ruta (desarrollo, tests o instalaciones portables).
pub const STEAM_ROOT_ENV: &str = "MIS_JUEGOS_STEAM_ROOT";

/// Carpeta raíz de Steam: la variable de entorno, o el registro de Windows.
pub fn steam_root() -> Option<PathBuf> {
    std::env::var_os(STEAM_ROOT_ENV)
        .map(PathBuf::from)
        .into_iter()
        .chain(registry_candidates())
        .find(|p| p.join("steamapps").is_dir())
}

#[cfg(windows)]
fn registry_candidates() -> Vec<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let read = |hive, key: &str, value: &str| -> Option<PathBuf> {
        let s: String = RegKey::predef(hive).open_subkey(key).ok()?.get_value(value).ok()?;
        Some(PathBuf::from(s))
    };
    [
        read(HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath"),
        read(HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"),
        read(HKEY_LOCAL_MACHINE, r"SOFTWARE\Valve\Steam", "InstallPath"),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(not(windows))]
fn registry_candidates() -> Vec<PathBuf> {
    Vec::new()
}
