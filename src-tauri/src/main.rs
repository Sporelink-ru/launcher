// Предотвращает открытие консоли на Windows в релизной сборке
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::command;
use serde::Serialize;
use std::process::Command;
use std::fs;
use serde_json::Value;
use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};

// Объявляем наши модули
mod auth;
mod storage;
mod discovery;

#[derive(Serialize)]
struct LoginResult {
    name: String,
    uuid: String,
    access_token: String,
}

#[command]
fn login(username: String, password: String) -> Result<LoginResult, String> {
    let alias = format!("{}@localhost:5000", username);
    match auth::login(&alias, &password) {
        Ok(resp) => Ok(LoginResult {
            name: resp.selected_profile.name,
            uuid: resp.selected_profile.id,
            access_token: resp.access_token,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[command]
fn get_skin_data(uuid: String) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
        
    let url = format!("http://localhost:5000/sessionserver/session/minecraft/profile/{}", uuid);
    println!("Fetching skin profile from: {}", url);
    
    let resp = client.get(&url).send().map_err(|e| {
        println!("Network error: {}", e);
        e.to_string()
    })?;
    
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;

    if let Some(props) = json["properties"].as_array() {
        for prop in props {
            if prop["name"] == "textures" {
                if let Some(val) = prop["value"].as_str() {
                    let decoded = base64::engine::general_purpose::STANDARD.decode(val).map_err(|e| e.to_string())?;
                    let tex_json: serde_json::Value = serde_json::from_slice(&decoded).map_err(|e| e.to_string())?;
                    
                    if let Some(skin_url) = tex_json["textures"]["SKIN"]["url"].as_str() {
                        println!("Downloading skin image from: {}", skin_url);
                        let img_resp = client.get(skin_url).send().map_err(|e| e.to_string())?;
                        let img_bytes = img_resp.bytes().map_err(|e| e.to_string())?;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&img_bytes);
                        return Ok(format!("data:image/png;base64,{}", b64));
                    }
                }
            }
        }
    }
    Err("No skin found in profile".to_string())
}

#[command]
fn launch_game(version: String, ram_mb: u32) -> Result<(), String> {
    launch_minecraft(&version, ram_mb).map_err(|e| e.to_string())
}

// ─── Логика запуска Minecraft ─────────────────────────────────────────
fn launch_minecraft(version: &str, ram_mb: u32) -> anyhow::Result<()> {
    let token_data = storage::load()?.context("Not logged in.")?;
    
    let injector_path = dirs::data_dir().unwrap().join("sporelink").join("authlib-injector.jar");
    if !injector_path.exists() {
        println!("Downloading authlib-injector...");
        let url = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
        let mut resp = reqwest::blocking::get(url)?;
        let mut file = fs::File::create(&injector_path)?;
        std::io::copy(&mut resp, &mut file)?;
    }

    let mc_dir = dirs::home_dir().unwrap().join(".minecraft");
    
    let version_dir = mc_dir.join("versions").join(version);
    let client_jar = version_dir.join(format!("{}.jar", version));
    let version_json = version_dir.join(format!("{}.json", version));

    if !client_jar.exists() || !version_json.exists() {
        anyhow::bail!("Client not found at {:?}. Please run version {} once in the official launcher.", version_dir, version);
    }

    let json_str = fs::read_to_string(&version_json)?;
    let json: Value = serde_json::from_str(&json_str)?;
    let main_class = json["mainClass"].as_str().unwrap_or("net.minecraft.client.main.Main");
    let mut classpath = vec![client_jar.to_string_lossy().to_string()];
    
    if let Some(libs) = json["libraries"].as_array() {
        for lib in libs {
            if let Some(rules) = lib.get("rules") {
                let mut allowed = false;
                for rule in rules.as_array().unwrap() {
                    let action = rule["action"].as_str().unwrap_or("");
                    let os_name = rule["os"]["name"].as_str().unwrap_or("");
                    if action == "allow" && (os_name.is_empty() || os_name == std::env::consts::OS) {
                        allowed = true;
                    }
                }
                if !allowed { continue; }
            }
            if let Some(artifact) = lib["downloads"]["artifact"].as_object() {
                if let Some(path) = artifact.get("path").and_then(|p| p.as_str()) {
                    let lib_path = mc_dir.join("libraries").join(path);
                    classpath.push(lib_path.to_string_lossy().to_string());
                }
            }
        }
    }

    let cp_separator = if cfg!(windows) { ";" } else { ":" };
    let cp_string = classpath.join(cp_separator);
    let api_root = "http://localhost:5000"; 
    let javaagent = format!("-javaagent:{}={}", injector_path.display(), api_root);

    println!("Launching Minecraft {} as {}...", version, token_data.profile.name);
    
    let mut cmd = Command::new("java");
    cmd.arg(format!("-Xmx{}M", ram_mb))
       .arg(javaagent)
       .arg("-cp").arg(cp_string)
       .arg(main_class)
       .arg("--username").arg(&token_data.profile.name)
       .arg("--uuid").arg(&token_data.profile.id)
       .arg("--accessToken").arg(&token_data.access_token)
       .arg("--version").arg(version)
       .arg("--gameDir").arg(&mc_dir)
       .arg("--assetsDir").arg(mc_dir.join("assets"))
       .arg("--assetIndex").arg(json["assetIndex"]["id"].as_str().unwrap_or(""));

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Minecraft exited with an error");
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![login, get_skin_data, launch_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}