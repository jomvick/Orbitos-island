export type AgentKind =
  | "opencode"
  | "claude"
  | "codex"
  | "antigravity"
  | "aider"
  | "gemini"
  | "cursor"
  | "kimi"
  | "qoder"
  | "qwen"
  | "factory"
  | "codebuddy"
  | (string & {});

export const AGENT_COLORS: Record<string, string> = {
  opencode: "#6366f1",
  claude: "#d97706",
  codex: "#059669",
  antigravity: "#7c3aed",
  aider: "#0891b2",
  gemini: "#2563eb",
  cursor: "#dc2626",
  kimi: "#c026d3",
  qoder: "#f59e0b",
  qwen: "#0ea5e9",
  factory: "#84cc16",
  codebuddy: "#f472b6",
};

export const AGENT_DISPLAY_NAMES: Record<string, string> = {
  opencode: "OpenCode",
  claude: "Claude",
  codex: "Codex",
  antigravity: "Antigravity",
  aider: "Aider",
  gemini: "Gemini",
  cursor: "Cursor",
  kimi: "Kimi",
  qoder: "Qoder",
  qwen: "Qwen",
  factory: "Factory",
  codebuddy: "Codebuddy",
};

export function getAgentColor(agent: string): string {
  return AGENT_COLORS[agent] ?? "#6b7280";
}

export function getAgentDisplayName(agent: string): string {
  return AGENT_DISPLAY_NAMES[agent] ?? agent;
}
