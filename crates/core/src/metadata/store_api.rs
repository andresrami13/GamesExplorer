use super::RateLimiter;
use crate::game::Metadata;
use crate::http::Http;
use crate::{persistence, Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CACHE_TTL: Duration = Duration::from_secs(30 * 24 * 3600);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreInfo {
    pub name: String,
    /// `game`, `dlc`, `demo`, `music`, `application`...
    pub kind: String,
    pub metadata: Metadata,
}

impl StoreInfo {
    pub fn is_playable(&self) -> bool {
        matches!(self.kind.as_str(), "game" | "demo")
    }
}

/// Lo que se guarda en disco. `info: None` = la tienda no tiene ficha (app retirada):
/// también se cachea, para no preguntar por ella en cada arranque.
#[derive(Serialize, Deserialize)]
struct Cached {
    fetched_at: u64,
    info: Option<StoreInfo>,
}

pub struct StoreClient {
    cache_dir: PathBuf,
    language: String,
    http: Http,
    limiter: Mutex<RateLimiter>,
}

impl StoreClient {
    pub fn new(cache_dir: PathBuf, language: &str) -> Self {
        StoreClient { cache_dir, language: language.into(), http: Http::new(), limiter: Mutex::new(RateLimiter::new()) }
    }

    /// Solo caché, sin red. Para pintar al instante.
    pub fn cached(&self, app_id: u32) -> Option<Option<StoreInfo>> {
        let cached: Cached = persistence::read_json(&self.cache_file(app_id)).ok()??;
        let age = now_secs().saturating_sub(cached.fetched_at);
        (Duration::from_secs(age) < CACHE_TTL).then_some(cached.info)
    }

    /// Caché o red. Bloquea el hilo mientras el limitador lo pida: llamar desde un hilo de fondo.
    pub fn fetch(&self, app_id: u32) -> Result<Option<StoreInfo>> {
        if let Some(info) = self.cached(app_id) {
            return Ok(info);
        }
        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={app_id}&l={}",
            self.language
        );
        let response = loop {
            let wait = self.limiter.lock().unwrap().delay(Instant::now());
            std::thread::sleep(wait);
            let response = self.http.get(&url)?;
            let mut limiter = self.limiter.lock().unwrap();
            match response.status {
                429 | 403 => limiter.on_throttled(Instant::now()),
                _ => {
                    limiter.on_success(Instant::now());
                    break response;
                }
            }
        };
        if response.status != 200 {
            return Err(Error::Http(format!("Store API respondió {}", response.status)));
        }
        let info = parse_appdetails(app_id, &response.body)?;
        persistence::write_json(&self.cache_file(app_id), &Cached { fetched_at: now_secs(), info: info.clone() })?;
        Ok(info)
    }

    fn cache_file(&self, app_id: u32) -> PathBuf {
        self.cache_dir.join(format!("{app_id}.{}.json", self.language))
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

#[derive(Deserialize)]
struct Envelope {
    success: bool,
    data: Option<Data>,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    #[serde(default)]
    short_description: String,
    #[serde(default)]
    genres: Vec<Genre>,
    release_date: Option<ReleaseDate>,
}

#[derive(Deserialize)]
struct Genre {
    description: String,
}

#[derive(Deserialize)]
struct ReleaseDate {
    date: String,
}

fn parse_appdetails(app_id: u32, body: &[u8]) -> Result<Option<StoreInfo>> {
    let mut map: std::collections::HashMap<String, Envelope> = serde_json::from_slice(body)?;
    let Some(envelope) = map.remove(&app_id.to_string()) else { return Ok(None) };
    let Some(data) = envelope.data.filter(|_| envelope.success) else { return Ok(None) };
    let description = decode_entities(data.short_description.trim());
    Ok(Some(StoreInfo {
        name: data.name,
        kind: data.kind,
        metadata: Metadata {
            description: (!description.is_empty()).then_some(description),
            genres: data.genres.into_iter().map(|g| g.description).collect(),
            year: data.release_date.and_then(|r| year_in(&r.date)),
        },
    }))
}

/// La fecha viene localizada ("13 dic. 2018", "Dec 13, 2018"): basta con el año.
fn year_in(date: &str) -> Option<u16> {
    date.split(|c: char| !c.is_ascii_digit())
        .filter(|part| part.len() == 4)
        .filter_map(|part| part.parse::<u16>().ok())
        .find(|year| (1970..=2100).contains(year))
}

/// `short_description` trae entidades HTML básicas; no trae etiquetas.
fn decode_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_appdetails_payload() {
        let body = br#"{"1145350":{"success":true,"data":{"type":"game","name":"Hades II",
            "short_description":"Lucha m\u00e1s all\u00e1 del &quot;Inframundo&quot; &amp; m\u00e1s",
            "genres":[{"id":"1","description":"Acci\u00f3n"},{"id":"25","description":"Aventura"}],
            "release_date":{"coming_soon":false,"date":"25 SEP 2025"}}}}"#;
        let info = parse_appdetails(1145350, body).unwrap().unwrap();
        assert_eq!(info.name, "Hades II");
        assert!(info.is_playable());
        assert_eq!(info.metadata.genres, ["Acción", "Aventura"]);
        assert_eq!(info.metadata.year, Some(2025));
        assert_eq!(info.metadata.description.as_deref(), Some("Lucha más allá del \"Inframundo\" & más"));
    }

    #[test]
    fn unsuccessful_lookup_is_none() {
        assert_eq!(parse_appdetails(1, br#"{"1":{"success":false}}"#).unwrap(), None);
    }

    #[test]
    fn extracts_year_from_localized_dates() {
        assert_eq!(year_in("13 dic. 2018"), Some(2018));
        assert_eq!(year_in("Dec 13, 2018"), Some(2018));
        assert_eq!(year_in("Próximamente"), None);
    }
}
