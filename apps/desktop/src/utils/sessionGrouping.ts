import type { AgentSession } from "@agentos/shared-schema";

export type GroupBy = "none" | "agent" | "project" | "state";

export interface SessionGroup {
  key: string;
  label: string;
  sessions: AgentSession[];
  count: number;
}

export function groupSessions(
  sessions: AgentSession[],
  groupBy: GroupBy
): SessionGroup[] {
  const groups = new Map<string, AgentSession[]>();

  for (const session of sessions) {
    let key: string;

    switch (groupBy) {
      case "agent":
        key = session.agent;
        break;
      case "project": {
        const parts = session.cwd?.split("/").filter(Boolean);
        key = parts?.slice(-2, -1)?.[0] ?? session.cwd ?? "unknown";
        break;
      }
      case "state":
        key = session.phase;
        break;
      default:
        key = "all";
    }

    const arr = groups.get(key) ?? [];
    arr.push(session);
    groups.set(key, arr);
  }

  const labelMap: Record<string, (key: string, count: number) => string> = {
    agent: (key) => key.charAt(0).toUpperCase() + key.slice(1),
    project: (key) => key,
    state: (key) => {
      const labels: Record<string, string> = {
        running: "Running",
        waiting_permission: "Waiting for Permission",
        waiting_question: "Needs Answer",
        completed: "Completed",
        failed: "Failed",
        paused: "Paused",
        orphaned: "Orphaned",
      };
      return labels[key] ?? key;
    },
    none: () => "All Sessions",
  };

  const sortOrder: Record<string, number> = {
    running: 0,
    waiting_permission: 1,
    waiting_question: 2,
    paused: 3,
    completed: 4,
    failed: 5,
    orphaned: 6,
  };

  return Array.from(groups.entries())
    .map(([key, groupSessions]) => ({
      key,
      label: (labelMap[groupBy] ?? labelMap.none)(key, groupSessions.length),
      sessions: groupSessions,
      count: groupSessions.length,
    }))
    .sort((a, b) => {
      const aOrder = sortOrder[a.key] ?? 99;
      const bOrder = sortOrder[b.key] ?? 99;
      return aOrder - bOrder;
    });
}
