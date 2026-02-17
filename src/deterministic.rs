use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::agent::AgentEvent;

#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    pub program: String,
    pub args: Vec<String>,
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        Self {
            program: "cargo".to_string(),
            args: vec!["test".to_string()],
        }
    }
}

pub struct TestRunnerAdapter {
    #[cfg(test)]
    config: TestRunnerConfig,
    event_tx: Sender<AgentEvent>,
    event_rx: Receiver<AgentEvent>,
}

impl TestRunnerAdapter {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            #[cfg(test)]
            config: TestRunnerConfig::default(),
            event_tx,
            event_rx,
        }
    }

    #[cfg(test)]
    pub fn with_config(config: TestRunnerConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            config,
            event_tx,
            event_rx,
        }
    }

    #[cfg(test)]
    pub fn run_tests(&self) {
        Self::spawn_run(self.config.clone(), self.event_tx.clone());
    }

    pub fn run_tests_with_command(&self, command: Option<&str>) {
        let tx = self.event_tx.clone();
        let normalized = command.map(str::trim).filter(|value| !value.is_empty());
        if let Some(command_line) = normalized {
            let config = TestRunnerConfig {
                program: "bash".to_string(),
                args: vec!["-lc".to_string(), command_line.to_string()],
            };
            Self::spawn_run(config, tx);
        } else {
            let _ = tx.send(AgentEvent::System(
                "Deterministic test runner failed: no test command configured in meta.json."
                    .to_string(),
            ));
            let _ = tx.send(AgentEvent::Completed {
                success: false,
                code: -2,
            });
        }
    }

    pub fn drain_events(&self) -> Vec<AgentEvent> {
        self.drain_events_limited(usize::MAX)
    }

    pub fn drain_events_limited(&self, max_events: usize) -> Vec<AgentEvent> {
        let mut events = Vec::new();
        if max_events == 0 {
            return events;
        }
        while events.len() < max_events {
            let Ok(event) = self.event_rx.try_recv() else {
                break;
            };
            events.push(event);
        }
        events
    }

    fn spawn_run(config: TestRunnerConfig, tx: Sender<AgentEvent>) {
        thread::spawn(move || {
            let mut command = Command::new(&config.program);
            command
                .args(&config.args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(err) => {
                    let _ = tx.send(AgentEvent::System(format!(
                        "Deterministic test runner failed to start: {err}"
                    )));
                    let _ = tx.send(AgentEvent::Completed {
                        success: false,
                        code: -1,
                    });
                    return;
                }
            };

            let mut readers = Vec::new();
            if let Some(stdout) = child.stdout.take() {
                readers.push(spawn_reader(stdout, tx.clone()));
            }
            if let Some(stderr) = child.stderr.take() {
                readers.push(spawn_reader(stderr, tx.clone()));
            }

            let wait_result = child.wait();
            for reader in readers {
                let _ = reader.join();
            }
            match wait_result {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    let _ = tx.send(AgentEvent::Completed {
                        success: status.success(),
                        code,
                    });
                    if !status.success() {
                        let _ = tx.send(AgentEvent::System(format!(
                            "Deterministic test runner exited with status code {code}"
                        )));
                    }
                }
                Err(err) => {
                    let _ = tx.send(AgentEvent::System(format!(
                        "Deterministic test runner wait failed: {err}"
                    )));
                    let _ = tx.send(AgentEvent::Completed {
                        success: false,
                        code: -1,
                    });
                }
            }
        });
    }
}

fn spawn_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    tx: Sender<AgentEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = tx.send(AgentEvent::Output(line));
        }
    })
}

#[cfg(test)]
#[path = "../tests/unit/deterministic_tests.rs"]
mod tests;
