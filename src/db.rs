use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::errors::{Result, VaulterError};
use crate::models::EnvVar;

pub fn vaulter_dir() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("VAULTER_HOME") {
        return Ok(PathBuf::from(custom));
    }
    Ok(dirs::home_dir()
        .ok_or(VaulterError::NoHomeDir)?
        .join(".vaulter"))
}

fn db_path() -> Result<PathBuf> {
    vaulter_dir().map(|d| d.join("vaulter.db"))
}

pub async fn open_db() -> Result<SqlitePool> {
    let path = db_path()?;
    if !path.exists() {
        init_db().await?;
    }
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    sqlx::query!("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    Ok(pool)
}

pub async fn init_db() -> Result<()> {
    let dir = vaulter_dir()?;
    fs::create_dir_all(&dir)?;
    set_restrictive_perms(&dir, 0o700)?;

    let path = dir.join("vaulter.db");
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;

    sqlx::query!("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    pool.close().await;
    set_restrictive_perms(&path, 0o600)?;
    Ok(())
}

#[cfg(unix)]
fn set_restrictive_perms(path: &std::path::Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_restrictive_perms(_path: &std::path::Path, _mode: u32) -> Result<()> {
    Ok(())
}

pub async fn get_active_vault(pool: &SqlitePool, dir: &str) -> Result<String> {
    let result = sqlx::query!("SELECT vault_name FROM dir_vaults WHERE dir = ?", dir)
        .fetch_optional(pool)
        .await?;

    Ok(result
        .map(|r| r.vault_name)
        .unwrap_or_else(|| "default".to_string()))
}

pub async fn set_active_vault(pool: &SqlitePool, dir: &str, name: &str) -> Result<()> {
    resolve_vault_id(pool, name).await?;
    sqlx::query!(
        "INSERT INTO dir_vaults (dir, vault_name) VALUES (?, ?)
         ON CONFLICT(dir) DO UPDATE SET vault_name = excluded.vault_name",
        dir,
        name
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn resolve_vault_id(pool: &SqlitePool, name: &str) -> Result<i64> {
    let row = sqlx::query!(r#"SELECT id as "id!" FROM vaults WHERE name = ?"#, name)
        .fetch_optional(pool)
        .await?;

    row.map(|r| r.id)
        .ok_or_else(|| VaulterError::VaultNotFound(name.to_string()))
}

pub async fn create_vault(pool: &SqlitePool, name: &str) -> Result<()> {
    sqlx::query!("INSERT INTO vaults (name) VALUES (?)", name)
        .execute(pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("2067") => {
                VaulterError::VaultAlreadyExists(name.to_string())
            }
            _ => e.into(),
        })?;
    Ok(())
}

pub async fn list_vaults(pool: &SqlitePool) -> Result<Vec<(String, String)>> {
    let rows = sqlx::query!("SELECT name, created_at FROM vaults ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| (r.name, r.created_at)).collect())
}

pub async fn delete_vault(pool: &SqlitePool, name: &str) -> Result<()> {
    if name == "default" {
        return Err(VaulterError::CannotDeleteDefault);
    }

    let id = resolve_vault_id(pool, name).await?;
    sqlx::query!("DELETE FROM vaults WHERE id = ?", id)
        .execute(pool)
        .await?;

    sqlx::query!("DELETE FROM dir_vaults WHERE vault_name = ?", name)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_var(pool: &SqlitePool, vault_id: i64, key: &str, value: &str) -> Result<()> {
    sqlx::query!(
        "INSERT INTO env_vars (vault_id, key, value) VALUES (?, ?, ?)
         ON CONFLICT(vault_id, key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        vault_id,
        key,
        value
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_var(pool: &SqlitePool, vault_id: i64, key: &str) -> Result<Option<String>> {
    let row = sqlx::query!(
        "SELECT value FROM env_vars WHERE vault_id = ? AND key = ?",
        vault_id,
        key
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.value))
}

pub async fn list_vars(pool: &SqlitePool, vault_id: i64) -> Result<Vec<EnvVar>> {
    let rows = sqlx::query!(
        "SELECT key, value FROM env_vars WHERE vault_id = ? ORDER BY key",
        vault_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| EnvVar::new(r.key, r.value))
        .collect())
}

pub async fn delete_var(pool: &SqlitePool, vault_id: i64, key: &str) -> Result<()> {
    sqlx::query!(
        "DELETE FROM env_vars WHERE vault_id = ? AND key = ?",
        vault_id,
        key
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn export_vars(pool: &SqlitePool, vault_names: &[String]) -> Result<Vec<EnvVar>> {
    use crate::models::{Key, Value};
    let mut map: HashMap<Key, Value> = HashMap::new();

    for name in vault_names {
        let vault_id = resolve_vault_id(pool, name).await?;
        let vars = list_vars(pool, vault_id).await?;
        for var in vars {
            map.insert(var.key, var.value);
        }
    }

    let mut result: Vec<EnvVar> = map
        .into_iter()
        .map(|(k, v)| EnvVar { key: k, value: v })
        .collect();
    result.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(result)
}
