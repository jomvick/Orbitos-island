import { AgentSession, getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";
import { PhaseDot } from "./PhaseDot";
import { motion } from "framer-motion";

const TERMINAL_KIND_LABELS: Record<string, string> = {
  Tmux: "tmux",
  Zellij: "zellij",
  Ghostty: "ghostty",
  WezTerm: "wez",
  Kitty: "kitty",
  Konsole: "konsole",
  Unknown: "term",
};

function formatDuration(ms: number): string {
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  const mins = Math.floor(ms / 60_000);
  const secs = ((ms % 60_000) / 1000).toFixed(0);
  return `${mins}m${secs}s`;
}

interface SessionRowProps {
  session: AgentSession;
  onJump: (session: AgentSession) => void;
  onOpenOverlay: (session: AgentSession) => void;
}

export function SessionRow({ session, onJump, onOpenOverlay }: SessionRowProps) {
  const handleClick = () => {
    if (session.permission || session.question) {
      onOpenOverlay(session);
      return;
    }
    if (session.jump_target || session.terminal) {
      onJump(session);
    }
  };

  const agentColor = getAgentColor(session.agent);
  const agentInitial = getAgentDisplayName(session.agent).charAt(0);
  const actionLabel = session.current_action
    ?? (session.cwd ? session.cwd.split("/").pop() : null)
    ?? getAgentDisplayName(session.agent);

  const terminalLabel = session.terminal_kind
    ? TERMINAL_KIND_LABELS[session.terminal_kind] ?? session.terminal_kind.slice(0, 4)
    : session.terminal
      ? session.terminal.slice(0, 4)
      : null;

  return (
    <button
      onClick={handleClick}
      className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
        hover:bg-white/[0.04] active:bg-white/[0.06] transition-all text-left group"
    >
      <div className="flex items-center gap-2 shrink-0">
        <div
          className="w-5 h-5 rounded flex items-center justify-center shrink-0"
          style={{
            backgroundColor: `${agentColor}33`,
            borderColor: `${agentColor}55`,
            borderWidth: 1,
          }}
        >
          <span className="font-bold leading-none text-[9px]" style={{ color: agentColor }}>
            {agentInitial}
          </span>
        </div>
        <PhaseDot phase={session.phase} />
      </div>

      <div className="flex-1 min-w-0 text-left">
        <p className="text-[12px] text-white/80 font-medium truncate leading-tight">
          {actionLabel}
        </p>
        <div className="flex items-center gap-2 mt-0.5">
          {session.cwd && (
            <span className="text-[10px] text-white/30 truncate max-w-[120px] font-mono">
              {session.cwd.split("/").slice(-2).join("/")}
            </span>
          )}
          <span className="text-[10px] text-white/20 font-mono tabular-nums">
            {formatDuration(session.duration_ms)}
          </span>
          {terminalLabel && (
            <span className="text-[10px] font-mono text-white/30 px-1.5 py-0.5 rounded-md bg-white/[0.04] border border-white/[0.06]">
              {terminalLabel}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1.5 shrink-0">
        {session.permission && (
          <motion.span
            layout
            className="w-3.5 h-3.5 rounded-full bg-amber-500 flex items-center justify-center text-[7px] font-bold text-black"
          >
            !
          </motion.span>
        )}
        {session.question && (
          <motion.span
            layout
            className="w-3.5 h-3.5 rounded-full bg-blue-500 flex items-center justify-center text-[7px] font-bold text-black"
          >
            ?
          </motion.span>
        )}
        <svg
          className="w-3 h-3 text-white/20 group-hover:text-white/40 transition-colors"
          fill="none" viewBox="0 0 24 24" stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
        </svg>
      </div>
    </button>
  );
}