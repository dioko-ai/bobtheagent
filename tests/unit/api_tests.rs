use super::*;
use crate::api::capabilities::CapabilityOperation;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn api_request_serialization_uses_domain_and_action_tags() {
    let request = ApiRequestContract::Workflow(WorkflowRequest::StartExecution);
    let value = serde_json::to_value(request).expect("request should serialize");
    assert_eq!(
        value,
        json!({
            "domain": "workflow",
            "request": {
                "action": "start_execution"
            }
        })
    );
}

#[test]
fn api_request_deserialization_reads_domain_and_action_tags() {
    let value = json!({
        "domain": "workflow",
        "request": {
            "action": "append_active_output",
            "line": "stream chunk"
        }
    });

    let request: ApiRequestContract =
        serde_json::from_value(value).expect("request should deserialize");
    assert_eq!(
        request,
        ApiRequestContract::Workflow(WorkflowRequest::AppendActiveOutput {
            line: "stream chunk".to_string(),
        })
    );
}

#[test]
fn api_response_deserialization_reads_domain_and_kind_tags() {
    let value = json!({
        "domain": "session",
        "response": {
            "kind": "planner_markdown",
            "markdown": "# plan"
        }
    });

    let response: ApiResponseContract =
        serde_json::from_value(value).expect("response should deserialize");
    assert_eq!(
        response,
        ApiResponseContract::Session(SessionResponse::PlannerMarkdown {
            markdown: "# plan".to_string(),
        })
    );
}

#[test]
fn api_response_serialization_uses_domain_and_kind_tags() {
    let response = ApiResponseContract::Workflow(WorkflowResponse::RecentFailures {
        failures: vec![TaskFailureContract {
            kind: WorkflowFailureKindContract::Test,
            top_task_id: 12,
            top_task_title: "Write regression test".to_string(),
            attempts: 2,
            reason: "missing coverage".to_string(),
            action_taken: "requeue test writer".to_string(),
        }],
    });
    let value = serde_json::to_value(response).expect("response should serialize");
    assert_eq!(
        value,
        json!({
            "domain": "workflow",
            "response": {
                "kind": "recent_failures",
                "failures": [
                    {
                        "kind": "test",
                        "top_task_id": 12,
                        "top_task_title": "Write regression test",
                        "attempts": 2,
                        "reason": "missing coverage",
                        "action_taken": "requeue test writer"
                    }
                ]
            }
        })
    );
}

#[test]
fn started_worker_job_parent_context_defaults_to_none() {
    let value = json!({
        "run": {
            "run_kind": "deterministic_test_run"
        },
        "role": "test_runner",
        "top_task_id": 7
    });

    let job: StartedWorkerJobContract =
        serde_json::from_value(value).expect("job should deserialize");
    assert_eq!(job.parent_context_key, None);
}

#[test]
fn request_envelope_defaults_request_id_and_metadata() {
    let value = json!({
        "capability": "workflow_execution_queue",
        "payload": {
            "domain": "workflow",
            "request": {
                "action": "start_next_job"
            }
        }
    });

    let envelope: RequestEnvelope<ApiRequestContract> =
        serde_json::from_value(value).expect("envelope should deserialize");
    assert_eq!(envelope.request_id, None);
    assert_eq!(envelope.metadata, RequestMetadata::default());
    assert_eq!(envelope.capability, CapabilityId::WorkflowExecutionQueue);
}

#[test]
fn error_envelope_defaults_retryable_and_details() {
    let result = json!({
        "status": "err",
        "error": {
            "code": "validation_failed",
            "message": "bad payload"
        }
    });

    let parsed: ApiResultEnvelope<ApiResponseContract> =
        serde_json::from_value(result).expect("error envelope should deserialize");
    assert_eq!(
        parsed,
        ApiResultEnvelope::Err {
            error: ApiErrorEnvelope {
                code: ApiErrorCode::ValidationFailed,
                message: "bad payload".to_string(),
                retryable: false,
                details: None,
            }
        }
    );
}

#[test]
fn response_envelope_err_defaults_retryable_and_details_when_omitted() {
    let encoded = json!({
        "request_id": "req-10",
        "capability": "workflow_execution_queue",
        "result": {
            "status": "err",
            "error": {
                "code": "external_failure",
                "message": "runner exited non-zero"
            }
        }
    });

    let decoded: ResponseEnvelope<ApiResponseContract> =
        serde_json::from_value(encoded).expect("response should deserialize");
    assert_eq!(
        decoded,
        ResponseEnvelope {
            request_id: Some("req-10".to_string()),
            capability: CapabilityId::WorkflowExecutionQueue,
            result: ApiResultEnvelope::Err {
                error: ApiErrorEnvelope {
                    code: ApiErrorCode::ExternalFailure,
                    message: "runner exited non-zero".to_string(),
                    retryable: false,
                    details: None,
                }
            },
        }
    );
}

#[test]
fn response_envelope_err_round_trips_with_details() {
    let envelope = ResponseEnvelope::<ApiResponseContract> {
        request_id: Some("req-9".to_string()),
        capability: CapabilityId::SessionFailureStorage,
        result: ApiResultEnvelope::Err {
            error: ApiErrorEnvelope {
                code: ApiErrorCode::Conflict,
                message: "append conflict".to_string(),
                retryable: true,
                details: Some(json!({
                    "file": "task_fails.json",
                    "line": 12
                })),
            },
        },
    };

    let encoded = serde_json::to_value(&envelope).expect("response should serialize");
    let decoded: ResponseEnvelope<ApiResponseContract> =
        serde_json::from_value(encoded).expect("response should deserialize");
    assert_eq!(decoded, envelope);
}

#[test]
fn capability_matrix_entries_are_unique_and_have_contract_metadata() {
    let mut ids = HashSet::new();
    for definition in CAPABILITY_MATRIX {
        assert!(
            ids.insert(definition.id),
            "duplicate capability id found: {:?}",
            definition.id
        );
        assert!(
            !definition.request_contract.is_empty(),
            "request contract must not be empty for {:?}",
            definition.id
        );
        assert!(
            !definition.response_contract.is_empty(),
            "response contract must not be empty for {:?}",
            definition.id
        );
        assert!(
            !definition.code_paths.is_empty(),
            "code paths must not be empty for {:?}",
            definition.id
        );
        for path in definition.code_paths {
            assert!(
                path.starts_with("src/") && path.contains("::"),
                "code path should use file::symbol format: {path}"
            );
        }
        assert!(
            !definition.notes.trim().is_empty(),
            "notes must not be empty for {:?}",
            definition.id
        );
    }
}

#[test]
fn capability_matrix_definitions_cover_expected_workflow_and_session_ids() {
    let ids: HashSet<_> = CAPABILITY_MATRIX.iter().map(|entry| entry.id).collect();
    assert!(ids.contains(&CapabilityId::WorkflowTaskGraphSync));
    assert!(ids.contains(&CapabilityId::WorkflowExecutionQueue));
    assert!(ids.contains(&CapabilityId::WorkflowContextProjection));
    assert!(ids.contains(&CapabilityId::SessionLifecycle));
    assert!(ids.contains(&CapabilityId::SessionPlannerStorage));
    assert!(ids.contains(&CapabilityId::SessionFailureStorage));
    assert!(ids.contains(&CapabilityId::SessionProjectContextStorage));

    let workflow_task_graph = capability_definition(CapabilityId::WorkflowTaskGraphSync)
        .expect("workflow task graph capability should exist");
    assert_eq!(
        workflow_task_graph.operation,
        CapabilityOperation::CommandQuery
    );
    assert!(
        workflow_task_graph
            .request_contract
            .contains("SyncPlannerTasks")
    );

    let session_failure = capability_definition(CapabilityId::SessionFailureStorage)
        .expect("session failure capability should exist");
    assert_eq!(session_failure.operation, CapabilityOperation::CommandQuery);
    assert!(session_failure.response_contract.contains("TaskFails"));

    let session_project = capability_definition(CapabilityId::SessionProjectContextStorage)
        .expect("session project-context capability should exist");
    assert_eq!(session_project.operation, CapabilityOperation::CommandQuery);
    assert!(session_project.request_contract.contains("ReadProjectInfo"));
    assert!(session_project.response_contract.contains("SessionMeta"));
}

#[test]
fn capability_matrix_covers_workflow_and_session_task_paths() {
    let workflow_task_graph = capability_definition(CapabilityId::WorkflowTaskGraphSync)
        .expect("workflow task graph capability should exist");
    assert_eq!(workflow_task_graph.domain, CapabilityDomain::Workflow);
    assert!(
        workflow_task_graph
            .code_paths
            .iter()
            .any(|path| *path == "src/workflow.rs::sync_planner_tasks_from_file")
    );
    assert!(
        workflow_task_graph
            .code_paths
            .iter()
            .any(|path| *path == "src/workflow.rs::planner_tasks_for_file")
    );

    let workflow_execution = capability_definition(CapabilityId::WorkflowExecutionQueue)
        .expect("workflow execution capability should exist");
    assert_eq!(workflow_execution.domain, CapabilityDomain::Workflow);
    assert!(
        workflow_execution
            .code_paths
            .iter()
            .any(|path| path.starts_with("src/workflow.rs::"))
    );

    let session_planner = capability_definition(CapabilityId::SessionPlannerStorage)
        .expect("session planner capability should exist");
    assert_eq!(session_planner.domain, CapabilityDomain::Session);
    assert!(
        session_planner
            .code_paths
            .iter()
            .any(|path| *path == "src/session_store.rs::read_tasks")
    );
    assert!(
        session_planner
            .code_paths
            .iter()
            .any(|path| *path == "src/session_store.rs::write_rolling_context")
    );

    let session_lifecycle = capability_definition(CapabilityId::SessionLifecycle)
        .expect("session lifecycle capability should exist");
    assert_eq!(session_lifecycle.domain, CapabilityDomain::Session);
    assert!(
        session_lifecycle
            .code_paths
            .iter()
            .any(|path| *path == "src/session_store.rs::initialize")
    );
}
