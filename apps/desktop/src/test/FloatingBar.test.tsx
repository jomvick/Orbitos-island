import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { FloatingBar } from "../components/FloatingBar";
import { useSessionStore } from "../stores/sessionStore";
import type { AgentSession } from "@agentos/shared-schema";

vi.mock("../hooks/useDaemonConnection", () => ({
  useDaemonConnection: () => ({
    connected: useSessionStore.getState().connected,
  }),
}));

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

describe("FloatingBar", () => {
  beforeEach(() => {
    useSessionStore.getState().syncSessions([]);
    useSessionStore.getState().setConnected(true);
    useSessionStore.getState().setExpanded(false);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows Orbitos Island when no active sessions", () => {
    render(<FloatingBar />);
    expect(screen.getByText("Orbitos Island")).toBeInTheDocument();
  });

  it("shows active agent name", () => {
    useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    render(<FloatingBar />);
    expect(screen.getByText("Claude")).toBeInTheDocument();
  });

  it("shows phase label for running session", () => {
    act(() => {
      useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    });
    render(<FloatingBar />);
    expect(screen.getByText(/running/i)).toBeInTheDocument();
  });

  it("shows needs permission label", () => {
    act(() => {
      useSessionStore.getState().upsertSession(
        makeSession("s1", "waiting_permission", {
          permission: {
            id: "p1",
            command: "test",
            description: "test perm",
            created_at: new Date().toISOString(),
            expires_at: new Date().toISOString(),
          },
        })
      );
    });
    render(<FloatingBar />);
    expect(screen.getByText(/needs permission/i)).toBeInTheDocument();
  });

  it("shows permission badge (!) when waiting for permission", () => {
    useSessionStore.getState().upsertSession(
      makeSession("s1", "waiting_permission", {
        permission: {
          id: "p1",
          command: "test",
          description: "test perm",
          created_at: new Date().toISOString(),
          expires_at: new Date().toISOString(),
        },
      })
    );
    render(<FloatingBar />);
    expect(screen.getByText("!")).toBeInTheDocument();
  });

  it("shows extra count when multiple active sessions", () => {
    useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    useSessionStore.getState().upsertSession(makeSession("s2", "running", { agent: "codex" }));
    render(<FloatingBar />);
    expect(screen.getByText("+1")).toBeInTheDocument();
  });

  it("hover panel opens with detail info", async () => {
    useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    render(<FloatingBar />);

    const bar = screen.getByText("Claude").closest("[tabindex]");
    if (bar) {
      fireEvent.mouseEnter(bar);
      act(() => { vi.advanceTimersByTime(100); });
    }

    expect(screen.getByText("Open Cockpit")).toBeInTheDocument();
    expect(screen.getByText("Stop")).toBeInTheDocument();
  });

  it("hover panel does not close immediately on mouseleave (grace period)", async () => {
    act(() => {
      useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    });
    render(<FloatingBar />);

    const bar = screen.getByText("Claude").closest("[tabindex]");
    if (bar) {
      act(() => {
        fireEvent.mouseEnter(bar);
      });
      act(() => { vi.advanceTimersByTime(100); });

      const liveBar = screen.getByText("Claude").closest("[tabindex]")!;
      act(() => {
        fireEvent.mouseLeave(liveBar);
        fireEvent.blur(liveBar);
      });

      expect(screen.getByText("Open Cockpit")).toBeInTheDocument();

      act(() => {
        vi.runOnlyPendingTimers();
      });

      // After timeout, it should be removed
      expect(screen.queryByText("Open Cockpit")).not.toBeInTheDocument();
    }
  });

  it("shows offline state when disconnected", () => {
    useSessionStore.getState().setConnected(false);
    render(<FloatingBar />);
    expect(screen.getByText(/Offline/)).toBeInTheDocument();
  });
});
