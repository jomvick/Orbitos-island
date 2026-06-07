import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AgentPill } from "../components/AgentPill";
import type { AgentSession } from "@agentos/shared-schema";

function makeSession(overrides?: Partial<AgentSession>): AgentSession {
  return {
    id: "test-1",
    agent: "claude",
    phase: "running",
    tokens_input: 100,
    tokens_output: 50,
    duration_ms: 5000,
    cwd: "/home/user/project",
    updated_at: new Date().toISOString(),
    ...overrides,
  } as AgentSession;
}

describe("AgentPill", () => {
  it("renders agent name", () => {
    render(<AgentPill session={makeSession()} />);
    expect(screen.getByText("Claude")).toBeTruthy();
  });

  it("renders compact variant", () => {
    render(<AgentPill session={makeSession()} compact />);
    expect(screen.getByText("Claude")).toBeTruthy();
  });

  it("shows phase label when showPhaseLabel is true", () => {
    render(<AgentPill session={makeSession({ phase: "waiting_permission" })} showPhaseLabel />);
    expect(screen.getByText("waiting permission")).toBeTruthy();
  });

  it("renders model name when not compact", () => {
    render(<AgentPill session={makeSession({ model: "claude-sonnet-4" })} />);
    expect(screen.getByText("4")).toBeTruthy();
  });

  it("does not render model name in compact mode", () => {
    render(<AgentPill session={makeSession({ model: "claude-sonnet-4" })} compact />);
    expect(screen.queryByText("4")).toBeNull();
  });

  it("calls onClick when clicked", async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(<AgentPill session={makeSession()} onClick={onClick} />);
    await user.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });
});
