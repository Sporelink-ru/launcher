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

pub fn login(alias: &str, password: &str) -> Result<LoginResponse> {
    let (username, hub_base) = discovery::parse_alias(alias)?;
    let wk = discovery::fetch(&hub_base)?;

    let url = format!("{}/authserver/authenticate", wk.auth_homeserver.trim_end_matches('/'));
    
    let client = reqwest::blocking::Client::new();
    let resp = client.post(&url)
        .json(&LoginRequest {
            username: username.clone(),
            password: password.to_string(),
            client_token: None,
        })
        .send()?;

    if !resp.status().is_success() {
        anyhow::bail!("Invalid credentials or server error");
    }

    let auth: LoginResponse = resp.json()?;
    Ok(auth)
}