import { useState, useEffect, useRef } from "react";
import { useSessionStore } from "../stores/sessionStore";
import { motion, AnimatePresence, useMotionValue, useTransform, animate } from "framer-motion";
import { useDaemonConnection } from "../hooks/useDaemonConnection";
import { useCursorEvents } from "../hooks/useCursorEvents";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";
import type { AgentSession } from "@agentos/shared-schema";
import { Dashboard } from "./Dashboard";
import { SessionRow } from "./SessionRow";
import { PhaseDot } from "./PhaseDot";

function AnimatedNumber({ value, duration = 0.6 }: { value: number; duration?: number }) {
const count = useMotionValue(0);
const rounded = useTransform(() => Math.round(count.get()).toLocaleString());

useEffect(() => {
const controls = animate(count, value, { duration, ease: "easeOut" });
return controls.stop;
}, [value]);

return <motion.span>{rounded}</motion.span>;
}

function AgentIcon({ agent, size = 22 }: { agent: string; size?: number }) {
const color = getAgentColor(agent);
const initial = getAgentDisplayName(agent).charAt(0);

return (
<motion.div
layout
initial={{ scale: 0.8, opacity: 0 }}
animate={{ scale: 1, opacity: 1 }}
exit={{ scale: 0.8, opacity: 0 }}
transition={{ type: "spring", stiffness: 400, damping: 25 }}
className="rounded-md flex items-center justify-center shrink-0"
style={{
width: size,
height: size,
backgroundColor: `${color}33`,
borderColor: `${color}55`,
borderWidth: 1,
borderStyle: "solid",
}}
>
<span
className="font-bold leading-none"
style={{ color, fontSize: size * 0.55 }}
>
{initial}
</span>
</motion.div>
);
}

const PRIORITY: Record<string, number> = {
waiting_permission: 0,
waiting_question: 1,
running: 2,
};

function pickPrioritySession(sessions: AgentSession[]): AgentSession | null {
if (sessions.length === 0) return null;
let best = sessions[0];
for (const s of sessions) {
if ((PRIORITY[s.phase] ?? 99) < (PRIORITY[best.phase] ?? 99)) {
best = s;
}
}
return best;
}

type PillBorderState = "offline" | "idle" | "running" | "waiting_permission" | "waiting_question" | "failed";

const BORDER_COLORS: Record<PillBorderState, string> = {
offline: "rgba(239,68,68,0.3)",
idle: "rgba(255,255,255,0.05)",
running: "rgba(255,255,255,0.07)",
waiting_permission: "rgba(245,158,11,0.2)",
waiting_question: "rgba(245,158,11,0.2)",
failed: "rgba(239,68,68,0.25)",
};

const HOVER_GRACE_MS = 80;

const safeInvoke = async (cmd: string, args?: any) => {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke(cmd, args);
  } catch (err) {
    console.error(`Tauri invoke failed for ${cmd}:`, err);
  }
};

export function FloatingBar() {
const sessions = useSessionStore((s) => s.sessions);
const setExpanded = useSessionStore((s) => s.setExpanded);
const setPendingOverlay = useSessionStore((s) => s.setPendingOverlay);
const isExpanded = useSessionStore((s) => s.isExpanded);
const { connected } = useDaemonConnection();
const [isHovered, setIsHovered] = useState(false);
const [isFocused, setIsFocused] = useState(false);
const [isDragging, setIsDragging] = useState(false);
const [stopAllConfirm, setStopAllConfirm] = useState(false);
const hoverTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
const containerRef = useRef<HTMLDivElement>(null);
const lastSize = useRef({ width: 364, height: 78 });
const shrinkTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
const currentWindowSize = useRef({ width: 364, height: 78 });

const { acquire, release } = useCursorEvents("floatingbar");

const handleMouseDown = (e: React.MouseEvent) => {
  const target = e.target as HTMLElement;
  if (
    target.closest("button") ||
    target.closest("a") ||
    target.closest("input") ||
    target.closest(".cursor-pointer")
  ) {
    return;
  }

  const startX = e.clientX;
  const startY = e.clientY;
  let dragStarted = false;

  const handleMouseMove = (event: MouseEvent) => {
    const deltaX = Math.abs(event.clientX - startX);
    const deltaY = Math.abs(event.clientY - startY);
    const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

    if (!dragStarted && distance > 5) {
      dragStarted = true;
      setIsDragging(true);
      safeInvoke("start_drag");
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  };

  document.addEventListener("mousemove", handleMouseMove);
  document.addEventListener("mouseup", handleMouseUp);
};

useEffect(() => {
  if (isExpanded || isFocused) {
    acquire();
  } else if (!isHovered) {
    release();
  }
}, [isExpanded, isFocused]);

useEffect(() => {
  if (!containerRef.current || typeof ResizeObserver === "undefined") return;

  const resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const element = containerRef.current;
      if (!element) continue;

      const width = element.offsetWidth;
      const height = element.offsetHeight;

      if (width === 0 || height === 0) continue;

      if (shrinkTimeoutRef.current) {
        clearTimeout(shrinkTimeoutRef.current);
        shrinkTimeoutRef.current = null;
      }

      if (width > currentWindowSize.current.width || height > currentWindowSize.current.height) {
        currentWindowSize.current = { width, height };
        safeInvoke("update_window_size", { width, height });
      } else {
        shrinkTimeoutRef.current = setTimeout(() => {
          currentWindowSize.current = { width, height };
          safeInvoke("update_window_size", { width, height });
        }, 350);
      }
    }
  });

  resizeObserver.observe(containerRef.current);
  return () => {
    resizeObserver.disconnect();
    if (shrinkTimeoutRef.current) clearTimeout(shrinkTimeoutRef.current);
  };
}, [isExpanded, isHovered, isFocused]);

useEffect(() => {
return () => {
if (hoverTimeoutRef.current) clearTimeout(hoverTimeoutRef.current);
};
}, []);

const activeSessions = Array.from(sessions.values()).filter((s) =>
["running", "waiting_permission", "waiting_question"].includes(s.phase)
);

const prioritySession = pickPrioritySession(activeSessions);
const extraCount = activeSessions.length - 1;

const pillState: PillBorderState = !connected
? "offline"
: !prioritySession
? "idle"
: prioritySession.phase === "waiting_permission"
? "waiting_permission"
: prioritySession.phase === "waiting_question"
? "waiting_question"
: prioritySession.phase === "failed"
? "failed"
: "running";

// Instantly pre-allocate physical window space on state transitions
useEffect(() => {
  const expandWindow = async () => {
    let targetWidth = 340;
    let targetHeight = 54;

    if (isExpanded) {
      targetWidth = 720;
      targetHeight = 700;
    } else if (isHovered || isFocused) {
      targetWidth = 420;
      targetHeight = activeSessions.length > 0
        ? Math.min(52 + activeSessions.length * 72 + 80, 520)
        : 200;
    } else {
      return;
    }

    const width = targetWidth + 24;
    const height = targetHeight + 24;
    currentWindowSize.current = { width, height };
    await safeInvoke("update_window_size", { width, height, center: isExpanded });
  };

  expandWindow();
}, [isExpanded, isHovered, isFocused, activeSessions.length]);

const getBorderColor = () => {
  if (!connected) return "rgba(239, 68, 68, 0.4)";
  if (prioritySession) {
    if (prioritySession.phase === "waiting_permission" || prioritySession.phase === "waiting_question") {
      return "rgba(245, 158, 11, 0.4)";
    }
    if (prioritySession.phase === "failed") {
      return "rgba(239, 68, 68, 0.4)";
    }
    const agentColor = getAgentColor(prioritySession.agent);
    return `${agentColor}66`;
  }
  return "rgba(255, 255, 255, 0.08)";
};

const getBoxShadow = () => {
  if (!connected) {
    return "0 12px 40px rgba(0, 0, 0, 0.5), 0 0 20px rgba(239, 68, 68, 0.15)";
  }
  if (prioritySession) {
    if (prioritySession.phase === "waiting_permission" || prioritySession.phase === "waiting_question") {
      return "0 12px 40px rgba(0, 0, 0, 0.5), 0 0 24px rgba(245, 158, 11, 0.25)";
    }
    if (prioritySession.phase === "failed") {
      return "0 12px 40px rgba(0, 0, 0, 0.5), 0 0 24px rgba(239, 68, 68, 0.2)";
    }
    const agentColor = getAgentColor(prioritySession.agent);
    return `0 12px 40px rgba(0, 0, 0, 0.5), 0 0 20px ${agentColor}22`;
  }
  return "0 12px 40px rgba(0, 0, 0, 0.4)";
};

const phaseLabel = !connected
? "Offline"
: !prioritySession
? "Orbitos Island"
: prioritySession.phase === "waiting_permission"
? "needs permission"
: prioritySession.phase === "waiting_question"
? "needs input"
: prioritySession.phase.replace("_", " ");

const hasAnyPermission = activeSessions.some((s) => s.permission != null);
const hasAnyQuestion = activeSessions.some((s) => s.question != null);

const handleStopAll = async () => {
  for (const s of activeSessions) {
    await safeInvoke("stop_agent", { sessionId: s.id });
  }
  setStopAllConfirm(false);
};

const handleSessionClick = (session: AgentSession) => {
    if (session.permission || session.question) {
      setPendingOverlay(session);
      return;
    }
    if (session.jump_target || session.terminal) {
      safeInvoke("jump_to_session", { sessionId: session.id });
    }
  };

const targetWidth = isExpanded ? 720 : 420;

return (
<div ref={containerRef} className="p-3 w-fit pointer-events-none">
<motion.div
layout
tabIndex={0}
onMouseDown={handleMouseDown}
className={`flex flex-col overflow-hidden pointer-events-auto border backdrop-blur-xl bg-[#0C0C0E]/80 rounded-3xl focus:outline-none transition-shadow ${
  isDragging ? "cursor-grabbing shadow-2xl scale-[1.01]" : "cursor-grab shadow-xl hover:scale-[1.005]"
}`}
style={{
borderColor: getBorderColor(),
boxShadow: getBoxShadow(),
}}
animate={{ width: targetWidth }}
transition={{ type: "spring", stiffness: 300, damping: 30 }}
onMouseEnter={() => {
          if (hoverTimeoutRef.current) clearTimeout(hoverTimeoutRef.current);
          acquire();
          setIsHovered(true);
        }}
        onMouseLeave={() => {
          hoverTimeoutRef.current = setTimeout(() => {
            if (!isExpanded) release();
            setIsHovered(false);
            setStopAllConfirm(false);
          }, HOVER_GRACE_MS);
        }}
onFocus={() => setIsFocused(true)}
onBlur={() => setIsFocused(false)}
>
{/* ── Main Content Row ── */}
<div className="flex items-center gap-3 px-5 py-3 min-h-[52px]">
{prioritySession ? (
<>
  {/* AgentIcon of priority session */}
  <AnimatePresence mode="popLayout">
    <AgentIcon key={prioritySession.agent} agent={prioritySession.agent} size={22} />
  </AnimatePresence>

  {/* PhaseDot */}
  <div className="flex-shrink-0">
    <PhaseDot phase={prioritySession.phase} />
  </div>

  {/* Agent Info */}
  <div className="flex items-center gap-2 overflow-hidden min-w-0">
    <motion.span
      layout
      className="text-[13px] font-semibold text-white/90 whitespace-nowrap"
    >
      {getAgentDisplayName(prioritySession.agent)}
    </motion.span>

    <AnimatePresence mode="wait">
      <motion.span
        key={prioritySession.phase}
        initial={{ opacity: 0, y: 6 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -6 }}
        transition={{ duration: 0.2 }}
        className="text-[11px] text-white/35 font-medium whitespace-nowrap"
      >
        — {phaseLabel}
      </motion.span>
    </AnimatePresence>
  </div>

  <div className="flex-1" />

  {/* Extra count badge */}
  {extraCount > 0 && (
    <span className="text-[10px] font-semibold text-white/50 whitespace-nowrap px-2 py-0.5 rounded-full bg-white/[0.06] border border-white/[0.08]">
      +{extraCount}
    </span>
  )}

  {/* Permission/Question badge */}
  <AnimatePresence>
    {(hasAnyPermission || hasAnyQuestion) && (
      <motion.div
        key="alert-badge"
        initial={{ scale: 0, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0, opacity: 0 }}
        transition={{ type: "spring", stiffness: 500, damping: 15 }}
        className="w-4 h-4 rounded-full bg-amber-500 flex items-center justify-center shrink-0"
      >
        <span className="text-[9px] font-bold text-black leading-none">!</span>
      </motion.div>
    )}
  </AnimatePresence>
</>
) : (
<>
  {/* PhaseDot */}
  <div className="flex-shrink-0">
    <PhaseDot phase={connected ? "completed" : "failed"} />
  </div>

  <motion.span className="text-[13px] font-semibold text-white/90 whitespace-nowrap">
    Orbitos Island
  </motion.span>

  {!connected && (
    <span className="text-[11px] text-white/35 font-medium">— Offline</span>
  )}
</>
)}
</div>

{/* ── Dashboard extension (Cockpit mode) ── */}
<AnimatePresence>
{isExpanded && <Dashboard embedded />}
</AnimatePresence>

{/* ── Hover/focus detail (Quick mode) ── */}
<AnimatePresence>
{(isHovered || isFocused) && !isExpanded && (
<motion.div
initial={{ height: 0, opacity: 0 }}
animate={{ height: "auto", opacity: 1 }}
exit={{ height: 0, opacity: 0 }}
transition={{ duration: 0.2, ease: "easeInOut" }}
className="overflow-hidden border-t border-white/[0.06]"
>
{activeSessions.length > 0 ? (
<div className="px-4 py-3 space-y-1">
  <div className="flex items-center justify-between mb-2 px-1">
    <span className="text-[10px] font-bold text-white/30 uppercase tracking-[0.1em]">
      Active Sessions ({activeSessions.length})
    </span>
  </div>

  <div className="space-y-0.5 max-h-[360px] overflow-y-auto custom-scrollbar">
    {activeSessions.map((s) => (
      <SessionRow
        key={s.id}
        session={s}
        onJump={(session) => safeInvoke("jump_to_session", { sessionId: session.id })}
        onOpenOverlay={(session) => setPendingOverlay(session)}
      />
    ))}
  </div>

  {/* Bottom actions */}
  <div className="flex gap-2 pt-3 border-t border-white/[0.06] mt-2">
    <button
      onClick={() => setExpanded(true)}
      className="flex-1 px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 active:scale-[0.98] text-[11px] font-semibold text-white/70 transition-all border border-white/[0.06]"
    >
      Open Cockpit
    </button>
    <div className="relative">
      <button
        onClick={() => setStopAllConfirm(!stopAllConfirm)}
        className="px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 active:scale-[0.98] text-[11px] font-semibold text-white/30 hover:text-red-400 transition-all border border-white/[0.06]"
      >
        Stop All
      </button>
      <AnimatePresence>
        {stopAllConfirm && (
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: -4 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: -4 }}
            transition={{ duration: 0.15 }}
            className="absolute bottom-full right-0 mb-2 w-64 p-4 rounded-xl bg-[#1a1a1e] border border-white/[0.08] shadow-2xl z-50"
          >
            <p className="text-[12px] font-semibold text-white/90 mb-1">
              Stop all {activeSessions.length} agents?
            </p>
            <p className="text-[10px] text-white/40 leading-relaxed mb-4">
              This will terminate {activeSessions.map(s => getAgentDisplayName(s.agent)).join(", ")} sessions.
            </p>
            <div className="flex gap-2">
              <button
                onClick={() => setStopAllConfirm(false)}
                className="flex-1 px-3 py-2 rounded-lg text-[11px] font-medium text-white/50 bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.06] transition-all"
              >
                Cancel
              </button>
              <button
                onClick={handleStopAll}
                className="flex-1 px-3 py-2 rounded-lg text-[11px] font-semibold text-white bg-red-500/80 hover:bg-red-500 border border-red-500/30 transition-all"
              >
                Stop
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  </div>
</div>
) : (
<div className="flex flex-col items-center py-6 space-y-3">
  <div className="relative flex items-center justify-center">
    <div className="w-10 h-10 rounded-full border-2 border-dashed border-white/10" />
    <div
      className="absolute w-2.5 h-2.5 rounded-full"
      style={{
        backgroundColor: connected ? "#22c55e" : "#ef4444",
      }}
    />
  </div>
  <p className="text-[12px] text-white/40 italic">
    {connected ? "Ready — no active sessions" : "Daemon offline"}
  </p>
  <button
    onClick={() => setExpanded(true)}
    className="px-6 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-[11px] font-semibold text-white/70 transition-all border border-white/[0.06]"
  >
    View History
  </button>
</div>
)}
</motion.div>
)}
</AnimatePresence>
</motion.div>
</div>
);
}
