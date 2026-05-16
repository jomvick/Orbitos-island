import type { AgentSession } from "@agentos/shared-schema";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";

const PHASE_ICONS: Record<string, string> = {
  running: "\u25CF",
  waiting_permission: "\u2753",
  waiting_question: "\u2753",
  completed: "\u2713",
  failed: "\u2717",
  paused: "\u23F8",
  orphaned: "\u25CB",
};

const PHASE_COLORS: Record<string, string> = {
  running: "#22c55e",
  waiting_permission: "#f59e0b",
  waiting_question: "#3b82f6",
  completed: "#4b5563",
  failed: "#ef4444",
  paused: "#4b5563",
  orphaned: "#4b5563",
};

interface AgentPillProps {
  session: AgentSession;
  compact?: boolean;
  onClick?: () => void;
  showPhaseLabel?: boolean;
}

export function AgentPill({ session, compact, onClick, showPhaseLabel }: AgentPillProps) {
  const color = getAgentColor(session.agent);
  const name = getAgentDisplayName(session.agent);
  const phaseColor = PHASE_COLORS[session.phase] ?? "#6b7280";

  return (
    <button
      onClick={onClick}
      className={`
        flex items-center gap-2 rounded-full
        transition-all duration-300 hover:scale-105 active:scale-95
        ${compact ? "px-2 py-1 text-[10px]" : "px-3 py-1.5 text-xs"}
      `}
      style={{
        backgroundColor: `${color}12`,
        borderColor: `${color}28`,
        borderWidth: 1,
        borderStyle: "solid",
      }}
    >
      <div 
        className="w-1.5 h-1.5 rounded-full shrink-0" 
        style={{ backgroundColor: phaseColor, boxShadow: `0 0 8px ${phaseColor}` }} 
      />
      {showPhaseLabel && (
        <span className="text-[9px] text-white/40 font-medium">
          {session.phase.replace("_", " ")}
        </span>
      )}
      <span className="text-white/90 font-semibold tracking-tight">
        {compact ? name.split(" ")[0] : name}
      </span>
      {!compact && session.model && (
        <span className="font-mono text-[10px] ml-0.5" style={{ color }}>
          {session.model.split("-").pop()}
        </span>
      )}
    </button>
  );
}
