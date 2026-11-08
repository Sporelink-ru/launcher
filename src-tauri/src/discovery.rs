use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WellKnown {
    pub protocol_version: String,
    pub server_id: String,
    pub server_name: String,
    pub auth_homeserver: String,
    pub public_keys: PublicKeys,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublicKeys {
    pub server_signing_key: String,
    #[serde(default)]
    pub auth_signing_key: Option<String>,
}

/// Fetch /.well-known/mc-federation from a hub.
pub fn fetch(hub_base: &str) -> Result<WellKnown> {
    let url = format!(
        "{}/.well-known/mc-federation",
        hub_base.trim_end_matches('/')
    );
    let wk: WellKnown = reqwest::blocking::get(&url)
        .with_context(|| format!("failed to GET {url}"))?
        .error_for_status()
        .with_context(|| format!("hub returned error for {url}"))?
        .json()
        .with_context(|| format!("invalid JSON from {url}"))?;
    Ok(wk)
}

/// Parse `user@host` into (user, base_url).
pub fn parse_alias(alias: &str) -> Result<(String, String)> {
    let (user, host) = alias
        .split_once('@')
        .context("expected format: user@host")?;
    if user.is_empty() || host.is_empty() {
        anyhow::bail!("empty user or host in alias");
    }
    let scheme = if host.starts_with("localhost") || host.starts_with("127.") {
        "http"
    } else {
        "https"
    };
    Ok((user.to_string(), format!("{scheme}://{host}")))
}

/// `sporelink resolve alice@localhost:5000` — show where Alice lives.
pub fn resolve(alias: &str) -> Result<()> {
    let (user, base) = parse_alias(alias)?;
    let wk = fetch(&base)?;

    println!("user:        {user}");
    println!("hub:         {} ({})", wk.server_name, wk.server_id);
    println!("auth_server: {}", wk.auth_homeserver);
    println!("protocol:    {}", wk.protocol_version);
    println!("capabilities: {:?}", wk.capabilities);

    Ok(())
}
