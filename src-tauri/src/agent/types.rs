use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Idle,
    Understanding,
    Planning,
    WaitingForApproval,
    Executing,
    WaitingForTool,
    Reflecting,
    Replanning,
    Completed,
    Failed,
    Cancelled,
    Paused,
    Scheduled,
    WaitingForUser,
}

impl TaskStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled
        )
    }

    pub fn can_pause(self) -> bool {
        matches!(
            self,
            Self::Understanding
                | Self::Planning
                | Self::Executing
                | Self::WaitingForTool
                | Self::Reflecting
                | Self::Replanning
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepStatus {
    Pending,
    Running,
    WaitingApproval,
    Done,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyDecision {
    Allow,
    Deny,
    RequireApproval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryKind {
    Working,
    Episodic,
    Semantic,
    Preference,
    TaskHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryScope {
    Session,
    Conversation,
    Task,
    User,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub step_id: String,
    pub description: String,
    pub status: StepStatus,
    pub dependencies: Vec<String>,
    pub required_tools: Vec<String>,
    pub expected_output: String,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub summary: Option<String>,
}

impl PlanStep {
    pub fn new(description: impl Into<String>, tools: &[&str], risk: RiskLevel) -> Self {
        Self {
            step_id: ids::new_id("step"),
            description: description.into(),
            status: StepStatus::Pending,
            dependencies: Vec::new(),
            required_tools: tools.iter().map(|name| (*name).to_string()).collect(),
            expected_output: String::new(),
            risk_level: risk,
            summary: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallRecord {
    pub id: String,
    pub tool: String,
    pub arguments_hash: String,
    pub sanitized_arguments: Value,
    pub result_summary: String,
    pub risk_level: RiskLevel,
    pub status: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub artifact_id: String,
    pub task_id: String,
    pub kind: String,
    pub path: String,
    pub size: u64,
    pub created_at: u64,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub task_id: String,
    pub user_id: String,
    pub conversation_id: String,
    pub agent_id: String,
    pub goal: String,
    pub status: TaskStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub current_step: Option<String>,
    #[serde(default)]
    pub plan: Vec<PlanStep>,
    #[serde(default)]
    pub context: Value,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallRecord>,
    #[serde(default)]
    pub approval_ids: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    pub result: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub cancelled: bool,
}

impl AgentTask {
    pub fn new(goal: String, agent_id: String, conversation_id: String) -> Self {
        let now = ids::now_ms();
        Self {
            task_id: ids::new_id("task"),
            user_id: "local".into(),
            conversation_id,
            agent_id,
            goal,
            status: TaskStatus::Idle,
            created_at: now,
            updated_at: now,
            current_step: None,
            plan: Vec::new(),
            context: Value::Object(Default::default()),
            tool_calls: Vec::new(),
            approval_ids: Vec::new(),
            errors: Vec::new(),
            artifacts: Vec::new(),
            result: None,
            metadata: Value::Object(Default::default()),
            cancelled: false,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = ids::now_ms();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub approval_id: String,
    pub task_id: String,
    pub step_id: Option<String>,
    pub tool: String,
    pub action: String,
    pub target: String,
    pub data_summary: String,
    pub reason: String,
    pub consequences: String,
    pub risk_level: RiskLevel,
    pub status: ApprovalStatus,
    pub created_at: u64,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalStatus {
    Pending,
    ApprovedOnce,
    ApprovedForTask,
    Denied,
    Edited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryItem {
    pub id: String,
    pub kind: MemoryKind,
    pub content: String,
    pub source: String,
    pub confidence: f32,
    pub created_at: u64,
    pub updated_at: u64,
    pub importance: u8,
    pub scope: MemoryScope,
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub memory_scope: MemoryScope,
    pub approval_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: String,
    pub task_template_goal: String,
    pub agent_id: String,
    pub cadence: String,
    pub enabled: bool,
    pub last_run: Option<u64>,
    pub next_run: u64,
    pub failure_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub risk_level: RiskLevel,
    pub permissions_required: Vec<String>,
    pub requires_confirmation: bool,
    pub timeout_ms: u64,
    pub supports_cancellation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    pub event: String,
    pub task_id: Option<String>,
    pub step_id: Option<String>,
    pub timestamp: u64,
    pub data: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMetrics {
    pub tasks_started: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tool_failures: u64,
    pub approvals_requested: u64,
    pub retries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub agent_id: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: u64,
    #[serde(default)]
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrant {
    pub level: String,
    pub key: String,
    pub allowed: bool,
}
