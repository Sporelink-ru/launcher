use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::storage;

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_token: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub client_token: String,
    pub selected_profile: Profile,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

pub fn login(alias: &str) -> Result<()> {
    // 1. Разбираем alias и находим Hub
    let (username, hub_base) = discovery::parse_alias(alias)?;
    let wk = discovery::fetch(&hub_base)?;

    println!("Hub:      {} ({})", wk.server_name, wk.server_id);
    println!("Auth:     {}", wk.auth_homeserver);
    println!("User:     {username}");

    // 2. Спрашиваем пароль (не эхо)
    let password = rpassword::prompt_password("Password: ")
        .context("failed to read password")?;

    // 3. Отправляем на auth_homeserver (а не hub_base — это важно для федерации)
    let url = format!(
        "{}/authserver/authenticate",
        wk.auth_homeserver.trim_end_matches('/')
    );
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&LoginRequest {
            username: username.clone(),
            password,
            client_token: None,
        })
        .send()
        .with_context(|| format!("POST {url} failed"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("auth failed ({status}): {body}");
    }

    let auth: LoginResponse = resp.json().context("invalid auth response")?;

    // 4. Сохраняем токен
    storage::save(&storage::StoredToken {
        hub_url: hub_base.trim_end_matches('/').to_string(),
        auth_homeserver: wk.auth_homeserver.clone(),
        server_id: wk.server_id.clone(),
        access_token: auth.access_token.clone(),
        client_token: auth.client_token.clone(),
        profile: auth.selected_profile.clone(),
    })?;

    println!();
    println!(
        "✓ Logged in as {} (uuid: {})",
        auth.selected_profile.name, auth.selected_profile.id
    );
    let preview: String = auth.access_token.chars().take(40).collect();
    println!("  JWT: {preview}...");
    println!("  Saved to: {}", storage::token_file_display()?);

    Ok(())
}