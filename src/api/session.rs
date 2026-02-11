use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub did: String,
    pub handle: String,
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub pds_endpoint: Option<String>,
}

fn session_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("skyscraper")
}

fn session_path() -> PathBuf {
    session_dir().join("session.json")
}

pub fn save_session(session: &SessionData) -> Result<()> {
    let dir = session_dir();
    std::fs::create_dir_all(&dir)?;

    // Restrict directory permissions to owner-only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }

    let json = serde_json::to_string_pretty(session)?;
    let path = session_path();
    std::fs::write(&path, &json)?;

    // Restrict file permissions to owner-only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn load_session() -> Result<Option<SessionData>> {
    let path = session_path();
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path)?;
    let session: SessionData = serde_json::from_str(&json)?;
    Ok(Some(session))
}

pub fn clear_session() -> Result<()> {
    let path = session_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn get_last_handle() -> Option<String> {
    load_session().ok()?.map(|s| s.handle)
}
