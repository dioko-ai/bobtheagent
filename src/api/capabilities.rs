use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityId {
    AppPromptPreparation,
    AppPlannerStateSync,
    AppExecutionControl,
    EventPolling,
    WorkflowTaskGraphSync,
    WorkflowExecutionQueue,
    WorkflowContextProjection,
    SessionLifecycle,
    SessionPlannerStorage,
    SessionFailureStorage,
    SessionProjectContextStorage,
    SubagentPromptGeneration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDomain {
    App,
    Events,
    Workflow,
    Session,
    Subagent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityOperation {
    Command,
    Query,
    CommandQuery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityDefinition {
    pub id: CapabilityId,
    pub domain: CapabilityDomain,
    pub operation: CapabilityOperation,
    pub request_contract: &'static str,
    pub response_contract: &'static str,
    pub code_paths: &'static [&'static str],
    pub notes: &'static str,
}

pub const CAPABILITY_MATRIX: &[CapabilityDefinition] = &[
    CapabilityDefinition {
        id: CapabilityId::AppPromptPreparation,
        domain: CapabilityDomain::App,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "AppRequest::{PrepareMasterPrompt,PreparePlannerPrompt,PrepareAttachDocsPrompt}",
        response_contract: "AppResponse::Prompt",
        code_paths: &[
            "src/app.rs::prepare_master_prompt",
            "src/app.rs::prepare_planner_prompt",
            "src/app.rs::prepare_attach_docs_prompt",
        ],
        notes: "Prepares transport-agnostic prompt payloads before adapter-specific execution.",
    },
    CapabilityDefinition {
        id: CapabilityId::AppPlannerStateSync,
        domain: CapabilityDomain::App,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "AppRequest::SyncPlannerTasks",
        response_contract: "AppResponse::PlannerTasksSynced",
        code_paths: &[
            "src/app.rs::sync_planner_tasks_from_file",
            "src/app.rs::planner_tasks_for_file",
        ],
        notes: "Synchronizes planner task state between UI-facing app state and workflow core.",
    },
    CapabilityDefinition {
        id: CapabilityId::AppExecutionControl,
        domain: CapabilityDomain::App,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "AppRequest::{StartExecution,StartNextWorkerJob,WorkerOutput,WorkerCompleted,DrainWorkerFailures}",
        response_contract: "AppResponse::{ExecutionMessages,StartedWorkerJob,WorkerCompleted,WorkerFailures}",
        code_paths: &[
            "src/app.rs::start_execution",
            "src/app.rs::start_next_worker_job",
            "src/app.rs::on_worker_output",
            "src/app.rs::on_worker_completed",
            "src/app.rs::drain_worker_failures",
        ],
        notes: "Coordinates execution lifecycle across worker adapters and surfaced status messages.",
    },
    CapabilityDefinition {
        id: CapabilityId::EventPolling,
        domain: CapabilityDomain::Events,
        operation: CapabilityOperation::Query,
        request_contract: "EventsRequest::NextEvent",
        response_contract: "EventsResponse::Event",
        code_paths: &["src/events.rs::next_event"],
        notes: "Maps terminal input/mouse transport into normalized app events.",
    },
    CapabilityDefinition {
        id: CapabilityId::WorkflowTaskGraphSync,
        domain: CapabilityDomain::Workflow,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "WorkflowRequest::{SyncPlannerTasks,PlannerTasksForFile}",
        response_contract: "WorkflowResponse::{Ack,PlannerTasks}",
        code_paths: &[
            "src/workflow.rs::sync_planner_tasks_from_file",
            "src/workflow.rs::planner_tasks_for_file",
        ],
        notes: "Validates and projects task-graph contracts independently of transport/UI concerns.",
    },
    CapabilityDefinition {
        id: CapabilityId::WorkflowExecutionQueue,
        domain: CapabilityDomain::Workflow,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "WorkflowRequest::{StartExecution,StartNextJob,FinishActiveJob,DrainRecentFailures}",
        response_contract: "WorkflowResponse::{StartExecution,StartedJob,FinishActiveJob,RecentFailures}",
        code_paths: &[
            "src/workflow.rs::start_execution",
            "src/workflow.rs::start_next_job",
            "src/workflow.rs::finish_active_job",
            "src/workflow.rs::drain_recent_failures",
        ],
        notes: "Owns transport-agnostic orchestration, retries, and failure progression.",
    },
    CapabilityDefinition {
        id: CapabilityId::WorkflowContextProjection,
        domain: CapabilityDomain::Workflow,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "WorkflowRequest::{RollingContextEntries,ReplaceRollingContextEntries,RightPaneBlockView}",
        response_contract: "WorkflowResponse::{RollingContext,RightPaneBlock}",
        code_paths: &[
            "src/workflow.rs::rolling_context_entries",
            "src/workflow.rs::replace_rolling_context_entries",
            "src/workflow.rs::right_pane_block_view",
        ],
        notes: "Projects normalized context and pane representations for any adapter.",
    },
    CapabilityDefinition {
        id: CapabilityId::SessionLifecycle,
        domain: CapabilityDomain::Session,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "SessionRequest::{Initialize,OpenExisting,ListSessions}",
        response_contract: "SessionResponse::{Initialized,Sessions}",
        code_paths: &[
            "src/session_store.rs::initialize",
            "src/session_store.rs::open_existing",
            "src/session_store.rs::list_sessions",
        ],
        notes: "Creates, resumes, and lists session storage roots without coupling to UI transport.",
    },
    CapabilityDefinition {
        id: CapabilityId::SessionPlannerStorage,
        domain: CapabilityDomain::Session,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "SessionRequest::{ReadTasks,ReadPlannerMarkdown,WriteRollingContext,ReadRollingContext}",
        response_contract: "SessionResponse::{Tasks,PlannerMarkdown,RollingContext,Ack}",
        code_paths: &[
            "src/session_store.rs::read_tasks",
            "src/session_store.rs::read_planner_markdown",
            "src/session_store.rs::write_rolling_context",
            "src/session_store.rs::read_rolling_context",
        ],
        notes: "Persists planner and rolling-context artifacts used by multiple adapters.",
    },
    CapabilityDefinition {
        id: CapabilityId::SessionFailureStorage,
        domain: CapabilityDomain::Session,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "SessionRequest::{ReadTaskFails,AppendTaskFails}",
        response_contract: "SessionResponse::{TaskFails,Ack}",
        code_paths: &[
            "src/session_store.rs::read_task_fails",
            "src/session_store.rs::append_task_fails",
        ],
        notes: "Stores durable failure envelopes for retries and user-facing reporting.",
    },
    CapabilityDefinition {
        id: CapabilityId::SessionProjectContextStorage,
        domain: CapabilityDomain::Session,
        operation: CapabilityOperation::CommandQuery,
        request_contract: "SessionRequest::{ReadProjectInfo,WriteProjectInfo,ReadSessionMeta}",
        response_contract: "SessionResponse::{ProjectInfo,SessionMeta,Ack}",
        code_paths: &[
            "src/session_store.rs::read_project_info",
            "src/session_store.rs::write_project_info",
            "src/session_store.rs::read_session_meta",
        ],
        notes: "Manages project-context and session-meta documents consumed across subagents.",
    },
    CapabilityDefinition {
        id: CapabilityId::SubagentPromptGeneration,
        domain: CapabilityDomain::Subagent,
        operation: CapabilityOperation::Query,
        request_contract: "SubagentRequest::*",
        response_contract: "SubagentResponse::{Prompt,IntroPrompt}",
        code_paths: &[
            "src/subagents/master.rs::*",
            "src/subagents/project_info.rs::*",
            "src/subagents/task_check.rs::*",
        ],
        notes: "Builds deterministic prompt contracts used by worker and master adapters.",
    },
];

pub fn capability_definition(id: CapabilityId) -> Option<&'static CapabilityDefinition> {
    CAPABILITY_MATRIX.iter().find(|entry| entry.id == id)
}
