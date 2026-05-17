import { useEffect, useCallback, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../stores/sessionStore";
import type { AgentSession } from "@agentos/shared-schema";

interface DaemonEventPayload {
  channel: string;
  data: AgentSession | null;
  timestamp: string;
}

const BASE_INTERVAL = 500;
const MAX_INTERVAL = 5000;
const PING_INTERVAL_MS = 5_000;
const REFRESH_ACTIVE_MS = 5_000;
const REFRESH_IDLE_MS = 20_000;

const ACTIVE_PHASES = new Set([
  "running",
  "waiting_permission",
  "waiting_question",
]);

function hasActiveSessions(): boolean {
  const sessions = useSessionStore.getState().sessions;
  for (const s of sessions.values()) {
    if (ACTIVE_PHASES.has(s.phase)) return true;
  }
  return false;
}

export function useDaemonConnection() {
  const [connected, setConnected] = useState(false);
  const upsertSession = useSessionStore((s) => s.upsertSession);
  const syncSessions = useSessionStore((s) => s.syncSessions);
  const setPendingOverlay = useSessionStore((s) => s.setPendingOverlay);
  const retryRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const refreshRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const attemptRef = useRef(0);

  const goOnline = useCallback(() => {
    setConnected(true);
    useSessionStore.getState().setConnected(true);
    attemptRef.current = 0;
  }, []);

  const goOffline = useCallback(() => {
    setConnected(false);
    useSessionStore.getState().setConnected(false);
    useSessionStore.getState().orphanRunningSessions();
  }, []);

  const syncAllSessions = useCallback(async () => {
    try {
      const result: any = await invoke("get_sessions", { filter: null });
      const sessions: AgentSession[] = result?.data ?? [];
      syncSessions(sessions);
    } catch {
      // silent — ping handler manages connectivity
    }
  }, [syncSessions]);

  const scheduleRetry = useCallback(() => {
    attemptRef.current += 1;
    const delay = Math.min(
      BASE_INTERVAL * Math.pow(2, attemptRef.current - 1),
      MAX_INTERVAL,
    );
    retryRef.current = setTimeout(async () => {
      try {
        const result: any = await invoke("get_sessions", { filter: null });
        const sessions: AgentSession[] = result?.data ?? [];
        syncSessions(sessions);
        goOnline();
      } catch {
        goOffline();
        scheduleRetry();
      }
    }, delay);
  }, [syncSessions, goOnline, goOffline]);

  useEffect(() => {
    let unlistenConnected: UnlistenFn | undefined;
    let unlistenDisconnected: UnlistenFn | undefined;
    let unlistenEvents: UnlistenFn | undefined;

    const setup = async () => {
      unlistenConnected = await listen<boolean>("daemon-connected", () => {
        goOnline();
        syncAllSessions();
      });

      unlistenDisconnected = await listen<boolean>(
        "daemon-disconnected",
        () => {
          goOffline();
        },
      );

      unlistenEvents = await listen<DaemonEventPayload>(
        "daemon-event",
        (event) => {
          const payload = event.payload;
          let session = payload.data as any;
          if (session) {
            // Map UniversalEvent to AgentSession structure if it's a raw event
            if (session.session_id && !session.id) {
              const eventKind = session.event;
              let phase = "running";
              if (eventKind === "session_completed") phase = "completed";
              else if (eventKind === "session_failed") phase = "failed";
              else if (eventKind === "session_paused") phase = "paused";
              else if (eventKind === "permission_requested") phase = "waiting_permission";
              else if (eventKind === "question_asked") phase = "waiting_question";

              session = {
                id: session.session_id,
                agent: session.agent,
                phase,
                tokens_input: session.tokens_input ?? 0,
                tokens_output: session.tokens_output ?? 0,
                duration_ms: session.duration_ms ?? 0,
                created_at: session.timestamp ?? new Date().toISOString(),
                updated_at: session.timestamp ?? new Date().toISOString(),
                last_heartbeat: session.timestamp ?? new Date().toISOString(),
                event_count: 1,
                cwd: session.cwd,
                branch: session.branch,
                model: session.model,
                permission: session.permission,
                question: session.question,
                jump_target: session.jump_target,
                plan: session.plan,
                diff: session.diff,
                error: session.error,
              };
            }

            upsertSession(session);
            if (
              session.phase === "waiting_permission" ||
              session.phase === "waiting_question"
            ) {
              setPendingOverlay(session);
            }
          }
        },
      );

      // First sync: immediate
      try {
        const result: any = await invoke("get_sessions", { filter: null });
        const sessions: AgentSession[] = result?.data ?? [];
        syncSessions(sessions);
        goOnline();
      } catch {
        scheduleRetry();
      }

      // Heartbeat ping every 5s to detect disconnection early
      let failureCount = 0;
      pingRef.current = setInterval(async () => {
        try {
          await invoke("ping");
          failureCount = 0;
          if (!useSessionStore.getState().connected) {
            console.log("reconnected to daemon, syncing state...");
            goOnline();
            syncAllSessions();
          }
        } catch {
          failureCount++;
          if (failureCount >= 2) {
            if (useSessionStore.getState().connected) {
              console.warn("daemon heartbeat failed 2x, going offline");
              goOffline();
            }
          }
        }
      }, PING_INTERVAL_MS);

      // Removed dynamic session refresh to prevent desync with real-time events.
      // We now rely solely on real-time daemon events for state updates,
      // and only use syncAllSessions for initial hydration and recovery.
    };

    setup();

    return () => {
      unlistenConnected?.();
      unlistenDisconnected?.();
      unlistenEvents?.();
      if (retryRef.current) clearTimeout(retryRef.current);
      if (pingRef.current) clearInterval(pingRef.current);
      if (refreshRef.current) clearInterval(refreshRef.current);
    };
  }, [
    syncAllSessions,
    syncSessions,
    upsertSession,
    setPendingOverlay,
    scheduleRetry,
    goOnline,
    goOffline,
  ]);

  return { connected };
}
