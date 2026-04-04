use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::errors::{Result, VaulterError};

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
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    Ok(pool)
}

pub async fn init_db() -> Result<()> {
    let dir = vaulter_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join("vaulter.db");
    let url = format!("sqlite:{}?mode=rwc", path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    pool.close().await;
    Ok(())
}

pub async fn get_active_vault(pool: &SqlitePool, dir: &str) -> Result<String> {
    let result: Option<(String,)> =
        sqlx::query_as("SELECT vault_name FROM dir_vaults WHERE dir = ?")
            .bind(dir)
            .fetch_optional(pool)
            .await?;

    Ok(result.map(|r| r.0).unwrap_or_else(|| "default".to_string()))
}

pub async fn set_active_vault(pool: &SqlitePool, dir: &str, name: &str) -> Result<()> {
    resolve_vault_id(pool, name).await?;
    sqlx::query(
        "INSERT INTO dir_vaults (dir, vault_name) VALUES (?, ?)
         ON CONFLICT(dir) DO UPDATE SET vault_name = excluded.vault_name",
    )
    .bind(dir)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn resolve_vault_id(pool: &SqlitePool, name: &str) -> Result<i64> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM vaults WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;

    row.map(|r| r.0)
        .ok_or_else(|| VaulterError::VaultNotFound(name.to_string()))
}

pub async fn create_vault(pool: &SqlitePool, name: &str) -> Result<()> {
    sqlx::query("INSERT INTO vaults (name) VALUES (?)")
        .bind(name)
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
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT name, created_at FROM vaults ORDER BY name")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

pub async fn delete_vault(pool: &SqlitePool, name: &str) -> Result<()> {
    if name == "default" {
        return Err(VaulterError::CannotDeleteDefault);
    }

    let id = resolve_vault_id(pool, name).await?;
    sqlx::query("DELETE FROM vaults WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM dir_vaults WHERE vault_name = ?")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_var(pool: &SqlitePool, vault_id: i64, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO env_vars (vault_id, key, value) VALUES (?, ?, ?)
         ON CONFLICT(vault_id, key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(vault_id)
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_var(pool: &SqlitePool, vault_id: i64, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM env_vars WHERE vault_id = ? AND key = ?")
            .bind(vault_id)
            .bind(key)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

pub async fn list_vars(pool: &SqlitePool, vault_id: i64) -> Result<Vec<(String, String)>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT key, value FROM env_vars WHERE vault_id = ? ORDER BY key")
            .bind(vault_id)
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

pub async fn delete_var(pool: &SqlitePool, vault_id: i64, key: &str) -> Result<()> {
    sqlx::query("DELETE FROM env_vars WHERE vault_id = ? AND key = ?")
        .bind(vault_id)
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn export_vars(pool: &SqlitePool, vault_names: &[String]) -> Result<Vec<(String, String)>> {
    let mut map: HashMap<String, String> = HashMap::new();

    for name in vault_names {
        let vault_id = resolve_vault_id(pool, name).await?;
        let vars = list_vars(pool, vault_id).await?;
        for (k, v) in vars {
            map.insert(k, v);
        }
    }

    let mut result: Vec<(String, String)> = map.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
