//! Lectura y escritura atómica de JSON: se escribe a un temporal y se renombra,
//! así un cierre a mitad de escritura nunca deja el archivo corrupto.

use crate::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    write_atomic(path, &serde_json::to_vec(value)?)
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trips_and_reports_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sub").join("data.json");
        assert_eq!(super::read_json::<Vec<u8>>(&file).unwrap(), None);
        super::write_json(&file, &vec![1u8, 2, 3]).unwrap();
        assert_eq!(super::read_json::<Vec<u8>>(&file).unwrap(), Some(vec![1, 2, 3]));
    }
}
