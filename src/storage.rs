use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::auth::Profile;

#[derive(Serialize, Deserialize)]
pub struct StoredToken {
    pub hub_url: String,
    pub auth_homeserver: String,
    pub server_id: String,
    pub access_token: String,
    pub client_token: String,
    pub profile: Profile,
}

fn token_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .context("no data directory available")?
        .join("sporelink");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn token_path() -> Result<PathBuf> {
    Ok(token_dir()?.join("token.json"))
}

pub fn token_file_display() -> Result<String> {
    Ok(token_path()?.display().to_string())
}

pub fn save(token: &StoredToken) -> Result<()> {
    let path = token_path()?;
    let json = serde_json::to_string_pretty(token)?;
    fs::write(&path, json)?;

    // На Unix ставим права 0600 — только владелец может читать
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn load() -> Result<Option<StoredToken>> {
    let path = token_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(&path)?;
    let token: StoredToken = serde_json::from_str(&json)?;
    Ok(Some(token))
}

pub fn whoami() -> Result<()> {
    match load()? {
        Some(t) => {
            println!("Logged in as:  {}", t.profile.name);
            println!("UUID:          {}", t.profile.id);
            println!("Hub:           {} ({})", t.server_id, t.hub_url);
            println!("Auth server:   {}", t.auth_homeserver);
            let head: String = t.access_token.chars().take(40).collect();
            println!("JWT (head):    {head}...");
            Ok(())
        }
        None => {
            println!("Not logged in.");
            Ok(())
        }
    }
}

pub fn logout() -> Result<()> {
    let path = token_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
        println!("✓ Logged out");
    } else {
        println!("Not logged in.");
    }
    Ok(())
}