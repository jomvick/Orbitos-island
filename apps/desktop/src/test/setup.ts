import "@testing-library/jest-dom";
import { vi, beforeEach } from "vitest";
import React from "react";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(vi.fn())),
}));

vi.mock("framer-motion", () => {
  const proxy = new Proxy(
    {},
    {
      get: (_target, prop) => {
        if (prop === "AnimatePresence") return ({ children }: any) => children;
        if (prop === "motion") {
          return new Proxy(
            {},
            {
              get: (_t, tag) => {
                const Tag = tag as string;
                return React.forwardRef(
                  ({ initial, animate, exit, transition, layout, whileHover, whileTap, variants, custom, ...rest }: any, ref: any) =>
                    React.createElement(Tag, { ...rest, ref })
                );
              },
            }
          );
        }
        if (typeof prop === "string" && prop[0] === prop[0].toUpperCase()) {
          return React.forwardRef(
            ({ initial, animate, exit, transition, layout, whileHover, whileTap, variants, custom, ...rest }: any, ref: any) =>
              React.createElement(prop as string, { ...rest, ref })
          );
        }
        return vi.fn();
      },
    }
  );
  return {
    AnimatePresence: (proxy as any).AnimatePresence,
    motion: (proxy as any).motion,
    useMotionValue: () => ({ get: () => 0, set: () => {} }),
    useTransform: (fn: any) => (typeof fn === "function" ? fn() : "0"),
    animate: () => ({ stop: () => {} }),
    useAnimate: () => [{ current: document.createElement('div') }, vi.fn(() => ({ stop: vi.fn() }))],
  };
});

beforeEach(() => {
  mockInvoke.mockReset();
});
