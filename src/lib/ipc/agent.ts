import { invoke } from "@tauri-apps/api/core";

export type TaskStatus =
  | "IDLE"
  | "UNDERSTANDING"
  | "PLANNING"
  | "WAITING_FOR_APPROVAL"
  | "EXECUTING"
  | "WAITING_FOR_TOOL"
  | "REFLECTING"
  | "REPLANNING"
  | "COMPLETED"
  | "FAILED"
  | "CANCELLED"
  | "PAUSED"
  | "SCHEDULED"
  | "WAITING_FOR_USER";

export type StepStatus = "PENDING" | "RUNNING" | "WAITING_APPROVAL" | "DONE" | "FAILED" | "SKIPPED";
export type RiskLevel = "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";
export type ApprovalStatus = "PENDING" | "APPROVED_ONCE" | "APPROVED_FOR_TASK" | "DENIED" | "EDITED";

export interface PlanStep {
  stepId: string;
  description: string;
  status: StepStatus;
  dependencies: string[];
  requiredTools: string[];
  expectedOutput: string;
  riskLevel: RiskLevel;
  summary?: string | null;
}

export interface Artifact {
  artifactId: string;
  taskId: string;
  kind: string;
  path: string;
  size: number;
  createdAt: number;
}

export interface AgentTask {
  taskId: string;
  userId: string;
  conversationId: string;
  agentId: string;
  goal: string;
  status: TaskStatus;
  createdAt: number;
  updatedAt: number;
  currentStep: string | null;
  plan: PlanStep[];
  toolCalls: Array<{ tool: string; resultSummary: string; status: string }>;
  approvalIds: string[];
  errors: string[];
  artifacts: Artifact[];
  result: string | null;
}

export interface Approval {
  approvalId: string;
  taskId: string;
  stepId: string | null;
  tool: string;
  action: string;
  target: string;
  dataSummary: string;
  reason: string;
  consequences: string;
  riskLevel: RiskLevel;
  status: ApprovalStatus;
  createdAt: number;
  arguments?: {
    query?: string;
    matches?: Array<{ id: string; content: string; source?: string }>;
  };
}

export interface MemoryItem {
  id: string;
  kind: string;
  content: string;
  source: string;
  importance: number;
  scope: string;
  createdAt: number;
}

export interface AgentProfile {
  id: string;
  name: string;
  systemPrompt: string;
  model: string | null;
  tools: string[];
  memoryScope: string;
  approvalPolicy: string;
}

export interface ChatMessage {
  id: string;
  role: string;
  content: string;
  createdAt: number;
  taskId?: string | null;
}

export interface Conversation {
  id: string;
  title: string;
  agentId: string;
  createdAt: number;
  updatedAt: number;
  messages: ChatMessage[];
}

export interface ScheduledTask {
  id: string;
  taskTemplateGoal: string;
  agentId: string;
  cadence: string;
  enabled: boolean;
  lastRun: number | null;
  nextRun: number;
  failureCount: number;
}

export interface ToolDefinition {
  name: string;
  description: string;
  riskLevel: RiskLevel;
  requiresConfirmation: boolean;
}

export interface AgentEvent {
  event: string;
  taskId: string | null;
  stepId: string | null;
  timestamp: number;
  data: Record<string, unknown>;
}

export interface AgentSnapshot {
  tasks: AgentTask[];
  memories: MemoryItem[];
  approvals: Approval[];
  profiles: AgentProfile[];
  schedules: ScheduledTask[];
  conversations: Conversation[];
}

export interface SetupStatus {
  platform: string;
  whisperInstalled: boolean;
  whisperDirectory: string;
  whisperModel: string;
  ollamaAvailable: boolean;
  ollamaModel: string | null;
  firstRun: boolean;
  autostart?: boolean;
  watchFolder?: string;
}

export interface AuditRow {
  timestamp: number;
  event: string;
  data: unknown;
}

export const AGENT_EVENTS = {
  event: "coda://agent-event",
  download: "coda://setup-download",
  screenAsk: "coda://screen-ask",
} as const;

export interface ScreenAskEvent {
  phase: "streaming" | "done" | "error" | string;
  question: string;
  text: string;
  error: string | null;
}

export const agentIpc = {
  overview: () => invoke<AgentSnapshot>("agent_overview"),
  createConversation: (agentId?: string) =>
    invoke<Conversation>("agent_create_conversation", { agentId }),
  createTask: (goal: string, agentId?: string, conversationId?: string) =>
    invoke<AgentTask>("agent_create_task", { input: { goal, agentId, conversationId } }),
  pause: (id: string) => invoke<AgentTask>("agent_pause_task", { id }),
  resume: (id: string) => invoke<AgentTask>("agent_resume_task", { id }),
  cancel: (id: string) => invoke<AgentTask>("agent_cancel_task", { id }),
  retry: (id: string) => invoke<AgentTask>("agent_retry_task", { id }),
  approve: (id: string, scope?: string) =>
    invoke<AgentTask>("agent_approve", { id, input: { scope } }),
  deny: (id: string) => invoke<AgentTask>("agent_deny", { id }),
  remember: (content: string) => invoke<MemoryItem>("agent_remember", { content }),
  recall: (query: string, k?: number) =>
    invoke<Array<{ id: string; text: string; source: string; score: number }>>("agent_recall", {
      query,
      k,
    }),
  forget: (id: string) => invoke<void>("agent_forget", { id }),
  tools: () => invoke<ToolDefinition[]>("agent_list_tools"),
  schedule: (goal: string, cadence?: string, agentId?: string) =>
    invoke<ScheduledTask>("agent_schedule", { goal, cadence, agentId }),
  deleteSchedule: (id: string) => invoke<void>("agent_delete_schedule", { id }),
  actionItems: (summary: string) => invoke<string[]>("agent_action_items", { summary }),
  startVoice: () => invoke("agent_start_voice"),
  stopVoice: (agentId?: string, conversationId?: string) =>
    invoke<AgentTask | null>("agent_stop_voice", { agentId, conversationId }),
  finishVoice: () => invoke<string>("agent_finish_voice"),
  abortVoice: () => invoke<void>("agent_abort_voice"),
  askScreen: (question?: string) => invoke<string>("agent_ask_screen", { question }),
  setupStatus: () => invoke<SetupStatus>("agent_setup_status"),
  downloadWhisper: () => invoke<SetupStatus>("agent_download_whisper_model"),
  setAutostart: (enabled: boolean) => invoke<SetupStatus>("agent_set_autostart", { enabled }),
  listAudit: (query?: string, limit?: number) =>
    invoke<AuditRow[]>("agent_list_audit", { query, limit }),
  exportAudit: () => invoke<string>("agent_export_audit"),
};
