import { describe, it, expect, vi, beforeEach } from "vitest";
import { useSessionStore } from "../stores/sessionStore";
import type { AgentSession } from "@agentos/shared-schema";

function makeSession(
  id: string,
  phase: AgentSession["phase"],
  overrides?: Partial<AgentSession>
): AgentSession {
  return {
    id,
    agent: "claude",
    phase,
    tokens_input: 100,
    tokens_output: 50,
    duration_ms: 5000,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
    event_count: 1,
    ...overrides,
  };
}

describe("Daemon disconnect orphaning integration", () => {
  beforeEach(() => {
    useSessionStore.getState().syncSessions([]);
    useSessionStore.getState().setConnected(true);
  });

  it("marks all active sessions as orphaned when daemon goes offline", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "running"));
    store.upsertSession(makeSession("s2", "waiting_permission"));
    store.upsertSession(makeSession("s3", "completed"));
    store.upsertSession(makeSession("s4", "failed"));

    store.setConnected(false);
    store.orphanRunningSessions();

    const state = useSessionStore.getState();
    expect(state.connected).toBe(false);
    expect(state.sessions.get("s1")!.phase).toBe("orphaned");
    expect(state.sessions.get("s2")!.phase).toBe("orphaned");
    expect(state.sessions.get("s3")!.phase).toBe("completed");
    expect(state.sessions.get("s4")!.phase).toBe("failed");
  });

  it("clears pending overlay on disconnect", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "waiting_permission"));
    store.setPendingOverlay(useSessionStore.getState().sessions.get("s1")!);

    expect(useSessionStore.getState().pendingOverlay).not.toBeNull();

    store.orphanRunningSessions();
    store.setPendingOverlay(null);

    expect(useSessionStore.getState().pendingOverlay).toBeNull();
  });

  it("preserves completed session history after reconnect", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "completed"));
    store.upsertSession(makeSession("s2", "running"));

    store.setConnected(false);
    store.orphanRunningSessions();

    expect(useSessionStore.getState().sessions.get("s1")!.phase).toBe("completed");
    expect(useSessionStore.getState().sessions.get("s2")!.phase).toBe("orphaned");

    store.setConnected(true);
    store.syncSessions([
      makeSession("s1", "completed"),
      makeSession("s3", "running"),
    ]);

    const state = useSessionStore.getState();
    expect(state.sessions.get("s1")!.phase).toBe("completed");
    expect(state.sessions.has("s2")).toBe(false);
    expect(state.sessions.get("s3")!.phase).toBe("running");
  });
});
