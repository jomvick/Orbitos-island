import { create } from "zustand";
import type { AgentSession } from "@agentos/shared-schema";

interface SessionStore {
  sessions: Map<string, AgentSession>;
  isExpanded: boolean;
  connected: boolean;
  pendingOverlay: AgentSession | null;
  setExpanded: (expanded: boolean) => void;
  setConnected: (connected: boolean) => void;
  toggleExpanded: () => void;
  upsertSession: (session: AgentSession) => void;
  syncSessions: (sessions: AgentSession[]) => void;
  removeSession: (id: string) => void;
  setPendingOverlay: (session: AgentSession | null) => void;
}

export const useSessionStore = create<SessionStore>((set) => ({
  sessions: new Map(),
  isExpanded: false,
  connected: false,
  pendingOverlay: null,
  setExpanded: (expanded) => set({ isExpanded: expanded }),
  setConnected: (connected) => set({ connected }),
  toggleExpanded: () => set((s) => ({ isExpanded: !s.isExpanded })),
  upsertSession: (session) =>
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.set(session.id, session);
      return { sessions };
    }),
  syncSessions: (incoming) =>
    set((state) => {
      const sessions = new Map<string, AgentSession>();
      for (const s of incoming) {
        sessions.set(s.id, s);
      }
      return { sessions };
    }),
  removeSession: (id) =>
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.delete(id);
      return { sessions };
    }),
  setPendingOverlay: (session) => set({ pendingOverlay: session }),
}));
