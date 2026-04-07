use std::env;
use std::fs;
use std::io;
use std::process::Command;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::{Cli, Commands};
use crate::db;
use crate::errors::Result;
use crate::models::EnvVar;

const ZSH_COMPLETION: &str = include_str!("../completions/vaulter.zsh");

fn current_dir() -> Result<String> {
    env::current_dir()
        .map(|d| d.to_string_lossy().into_owned())
        .map_err(|_| crate::errors::VaulterError::NoCwd)
}

async fn resolve_vault(explicit: &Option<String>, pool: &sqlx::sqlite::SqlitePool) -> Result<String> {
    match explicit {
        Some(name) => Ok(name.clone()),
        None => db::get_active_vault(pool, &current_dir()?).await,
    }
}

/// Parse set args: either ["KEY", "VALUE"] or ["KEY=VAL", "KEY2=VAL2", ...]
fn parse_set_args(args: &[String]) -> Result<Vec<EnvVar>> {
    if args.is_empty() {
        return Err(crate::errors::VaulterError::InvalidSetArgs);
    }

    if args[0].contains('=') {
        let mut pairs = Vec::new();
        for arg in args {
            let (key, value) = arg
                .split_once('=')
                .ok_or(crate::errors::VaulterError::InvalidSetArgs)?;
            if key.is_empty() {
                return Err(crate::errors::VaulterError::InvalidSetArgs);
            }
            pairs.push(EnvVar::new(key, value));
        }
        Ok(pairs)
    } else if args.len() == 2 {
        Ok(vec![EnvVar::new(args[0].as_str(), args[1].as_str())])
    } else {
        Err(crate::errors::VaulterError::InvalidSetArgs)
    }
}

/// Parse .env file content into key-value pairs
pub fn parse_env(content: &str) -> Vec<EnvVar> {
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.strip_prefix("export ").unwrap_or(l))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| {
            EnvVar::new(k.trim(), v.trim().trim_matches('"').trim_matches('\''))
        })
        .collect()
}

async fn internal_complete(kind: &str, vault: Option<&str>) -> Result<()> {
    match kind {
        "vaults" => {
            let pool = db::open_db().await?;
            for (name, _) in db::list_vaults(&pool).await? {
                println!("{name}");
            }
        }
        "vars" => {
            let pool = db::open_db().await?;
            let vault_name = match vault {
                Some(v) => v.to_string(),
                None => db::get_active_vault(&pool, &current_dir()?).await?,
            };
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            for var in db::list_vars(&pool, vault_id).await? {
                println!("{}", var.key);
            }
        }
        "shells" => {
            for s in ["zsh", "bash", "fish", "powershell", "elvish"] {
                println!("{s}");
            }
        }
        _ => {}
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}

pub async fn run(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Init => {
            db::init_db().await?;
            println!("vaulter initialized at {}", db::vaulter_dir()?.display());
        }

        Commands::Debug => {
            let dir = db::vaulter_dir()?;
            let db_path = dir.join("vaulter.db");
            let db_exists = db_path.exists();

            println!("vaulter {}", env!("CARGO_PKG_VERSION"));
            println!("  git sha:     {}", env!("VAULTER_GIT_SHA"));
            println!("  build:       {}", if cfg!(debug_assertions) { "debug" } else { "release" });
            println!("  target:      {}", std::env::consts::OS);
            println!("  arch:        {}", std::env::consts::ARCH);
            println!();
            println!("paths:");
            println!("  vaulter dir: {}", dir.display());
            println!("  database:    {}", db_path.display());
            println!("  VAULTER_HOME: {}", std::env::var("VAULTER_HOME").unwrap_or_else(|_| "(not set)".into()));
            println!("  shell:       {}", std::env::var("SHELL").unwrap_or_else(|_| "(unknown)".into()));
            println!();

            if db_exists {
                let meta = fs::metadata(&db_path)?;
                println!("database:");
                println!("  size:        {} bytes", meta.len());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    println!("  perms:       {:o}", meta.permissions().mode() & 0o777);
                }

                let pool = db::open_db().await?;
                let vaults = db::list_vaults(&pool).await?;
                let active = db::get_active_vault(&pool, &current_dir()?).await?;
                println!("  vaults:      {}", vaults.len());
                println!("  active:      {} (for {})", active, current_dir()?);
            } else {
                println!("database: not initialized");
            }
        }

        Commands::Create { name } => {
            let pool = db::open_db().await?;
            db::create_vault(&pool, &name).await?;
            println!("vault '{name}' created");
        }

        Commands::List => {
            let pool = db::open_db().await?;
            let active = db::get_active_vault(&pool, &current_dir()?).await?;
            let vaults = db::list_vaults(&pool).await?;
            for (name, created_at) in vaults {
                let marker = if name == active { " *" } else { "" };
                println!("{name}{marker}  (created {created_at})");
            }
        }

        Commands::Delete { name } => {
            let pool = db::open_db().await?;
            db::delete_vault(&pool, &name).await?;
            println!("vault '{name}' deleted");
        }

        Commands::Use { vault } => {
            let pool = db::open_db().await?;
            let dir = current_dir()?;
            db::set_active_vault(&pool, &dir, &vault).await?;
            println!("switched to vault '{vault}' for {dir}");
        }

        Commands::Switch { name } => {
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&name, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            db::set_active_vault(&pool, &current_dir()?, &vault_name).await?;
            let vars = db::list_vars(&pool, vault_id).await?;
            for var in vars {
                println!("export {}={}", var.key, shell_quote(&var.value));
            }
        }

        Commands::Set { args, vault } => {
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&vault, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            let pairs = parse_set_args(&args)?;
            for var in &pairs {
                db::set_var(&pool, vault_id, &var.key, &var.value).await?;
                println!("{}={} (vault: {vault_name})", var.key, var.value);
            }
        }

        Commands::Get { key, vault } => {
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&vault, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            match db::get_var(&pool, vault_id, &key).await? {
                Some(val) => println!("{val}"),
                None => {
                    eprintln!("key '{key}' not found in vault '{vault_name}'");
                    std::process::exit(1);
                }
            }
        }

        Commands::Show { vault } => {
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&vault, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            let vars = db::list_vars(&pool, vault_id).await?;
            if vars.is_empty() {
                println!("vault '{vault_name}' has no variables");
            } else {
                println!("vault: {vault_name}");
                for var in vars {
                    println!("  {}={}", var.key, var.value);
                }
            }
        }

        Commands::Unset { key, vault } => {
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&vault, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            db::delete_var(&pool, vault_id, &key).await?;
            println!("unset '{key}' from vault '{vault_name}'");
        }

        Commands::Export { vault } => {
            let pool = db::open_db().await?;
            let vault_names = if vault.is_empty() {
                vec![db::get_active_vault(&pool, &current_dir()?).await?]
            } else {
                vault
            };
            let vars = db::export_vars(&pool, &vault_names).await?;
            for var in vars {
                println!("export {}={}", var.key, shell_quote(&var.value));
            }
        }

        Commands::Import { file, vault } => {
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&vault, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            let content = fs::read_to_string(&file)?;
            let pairs = parse_env(&content);
            let mut count = 0;
            for var in &pairs {
                db::set_var(&pool, vault_id, &var.key, &var.value).await?;
                count += 1;
            }
            println!("imported {count} variables into vault '{vault_name}' from {file}");
        }

        Commands::Completions { shell } => {
            if matches!(shell, Shell::Zsh) {
                print!("{}", ZSH_COMPLETION);
            } else {
                let mut cmd = Cli::command();
                let name = cmd.get_name().to_string();
                generate(shell, &mut cmd, name, &mut io::stdout());
            }
        }

        Commands::InternalComplete { kind, vault } => {
            // Silent on any failure — completion must never spam the shell.
            let _ = internal_complete(&kind, vault.as_deref()).await;
        }

        Commands::Run { args, cmd } => {
            if cmd.is_empty() {
                eprintln!("usage: vaulter run [with <vault>] -- <command>");
                std::process::exit(1);
            }
            let vault = match args.as_slice() {
                [] => None,
                [keyword, name] if keyword == "with" => Some(name.clone()),
                _ => {
                    eprintln!("usage: vaulter run [with <vault>] -- <command>");
                    std::process::exit(1);
                }
            };
            let pool = db::open_db().await?;
            let vault_name = resolve_vault(&vault, &pool).await?;
            let vault_id = db::resolve_vault_id(&pool, &vault_name).await?;
            let vars = db::list_vars(&pool, vault_id).await?;

            let status = Command::new(&cmd[0])
                .args(&cmd[1..])
                .envs(vars.into_iter().map(|v| (v.key, v.value)))
                .status()?;

            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Ok(())
}
