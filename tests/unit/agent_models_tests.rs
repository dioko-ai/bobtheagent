use super::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

fn home_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn unique_temp_home(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ))
}

#[test]
fn defaults_use_large_smart_for_all_agent_slots() {
    let routing = CodexAgentModelRouting::default();

    let master = routing.profile_for(CodexAgentKind::Master);
    let auditor = routing.profile_for(CodexAgentKind::WorkerAuditor);
    let docs = routing.profile_for(CodexAgentKind::DocsAttach);

    assert_eq!(master.model, "gpt-5.3-codex");
    assert_eq!(master.thinking_effort.as_deref(), Some("medium"));
    assert_eq!(auditor.model, "gpt-5.3-codex");
    assert_eq!(docs.model, "gpt-5.3-codex");
}

#[test]
fn empty_toml_still_uses_embedded_defaults() {
    let routing = CodexAgentModelRouting::from_toml_str("").expect("parse should succeed");
    let master = routing.profile_for(CodexAgentKind::Master);
    assert_eq!(master.model, "gpt-5.3-codex");
    assert_eq!(master.thinking_effort.as_deref(), Some("medium"));
}

#[test]
fn defaults_include_supergenious_alias_labels() {
    let routing = CodexAgentModelRouting::default();
    let parsed = CodexAgentModelRouting::from_toml_str(
        r#"
        [codex.agent_profiles]
        worker_implementor = "small-supergenious"
        "#,
    )
    .expect("parse should succeed");

    let baseline = routing.profile_for(CodexAgentKind::WorkerImplementor);
    let aliased = parsed.profile_for(CodexAgentKind::WorkerImplementor);
    assert_eq!(aliased.model, "gpt-5.1-codex-mini");
    assert_eq!(aliased.thinking_effort.as_deref(), Some("xhigh"));
    assert_ne!(aliased, baseline);
}

#[test]
fn toml_overrides_profiles_and_agent_assignments() {
    let routing = CodexAgentModelRouting::from_toml_str(
        r#"
        [codex.model_profiles.custom-max]
        model = "gpt-5.3-codex"
        thinking_effort = "xhigh"

        [codex.agent_profiles]
        master = "CUSTOM-max"
        worker_implementor = "custom-max"
        "#,
    )
    .expect("parse should succeed");

    let master = routing.profile_for(CodexAgentKind::Master);
    let implementor = routing.profile_for(CodexAgentKind::WorkerImplementor);
    let task_check = routing.profile_for(CodexAgentKind::TaskCheck);

    assert_eq!(master.model, "gpt-5.3-codex");
    assert_eq!(master.thinking_effort.as_deref(), Some("xhigh"));
    assert_eq!(implementor.thinking_effort.as_deref(), Some("xhigh"));
    assert_eq!(task_check.model, "gpt-5.3-codex");
    assert_eq!(task_check.thinking_effort.as_deref(), Some("medium"));
}

#[test]
fn unknown_profile_assignment_falls_back_to_large_smart() {
    let routing = CodexAgentModelRouting::from_toml_str(
        r#"
        [codex.agent_profiles]
        master = "does-not-exist"
        "#,
    )
    .expect("parse should succeed");

    let master = routing.profile_for(CodexAgentKind::Master);
    assert_eq!(master.model, "gpt-5.3-codex");
    assert_eq!(master.thinking_effort.as_deref(), Some("medium"));
}

#[test]
fn load_from_metaagent_config_creates_default_config_when_missing() {
    let _guard = home_env_lock().lock().expect("lock HOME mutex");
    let old_home = std::env::var_os("HOME");
    let temp_home = unique_temp_home("metaagent-home-default-config");
    fs::create_dir_all(&temp_home).expect("create temp HOME");
    unsafe { std::env::set_var("HOME", &temp_home) };

    let routing = CodexAgentModelRouting::load_from_metaagent_config()
        .expect("loading from missing config should succeed");
    let config_path = temp_home.join(".metaagent/config.toml");
    let config_text = fs::read_to_string(&config_path).expect("default config should be created");
    assert!(config_text.contains("[storage]"));
    assert!(config_text.contains("[codex.agent_profiles]"));
    assert_eq!(
        routing.profile_for(CodexAgentKind::Master).model,
        "gpt-5.3-codex"
    );

    match old_home {
        Some(value) => unsafe { std::env::set_var("HOME", value) },
        None => unsafe { std::env::remove_var("HOME") },
    }
    let _ = fs::remove_dir_all(&temp_home);
}

#[test]
fn load_from_metaagent_config_reports_invalid_data_for_malformed_file() {
    let _guard = home_env_lock().lock().expect("lock HOME mutex");
    let old_home = std::env::var_os("HOME");
    let temp_home = unique_temp_home("metaagent-home-bad-config");
    let config_dir = temp_home.join(".metaagent");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("config.toml"),
        "[codex.model_profiles.bad\n",
    )
    .expect("write malformed config");
    unsafe { std::env::set_var("HOME", &temp_home) };

    let err = CodexAgentModelRouting::load_from_metaagent_config()
        .expect_err("malformed config should fail");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

    match old_home {
        Some(value) => unsafe { std::env::set_var("HOME", value) },
        None => unsafe { std::env::remove_var("HOME") },
    }
    let _ = fs::remove_dir_all(&temp_home);
}
