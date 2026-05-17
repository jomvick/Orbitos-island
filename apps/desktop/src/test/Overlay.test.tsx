import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Overlay } from "../components/Overlay";
import { useSessionStore } from "../stores/sessionStore";
import type { AgentSession } from "@agentos/shared-schema";

function makePermissionSession(): AgentSession {
  return {
    id: "s1",
    agent: "claude",
    phase: "waiting_permission",
    tokens_input: 100,
    tokens_output: 50,
    duration_ms: 5000,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
    event_count: 1,
    permission: {
      id: "p1",
      command: "rm -rf /tmp/test",
      description: "Delete temp directory",
      created_at: new Date().toISOString(),
      expires_at: new Date(Date.now() + 300000).toISOString(),
    },
  };
}

function makeQuestionSession(): AgentSession {
  return {
    id: "s2",
    agent: "claude",
    phase: "waiting_question",
    tokens_input: 100,
    tokens_output: 50,
    duration_ms: 5000,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
    event_count: 1,
    question: {
      id: "q1",
      question: "Which approach do you prefer?",
      options: ["Option A", "Option B"],
      created_at: new Date().toISOString(),
    },
  };
}

describe("Overlay", () => {
  beforeEach(() => {
    useSessionStore.getState().syncSessions([]);
    useSessionStore.getState().setPendingOverlay(null);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders nothing when no pending overlay", () => {
    const { container } = render(<Overlay />);
    expect(container.innerHTML).toBe("");
  });

  it("renders permission overlay with approve/reject buttons", () => {
    useSessionStore.getState().setPendingOverlay(makePermissionSession());
    render(<Overlay />);

    expect(screen.getByText("Delete temp directory")).toBeInTheDocument();
    expect(screen.getByText("Approve Action")).toBeInTheDocument();
    expect(screen.getByText("Reject")).toBeInTheDocument();
    expect(screen.getByText("rm -rf /tmp/test")).toBeInTheDocument();
  });

  it("renders question overlay with options", () => {
    useSessionStore.getState().setPendingOverlay(makeQuestionSession());
    render(<Overlay />);

    expect(screen.getByText("Which approach do you prefer?")).toBeInTheDocument();
    expect(screen.getByText("Option A")).toBeInTheDocument();
    expect(screen.getByText("Option B")).toBeInTheDocument();
  });

  it("dismisses overlay on background click", () => {
    useSessionStore.getState().setPendingOverlay(makePermissionSession());
    const { container } = render(<Overlay />);
    const backdrop = container.firstChild as HTMLElement;
    fireEvent.click(backdrop);

    expect(useSessionStore.getState().pendingOverlay).toBeNull();
  });

  it("auto-dismisses permission overlay after timeout", () => {
    useSessionStore.getState().setPendingOverlay(makePermissionSession());
    render(<Overlay />);

    expect(useSessionStore.getState().pendingOverlay).not.toBeNull();

    act(() => {
      vi.advanceTimersByTime(5 * 60 * 1000);
    });

    expect(useSessionStore.getState().pendingOverlay).toBeNull();
  });

  it("does not auto-dismiss before timeout", () => {
    useSessionStore.getState().setPendingOverlay(makePermissionSession());
    render(<Overlay />);

    act(() => {
      vi.advanceTimersByTime(4 * 60 * 1000);
    });

    expect(useSessionStore.getState().pendingOverlay).not.toBeNull();
  });

  it("clears timeout when overlay is dismissed manually", () => {
    const session = makePermissionSession();
    useSessionStore.getState().setPendingOverlay(session);
    const { unmount } = render(<Overlay />);

    useSessionStore.getState().setPendingOverlay(null);
    unmount();

    act(() => {
      vi.advanceTimersByTime(5 * 60 * 1000);
    });

    expect(useSessionStore.getState().pendingOverlay).toBeNull();
  });
});
