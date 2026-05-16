import { useState, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { useSessionStore } from "../stores/sessionStore";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";

interface Command {
  id: string;
  label: string;
  description: string;
  icon: string;
  action: () => void;
  keywords: string[];
}

export function CommandPalette() {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const sessions = useSessionStore((s) => s.sessions);
  const setExpanded = useSessionStore((s) => s.setExpanded);
  const setPendingOverlay = useSessionStore((s) => s.setPendingOverlay);

  const allSessions = useMemo(
    () => Array.from(sessions.values()),
    [sessions]
  );

  const commands: Command[] = useMemo(() => {
    const cmds: Command[] = [
      {
        id: "toggle-dashboard",
        label: "Toggle Dashboard",
        description: "Open or close the agent dashboard",
        icon: "\u25A6",
        action: () => {
          setExpanded(true);
          setIsOpen(false);
        },
        keywords: ["dashboard", "panel", "open", "toggle", "show"],
      },
      {
        id: "focus-terminal",
        label: "Focus Terminal",
        description: "Jump back to the active terminal",
        icon: "\u2328",
        action: async () => {
          const firstActive = allSessions.find((s) =>
            ["running", "waiting_permission", "waiting_question"].includes(s.phase)
          );
          if (firstActive) {
            try {
              await invoke("jump_to_session", { sessionId: firstActive.id });
            } catch (e) {
              console.error("focus terminal failed:", e);
            }
          }
          setIsOpen(false);
        },
        keywords: ["terminal", "jump", "focus", "tmux", "pane"],
      },
    ];

    for (const session of allSessions.slice(0, 10)) {
      const name = getAgentDisplayName(session.agent);
      cmds.push({
        id: `focus-${session.id}`,
        label: `Focus ${name}`,
        description: session.cwd || session.model || "Jump to session",
        icon: "\u25CF",
        action: async () => {
          try {
            await invoke("jump_to_session", { sessionId: session.id });
          } catch (e) {
            console.error("focus session failed:", e);
          }
          setIsOpen(false);
        },
        keywords: [
          name.toLowerCase(),
          session.agent,
          "focus",
          "jump",
          "goto",
        ],
      });
    }

    cmds.push({
      id: "approve-all",
      label: "Approve All Permissions",
      description: "Approve all pending permission requests",
      icon: "\u2713",
      action: async () => {
        for (const s of allSessions) {
          if (s.phase === "waiting_permission" && s.permission) {
            try {
              await invoke("resolve_permission", {
                permissionId: s.permission.id,
                approved: true,
                response: null as string | null,
              });
            } catch (e) {
              console.error("approve all failed:", e);
            }
          }
        }
        setIsOpen(false);
      },
      keywords: ["approve", "permission", "allow", "all", "accept"],
    });

    cmds.push({
      id: "dismiss-overlay",
      label: "Dismiss Overlay",
      description: "Close any open overlay or notification",
      icon: "\u2715",
      action: () => {
        setPendingOverlay(null);
        setIsOpen(false);
      },
      keywords: ["dismiss", "close", "overlay", "notification", "cancel"],
    });

    return cmds;
  }, [allSessions, setExpanded, setPendingOverlay]);

  const filtered = useMemo(
    () =>
      query.trim()
        ? commands.filter(
            (c) =>
              c.label.toLowerCase().includes(query.toLowerCase()) ||
              c.keywords.some((k) =>
                k.toLowerCase().includes(query.toLowerCase())
              )
          )
        : commands,
    [commands, query]
  );

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.altKey) && e.key === " ") {
        e.preventDefault();
        setIsOpen((prev) => !prev);
        setQuery("");
      }
      if (e.key === "Escape" && isOpen) {
        setIsOpen(false);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isOpen]);

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50);
      setSelectedIndex(0);
    }
  }, [isOpen]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && filtered[selectedIndex]) {
      filtered[selectedIndex].action();
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-[200] flex items-start justify-center pt-[15vh]"
          onClick={() => setIsOpen(false)}
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: -10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: -10 }}
            transition={{ duration: 0.15, ease: "easeOut" }}
            className="w-[420px] max-h-[320px] overflow-hidden rounded-2xl
              bg-[rgba(20,20,25,0.95)] backdrop-blur-xl
              border border-glass-medium shadow-2xl shadow-black/50 font-sans"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center gap-3 px-4 py-3 border-b border-glass-light">
              <span className="text-text-dim text-sm font-mono">{">"}</span>
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setSelectedIndex(0);
                }}
                onKeyDown={handleKeyDown}
                placeholder="Search commands, agents, sessions..."
                className="flex-1 bg-transparent text-sm text-white/70
                  placeholder-white/20 outline-none"
              />
            </div>

            <div className="overflow-y-auto max-h-[260px] py-1">
              {filtered.length === 0 && (
                <p className="text-xs text-white/20 text-center py-6">
                  No results for "{query}"
                </p>
              )}
              {filtered.map((cmd, i) => (
                <button
                  key={cmd.id}
                  onClick={cmd.action}
                  onMouseEnter={() => setSelectedIndex(i)}
                  className={`w-full flex items-center gap-3 px-4 py-2.5 text-left
                    transition-colors ${
                      i === selectedIndex
                        ? "bg-white/[0.06]"
                        : "hover:bg-white/[0.03]"
                    }`}
                >
                  <span className="text-xs w-5 text-center text-white/30">
                    {cmd.icon}
                  </span>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-white/70 truncate">
                      {cmd.label}
                    </p>
                    <p className="text-[10px] text-white/30 truncate">
                      {cmd.description}
                    </p>
                  </div>
                </button>
              ))}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
