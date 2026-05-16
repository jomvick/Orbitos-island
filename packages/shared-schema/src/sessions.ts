import type { AgentKind } from "./agents";
import type { SessionPhase } from "./events";

export interface JumpTarget {
  session_id: string;
  terminal: string;
  pane?: string;
  cwd?: string;
  pid?: number;
}

export interface PermissionRequest {
  id: string;
  command: string;
  description: string;
  context?: string;
  created_at: string;
  expires_at: string;
}

export interface QuestionPrompt {
  id: string;
  question: string;
  options: string[];
  context?: string;
  created_at: string;
}

export interface PlanItem {
  action: string;
  file?: string;
  details?: string;
}

export interface PlanProposal {
  id: string;
  summary: string;
  items: PlanItem[];
  reasoning?: string;
  created_at: string;
}

export interface FileDiff {
  file_path: string;
  diff_content: string;
  language?: string;
  status?: string;
}

export interface DiffPayload {
  id: string;
  session_id: string;
  files: FileDiff[];
  summary?: string;
  created_at: string;
}

export interface AgentSession {
  id: string;
  agent: AgentKind;
  phase: SessionPhase;
  cwd?: string;
  branch?: string;
  model?: string;
  tokens_input: number;
  tokens_output: number;
  duration_ms: number;
  terminal?: string;
  pane?: string;
  permission?: PermissionRequest;
  question?: QuestionPrompt;
  jump_target?: JumpTarget;
  plan?: PlanProposal;
  diff?: DiffPayload;
  error?: string;
  metadata?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
  last_heartbeat: string;
  event_count: number;
}

export interface UniversalEvent {
  id: string;
  agent: AgentKind;
  event: string;
  session_id: string;
  cwd?: string;
  branch?: string;
  model?: string;
  tokens_input?: number;
  tokens_output?: number;
  duration_ms?: number;
  terminal?: string;
  pane?: string;
  permission?: PermissionRequest;
  question?: QuestionPrompt;
  jump_target?: JumpTarget;
  plan?: PlanProposal;
  diff?: DiffPayload;
  error?: string;
  metadata?: Record<string, unknown>;
  timestamp: string;
}
