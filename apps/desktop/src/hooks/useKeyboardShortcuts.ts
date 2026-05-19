import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../stores/sessionStore";

export function useKeyboardShortcuts() {
  const toggleExpanded = useSessionStore((s) => s.toggleExpanded);

  useEffect(() => {
    const handler = async (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        useSessionStore.getState().setExpanded(false);
        useSessionStore.getState().setPendingOverlay(null);
      }

      if (e.altKey && e.key === "a") {
        toggleExpanded();
      }

      const overlay = useSessionStore.getState().pendingOverlay;
      if (overlay?.question) {
        const num = parseInt(e.key, 10);
        if (num >= 1 && num <= overlay.question.options.length) {
          const answer = overlay.question.options[num - 1];
          try {
            await invoke("answer_question", {
              questionId: overlay.question.id,
              answer,
            });
          } catch (err) {
            console.error("answer failed:", err);
          }
          useSessionStore.getState().setPendingOverlay(null);
        }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleExpanded]);
}
