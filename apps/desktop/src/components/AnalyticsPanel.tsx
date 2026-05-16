import type { AgentSession } from "@agentos/shared-schema";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";

interface AnalyticsPanelProps {
  sessions: AgentSession[];
}

export function AnalyticsPanel({ sessions }: AnalyticsPanelProps) {
  const activeSessions = sessions.filter((s) =>
    ["running", "waiting_permission", "waiting_question", "paused"].includes(
      s.phase
    )
  );
  const completedSessions = sessions.filter(
    (s) => s.phase === "completed"
  );
  const failedSessions = sessions.filter((s) => s.phase === "failed");

  const totalTokensInput = sessions.reduce(
    (sum, s) => sum + s.tokens_input,
    0
  );
  const totalTokensOutput = sessions.reduce(
    (sum, s) => sum + s.tokens_output,
    0
  );
  const totalDurationMs = completedSessions.reduce(
    (sum, s) => sum + s.duration_ms,
    0
  );

  const agentGroups = sessions.reduce<Record<string, AgentSession[]>>(
    (acc, s) => {
      (acc[s.agent] ??= []).push(s);
      return acc;
    },
    {}
  );

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-4 gap-2">
        <StatCard label="Active" value={activeSessions.length} color="#22c55e" />
        <StatCard
          label="Total tokens"
          value={`${((totalTokensInput + totalTokensOutput) / 1000).toFixed(
            1
          )}k`}
          color="#6366f1"
        />
        <StatCard
          label="Completed"
          value={completedSessions.length}
          color="#6b7280"
        />
        <StatCard
          label="Time"
          value={`${(totalDurationMs / 3600000).toFixed(1)}h`}
          color="#f59e0b"
        />
      </div>

      <div>
        <h4 className="text-[10px] font-semibold text-white/30 uppercase tracking-wider mb-2">
          Per Agent
        </h4>
        <div className="space-y-1">
          {Object.entries(agentGroups).map(([agent, agentSessions]) => {
            const color = getAgentColor(agent);
            const name = getAgentDisplayName(agent);
            const agentTokens = agentSessions.reduce(
              (sum, s) => sum + s.tokens_input + s.tokens_output,
              0
            );
            const agentDuration = agentSessions
              .filter((s) => s.phase === "completed")
              .reduce((sum, s) => sum + s.duration_ms, 0);

            return (
              <div
                key={agent}
                className="flex items-center gap-3 px-3 py-2 rounded-lg bg-white/[0.03]"
              >
                <div
                  className="w-2 h-2 rounded-full shrink-0"
                  style={{ backgroundColor: color }}
                />
                <span className="text-xs text-white/70 flex-1">{name}</span>
                <span className="text-[10px] text-white/30">
                  {agentSessions.length} sessions
                </span>
                <span className="text-[10px] text-white/30">
                  {agentTokens.toLocaleString()} tok
                </span>
                <span className="text-[10px] text-white/30">
                  {(agentDuration / 60000).toFixed(0)}m
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function StatCard({
  label,
  value,
  color,
}: {
  label: string;
  value: string | number;
  color: string;
}) {
  return (
    <div
      className="px-3 py-2 rounded-lg"
      style={{ backgroundColor: `${color}10` }}
    >
      <p className="text-[10px] text-white/30 uppercase tracking-wider">
        {label}
      </p>
      <p className="text-lg font-semibold mt-0.5" style={{ color }}>
        {value}
      </p>
    </div>
  );
}
