import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export type CursorConsumer =
  | "floatingbar"
  | "overlay"
  | "commandpalette"
  | "settings";

const activeConsumers = new Set<CursorConsumer>();

export { activeConsumers as _activeConsumers };

async function syncCursor() {
  await invoke("set_ignore_cursor", { ignore: activeConsumers.size === 0 });
}

export function useCursorEvents(consumer: CursorConsumer) {
  const acquire = () => {
    activeConsumers.add(consumer);
    invoke("set_ignore_cursor", { ignore: false });
  };
  const release = () => {
    activeConsumers.delete(consumer);
    if (activeConsumers.size === 0) {
      invoke("set_ignore_cursor", { ignore: true });
    }
  };

  useEffect(() => {
    return () => release();
  }, []);

  return { acquire, release };
}
