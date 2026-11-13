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
use std::path::{Path, PathBuf};
use std::io::Write;

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

#[derive(serde::Deserialize)]
struct VersionManifest {
    versions: Vec<VersionEntry>,
}

#[derive(serde::Deserialize)]
struct VersionEntry {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
}

#[derive(serde::Deserialize)]
struct VersionDetails {
    id: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    libraries: Vec<Library>,
    downloads: Downloads,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    #[serde(rename = "javaVersion")]
    java_version: Option<JavaVersion>,
}

#[derive(serde::Deserialize)]
struct JavaVersion {
    #[serde(rename = "majorVersion")]
    major_version: u32,
}

#[derive(serde::Deserialize)]
struct Library {
    name: String,
    downloads: Option<LibraryDownloads>,
    rules: Option<Vec<Rule>>,
    natives: Option<std::collections::HashMap<String, String>>,
}

#[derive(serde::Deserialize)]
struct Rule {
    action: String,
    os: Option<RuleOs>,
}

#[derive(serde::Deserialize)]
struct RuleOs {
    name: Option<String>,
}

#[derive(serde::Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
}

#[derive(serde::Deserialize)]
struct Artifact {
    path: Option<String>,
    url: String,
}

#[derive(serde::Deserialize)]
struct Downloads {
    client: Artifact,
}

#[derive(serde::Deserialize)]
struct AssetIndex {
    id: String,
    url: String,
}

#[command]
fn get_minecraft_versions() -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .and_then(|r| r.json::<VersionManifest>());

    match res {
        Ok(manifest) => {
            let versions = manifest.versions.into_iter()
                .filter(|v| v.version_type == "release")
                .map(|v| v.id)
                .collect();
            Ok(versions)
        },
        Err(e) => {
            println!("[Rust] Failed to fetch versions: {}", e);
            // Возвращаем базовый список, если API недоступно
            Ok(vec!["1.21.4".to_string(), "1.20.4".to_string(), "1.19.4".to_string(), "1.16.5".to_string(), "1.8.9".to_string()])
        }
    }
}

#[command]
async fn download_game_and_java(version: String, window: tauri::Window) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mc_dir = dirs::data_dir().ok_or("Не удалось найти папку AppData")?.join("sporelink").join("minecraft");
        let version_dir = mc_dir.join("versions").join(&version);
        fs::create_dir_all(&version_dir).map_err(|e| format!("Ошибка создания папки: {}", e))?;
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        window.emit("launch_status", "Получение манифеста версии...").unwrap();
        window.emit("launch_progress", 5).unwrap();
        
        let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let manifest: VersionManifest = client.get(manifest_url).send()
            .map_err(|e| format!("Нет связи с Mojang: {}", e))?
            .json().map_err(|e| format!("Ошибка парсинга манифеста: {}", e))?;
            
        let version_url = manifest.versions.iter().find(|v| v.id == version)
            .ok_or_else(|| "Версия не найдена в манифесте")?.url.clone();

        let version_json_path = version_dir.join(format!("{}.json", version));
        if !version_json_path.exists() {
            let json_bytes = client.get(&version_url).send()
                .map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
            fs::write(&version_json_path, &json_bytes).map_err(|e| e.to_string())?;
        }
        window.emit("launch_progress", 10).unwrap();

        let version_details: VersionDetails = serde_json::from_reader(fs::File::open(&version_json_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        // 2. Java
        if let Some(java_ver) = &version_details.java_version {
            let jre_dir = mc_dir.join("runtime").join(format!("java-{}", java_ver.major_version));
            let java_exe = if cfg!(windows) { jre_dir.join("bin").join("java.exe") } else { jre_dir.join("bin").join("java") };
            
            if !java_exe.exists() {
                window.emit("launch_status", format!("Скачивание Java {}...", java_ver.major_version)).unwrap();
                let os = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
                let arch = if cfg!(target_arch = "x86_64") { "x64" } else { "aarch64" };
                let url = format!("https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse", java_ver.major_version, os, arch);
                
                let java_client = reqwest::blocking::Client::builder().user_agent("SporelinkLauncher/0.1.0").build().map_err(|e| e.to_string())?;
                let resp = java_client.get(&url).send().map_err(|e| format!("Сеть: {}", e))?;
                if !resp.status().is_success() { return Err(format!("Adoptium вернул HTTP {}", resp.status())); }

                let bytes = resp.bytes().map_err(|e| e.to_string())?;
                let zip_path = mc_dir.join("runtime").join(format!("jre-{}.tar.gz", java_ver.major_version));
                fs::create_dir_all(mc_dir.join("runtime")).map_err(|e| e.to_string())?;
                fs::write(&zip_path, &bytes).map_err(|e| e.to_string())?;

                let tar_gz = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
                let tar = flate2::read::GzDecoder::new(tar_gz);
                let mut archive = tar::Archive::new(tar);

                for entry in archive.entries().map_err(|e| e.to_string())? {
                    let mut entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path().map_err(|e| e.to_string())?;
                    let components: Vec<_> = path.components().collect();
                    if components.len() > 1 {
                        let mut new_path = std::path::PathBuf::new();
                        for comp in components.iter().skip(1) { new_path.push(comp); }
                        let outpath = jre_dir.join(new_path);
                        if let Some(p) = outpath.parent() { if !p.exists() { fs::create_dir_all(p).map_err(|e| e.to_string())?; } }
                        entry.unpack(&outpath).map_err(|e| e.to_string())?;
                    }
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = fs::metadata(&java_exe) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&java_exe, perms).unwrap();
                    }
                }
            }
        }
        window.emit("launch_progress", 30).unwrap();

        // 3. Client.jar
        window.emit("launch_status", "Скачивание клиента...").unwrap();
        let client_jar = version_dir.join(format!("{}.jar", version));
        if !client_jar.exists() {
            let bytes = client.get(&version_details.downloads.client.url).send()
                .map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
            fs::write(&client_jar, bytes).map_err(|e| e.to_string())?;
        }
        window.emit("launch_progress", 40).unwrap();

        // 4. Библиотеки
        window.emit("launch_status", "Скачивание библиотек...").unwrap();
        let lib_dir = mc_dir.join("libraries");
        let total_libs = version_details.libraries.len();
        for (i, lib) in version_details.libraries.iter().enumerate() {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    if let Some(path) = &artifact.path {
                        let dest = lib_dir.join(path);
                        if !dest.exists() {
                            if let Some(parent) = dest.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                            if let Ok(resp) = client.get(&artifact.url).send() {
                                if let Ok(bytes) = resp.bytes() { let _ = fs::write(&dest, bytes); }
                            }
                        }
                    }
                }
            }
            if i % 10 == 0 {
                let progress = 40 + (i * 20 / total_libs.max(1));
                window.emit("launch_progress", progress).unwrap();
                window.emit("launch_status", format!("Библиотеки: {}/{}", i, total_libs)).unwrap();
            }
        }
        window.emit("launch_progress", 60).unwrap();

        // 5. Ассеты
        window.emit("launch_status", "Загрузка индекса ресурсов...").unwrap();
        let asset_index_path = mc_dir.join("assets").join("indexes").join(format!("{}.json", version_details.asset_index.id));
        if !asset_index_path.exists() {
            fs::create_dir_all(mc_dir.join("assets").join("indexes")).map_err(|e| e.to_string())?;
            let bytes = client.get(&version_details.asset_index.url).send()
                .map_err(|e| e.to_string())?.bytes().map_err(|e| e.to_string())?;
            fs::write(&asset_index_path, &bytes).map_err(|e| e.to_string())?;
        }

        let asset_index_json: serde_json::Value = serde_json::from_reader(fs::File::open(&asset_index_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
            
        if let Some(objects) = asset_index_json["objects"].as_object() {
            let total_assets = objects.len();
            let mut downloaded = 0;
            
            for (_, obj) in objects {
                if let Some(hash) = obj["hash"].as_str() {
                    let prefix = &hash[0..2];
                    let asset_path = mc_dir.join("assets").join("objects").join(prefix).join(hash);
                    if !asset_path.exists() {
                        if let Some(parent) = asset_path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                        let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);
                        if let Ok(resp) = client.get(&url).send() {
                            if let Ok(bytes) = resp.bytes() { let _ = fs::write(&asset_path, bytes); }
                        }
                    }
                    downloaded += 1;
                    
                    if downloaded % 50 == 0 {
                        let progress = 60 + (downloaded * 40 / total_assets.max(1));
                        window.emit("launch_progress", progress).unwrap();
                        window.emit("launch_status", format!("Ресурсы: {}/{}", downloaded, total_assets)).unwrap();
                    }
                }
            }
        }

        window.emit("launch_status", "Загрузка завершена!").unwrap();
        window.emit("launch_progress", 100).unwrap();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())? 
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
        
        // Запрашиваем у GitHub последнюю версию
        let client = reqwest::blocking::Client::builder()
            .user_agent("SporelinkLauncher/0.1.0")
            .build()
            .unwrap();
            
        let api_url = "https://api.github.com/repos/yushijinhun/authlib-injector/releases/latest";
        let mut download_url = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.7/authlib-injector-1.2.7.jar".to_string();

        if let Ok(resp) = client.get(api_url).send() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(assets) = json["assets"].as_array() {
                    for asset in assets {
                        if let Some(name) = asset["name"].as_str() {
                            if name.ends_with(".jar") {
                                if let Some(url) = asset["browser_download_url"].as_str() {
                                    download_url = url.to_string();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut resp = client.get(&download_url).send()?;
        let mut file = fs::File::create(&injector_path)?;
        std::io::copy(&mut resp, &mut file)?;
    }

    window.emit("launch_status", "Сборка Classpath...").ok();
    let mc_dir = dirs::data_dir().unwrap().join("sporelink").join("minecraft");
    let version_dir = mc_dir.join("versions").join(version);
    let client_jar = version_dir.join(format!("{}.jar", version));
    let version_json = version_dir.join(format!("{}.json", version));

    if !client_jar.exists() || !version_json.exists() {
        anyhow::bail!("Клиент не найден. Нажмите ИГРАТЬ, чтобы скачать версию {}.", version);
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

    // --- ИСПОЛЬЗУЕМ СКАЧАННУЮ JAVA ---
    let java_major = json["javaVersion"]["majorVersion"].as_u64().unwrap_or(17);
    let jre_dir = mc_dir.join("runtime").join(format!("java-{}", java_major));
    let java_exe = if cfg!(windows) { jre_dir.join("bin").join("java.exe") } else { jre_dir.join("bin").join("java") };
    
    // Если скачанной Java нет, используем системную (fallback)
    let java_cmd = if java_exe.exists() {
        java_exe.to_string_lossy().to_string()
    } else {
        "java".to_string()
    };

    window.emit("launch_status", format!("Запуск Java (версия {})...", java_major)).ok();
    
    let mut cmd = Command::new(&java_cmd);
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

    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                println!("[MC] {}", line); 
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                println!("[MC ERROR] {}", line); 
            }
        });
    }

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
            get_news,
            get_minecraft_versions,
            download_game_and_java
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}