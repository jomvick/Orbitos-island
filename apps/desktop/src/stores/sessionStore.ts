import { create } from "zustand";
import type { AgentSession } from "@agentos/shared-schema";

interface SessionStore {
  sessions: Map<string, AgentSession>;
  isExpanded: boolean;
  connected: boolean;
  pendingOverlay: AgentSession | null;
  pendingPermission: AgentSession | null;
  pendingQuestion: AgentSession | null;
  setExpanded: (expanded: boolean) => void;
  setConnected: (connected: boolean) => void;
  toggleExpanded: () => void;
  upsertSession: (session: AgentSession) => void;
  syncSessions: (sessions: AgentSession[]) => void;
  removeSession: (id: string) => void;
  setPendingOverlay: (session: AgentSession | null) => void;
  setPendingPermission: (session: AgentSession | null) => void;
  setPendingQuestion: (session: AgentSession | null) => void;
  orphanRunningSessions: () => void;
}

function derivePendingOverlay(
  pendingPermission: AgentSession | null,
  pendingQuestion: AgentSession | null,
): AgentSession | null {
  return pendingPermission ?? pendingQuestion ?? null;
}

export const useSessionStore = create<SessionStore>((set) => ({
  sessions: new Map(),
  isExpanded: false,
  connected: false,
  pendingOverlay: null,
  pendingPermission: null,
  pendingQuestion: null,
  setExpanded: (expanded) => set({ isExpanded: expanded }),
  setConnected: (connected) => set({ connected }),
  toggleExpanded: () => set((s) => ({ isExpanded: !s.isExpanded })),
  upsertSession: (session) =>
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.set(session.id, session);

      let pendingPermission = state.pendingPermission;
      let pendingQuestion = state.pendingQuestion;

      if (session.phase === "waiting_permission" && session.permission) {
        pendingPermission = session;
      } else if (state.pendingPermission?.id === session.id && session.phase !== "waiting_permission") {
        pendingPermission = null;
      }

      if (session.phase === "waiting_question" && session.question) {
        pendingQuestion = session;
      } else if (state.pendingQuestion?.id === session.id && session.phase !== "waiting_question") {
        pendingQuestion = null;
      }

      const pendingOverlay = derivePendingOverlay(pendingPermission, pendingQuestion);

      return { sessions, pendingPermission, pendingQuestion, pendingOverlay };
    }),
  orphanRunningSessions: () =>
    set((state) => {
      const sessions = new Map(state.sessions);
      let changed = false;
      for (const [id, session] of sessions) {
        if (session.phase === "running" || session.phase === "waiting_permission" || session.phase === "waiting_question") {
          sessions.set(id, { ...session, phase: "orphaned" });
          changed = true;
        }
      }
      return changed
        ? { sessions, pendingOverlay: null, pendingPermission: null, pendingQuestion: null }
        : state;
    }),
  syncSessions: (incoming) =>
    set((state) => {
      const sessions = new Map<string, AgentSession>();
      for (const s of incoming) {
        sessions.set(s.id, s);
      }
      const pendingPermission = state.pendingPermission && sessions.has(state.pendingPermission.id)
        ? sessions.get(state.pendingPermission.id)!
        : null;
      const pendingQuestion = state.pendingQuestion && sessions.has(state.pendingQuestion.id)
        ? sessions.get(state.pendingQuestion.id)!
        : null;
      const pendingOverlay = derivePendingOverlay(pendingPermission, pendingQuestion);
      return { sessions, pendingPermission, pendingQuestion, pendingOverlay };
    }),
  removeSession: (id) =>
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.delete(id);
      const pendingPermission = state.pendingPermission?.id === id ? null : state.pendingPermission;
      const pendingQuestion = state.pendingQuestion?.id === id ? null : state.pendingQuestion;
      const pendingOverlay = derivePendingOverlay(pendingPermission, pendingQuestion);
      return { sessions, pendingPermission, pendingQuestion, pendingOverlay };
    }),
  setPendingOverlay: (session) =>
    set((state) => {
      let pendingPermission = state.pendingPermission;
      let pendingQuestion = state.pendingQuestion;
      if (session) {
        if (session.phase === "waiting_permission" && session.permission) {
          pendingPermission = session;
        } else if (session.phase === "waiting_question" && session.question) {
          pendingQuestion = session;
        }
      } else {
        pendingPermission = null;
        pendingQuestion = null;
      }
      return {
        pendingOverlay: derivePendingOverlay(pendingPermission, pendingQuestion),
        pendingPermission,
        pendingQuestion,
      };
    }),
  setPendingPermission: (session) =>
    set((state) => ({
      pendingPermission: session,
      pendingOverlay: derivePendingOverlay(session, state.pendingQuestion),
    })),
  setPendingQuestion: (session) =>
    set((state) => ({
      pendingQuestion: session,
      pendingOverlay: derivePendingOverlay(state.pendingPermission, session),
    })),
}));