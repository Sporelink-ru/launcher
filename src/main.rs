mod auth;
mod discovery;
mod storage;

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