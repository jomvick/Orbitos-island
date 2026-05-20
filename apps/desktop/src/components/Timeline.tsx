import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";
import type { SessionPhase } from "@agentos/shared-schema";

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

interface TimelineEntry {
  session_id: string;
  agent: string;
  event_kind: string;
  timestamp: string;
  summary: string;
}

interface TimelineResponse {
  data: TimelineEntry[];
}

const PAGE_SIZE = 20;

const AGENT_OPTIONS = [
  { value: "", label: "All agents" },
  { value: "claude", label: "Claude" },
  { value: "opencode", label: "OpenCode" },
  { value: "codex", label: "Codex" },
  { value: "aider", label: "Aider" },
  { value: "gemini", label: "Gemini" },
  { value: "antigravity", label: "Antigravity" },
  { value: "cursor", label: "Cursor" },
  { value: "copilot", label: "Copilot" },
  { value: "deepseek", label: "DeepSeek" },
];

const PHASE_OPTIONS: { value: string; label: string }[] = [
  { value: "", label: "All events" },
  { value: "running", label: "Started" },
  { value: "completed", label: "Completed" },
  { value: "failed", label: "Failed" },
  { value: "waiting_permission", label: "Permissions" },
  { value: "waiting_question", label: "Questions" },
];

function phaseLabel(phase: string): string {
  return PHASE_OPTIONS.find((p) => p.value === phase)?.label ?? phase;
}

export function Timeline() {
  const [entries, setEntries] = useState<TimelineEntry[]>([]);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);
  const [agentFilter, setAgentFilter] = useState("");
  const [phaseFilter, setPhaseFilter] = useState("");

  const fetchTimeline = useCallback(
    async (reset: boolean) => {
      setLoading(true);
      const currentOffset = reset ? 0 : offset;
      try {
        const result: TimelineResponse = await invoke("get_timeline", {
          limit: PAGE_SIZE,
          offset: currentOffset,
          agent: agentFilter || null,
          phase: phaseFilter || null,
        });
        const data = result?.data ?? [];
        if (reset) {
          setEntries(data);
          setOffset(PAGE_SIZE);
        } else {
          setEntries((prev) => [...prev, ...data]);
          setOffset((prev) => prev + PAGE_SIZE);
        }
        setHasMore(data.length === PAGE_SIZE);
      } catch {
        // silent — handled by connection status
      } finally {
        setLoading(false);
      }
    },
    [offset, agentFilter, phaseFilter]
  );

  useEffect(() => {
    setOffset(0);
    fetchTimeline(true);
  }, [agentFilter, phaseFilter]);

  return (
    <div className="space-y-3">
      {/* Filters */}
      <div className="flex gap-2">
        <select
          value={agentFilter}
          onChange={(e) => setAgentFilter(e.target.value)}
          className="px-2 py-1 rounded-lg bg-white/5 border border-white/10 text-[11px] text-white/60 focus:outline-none focus:border-indigo-500/50"
        >
          {AGENT_OPTIONS.map((o) => (
            <option key={o.value} value={o.value} className="bg-[#1a1a1e]">
              {o.label}
            </option>
          ))}
        </select>
        <select
          value={phaseFilter}
          onChange={(e) => setPhaseFilter(e.target.value)}
          className="px-2 py-1 rounded-lg bg-white/5 border border-white/10 text-[11px] text-white/60 focus:outline-none focus:border-indigo-500/50"
        >
          {PHASE_OPTIONS.map((o) => (
            <option key={o.value} value={o.value} className="bg-[#1a1a1e]">
              {o.label}
            </option>
          ))}
        </select>
      </div>

      {/* Timeline entries */}
      <div className="space-y-0.5">
        {entries.map((entry) => {
          const color = getAgentColor(entry.agent);
          const agentName = getAgentDisplayName(entry.agent);
          const time = new Date(entry.timestamp);
          const timeStr = time.toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
          });

          return (
            <div
              key={`${entry.session_id}-${entry.timestamp}`}
              className="flex items-start gap-3 px-3 py-1.5 rounded-lg
                hover:bg-white/[0.04] transition-colors group"
            >
              <div
                className="w-1.5 h-1.5 rounded-full mt-1.5 shrink-0"
                style={{ backgroundColor: color }}
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-medium text-white/70">
                    {agentName}
                  </span>
                  <span className="text-[10px] text-white/30">
                    {EVENT_LABELS[entry.event_kind] ?? entry.event_kind}
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

        {entries.length === 0 && !loading && (
          <div className="flex flex-col items-center py-10 opacity-20">
            <p className="text-xs font-medium">No timeline events</p>
          </div>
        )}
      </div>

      {/* Load More */}
      {hasMore && (
        <button
          onClick={() => fetchTimeline(false)}
          disabled={loading}
          className="w-full py-2 rounded-xl bg-white/5 hover:bg-white/10
            text-[11px] font-semibold text-white/50 hover:text-white/70
            transition-all border border-white/[0.06] disabled:opacity-30"
        >
          {loading ? "Loading..." : "Load More"}
        </button>
      )}
    </div>
  );
}