use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "domain", content = "request", rename_all = "snake_case")]
pub enum ApiRequestContract {
    App(AppRequest),
    Events(EventsRequest),
    Workflow(WorkflowRequest),
    Session(SessionRequest),
    Subagent(SubagentRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "domain", content = "response", rename_all = "snake_case")]
pub enum ApiResponseContract {
    App(AppResponse),
    Events(EventsResponse),
    Workflow(WorkflowResponse),
    Session(SessionResponse),
    Subagent(SubagentResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AppRequest {
    Tick,
    Quit,
    PaneNext,
    PanePrevious,
    SubmitChatMessage,
    SubmitDirectMessage {
        message: String,
    },
    SyncPlannerTasks {
        tasks: Vec<PlannerTaskEntryContract>,
    },
    PrepareMasterPrompt {
        message: String,
        tasks_file: String,
    },
    PreparePlannerPrompt {
        message: String,
        planner_file: String,
        project_info_file: String,
    },
    PrepareAttachDocsPrompt {
        tasks_file: String,
    },
    StartExecution,
    StartNextWorkerJob,
    WorkerOutput {
        line: String,
    },
    WorkerSystemOutput {
        line: String,
    },
    WorkerCompleted {
        success: bool,
        exit_code: i32,
    },
    DrainWorkerFailures,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppResponse {
    Ack,
    Prompt {
        text: String,
    },
    SubmittedChat {
        message: Option<String>,
    },
    PlannerTasksSynced {
        count: usize,
    },
    ExecutionMessages {
        messages: Vec<String>,
    },
    StartedWorkerJob {
        job: Option<StartedWorkerJobContract>,
    },
    WorkerCompleted {
        system_messages: Vec<String>,
        new_context_entries: Vec<String>,
    },
    WorkerFailures {
        failures: Vec<TaskFailureContract>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerRoleContract {
    Implementor,
    Auditor,
    TestWriter,
    TestRunner,
    FinalAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "run_kind", content = "payload", rename_all = "snake_case")]
pub enum JobRunContract {
    AgentPrompt(String),
    DeterministicTestRun,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartedWorkerJobContract {
    pub run: JobRunContract,
    pub role: WorkerRoleContract,
    pub top_task_id: u64,
    #[serde(default)]
    pub parent_context_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EventsRequest {
    NextEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventsResponse {
    Event { event: AppEventContract },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AppEventContract {
    Tick,
    Quit,
    NextPane,
    PrevPane,
    MoveUp,
    MoveDown,
    CursorLeft,
    CursorRight,
    ScrollChatUp,
    ScrollChatDown,
    ScrollRightUpGlobal,
    ScrollRightDownGlobal,
    InputChar(char),
    Backspace,
    Submit,
    MouseScrollUp,
    MouseScrollDown,
    MouseLeftClick { column: u16, row: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WorkflowRequest {
    SyncPlannerTasks {
        tasks: Vec<PlannerTaskEntryContract>,
    },
    PlannerTasksForFile,
    StartExecution,
    StartNextJob,
    AppendActiveOutput {
        line: String,
    },
    FinishActiveJob {
        success: bool,
        exit_code: i32,
    },
    DrainRecentFailures,
    RollingContextEntries,
    ReplaceRollingContextEntries {
        entries: Vec<String>,
    },
    RightPaneBlockView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowResponse {
    Ack,
    PlannerTasks {
        tasks: Vec<PlannerTaskEntryContract>,
    },
    StartExecution {
        messages: Vec<String>,
    },
    StartedJob {
        job: Option<StartedWorkerJobContract>,
    },
    FinishActiveJob {
        messages: Vec<String>,
    },
    RecentFailures {
        failures: Vec<TaskFailureContract>,
    },
    RollingContext {
        entries: Vec<String>,
    },
    RightPaneBlock {
        lines: Vec<String>,
        toggles: Vec<RightPaneToggleContract>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RightPaneToggleContract {
    pub line_index: usize,
    pub task_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SessionRequest {
    Initialize { cwd: String },
    OpenExisting { cwd: String, session_dir: String },
    ListSessions,
    ReadTasks,
    ReadPlannerMarkdown,
    WriteRollingContext { entries: Vec<String> },
    ReadRollingContext,
    ReadTaskFails,
    AppendTaskFails { entries: Vec<TaskFailureContract> },
    ReadProjectInfo,
    WriteProjectInfo { markdown: String },
    ReadSessionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionResponse {
    Initialized {
        session: SessionStoreSnapshotContract,
    },
    Sessions {
        sessions: Vec<SessionListEntryContract>,
    },
    Tasks {
        tasks: Vec<PlannerTaskEntryContract>,
    },
    PlannerMarkdown {
        markdown: String,
    },
    RollingContext {
        entries: Vec<String>,
    },
    TaskFails {
        entries: Vec<TaskFailureContract>,
    },
    ProjectInfo {
        markdown: String,
    },
    SessionMeta {
        meta: SessionMetaContract,
    },
    Ack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionStoreSnapshotContract {
    pub session_dir: String,
    pub tasks_file: String,
    pub planner_file: String,
    pub task_fails_file: String,
    pub project_info_file: String,
    pub session_meta_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SubagentRequest {
    BuildMasterPrompt {
        tasks_file: String,
        workflow_prompt: String,
    },
    BuildConvertPlanPrompt {
        planner_file: String,
        tasks_file: String,
    },
    BuildSessionIntroIfNeeded {
        prompt: String,
        session_dir: String,
        session_meta_file: String,
        #[serde(default)]
        project_info: Option<String>,
        intro_needed: bool,
    },
    BuildFailureReportPrompt {
        task_fails_file: String,
        failed_this_cycle: Vec<TaskFailureContract>,
        has_test_failure: bool,
    },
    SplitAuditsCommandPrompt,
    MergeAuditsCommandPrompt,
    SplitTestsCommandPrompt,
    MergeTestsCommandPrompt,
    BuildProjectInfoPrompt {
        cwd: String,
        question: String,
        output_path: String,
    },
    BuildSessionMetaPrompt {
        user_prompt: String,
        output_path: String,
    },
    BuildTaskCheckPrompt {
        tasks_file: String,
        project_info_file: String,
        session_meta_file: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubagentResponse {
    Prompt {
        text: String,
    },
    IntroPrompt {
        text: String,
        intro_needed_after: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionListEntryContract {
    pub session_dir: String,
    pub workspace: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub created_at_label: Option<String>,
    pub created_at_epoch_secs: u64,
    pub last_used_epoch_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMetaContract {
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub stack_description: String,
    #[serde(default)]
    pub test_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerTaskKindContract {
    Task,
    Implementor,
    Auditor,
    TestWriter,
    TestRunner,
    FinalAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerTaskStatusContract {
    Pending,
    InProgress,
    NeedsChanges,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerTaskDocContract {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerTaskEntryContract {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub docs: Vec<PlannerTaskDocContract>,
    #[serde(default)]
    pub kind: PlannerTaskKindContract,
    #[serde(default)]
    pub status: PlannerTaskStatusContract,
    #[serde(default)]
    pub parent_id: Option<String>,
    pub order: Option<u32>,
}

impl Default for PlannerTaskKindContract {
    fn default() -> Self {
        Self::Task
    }
}

impl Default for PlannerTaskStatusContract {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowFailureKindContract {
    Audit,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskFailureContract {
    pub kind: WorkflowFailureKindContract,
    pub top_task_id: u64,
    pub top_task_title: String,
    pub attempts: u8,
    pub reason: String,
    pub action_taken: String,
}
