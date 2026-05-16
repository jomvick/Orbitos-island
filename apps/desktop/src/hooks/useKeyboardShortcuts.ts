import { useEffect } from "react";
import { useSessionStore } from "../stores/sessionStore";

export function useKeyboardShortcuts() {
  const toggleExpanded = useSessionStore((s) => s.toggleExpanded);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        useSessionStore.getState().setExpanded(false);
        useSessionStore.getState().setPendingOverlay(null);
      }

      if (e.altKey && e.key === "a") {
        toggleExpanded();
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleExpanded]);
}
