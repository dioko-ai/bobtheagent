use super::*;
use std::fs;

#[test]
fn expands_home_paths() {
    let expanded = expand_home("~/.metaagent/sessions").expect("home path should expand");
    assert!(expanded.is_absolute());
}

#[test]
fn embedded_default_config_includes_storage_section() {
    let parsed: MetaAgentConfig = toml::from_str(crate::default_config::DEFAULT_CONFIG_TOML)
        .expect("embedded default config should parse");
    assert_eq!(parsed.storage.root_dir, "~/.metaagent/sessions");
}

#[test]
fn planner_task_status_defaults_to_pending() {
    let parsed: PlannerTaskFileEntry =
        serde_json::from_str("{\"id\":\"a\",\"title\":\"Task A\",\"parent_id\":null,\"order\":0}")
            .expect("json should parse");
    assert!(parsed.details.is_empty());
    assert!(parsed.docs.is_empty());
    assert!(matches!(parsed.status, PlannerTaskStatusFile::Pending));
    assert!(matches!(parsed.kind, PlannerTaskKindFile::Task));
}

#[test]
fn planner_task_accepts_numeric_or_string_ids() {
    let numeric: PlannerTaskFileEntry =
        serde_json::from_str("{\"id\":1,\"title\":\"Task\",\"parent_id\":2,\"order\":0}")
            .expect("numeric ids should parse");
    assert_eq!(numeric.id, "1");
    assert_eq!(numeric.parent_id.as_deref(), Some("2"));

    let stringy: PlannerTaskFileEntry =
        serde_json::from_str("{\"id\":\"a\",\"title\":\"Task\",\"parent_id\":\"b\",\"order\":0}")
            .expect("string ids should parse");
    assert_eq!(stringy.id, "a");
    assert_eq!(stringy.parent_id.as_deref(), Some("b"));
}

#[test]
fn planner_task_parses_details_field() {
    let parsed: PlannerTaskFileEntry = serde_json::from_str(
            "{\"id\":\"a\",\"title\":\"Task\",\"details\":\"More context\",\"parent_id\":null,\"order\":0}",
        )
        .expect("json should parse");
    assert_eq!(parsed.details, "More context");
}

#[test]
fn planner_task_parses_docs_field() {
    let parsed: PlannerTaskFileEntry = serde_json::from_str(
            "{\"id\":\"a\",\"title\":\"Task\",\"details\":\"ctx\",\"docs\":[{\"title\":\"Ratatui docs\",\"url\":\"https://docs.rs/ratatui/latest/ratatui/\",\"summary\":\"widgets and rendering\"}],\"parent_id\":null,\"order\":0}",
        )
        .expect("json should parse");
    assert_eq!(parsed.docs.len(), 1);
    assert_eq!(parsed.docs[0].title, "Ratatui docs");
}

#[test]
fn planner_task_parses_legacy_docs_string_field() {
    let parsed: PlannerTaskFileEntry = serde_json::from_str(
            "{\"id\":\"a\",\"title\":\"Task\",\"details\":\"ctx\",\"docs\":\"src/app.rs, src/main.rs\",\"parent_id\":null,\"order\":0}",
        )
        .expect("json should parse");
    assert_eq!(parsed.docs.len(), 2);
    assert_eq!(parsed.docs[0].title, "src/app.rs");
    assert_eq!(parsed.docs[1].title, "src/main.rs");
}

#[test]
fn planner_task_parses_empty_legacy_docs_string_as_empty_docs() {
    let parsed: PlannerTaskFileEntry = serde_json::from_str(
            "{\"id\":\"a\",\"title\":\"Task\",\"details\":\"ctx\",\"docs\":\"\",\"parent_id\":null,\"order\":0}",
        )
        .expect("json should parse");
    assert!(parsed.docs.is_empty());
}

#[test]
fn list_sessions_in_root_orders_by_last_used_desc() {
    let base = std::env::temp_dir().join(format!(
        "metaagent-session-list-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("base dir");

    let s1 = base.join("s1");
    let s2 = base.join("s2");
    fs::create_dir_all(&s1).expect("s1");
    fs::create_dir_all(&s2).expect("s2");
    fs::write(
        s1.join("metadata.json"),
        serde_json::to_string_pretty(&SessionMetadata {
            workspace: "/tmp/w1".to_string(),
            created_at_epoch_secs: 10,
            last_used_epoch_secs: 20,
        })
        .expect("serialize"),
    )
    .expect("write s1 metadata");
    fs::write(
        s2.join("metadata.json"),
        serde_json::to_string_pretty(&SessionMetadata {
            workspace: "/tmp/w2".to_string(),
            created_at_epoch_secs: 15,
            last_used_epoch_secs: 30,
        })
        .expect("serialize"),
    )
    .expect("write s2 metadata");
    fs::write(
        s2.join("meta.json"),
        serde_json::to_string_pretty(&SessionMetaFile {
            title: "Planner Session".to_string(),
            created_at: "2026-02-16T12:00:00Z".to_string(),
            stack_description: "Rust + Ratatui terminal UI app".to_string(),
            test_command: Some("cargo test".to_string()),
        })
        .expect("serialize"),
    )
    .expect("write s2 session meta");

    let listed = list_sessions_in_root(&base).expect("list sessions");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].session_dir, s2);
    assert_eq!(listed[1].session_dir, s1);
    assert_eq!(listed[0].title.as_deref(), Some("Planner Session"));
    assert_eq!(
        listed[0].created_at_label.as_deref(),
        Some("2026-02-16T12:00:00Z")
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn open_existing_supports_rolling_context_round_trip() {
    let base = std::env::temp_dir().join(format!(
        "metaagent-session-open-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ));
    let session_dir = base.join("session-a");
    fs::create_dir_all(&session_dir).expect("session dir");
    let cwd = std::env::current_dir().expect("cwd");
    let store = SessionStore::open_existing(&cwd, &session_dir).expect("open existing");

    store
        .write_rolling_context(&["one".to_string(), "two".to_string()])
        .expect("write context");
    let read_back = store.read_rolling_context().expect("read context");
    assert_eq!(read_back, vec!["one".to_string(), "two".to_string()]);

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn create_unique_session_dir_avoids_same_second_workspace_collision() {
    let base = std::env::temp_dir().join(format!(
        "metaagent-session-unique-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("base dir");

    let first = create_unique_session_dir(&base, 42, "workspace").expect("first dir");
    let second = create_unique_session_dir(&base, 42, "workspace").expect("second dir");

    assert_ne!(first, second);
    assert!(first.ends_with("42-workspace"));
    assert!(second.ends_with("42-workspace-1"));
    assert!(first.is_dir());
    assert!(second.is_dir());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn task_fails_round_trip_append() {
    let base = std::env::temp_dir().join(format!(
        "metaagent-session-fails-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos()
    ));
    let session_dir = base.join("session-a");
    fs::create_dir_all(&session_dir).expect("session dir");
    let cwd = std::env::current_dir().expect("cwd");
    let store = SessionStore::open_existing(&cwd, &session_dir).expect("open existing");

    store
        .append_task_fails(&[TaskFailFileEntry {
            kind: "audit".to_string(),
            top_task_id: 1,
            top_task_title: "Task A".to_string(),
            attempts: 4,
            reason: "Critical blocker".to_string(),
            action_taken: "Continued".to_string(),
            created_at_epoch_secs: 123,
        }])
        .expect("append fails");
    let read_back = store.read_task_fails().expect("read fails");
    assert_eq!(read_back.len(), 1);
    assert_eq!(read_back[0].kind, "audit");
    assert_eq!(read_back[0].top_task_id, 1);

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn session_meta_file_defaults_stack_description_when_missing() {
    let parsed: SessionMetaFile = serde_json::from_str(
        "{\"title\":\"Planner Session\",\"created_at\":\"2026-02-16T12:00:00Z\"}",
    )
    .expect("session meta should parse");
    assert_eq!(parsed.title, "Planner Session");
    assert_eq!(parsed.created_at, "2026-02-16T12:00:00Z");
    assert!(parsed.stack_description.is_empty());
    assert!(parsed.test_command.is_none());
}

#[test]
fn session_meta_file_parses_test_command_as_string_or_null() {
    let with_command: SessionMetaFile = serde_json::from_str(
            "{\"title\":\"Planner Session\",\"created_at\":\"2026-02-16T12:00:00Z\",\"stack_description\":\"Rust\",\"test_command\":\"cargo test\"}",
        )
        .expect("session meta with command should parse");
    assert_eq!(with_command.test_command.as_deref(), Some("cargo test"));

    let without_tests: SessionMetaFile = serde_json::from_str(
            "{\"title\":\"Planner Session\",\"created_at\":\"2026-02-16T12:00:00Z\",\"stack_description\":\"Rust\",\"test_command\":null}",
        )
        .expect("session meta with null command should parse");
    assert!(without_tests.test_command.is_none());
}
