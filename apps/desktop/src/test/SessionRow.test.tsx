import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SessionRow } from "../components/SessionRow";
import type { AgentSession, JumpTarget, PermissionRequest } from "@agentos/shared-schema";

const defaultSession: AgentSession = {
  id: "session-1",
  agent: "opencode",
  phase: "running",
  tokens_input: 500,
  tokens_output: 200,
  duration_ms: 15000,
  cwd: "/home/user/project/src",
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
  last_heartbeat: new Date().toISOString(),
  event_count: 10,
  current_action: "refactoring auth module",
  terminal: "tmux",
  terminal_kind: "Tmux",
  model: "claude-sonnet-4",
  branch: "main",
};

function makeSession(overrides?: Partial<AgentSession>): AgentSession {
  return { ...defaultSession, ...overrides };
}

describe("SessionRow", () => {
  it("renders agent action", () => {
    render(<SessionRow session={makeSession()} onJump={vi.fn()} onOpenOverlay={vi.fn()} />);
    expect(screen.getByText("refactoring auth module")).toBeTruthy();
  });

  it("renders cwd fallback when no current_action", () => {
    render(
      <SessionRow
        session={makeSession({ current_action: undefined })}
        onJump={vi.fn()}
        onOpenOverlay={vi.fn()}
      />
    );
    expect(screen.getByText("src")).toBeTruthy();
  });

  it("shows terminal badge", () => {
    const { container } = render(
      <SessionRow session={makeSession()} onJump={vi.fn()} onOpenOverlay={vi.fn()} />
    );
    expect(container.querySelector("[class*='rounded-md']")).toBeTruthy();
  });

  it("calls onJump when clicked on running session", async () => {
    const onJump = vi.fn();
    const user = userEvent.setup();
    render(
      <SessionRow
        session={makeSession({ jump_target: { session_id: "session-1", terminal: "tmux", pane: "1:0" } })}
        onJump={onJump}
        onOpenOverlay={vi.fn()}
      />
    );
    await user.click(screen.getByRole("button"));
    expect(onJump).toHaveBeenCalledWith(expect.objectContaining({ id: "session-1" }));
  });

  it("calls onOpenOverlay when session has permission", async () => {
    const onOpenOverlay = vi.fn();
    const user = userEvent.setup();
    render(
      <SessionRow
        session={makeSession({ permission: { id: "perm-1", command: "npm test", description: "run tests", created_at: new Date().toISOString(), expires_at: new Date().toISOString() } })}
        onJump={vi.fn()}
        onOpenOverlay={onOpenOverlay}
      />
    );
    await user.click(screen.getByRole("button"));
    expect(onOpenOverlay).toHaveBeenCalledWith(expect.objectContaining({ id: "session-1" }));
  });

  it("renders permission badge when session has permission", () => {
    const { container } = render(
      <SessionRow
        session={makeSession({ permission: { command: "npm test" } as any })}
        onJump={vi.fn()}
        onOpenOverlay={vi.fn()}
      />
    );
    const badges = container.querySelectorAll("[class*='rounded-full']");
    expect(badges.length).toBeGreaterThan(0);
  });
});
