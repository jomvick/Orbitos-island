import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { TokenBars } from "../components/TokenBars";

describe("TokenBars", () => {
  it("renders 5 bars", () => {
    const { container } = render(<TokenBars tokensConsumed={0} />);
    const bars = container.querySelectorAll("[class*='w-[3px]']");
    expect(bars.length).toBe(5);
  });

  it("fills no bars when tokens are 0", () => {
    const { container } = render(<TokenBars tokensConsumed={0} />);
    const bars = container.querySelectorAll("[class*='w-[3px]']");
    bars.forEach((bar) => {
      expect((bar as HTMLElement).style.backgroundColor).toBe(
        "rgba(255, 255, 255, 0.08)"
      );
    });
  });

  it("fills all bars when tokens exceed max", () => {
    const { container } = render(
      <TokenBars tokensConsumed={60000} maxTokens={50000} />
    );
    const bars = container.querySelectorAll("[class*='w-[3px]']");
    bars.forEach((bar) => {
      expect((bar as HTMLElement).style.backgroundColor).toBe(
        "rgba(255, 255, 255, 0.6)"
      );
    });
  });

  it("fills partial bars for partial consumption", () => {
    const { container } = render(
      <TokenBars tokensConsumed={25000} maxTokens={50000} />
    );
    const bars = container.querySelectorAll("[class*='w-[3px]']");
    const filled = Array.from(bars).filter(
      (b) => {
        const bg = (b as HTMLElement).style.backgroundColor;
        return bg === "rgba(255, 255, 255, 0.6)" || bg === "rgba(255,255,255,0.6)";
      }
    );
    expect(filled.length).toBeGreaterThan(0);
    expect(filled.length).toBeLessThan(5);
  });

  it("uses custom maxTokens", () => {
    const { container: c1 } = render(
      <TokenBars tokensConsumed={100000} maxTokens={1000000} />
    );
    const bars1 = c1.querySelectorAll("[class*='w-[3px]']");
    const filled1 = Array.from(bars1).filter(
      (b) => {
        const bg = (b as HTMLElement).style.backgroundColor;
        return bg === "rgba(255, 255, 255, 0.6)" || bg === "rgba(255,255,255,0.6)";
      }
    );
    expect(filled1.length).toBeLessThan(5);
  });
});
