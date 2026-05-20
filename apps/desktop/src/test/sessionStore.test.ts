import { describe, it, expect, beforeEach } from "vitest";
import { useSessionStore } from "../stores/sessionStore";
import type { AgentSession } from "@agentos/shared-schema";

function makeSession(
  id: string,
  phase: AgentSession["phase"],
  overrides?: Partial<AgentSession>,
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

function makePermissionSession(
  id: string,
  overrides?: Partial<AgentSession>,
): AgentSession {
  return makeSession(id, "waiting_permission", {
    permission: {
      id: `p-${id}`,
      command: "rm -rf /tmp",
      description: "Remove temp directory",
      created_at: new Date().toISOString(),
      expires_at: new Date().toISOString(),
    },
    ...overrides,
  });
}

function makeQuestionSession(
  id: string,
  overrides?: Partial<AgentSession>,
): AgentSession {
  return makeSession(id, "waiting_question", {
    question: {
      id: `q-${id}`,
      question: "Continue?",
      options: ["yes", "no"],
      created_at: new Date().toISOString(),
    },
    ...overrides,
  });
}

describe("sessionStore", () => {
  beforeEach(() => {
    const store = useSessionStore.getState();
    store.syncSessions([]);
    store.setConnected(true);
    store.setExpanded(false);
    store.setPendingOverlay(null);
  });

  it("upserts a session", () => {
    const store = useSessionStore.getState();
    const session = makeSession("s1", "running");
    store.upsertSession(session);

    const stored = useSessionStore.getState().sessions.get("s1");
    expect(stored).toBeDefined();
    expect(stored!.phase).toBe("running");
  });

  it("updates existing session on upsert", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "running", { tokens_input: 100 }));
    store.upsertSession(makeSession("s1", "running", { tokens_input: 200 }));

    const stored = useSessionStore.getState().sessions.get("s1");
    expect(stored!.tokens_input).toBe(200);
    expect(useSessionStore.getState().sessions.size).toBe(1);
  });

  it("removes a session", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "running"));
    store.removeSession("s1");

    expect(useSessionStore.getState().sessions.has("s1")).toBe(false);
  });

  it("syncs sessions replacing all existing", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "running"));
    store.syncSessions([makeSession("s2", "completed")]);

    const state = useSessionStore.getState();
    expect(state.sessions.has("s1")).toBe(false);
    expect(state.sessions.has("s2")).toBe(true);
  });

  it("orphans running sessions on disconnect", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "running"));
    store.upsertSession(makeSession("s2", "waiting_permission"));
    store.upsertSession(makeSession("s3", "waiting_question"));
    store.upsertSession(makeSession("s4", "completed"));
    store.upsertSession(makeSession("s5", "failed"));

    store.orphanRunningSessions();

    const state = useSessionStore.getState();
    expect(state.sessions.get("s1")!.phase).toBe("orphaned");
    expect(state.sessions.get("s2")!.phase).toBe("orphaned");
    expect(state.sessions.get("s3")!.phase).toBe("orphaned");
    expect(state.sessions.get("s4")!.phase).toBe("completed");
    expect(state.sessions.get("s5")!.phase).toBe("failed");
  });

  it("does not clear history on orphan", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "completed"));
    store.upsertSession(makeSession("s2", "running"));

    store.orphanRunningSessions();

    const state = useSessionStore.getState();
    expect(state.sessions.size).toBe(2);
    expect(state.sessions.get("s1")!.phase).toBe("completed");
    expect(state.sessions.get("s2")!.phase).toBe("orphaned");
  });

  it("no-ops orphanRunningSessions when no active sessions exist", () => {
    const store = useSessionStore.getState();
    store.upsertSession(makeSession("s1", "completed"));
    store.upsertSession(makeSession("s2", "failed"));

    const before = useSessionStore.getState();
    store.orphanRunningSessions();
    const after = useSessionStore.getState();

    expect(after.sessions).toBe(before.sessions);
  });

  it("toggles expanded", () => {
    expect(useSessionStore.getState().isExpanded).toBe(false);
    useSessionStore.getState().toggleExpanded();
    expect(useSessionStore.getState().isExpanded).toBe(true);
    useSessionStore.getState().toggleExpanded();
    expect(useSessionStore.getState().isExpanded).toBe(false);
  });

  it("sets pending overlay", () => {
    const session = makePermissionSession("s1");
    useSessionStore.getState().setPendingOverlay(session);
    expect(useSessionStore.getState().pendingOverlay).toEqual(session);
    useSessionStore.getState().setPendingOverlay(null);
    expect(useSessionStore.getState().pendingOverlay).toBeNull();
  });

  it("upsertSession auto-sets pendingPermission for waiting_permission", () => {
    const session = makePermissionSession("s1");
    useSessionStore.getState().upsertSession(session);

    const state = useSessionStore.getState();
    expect(state.pendingPermission).toEqual(session);
    expect(state.pendingOverlay).toEqual(session);
  });

  it("upsertSession auto-sets pendingQuestion for waiting_question", () => {
    const session = makeQuestionSession("s1");
    useSessionStore.getState().upsertSession(session);

    const state = useSessionStore.getState();
    expect(state.pendingQuestion).toEqual(session);
    expect(state.pendingOverlay).toEqual(session);
  });

  it("upsertSession clears pendingPermission when phase changes away", () => {
    const permSession = makePermissionSession("s1");
    useSessionStore.getState().upsertSession(permSession);
    expect(useSessionStore.getState().pendingPermission).toEqual(permSession);

    useSessionStore.getState().upsertSession(makeSession("s1", "completed"));
    expect(useSessionStore.getState().pendingPermission).toBeNull();
  });

  it("upsertSession clears pendingQuestion when phase changes away", () => {
    const qSession = makeQuestionSession("s1");
    useSessionStore.getState().upsertSession(qSession);
    expect(useSessionStore.getState().pendingQuestion).toEqual(qSession);

    useSessionStore.getState().upsertSession(makeSession("s1", "completed"));
    expect(useSessionStore.getState().pendingQuestion).toBeNull();
  });

  it("pendingOverlay prioritizes permission over question", () => {
    const permSession = makePermissionSession("s1");
    const qSession = makeQuestionSession("s2");
    useSessionStore.getState().upsertSession(permSession);
    useSessionStore.getState().upsertSession(qSession);

    expect(useSessionStore.getState().pendingOverlay!.id).toBe("s1");
  });

  it("orphan clears pendingPermission and pendingQuestion", () => {
    const permSession = makePermissionSession("s1");
    const qSession = makeQuestionSession("s2");
    useSessionStore.getState().upsertSession(permSession);
    useSessionStore.getState().upsertSession(qSession);

    useSessionStore.getState().orphanRunningSessions();

    const state = useSessionStore.getState();
    expect(state.pendingPermission).toBeNull();
    expect(state.pendingQuestion).toBeNull();
    expect(state.pendingOverlay).toBeNull();
  });

  it("removeSession clears pending fields for removed session", () => {
    const permSession = makePermissionSession("s1");
    useSessionStore.getState().upsertSession(permSession);

    useSessionStore.getState().removeSession("s1");

    expect(useSessionStore.getState().pendingPermission).toBeNull();
    expect(useSessionStore.getState().pendingOverlay).toBeNull();
  });

  it("syncSessions preserves pending state for still-existing sessions", () => {
    const permSession = makePermissionSession("s1");
    useSessionStore.getState().upsertSession(permSession);

    useSessionStore.getState().syncSessions([makePermissionSession("s1")]);

    expect(useSessionStore.getState().pendingPermission!.id).toBe("s1");
  });
});