use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_home(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ))
}

fn with_temp_home<T>(prefix: &str, f: impl FnOnce(&Path) -> T) -> T {
    let _guard = crate::artifact_io::home_env_test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let temp_home = unique_temp_home(prefix);
    fs::create_dir_all(&temp_home).expect("create temp home");

    let prior_home = std::env::var_os("HOME");
    // SAFETY: tests serialize HOME mutation with HOME_LOCK and restore afterward.
    unsafe {
        std::env::set_var("HOME", &temp_home);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&temp_home)));

    // SAFETY: restoration mirrors the guarded mutation above.
    unsafe {
        if let Some(value) = prior_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
    }
    fs::remove_dir_all(&temp_home).ok();

    match result {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

#[test]
fn ensure_default_metaagent_config_prefers_bob_when_both_configs_exist() {
    with_temp_home("artifact-io-prefers-bob", |home| {
        let bob_config = home.join(".bob/config.toml");
        let legacy_config = home.join(".metaagent/config.toml");
        fs::create_dir_all(bob_config.parent().expect("bob config parent"))
            .expect("create bob config dir");
        fs::create_dir_all(legacy_config.parent().expect("legacy config parent"))
            .expect("create legacy config dir");
        fs::write(
            &bob_config,
            r#"
            [backend]
            selected = "codex"
            "#,
        )
        .expect("write bob config");
        fs::write(
            &legacy_config,
            r#"
            [backend]
            selected = "claude"
            "#,
        )
        .expect("write legacy config");

        let active_config = ensure_default_metaagent_config().expect("resolve active config");
        assert_eq!(active_config, bob_config);
        assert_eq!(
            runtime_storage_dir().expect("resolve runtime storage dir"),
            home.join(".bob")
        );
    });
}

#[test]
fn ensure_default_metaagent_config_falls_back_to_legacy_when_only_legacy_config_exists() {
    with_temp_home("artifact-io-legacy-fallback", |home| {
        let legacy_config = home.join(".metaagent/config.toml");
        fs::create_dir_all(legacy_config.parent().expect("legacy config parent"))
            .expect("create legacy config dir");
        fs::write(
            &legacy_config,
            r#"
            [backend]
            selected = "claude"
            "#,
        )
        .expect("write legacy config");

        let active_config = ensure_default_metaagent_config().expect("resolve active config");
        assert_eq!(active_config, legacy_config);
        assert_eq!(
            runtime_storage_dir().expect("resolve runtime storage dir"),
            home.join(".metaagent")
        );
    });
}

#[test]
fn ensure_default_metaagent_config_initializes_fresh_home_under_bob() {
    with_temp_home("artifact-io-fresh-init", |home| {
        let expected_config = home.join(".bob/config.toml");
        let active_config = ensure_default_metaagent_config().expect("initialize default config");
        assert_eq!(active_config, expected_config);
        assert!(expected_config.exists());
        assert_eq!(
            runtime_storage_dir().expect("resolve runtime storage dir"),
            home.join(".bob")
        );

        let config_text = fs::read_to_string(&expected_config).expect("read initialized config");
        assert!(config_text.contains("root_dir = \"~/.bob/sessions\""));
    });
}

#[test]
fn ensure_default_metaagent_config_writes_legacy_compat_file_when_missing() {
    with_temp_home("artifact-io-legacy-compat-create", |home| {
        let active_config = ensure_default_metaagent_config().expect("initialize default config");
        let legacy_config = home.join(".metaagent/config.toml");

        assert_eq!(active_config, home.join(".bob/config.toml"));
        assert!(legacy_config.exists());

        let active_text = fs::read_to_string(&active_config).expect("read active config");
        let legacy_text = fs::read_to_string(&legacy_config).expect("read legacy config");
        assert_eq!(legacy_text, active_text);
    });
}

#[test]
fn ensure_default_metaagent_config_keeps_existing_legacy_compat_file() {
    with_temp_home("artifact-io-legacy-compat-preserve", |home| {
        let bob_config = home.join(".bob/config.toml");
        let legacy_config = home.join(".metaagent/config.toml");
        fs::create_dir_all(bob_config.parent().expect("bob config parent"))
            .expect("create bob config dir");
        fs::create_dir_all(legacy_config.parent().expect("legacy config parent"))
            .expect("create legacy config dir");
        fs::write(
            &bob_config,
            r#"
            [backend]
            selected = "codex"
            "#,
        )
        .expect("write bob config");
        let sentinel_legacy = r#"
            [backend]
            selected = "claude"
            [storage]
            root_dir = "~/.metaagent/custom"
        "#;
        fs::write(&legacy_config, sentinel_legacy).expect("write legacy sentinel config");

        ensure_default_metaagent_config().expect("merge config");

        let preserved_legacy = fs::read_to_string(&legacy_config).expect("read legacy config");
        assert_eq!(preserved_legacy, sentinel_legacy);
    });
}

#[test]
fn runtime_storage_dir_prefers_bob_when_both_dirs_exist_without_configs() {
    with_temp_home("artifact-io-dir-precedence", |home| {
        fs::create_dir_all(home.join(".bob")).expect("create bob dir");
        fs::create_dir_all(home.join(".metaagent")).expect("create legacy dir");

        assert_eq!(
            runtime_storage_dir().expect("resolve runtime storage dir"),
            home.join(".bob")
        );
    });
}
