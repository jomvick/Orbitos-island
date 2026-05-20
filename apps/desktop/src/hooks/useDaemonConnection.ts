import { useEffect, useCallback, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../stores/sessionStore";
import { shouldPlaySound } from "../stores/settingsStore";
import type { AgentSession, UniversalEvent } from "@agentos/shared-schema";

interface DaemonCommandResponse<T> {
  data: T;
}

interface DaemonEventPayload {
  channel: string;
  data: UniversalEvent | AgentSession | null;
  timestamp: string;
}

const BASE_INTERVAL = 500;
const MAX_INTERVAL = 5000;
const PING_INTERVAL_MS = 5_000;

export function useDaemonConnection() {
  const [connected, setConnected] = useState(false);
  const upsertSession = useSessionStore((s) => s.upsertSession);
  const syncSessions = useSessionStore((s) => s.syncSessions);
  const retryRef = useRef<ReturnType<typeof setTimeout> | null>(null);
const pingRef = useRef<ReturnType<typeof setInterval> | null>(null);
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
      const result: DaemonCommandResponse<AgentSession[]> = await invoke("get_sessions", { filter: null });
      const sessions = result?.data ?? [];
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
        const result: DaemonCommandResponse<AgentSession[]> = await invoke("get_sessions", { filter: null });
        const sessions = result?.data ?? [];
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
          const raw = payload.data;
          if (!raw) return;

          const mapped: AgentSession = "id" in raw && "phase" in raw
            ? raw
            : {
                id: raw.session_id,
                agent: raw.agent,
                phase: (() => {
                  const k = raw.event;
                  if (k === "session_completed") return "completed";
                  if (k === "session_failed") return "failed";
                  if (k === "session_paused") return "paused";
                  if (k === "permission_requested") return "waiting_permission";
                  if (k === "question_asked") return "waiting_question";
                  return "running";
                })(),
                tokens_input: raw.tokens_input ?? 0,
                tokens_output: raw.tokens_output ?? 0,
                duration_ms: raw.duration_ms ?? 0,
                created_at: raw.timestamp ?? new Date().toISOString(),
                updated_at: raw.timestamp ?? new Date().toISOString(),
                last_heartbeat: raw.timestamp ?? new Date().toISOString(),
                event_count: 1,
                cwd: raw.cwd,
                branch: raw.branch,
                model: raw.model,
                permission: raw.permission,
                question: raw.question,
                jump_target: raw.jump_target,
                plan: raw.plan,
                diff: raw.diff,
                error: raw.error,
                current_action: raw.current_action,
              };

          if (!("id" in raw)) {
            const eventKind = (raw as UniversalEvent).event;
            if (eventKind === "permission_requested" && shouldPlaySound("permission_request")) {
              invoke("play_sound", { sound: "permission_request" }).catch(() => {});
            } else if (eventKind === "session_failed" && shouldPlaySound("task_error")) {
              invoke("play_sound", { sound: "task_error" }).catch(() => {});
            } else if (eventKind === "session_completed" && shouldPlaySound("task_completed")) {
              invoke("play_sound", { sound: "task_completed" }).catch(() => {});
            }
          }

          upsertSession(mapped);
        },
      );

      // First sync: immediate
      try {
        const result: DaemonCommandResponse<AgentSession[]> = await invoke("get_sessions", { filter: null });
        const sessions = result?.data ?? [];
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
    scheduleRetry,
    goOnline,
    goOffline,
  ]);

  return { connected };
}
