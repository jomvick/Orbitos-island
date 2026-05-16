import { useMemo } from "react";
import type { AgentSession } from "@agentos/shared-schema";
import { getAgentColor } from "@agentos/shared-schema";

interface ActivityGraphProps {
  sessions: AgentSession[];
}

export function ActivityGraph({ sessions }: ActivityGraphProps) {
  const agents = useMemo(() => {
    const map = new Map<string, { sessions: number; tokens: number; duration: number }>();
    for (const s of sessions) {
      const existing = map.get(s.agent) ?? {
        sessions: 0,
        tokens: 0,
        duration: 0,
      };
      existing.sessions++;
      existing.tokens += s.tokens_input + s.tokens_output;
      existing.duration +=
        s.phase === "completed" || s.phase === "failed" ? s.duration_ms : 0;
      map.set(s.agent, existing);
    }
    return Array.from(map.entries()).sort((a, b) => b[1].tokens - a[1].tokens);
  }, [sessions]);

  const maxTokens = Math.max(...agents.map(([, d]) => d.tokens), 1);
  const maxDuration = Math.max(...agents.map(([, d]) => d.duration), 1);

  return (
    <div className="space-y-4">
      <div>
        <h4 className="text-[10px] font-semibold text-white/30 uppercase tracking-wider mb-2">
          Token Usage
        </h4>
        <div className="space-y-1.5">
          {agents.map(([agent, data]) => {
            const color = getAgentColor(agent);
            const pct = (data.tokens / maxTokens) * 100;
            return (
              <div key={agent} className="space-y-0.5">
                <div className="flex justify-between text-[10px]">
                  <span className="text-white/50">{agent}</span>
                  <span className="text-white/30">
                    {data.tokens.toLocaleString()} tok
                  </span>
                </div>
                <div className="h-2 rounded-full bg-white/[0.06] overflow-hidden">
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{
                      width: `${Math.max(pct, 2)}%`,
                      backgroundColor: color,
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <div>
        <h4 className="text-[10px] font-semibold text-white/30 uppercase tracking-wider mb-2">
          Duration (completed only)
        </h4>
        <div className="space-y-1.5">
          {agents
            .filter(([, d]) => d.duration > 0)
            .map(([agent, data]) => {
              const color = getAgentColor(agent);
              const pct = (data.duration / maxDuration) * 100;
              const mins = Math.round(data.duration / 60000);
              return (
                <div key={agent} className="space-y-0.5">
                  <div className="flex justify-between text-[10px]">
                    <span className="text-white/50">{agent}</span>
                    <span className="text-white/30">{mins}m</span>
                  </div>
                  <div className="h-2 rounded-full bg-white/[0.06] overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all duration-500"
                      style={{
                        width: `${Math.max(pct, 2)}%`,
                        backgroundColor: color,
                        opacity: 0.6,
                      }}
                    />
                  </div>
                </div>
              );
            })}
        </div>
      </div>
    </div>
  );
}
