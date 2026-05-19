import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act, waitFor } from "@testing-library/react";
import { FloatingBar } from "../components/FloatingBar";
import { useSessionStore } from "../stores/sessionStore";
import type { AgentSession } from "@agentos/shared-schema";

vi.mock("../hooks/useDaemonConnection", () => ({
  useDaemonConnection: () => ({
    connected: useSessionStore.getState().connected,
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
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

  it("shows active agent name on the pill", () => {
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

  it("shows needs permission label on the pill", () => {
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

  it("shows alert badge (!) when any session waits for permission", () => {
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

  it("hover panel shows list of active sessions", async () => {
    useSessionStore.getState().upsertSession(
      makeSession("s1", "running", { cwd: "/home/project" })
    );
    render(<FloatingBar />);

    const bar = screen.getByText("Claude").closest("[tabindex]");
    if (bar) {
      fireEvent.mouseEnter(bar);
      act(() => { vi.advanceTimersByTime(100); });
    }

    expect(screen.getByText(/Active Sessions/)).toBeInTheDocument();
    expect(screen.getByText("Open Cockpit")).toBeInTheDocument();
    expect(screen.getByText("Stop All")).toBeInTheDocument();
  });

  it("hover panel closes after mouseleave", async () => {
    act(() => {
      useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    });
    render(<FloatingBar />);

    const getBar = () => screen.getAllByText("Claude")[0].closest("[tabindex]")!;

    act(() => {
      fireEvent.mouseEnter(getBar());
      vi.advanceTimersByTime(100);
    });

    expect(screen.getByText("Open Cockpit")).toBeInTheDocument();

    act(() => {
      fireEvent.mouseLeave(getBar());
      fireEvent.blur(getBar());
      vi.runOnlyPendingTimers();
    });

    expect(screen.queryByText("Open Cockpit")).not.toBeInTheDocument();
  });

  it("shows offline state when disconnected", () => {
    useSessionStore.getState().setConnected(false);
    render(<FloatingBar />);
    expect(screen.getByText(/Offline/)).toBeInTheDocument();
  });

  it("priority session is shown when mixed phases", () => {
    useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    useSessionStore.getState().upsertSession(
      makeSession("s2", "waiting_permission", {
        agent: "gemini",
        permission: {
          id: "p1",
          command: "test",
          description: "test",
          created_at: new Date().toISOString(),
          expires_at: new Date().toISOString(),
        },
      })
    );
    render(<FloatingBar />);
    // The pill should show the waiting_permission session (gemini), not Claude
    expect(screen.getByText("Gemini")).toBeInTheDocument();
  });

  it("shows Stop All confirmation popover", async () => {
    useSessionStore.getState().upsertSession(makeSession("s1", "running"));
    useSessionStore.getState().upsertSession(makeSession("s2", "running"));
    render(<FloatingBar />);

    const bar = screen.getByText("Claude").closest("[tabindex]");
    if (bar) {
      fireEvent.mouseEnter(bar);
      act(() => { vi.advanceTimersByTime(100); });
    }

    fireEvent.click(screen.getByText("Stop All"));
    expect(screen.getByText(/Stop all 2 agents/i)).toBeInTheDocument();
    expect(screen.getByText("Cancel")).toBeInTheDocument();
    expect(screen.getByText("Stop")).toBeInTheDocument();
  });
});
