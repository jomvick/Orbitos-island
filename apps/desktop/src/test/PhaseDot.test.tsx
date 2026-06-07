import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { PhaseDot } from "../components/PhaseDot";

describe("PhaseDot", () => {
  it("renders a dot element", () => {
    const { container } = render(<PhaseDot phase="running" />);
    const dots = container.querySelectorAll("[class*='rounded-full']");
    expect(dots.length).toBeGreaterThanOrEqual(1);
  });

  it("renders for each phase variant", () => {
    const phases = ["running", "waiting_permission", "waiting_question", "completed", "failed", "paused", "orphaned"];
    for (const phase of phases) {
      const { container } = render(<PhaseDot phase={phase} />);
      expect(container.firstChild).toBeTruthy();
    }
  });

  it("handles unknown phase gracefully", () => {
    const { container } = render(<PhaseDot phase="unknown" />);
    expect(container.firstChild).toBeTruthy();
  });
});
