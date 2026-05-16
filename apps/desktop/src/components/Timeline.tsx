import type { AgentSession } from "@agentos/shared-schema";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";

const EVENT_LABELS: Record<string, string> = {
  session_started: "started task",
  session_completed: "completed task",
  session_failed: "failed",
  permission_requested: "requested permission",
  question_asked: "asked a question",
  activity_updated: "updated activity",
  heartbeat: "heartbeat",
  token_usage: "token usage",
};

interface TimelineProps {
  sessions: AgentSession[];
}

export function Timeline({ sessions }: TimelineProps) {
  const sorted = [...sessions].sort(
    (a, b) =>
      new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
  );

  const entries = sorted.slice(0, 20).map((s) => ({
    id: s.id,
    agent: s.agent,
    agentName: getAgentDisplayName(s.agent),
    color: getAgentColor(s.agent),
    event: s.phase === "completed" ? "session_completed" 
         : s.phase === "failed" ? "session_failed"
         : s.phase === "running" ? "session_started"
         : "activity_updated",
    timestamp: s.updated_at,
    summary: s.cwd || s.model || "",
  }));

  return (
    <div className="space-y-0.5">
      {entries.map((entry) => {
        const time = new Date(entry.timestamp);
        const timeStr = time.toLocaleTimeString([], {
          hour: "2-digit",
          minute: "2-digit",
        });

        return (
          <div
            key={entry.id}
            className="flex items-start gap-3 px-3 py-1.5 rounded-lg
              hover:bg-white/[0.04] transition-colors group"
          >
            <div
              className="w-1.5 h-1.5 rounded-full mt-1.5 shrink-0"
              style={{ backgroundColor: entry.color }}
            />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-xs font-medium text-white/70">
                  {entry.agentName}
                </span>
                <span className="text-[10px] text-white/30">
                  {EVENT_LABELS[entry.event] ?? entry.event}
                </span>
              </div>
              {entry.summary && (
                <p className="text-[10px] text-white/20 truncate mt-0.5">
                  {entry.summary}
                </p>
              )}
            </div>
            <span className="text-[10px] text-white/20 shrink-0">
              {timeStr}
            </span>
          </div>
        );
      })}
    </div>
  );
}
