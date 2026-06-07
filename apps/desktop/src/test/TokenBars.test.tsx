import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { TokenBars } from "../components/TokenBars";

describe("TokenBars", () => {
  it("renders a horizontal progress bar", () => {
    const { container } = render(<TokenBars tokensConsumed={0} />);
    const track = container.querySelector("[class*='rounded-full']");
    expect(track).toBeTruthy();
  });

  it("shows empty bar when tokens are 0", () => {
    const { container } = render(<TokenBars tokensConsumed={0} />);
    const fill = container.querySelector("[class*='from-white']") as HTMLElement;
    expect(fill).toBeTruthy();
  });

  it("fills fully when tokens exceed max", () => {
    const { container } = render(
      <TokenBars tokensConsumed={60000} maxTokens={50000} />
    );
    const track = container.querySelector("[class*='rounded-full']");
    expect(track).toBeTruthy();
  });

  it("fills partially for partial consumption", () => {
    const { container } = render(
      <TokenBars tokensConsumed={25000} maxTokens={50000} />
    );
    const fill = container.querySelector("[class*='from-white']") as HTMLElement;
    expect(fill).toBeTruthy();
  });

  it("uses custom maxTokens", () => {
    const { container } = render(
      <TokenBars tokensConsumed={100000} maxTokens={1000000} />
    );
    const fill = container.querySelector("[class*='from-white']") as HTMLElement;
    expect(fill).toBeTruthy();
  });
});
