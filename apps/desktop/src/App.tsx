import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { FloatingBar } from "./components/FloatingBar";
import { Overlay } from "./components/Overlay";
import { CommandPalette } from "./components/CommandPalette";
import { Settings } from "./components/Settings";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useCursorEvents } from "./hooks/useCursorEvents";
import { AnimatePresence, motion } from "framer-motion";

type View = "main" | "settings";

export default function App() {
  const [view, setView] = useState<View>("main");
  useKeyboardShortcuts();
  const { acquire, release } = useCursorEvents("settings");

  useEffect(() => {
    if (view === "settings") {
      acquire();
    } else {
      release();
    }
  }, [view]);

  useEffect(() => {
    const unlisten = listen<string>("navigate", (event) => {
      setView(event.payload === "/settings" ? "settings" : "main");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleBack = () => {
    setView("main");
  };

  return (
    <div className="flex flex-col items-center pt-4 bg-transparent select-none pointer-events-none w-full h-screen">
      <AnimatePresence mode="wait">
        {view === "main" ? (
          <motion.div
            key="main"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex flex-col items-center pointer-events-none"
          >
            <FloatingBar />
            <Overlay />
            <CommandPalette />
          </motion.div>
        ) : (
          <motion.div
            key="settings"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
            className="w-[560px] h-[600px] rounded-2xl overflow-hidden border border-white/[0.08] shadow-2xl pointer-events-auto"
            style={{
              backdropFilter: "blur(24px)",
              backgroundColor: "rgba(12, 12, 14, 0.92)",
            }}
          >
            <div className="relative h-full flex flex-col">
              <button
                onClick={handleBack}
                className="absolute top-3 left-3 z-10 w-7 h-7 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center transition-colors"
              >
                <svg className="w-3.5 h-3.5 text-white/60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                </svg>
              </button>
              <Settings />
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
