use crate::{vdf, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Diferencia entre un SteamID64 y el accountid que nombra la carpeta `userdata/<accountid>`.
const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;

#[derive(Debug, Default, PartialEq)]
pub struct Usage {
    pub playtime_minutes: Option<u32>,
    pub last_played: Option<i64>,
}

/// Horas y última sesión por appid, sacadas de `userdata/<id>/config/localconfig.vdf`
/// del usuario que inició sesión más recientemente.
pub fn read_usage(root: &Path) -> Result<HashMap<u32, Usage>> {
    let Some(file) = localconfig_path(root) else { return Ok(HashMap::new()) };
    let doc = vdf::parse(&std::fs::read_to_string(file)?)?;
    let Some(apps) = doc.path(&["UserLocalConfigStore", "Software", "Valve", "Steam", "apps"]) else {
        return Ok(HashMap::new());
    };
    Ok(apps
        .entries()
        .iter()
        .filter_map(|(id, app)| {
            let usage = Usage {
                playtime_minutes: app.num("Playtime"),
                last_played: app.num::<i64>("LastPlayed").filter(|t| *t > 0),
            };
            Some((id.parse().ok()?, usage))
        })
        .collect())
}

fn localconfig_path(root: &Path) -> Option<PathBuf> {
    let file_for = |account: &str| root.join("userdata").join(account).join("config").join("localconfig.vdf");
    if let Some(account) = most_recent_account(root) {
        let file = file_for(&account.to_string());
        if file.is_file() {
            return Some(file);
        }
    }
    // Sin loginusers.vdf válido: el localconfig modificado más recientemente.
    std::fs::read_dir(root.join("userdata"))
        .ok()?
        .flatten()
        .map(|e| file_for(&e.file_name().to_string_lossy()))
        .filter_map(|f| Some((f.metadata().ok()?.modified().ok()?, f)))
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, f)| f)
}

fn most_recent_account(root: &Path) -> Option<u64> {
    let text = std::fs::read_to_string(root.join("config").join("loginusers.vdf")).ok()?;
    let doc = vdf::parse(&text).ok()?;
    let users = doc.get("users")?.entries();
    let pick = users
        .iter()
        .find(|(_, u)| u.str("MostRecent") == Some("1"))
        .or_else(|| users.iter().max_by_key(|(_, u)| u.num::<u64>("Timestamp").unwrap_or(0)))?;
    pick.0.parse::<u64>().ok()?.checked_sub(STEAM_ID64_BASE)
}
