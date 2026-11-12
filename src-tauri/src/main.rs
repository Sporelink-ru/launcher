// Предотвращает открытие консоли на Windows в релизной сборке
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::command;
use serde::Serialize;
use std::process::Command;
use std::fs;
use serde_json::Value;
use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use tauri::Emitter;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

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
        Ok(resp) => {
            // Сохраняем свежий токен на диск!
            let stored = storage::StoredToken {
                hub_url: "http://localhost:5000".to_string(),
                auth_homeserver: "http://localhost:5000".to_string(),
                server_id: "hub-0001".to_string(),
                access_token: resp.access_token.clone(),
                client_token: resp.client_token.clone(),
                profile: resp.selected_profile.clone(),
            };
            if let Err(e) = storage::save(&stored) {
                println!("Failed to save token: {}", e);
            }

            Ok(LoginResult {
                name: resp.selected_profile.name,
                uuid: resp.selected_profile.id,
                access_token: resp.access_token,
            })
        },
        Err(e) => Err(e.to_string()),
    }
}

#[command]
fn get_saved_session() -> Result<Option<LoginResult>, String> {
    match storage::load() {
        Ok(Some(token)) => Ok(Some(LoginResult {
            name: token.profile.name,
            uuid: token.profile.id,
            access_token: token.access_token,
        })),
        _ => Ok(None),
    }
}

#[derive(serde::Serialize)]
struct TexturesData {
    skin: Option<String>,
    cape: Option<String>,
}

#[command]
fn get_textures_data(uuid: String) -> Result<TexturesData, String> {
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let url = format!("http://localhost:5000/sessionserver/session/minecraft/profile/{}", uuid);
    
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;

    let mut skin_b64 = None;
    let mut cape_b64 = None;

    if let Some(props) = json["properties"].as_array() {
        for prop in props {
            if prop["name"] == "textures" {
                if let Some(val) = prop["value"].as_str() {
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(val) {
                        if let Ok(tex_json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                            // Скачиваем скин
                            if let Some(skin_url) = tex_json["textures"]["SKIN"]["url"].as_str() {
                                if let Ok(img) = client.get(skin_url).send().and_then(|r| r.bytes()) {
                                    skin_b64 = Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&img)));
                                }
                            }
                            // Скачиваем плащ
                            if let Some(cape_url) = tex_json["textures"]["CAPE"]["url"].as_str() {
                                if let Ok(img) = client.get(cape_url).send().and_then(|r| r.bytes()) {
                                    cape_b64 = Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&img)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(TexturesData { skin: skin_b64, cape: cape_b64 })
}

#[command]
fn upload_texture(uuid: String, file_bytes: Vec<u8>, model: String, texture_type: String) -> Result<(), String> {
    let token_data = storage::load().map_err(|e| e.to_string())?.ok_or("Not logged in")?;
    let client = reqwest::blocking::Client::new();
    let url = format!("http://localhost:5000/api/user/profile/{}/{}", uuid, texture_type);
    
    let part = reqwest::blocking::multipart::Part::bytes(file_bytes)
        .file_name("texture.png").mime_str("image/png").map_err(|e| e.to_string())?;
        
    let mut form = reqwest::blocking::multipart::Form::new().part("file", part);
    if texture_type == "skin" { form = form.text("model", model); }

    let resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token_data.access_token))
        .multipart(form).send().map_err(|e| e.to_string())?;

    if !resp.status().is_success() { return Err(format!("Server returned: {}", resp.status())); }
    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize)]
struct NewsItem {
    id: i32,
    title: String,
    content: String,
    date: String,
}

#[command]
fn get_news() -> Result<Vec<NewsItem>, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get("http://localhost:5000/api/news")
        .send()
        .map_err(|e| e.to_string())?;
        
    let news = resp.json::<Vec<NewsItem>>()
        .map_err(|e| e.to_string())?;
        
    Ok(news)
}

#[command]
fn launch_game(window: tauri::Window, version: String, ram_mb: u32) -> Result<(), String> {
    launch_minecraft(window, &version, ram_mb).map_err(|e| e.to_string())
}

fn launch_minecraft(window: tauri::Window, version: &str, ram_mb: u32) -> anyhow::Result<()> {
    let token_data = storage::load()?.context("Not logged in.")?;
    
    window.emit("launch_status", "Проверка authlib-injector...").ok();
    
    let injector_path = dirs::data_dir().unwrap().join("sporelink").join("authlib-injector.jar");
    if !injector_path.exists() {
        window.emit("launch_status", "Скачивание authlib-injector...").ok();
        let url = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
        let mut resp = reqwest::blocking::get(url)?;
        let mut file = fs::File::create(&injector_path)?;
        std::io::copy(&mut resp, &mut file)?;
    }

    window.emit("launch_status", "Сборка Classpath...").ok();
    let mc_dir = dirs::home_dir().unwrap().join(".minecraft");
    let version_dir = mc_dir.join("versions").join(version);
    let client_jar = version_dir.join(format!("{}.jar", version));
    let version_json = version_dir.join(format!("{}.json", version));

    if !client_jar.exists() || !version_json.exists() {
        anyhow::bail!("Клиент не найден. Запустите версию {} через официальный лаунчер.", version);
    }

    let json_str = fs::read_to_string(&version_json)?;
    let json: serde_json::Value = serde_json::from_str(&json_str)?;
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

    window.emit("launch_status", "Запуск Java-процесса...").ok();
    
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
       .arg("--assetIndex").arg(json["assetIndex"]["id"].as_str().unwrap_or(""))
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Не удалось запустить Java")?;
    
    window.emit("launch_status", "Игра запущена. Ожидание закрытия...").ok();
    window.emit("launch_progress", 100).ok();

    // Читаем stdout в отдельном потоке, чтобы не блокировать лаунчер
    if let Some(stdout) = child.stdout.take() {
        let window_clone = window.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                window_clone.emit("launch_log", line).ok();
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let window_clone = window.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                window_clone.emit("launch_log", line).ok();
            }
        });
    }

    // Ждём закрытия игры в фоне
    std::thread::spawn(move || {
        let _ = child.wait();
        window.emit("game_closed", ()).ok();
    });

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            login,
            get_textures_data,
            launch_game,
            get_saved_session,
            upload_texture,
            get_news
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}