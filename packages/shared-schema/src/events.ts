export type EventKind =
  | "session_started"
  | "activity_updated"
  | "permission_requested"
  | "question_asked"
  | "session_completed"
  | "session_failed"
  | "session_paused"
  | "heartbeat"
  | "token_usage"
  | "jump_target_updated"
  | "actionable_state_resolved";

export type SessionPhase =
  | "running"
  | "waiting_permission"
  | "waiting_question"
  | "completed"
  | "failed"
  | "paused"
  | "orphaned";

export const PHASE_DISPLAY: Record<SessionPhase, string> = {
  running: "Running",
  waiting_permission: "Waiting…",
  waiting_question: "Asking…",
  completed: "Complete",
  failed: "Failed",
  paused: "Paused",
  orphaned: "Lost",
};

export const PHASE_ORDER: Record<SessionPhase, number> = {
  running: 0,
  waiting_permission: 1,
  waiting_question: 1,
  paused: 2,
  completed: 3,
  failed: 3,
  orphaned: 4,
};
