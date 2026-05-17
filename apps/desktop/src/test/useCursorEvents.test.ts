import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";

const mockInvoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { useCursorEvents, _activeConsumers } from "../hooks/useCursorEvents";

describe("useCursorEvents", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    const { activeConsumers } = getActiveConsumers();
    activeConsumers.clear();
  });

  it("calls set_ignore_cursor with false when a consumer acquires", () => {
    const { result } = renderHook(() => useCursorEvents("floatingbar"));

    act(() => {
      result.current.acquire();
    });

    expect(mockInvoke).toHaveBeenCalledWith("set_ignore_cursor", { ignore: false });
  });

  it("calls set_ignore_cursor with true when the last consumer releases", () => {
    const { result: r1 } = renderHook(() => useCursorEvents("floatingbar"));
    const { result: r2 } = renderHook(() => useCursorEvents("overlay"));

    act(() => {
      r1.current.acquire();
      r2.current.acquire();
    });

    mockInvoke.mockClear();

    act(() => {
      r1.current.release();
    });

    expect(mockInvoke).not.toHaveBeenCalledWith("set_ignore_cursor", { ignore: true });

    act(() => {
      r2.current.release();
    });

    expect(mockInvoke).toHaveBeenCalledWith("set_ignore_cursor", { ignore: true });
  });

  it("does not call set_ignore_cursor true if multiple consumers are active", () => {
    const { result: r1 } = renderHook(() => useCursorEvents("floatingbar"));
    const { result: r2 } = renderHook(() => useCursorEvents("commandpalette"));

    act(() => {
      r1.current.acquire();
      r2.current.acquire();
    });

    mockInvoke.mockClear();

    act(() => {
      r1.current.release();
    });

    expect(mockInvoke).not.toHaveBeenCalledWith("set_ignore_cursor", { ignore: true });
  });

  it("releases consumer on unmount", () => {
    const { result, unmount } = renderHook(() => useCursorEvents("overlay"));

    act(() => {
      result.current.acquire();
    });

    mockInvoke.mockClear();
    unmount();

    expect(mockInvoke).toHaveBeenCalledWith("set_ignore_cursor", { ignore: true });
  });

  it("does not call invoke when never acquired", () => {
    mockInvoke.mockClear();
    const { unmount } = renderHook(() => useCursorEvents("floatingbar"));
    unmount();
    // It will call release on unmount, which calls invoke true if size is 0
    expect(mockInvoke).toHaveBeenCalledWith("set_ignore_cursor", { ignore: true });
  });
});

function getActiveConsumers() {
  return { activeConsumers: _activeConsumers ?? new Set() };
}
