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
          const session = payload.data;
          if (session) {
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

      // Dynamic session refresh — adapts to activity
      const scheduleRefresh = () => {
        const idle = !hasActiveSessions();
        const ms = idle ? REFRESH_IDLE_MS : REFRESH_ACTIVE_MS;
        refreshRef.current = setInterval(async () => {
          if (!useSessionStore.getState().connected) {
            clearInterval(refreshRef.current!);
            return;
          }
          await syncAllSessions();
          // Re-evaluate interval on next cycle
          clearInterval(refreshRef.current!);
          scheduleRefresh();
        }, ms);
      };
      scheduleRefresh();
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
