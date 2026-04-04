use std::env;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::sqlite::SqlitePool;
use vaulter::commands;
use vaulter::db;
use vaulter::errors::VaulterError;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestEnv {
    tmp_dir: std::path::PathBuf,
}

impl TestEnv {
    async fn new() -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp_dir = env::temp_dir().join(format!(
            "vaulter_test_{}_{id}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        unsafe { env::set_var("VAULTER_HOME", &tmp_dir) };

        db::init_db().await.unwrap();

        TestEnv { tmp_dir }
    }

    async fn pool(&self) -> SqlitePool {
        db::open_db().await.unwrap()
    }

    fn dir(&self) -> String {
        self.tmp_dir.to_string_lossy().into_owned()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        unsafe { env::remove_var("VAULTER_HOME") };
        let _ = fs::remove_dir_all(&self.tmp_dir);
    }
}

// ─── Init ────────────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_init_creates_db_and_default_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vaults = db::list_vaults(&pool).await.unwrap();
    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].0, "default");
}

#[tokio::test(flavor = "current_thread")]
async fn test_double_init_is_safe() {
    let env = TestEnv::new().await;
    db::init_db().await.unwrap();

    let pool = env.pool().await;
    let vaults = db::list_vaults(&pool).await.unwrap();
    assert_eq!(vaults.len(), 1);
}

// ─── Active vault (per-directory) ────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_active_vault_defaults_to_default() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let active = db::get_active_vault(&pool, &env.dir()).await.unwrap();
    assert_eq!(active, "default");
}

#[tokio::test(flavor = "current_thread")]
async fn test_use_switches_active_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "staging").await.unwrap();
    db::set_active_vault(&pool, &env.dir(), "staging").await.unwrap();

    let active = db::get_active_vault(&pool, &env.dir()).await.unwrap();
    assert_eq!(active, "staging");
}

#[tokio::test(flavor = "current_thread")]
async fn test_use_nonexistent_vault_fails() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let result = db::set_active_vault(&pool, &env.dir(), "nonexistent").await;
    assert!(matches!(result, Err(VaulterError::VaultNotFound(_))));
}

#[tokio::test(flavor = "current_thread")]
async fn test_active_vault_is_per_directory() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "vault_a").await.unwrap();
    db::create_vault(&pool, "vault_b").await.unwrap();

    let dir_a = format!("{}/project_a", env.dir());
    let dir_b = format!("{}/project_b", env.dir());

    db::set_active_vault(&pool, &dir_a, "vault_a").await.unwrap();
    db::set_active_vault(&pool, &dir_b, "vault_b").await.unwrap();

    assert_eq!(db::get_active_vault(&pool, &dir_a).await.unwrap(), "vault_a");
    assert_eq!(db::get_active_vault(&pool, &dir_b).await.unwrap(), "vault_b");
    assert_eq!(
        db::get_active_vault(&pool, &format!("{}/other", env.dir())).await.unwrap(),
        "default"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_switching_directory_changes_active_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "frontend").await.unwrap();
    db::create_vault(&pool, "backend").await.unwrap();

    let dir_fe = format!("{}/frontend", env.dir());
    let dir_be = format!("{}/backend", env.dir());
    let dir_new = format!("{}/newproject", env.dir());

    db::set_active_vault(&pool, &dir_fe, "frontend").await.unwrap();
    db::set_active_vault(&pool, &dir_be, "backend").await.unwrap();

    assert_eq!(db::get_active_vault(&pool, &dir_fe).await.unwrap(), "frontend");
    assert_eq!(db::get_active_vault(&pool, &dir_be).await.unwrap(), "backend");
    assert_eq!(db::get_active_vault(&pool, &dir_new).await.unwrap(), "default");
    assert_eq!(db::get_active_vault(&pool, &dir_fe).await.unwrap(), "frontend");
}

// ─── Vault CRUD ──────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_create_and_list_vaults() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "dev").await.unwrap();
    db::create_vault(&pool, "staging").await.unwrap();

    let vaults = db::list_vaults(&pool).await.unwrap();
    let names: Vec<&str> = vaults.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, vec!["default", "dev", "staging"]);
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_duplicate_vault_fails() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "dup").await.unwrap();
    let result = db::create_vault(&pool, "dup").await;
    assert!(matches!(result, Err(VaulterError::VaultAlreadyExists(_))));
}

#[tokio::test(flavor = "current_thread")]
async fn test_delete_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "temp").await.unwrap();
    db::delete_vault(&pool, "temp").await.unwrap();

    let vaults = db::list_vaults(&pool).await.unwrap();
    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].0, "default");
}

#[tokio::test(flavor = "current_thread")]
async fn test_cannot_delete_default_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let result = db::delete_vault(&pool, "default").await;
    assert!(matches!(result, Err(VaulterError::CannotDeleteDefault)));
}

#[tokio::test(flavor = "current_thread")]
async fn test_delete_vault_clears_dir_bindings() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "temp").await.unwrap();
    db::set_active_vault(&pool, &env.dir(), "temp").await.unwrap();
    assert_eq!(db::get_active_vault(&pool, &env.dir()).await.unwrap(), "temp");

    db::delete_vault(&pool, "temp").await.unwrap();

    assert_eq!(db::get_active_vault(&pool, &env.dir()).await.unwrap(), "default");
}

// ─── Env var CRUD ────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_set_and_get_var() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    db::set_var(&pool, vault_id, "API_KEY", "secret123").await.unwrap();

    let val = db::get_var(&pool, vault_id, "API_KEY").await.unwrap();
    assert_eq!(val, Some("secret123".to_string()));
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_nonexistent_var_returns_none() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    let val = db::get_var(&pool, vault_id, "NOPE").await.unwrap();
    assert_eq!(val, None);
}

#[tokio::test(flavor = "current_thread")]
async fn test_set_var_upserts() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    db::set_var(&pool, vault_id, "KEY", "v1").await.unwrap();
    db::set_var(&pool, vault_id, "KEY", "v2").await.unwrap();

    let val = db::get_var(&pool, vault_id, "KEY").await.unwrap();
    assert_eq!(val, Some("v2".to_string()));
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_vars() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    db::set_var(&pool, vault_id, "B_KEY", "bval").await.unwrap();
    db::set_var(&pool, vault_id, "A_KEY", "aval").await.unwrap();

    let vars = db::list_vars(&pool, vault_id).await.unwrap();
    assert_eq!(
        vars,
        vec![
            ("A_KEY".to_string(), "aval".to_string()),
            ("B_KEY".to_string(), "bval".to_string()),
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_delete_var() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    db::set_var(&pool, vault_id, "KEY", "val").await.unwrap();
    db::delete_var(&pool, vault_id, "KEY").await.unwrap();

    let val = db::get_var(&pool, vault_id, "KEY").await.unwrap();
    assert_eq!(val, None);
}

#[tokio::test(flavor = "current_thread")]
async fn test_same_key_different_vaults() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "other").await.unwrap();
    let default_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    let other_id = db::resolve_vault_id(&pool, "other").await.unwrap();

    db::set_var(&pool, default_id, "KEY", "from_default").await.unwrap();
    db::set_var(&pool, other_id, "KEY", "from_other").await.unwrap();

    assert_eq!(
        db::get_var(&pool, default_id, "KEY").await.unwrap(),
        Some("from_default".to_string())
    );
    assert_eq!(
        db::get_var(&pool, other_id, "KEY").await.unwrap(),
        Some("from_other".to_string())
    );
}

// ─── Export ──────────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_export_single_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    db::set_var(&pool, vault_id, "A", "1").await.unwrap();
    db::set_var(&pool, vault_id, "B", "2").await.unwrap();

    let vars = db::export_vars(&pool, &["default".to_string()]).await.unwrap();
    assert_eq!(
        vars,
        vec![
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string()),
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_export_multi_vault_deduplication() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "over").await.unwrap();
    let default_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    let over_id = db::resolve_vault_id(&pool, "over").await.unwrap();

    db::set_var(&pool, default_id, "SHARED", "default_val").await.unwrap();
    db::set_var(&pool, default_id, "ONLY_DEFAULT", "yes").await.unwrap();
    db::set_var(&pool, over_id, "SHARED", "over_val").await.unwrap();
    db::set_var(&pool, over_id, "ONLY_OVER", "yes").await.unwrap();

    let vars = db::export_vars(
        &pool,
        &["default".to_string(), "over".to_string()],
    )
    .await
    .unwrap();

    assert_eq!(
        vars,
        vec![
            ("ONLY_DEFAULT".to_string(), "yes".to_string()),
            ("ONLY_OVER".to_string(), "yes".to_string()),
            ("SHARED".to_string(), "over_val".to_string()),
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_export_nonexistent_vault_fails() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let result = db::export_vars(&pool, &["ghost".to_string()]).await;
    assert!(matches!(result, Err(VaulterError::VaultNotFound(_))));
}

// ─── Switch ─────────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_switch_sets_active_vault_for_dir() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "myvault").await.unwrap();
    db::set_active_vault(&pool, &env.dir(), "myvault").await.unwrap();

    let active = db::get_active_vault(&pool, &env.dir()).await.unwrap();
    assert_eq!(active, "myvault");
}

// ─── Import (.env parsing) ───────────────────────────────────────────

#[test]
fn test_parse_env_basic() {
    let content = "KEY=value\nDB_HOST=localhost\n";
    let pairs = commands::parse_env(content);
    assert_eq!(
        pairs,
        vec![
            ("KEY".to_string(), "value".to_string()),
            ("DB_HOST".to_string(), "localhost".to_string()),
        ]
    );
}

#[test]
fn test_parse_env_skips_comments_and_blanks() {
    let content = "# this is a comment\n\nKEY=value\n  # indented comment\n\n";
    let pairs = commands::parse_env(content);
    assert_eq!(pairs, vec![("KEY".to_string(), "value".to_string())]);
}

#[test]
fn test_parse_env_strips_quotes() {
    let content = "A=\"double quoted\"\nB='single quoted'\nC=unquoted\n";
    let pairs = commands::parse_env(content);
    assert_eq!(
        pairs,
        vec![
            ("A".to_string(), "double quoted".to_string()),
            ("B".to_string(), "single quoted".to_string()),
            ("C".to_string(), "unquoted".to_string()),
        ]
    );
}

#[test]
fn test_parse_env_handles_export_prefix() {
    let content = "export API_KEY=secret\nexport DB=postgres\nPLAIN=val\n";
    let pairs = commands::parse_env(content);
    assert_eq!(
        pairs,
        vec![
            ("API_KEY".to_string(), "secret".to_string()),
            ("DB".to_string(), "postgres".to_string()),
            ("PLAIN".to_string(), "val".to_string()),
        ]
    );
}

#[test]
fn test_parse_env_value_with_equals() {
    let content = "URL=postgres://user:pass@host/db?opt=1\n";
    let pairs = commands::parse_env(content);
    assert_eq!(
        pairs,
        vec![("URL".to_string(), "postgres://user:pass@host/db?opt=1".to_string())]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_import_writes_to_vault() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    let vault_id = db::resolve_vault_id(&pool, "default").await.unwrap();
    let pairs = commands::parse_env("A=1\nB=2\n");
    for (key, value) in &pairs {
        db::set_var(&pool, vault_id, key, value).await.unwrap();
    }

    let vars = db::list_vars(&pool, vault_id).await.unwrap();
    assert_eq!(
        vars,
        vec![
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string()),
        ]
    );
}

// ─── Cascade delete ──────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn test_deleting_vault_removes_its_vars() {
    let env = TestEnv::new().await;
    let pool = env.pool().await;

    db::create_vault(&pool, "temp").await.unwrap();
    let vault_id = db::resolve_vault_id(&pool, "temp").await.unwrap();
    db::set_var(&pool, vault_id, "KEY", "val").await.unwrap();

    db::delete_vault(&pool, "temp").await.unwrap();

    db::create_vault(&pool, "temp").await.unwrap();
    let new_id = db::resolve_vault_id(&pool, "temp").await.unwrap();
    let vars = db::list_vars(&pool, new_id).await.unwrap();
    assert!(vars.is_empty());
}
