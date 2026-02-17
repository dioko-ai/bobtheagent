use super::*;
use std::time::{Duration, Instant};

#[test]
fn deterministic_runner_streams_output_and_completes() {
    let runner = TestRunnerAdapter::with_config(TestRunnerConfig {
        program: "bash".to_string(),
        args: vec![
            "-lc".to_string(),
            "printf 'runner-out\\n'; printf 'runner-err\\n' 1>&2".to_string(),
        ],
    });
    runner.run_tests();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_output = false;
    let mut saw_completed = false;
    while Instant::now() < deadline {
        for event in runner.drain_events() {
            match event {
                AgentEvent::Output(line) if line == "runner-out" || line == "runner-err" => {
                    saw_output = true;
                }
                AgentEvent::Completed { success, code } => {
                    assert!(success);
                    assert_eq!(code, 0);
                    saw_completed = true;
                }
                AgentEvent::Output(_) | AgentEvent::System(_) => {}
            }
        }
        if saw_output && saw_completed {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(saw_output);
    assert!(saw_completed);
}

#[test]
fn deterministic_runner_reports_spawn_failure() {
    let runner = TestRunnerAdapter::with_config(TestRunnerConfig {
        program: "__no_such_runner__".to_string(),
        args: Vec::new(),
    });
    runner.run_tests();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_error = false;
    let mut saw_completed = false;
    while Instant::now() < deadline {
        for event in runner.drain_events() {
            match event {
                AgentEvent::System(line) if line.contains("failed to start") => {
                    saw_error = true;
                }
                AgentEvent::Completed { success, code } => {
                    assert!(!success);
                    assert_eq!(code, -1);
                    saw_completed = true;
                }
                AgentEvent::Output(_) | AgentEvent::System(_) => {}
            }
        }
        if saw_error && saw_completed {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(saw_error);
    assert!(saw_completed);
}

#[test]
fn deterministic_runner_emits_completed_after_output_is_drained() {
    let runner = TestRunnerAdapter::with_config(TestRunnerConfig {
        program: "bash".to_string(),
        args: vec![
            "-lc".to_string(),
            "(sleep 0.05; printf 'late\\n') & printf 'early\\n'".to_string(),
        ],
    });
    runner.run_tests();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_completed = false;
    let mut saw_late = false;
    let mut output_after_completed = false;
    while Instant::now() < deadline {
        for event in runner.drain_events() {
            match event {
                AgentEvent::Output(line) => {
                    if saw_completed {
                        output_after_completed = true;
                    }
                    if line.trim() == "late" {
                        saw_late = true;
                    }
                }
                AgentEvent::Completed { .. } => {
                    saw_completed = true;
                }
                AgentEvent::System(_) => {}
            }
        }
        if saw_completed && saw_late {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let extra_poll_deadline = Instant::now() + Duration::from_millis(150);
    while Instant::now() < extra_poll_deadline {
        for event in runner.drain_events() {
            if let AgentEvent::Output(_) = event
                && saw_completed
            {
                output_after_completed = true;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(saw_completed, "expected completed event");
    assert!(saw_late, "expected delayed output line");
    assert!(
        !output_after_completed,
        "saw output after completed event, which can corrupt next-job context"
    );
}

#[test]
fn deterministic_runner_runs_explicit_command_string() {
    let runner = TestRunnerAdapter::new();
    runner.run_tests_with_command(Some("printf 'from-meta-command\\n'"));

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_output = false;
    let mut saw_completed = false;
    while Instant::now() < deadline {
        for event in runner.drain_events() {
            match event {
                AgentEvent::Output(line) if line == "from-meta-command" => {
                    saw_output = true;
                }
                AgentEvent::Completed { success, code } => {
                    assert!(success);
                    assert_eq!(code, 0);
                    saw_completed = true;
                }
                AgentEvent::Output(_) | AgentEvent::System(_) => {}
            }
        }
        if saw_output && saw_completed {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(saw_output);
    assert!(saw_completed);
}

#[test]
fn deterministic_runner_skips_with_success_when_command_missing() {
    let runner = TestRunnerAdapter::new();
    runner.run_tests_with_command(None);

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_skip_message = false;
    let mut saw_completed = false;
    while Instant::now() < deadline {
        for event in runner.drain_events() {
            match event {
                AgentEvent::System(line) if line.contains("skipped") => {
                    saw_skip_message = true;
                }
                AgentEvent::Completed { success, code } => {
                    assert!(success);
                    assert_eq!(code, 0);
                    saw_completed = true;
                }
                AgentEvent::Output(_) | AgentEvent::System(_) => {}
            }
        }
        if saw_skip_message && saw_completed {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(saw_skip_message);
    assert!(saw_completed);
}

#[test]
fn drain_events_limited_respects_max_and_preserves_queue() {
    let runner = TestRunnerAdapter::new();
    for idx in 0..4 {
        runner
            .event_tx
            .send(AgentEvent::Output(format!("line-{idx}")))
            .expect("send should succeed");
    }

    let first = runner.drain_events_limited(1);
    assert_eq!(first.len(), 1);
    assert!(matches!(first[0], AgentEvent::Output(_)));

    let second = runner.drain_events_limited(10);
    assert_eq!(second.len(), 3);
    assert!(second.iter().all(|e| matches!(e, AgentEvent::Output(_))));
}
