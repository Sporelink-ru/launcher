mod auth;
mod discovery;
mod storage;
use std::process::Command;
use std::fs;
use std::io::Write;
use serde_json::Value;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "resolve" => {
            if args.len() < 3 {
                bail!("usage: sporelink resolve <user@host>");
            }
            discovery::resolve(&args[2])
        }
        "login" => {
            if args.len() < 3 {
                bail!("usage: sporelink login <user@host>");
            }
            auth::login(&args[2])
        }
        "whoami" => storage::whoami(),
        "logout" => storage::logout(),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        "launch" => {
            if args.len() < 3 {
                bail!("usage: sporelink launch <version>");
            }
            launch(&args[2])
        }
        cmd => bail!("unknown command: {cmd}"),
    }
}

fn print_usage() {
    println!("Sporelink launcher v0.1");
    println!();
    println!("Usage:");
    println!("  sporelink resolve <user@host>    Resolve user@host via well-known");
    println!("  sporelink login   <user@host>    Authenticate and save JWT locally");
    println!("  sporelink whoami                 Show current saved token");
    println!("  sporelink logout                 Remove saved token");
    println!();
    println!("Example:");
    println!("  sporelink login alice@localhost:5000");
}

pub fn launch(version: &str) -> Result<()> {
    let token_data = storage::load()?.ok_or_else(|| anyhow::anyhow!("Not logged in. Run `login` first."))?;
    
    // 1. Скачиваем authlib-injector
    let injector_path = dirs::data_dir().unwrap().join("sporelink").join("authlib-injector.jar");
    if !injector_path.exists() {
        println!("Downloading authlib-injector...");
        let url = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
        let mut resp = reqwest::blocking::get(url)?;
        let mut file = fs::File::create(&injector_path)?;
        std::io::copy(&mut resp, &mut file)?;
    }

    // 2. Пути к клиенту и манифесту
    let mc_dir = dirs::home_dir().unwrap().join(".minecraft");
    let version_dir = mc_dir.join("versions").join(version);
    let client_jar = version_dir.join(format!("{}.jar", version));
    let version_json = version_dir.join(format!("{}.json", version));

    if !client_jar.exists() || !version_json.exists() {
        anyhow::bail!("Client not found at {:?}. Please run version {} once in the official launcher to download it.", version_dir, version);
    }

    // 3. Парсим JSON, чтобы собрать classpath
    println!("Building classpath from manifest...");
    let json_str = fs::read_to_string(&version_json)?;
    let json: Value = serde_json::from_str(&json_str)?;
    
    let main_class = json["mainClass"].as_str().unwrap_or("net.minecraft.client.main.Main");
    let mut classpath = vec![client_jar.to_string_lossy().to_string()];
    
    if let Some(libs) = json["libraries"].as_array() {
        for lib in libs {
            // Пропускаем библиотеки, которые не для нашей ОС (например, Windows-only на Linux)
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

    // Разделитель для classpath: ';' для Windows, ':' для Linux/Mac
    let cp_separator = if cfg!(windows) { ";" } else { ":" };
    let cp_string = classpath.join(cp_separator);

    // 4. Аргументы JVM (authlib-injector)
    let api_root = "http://localhost:5000"; 
    let javaagent = format!("-javaagent:{}={}", injector_path.display(), api_root);

    // 5. Запуск
    println!("Launching Minecraft {} as {}...", version, token_data.profile.name);
    
    let mut cmd = Command::new("java");
    cmd.arg("-Xmx2G")
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
        anyhow::bail!("Minecraft exited with error");
    }
    Ok(())
}