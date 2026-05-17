import { useState, useEffect, useRef } from "react";
import { useSessionStore } from "../stores/sessionStore";
import { motion, AnimatePresence, useMotionValue, useTransform, animate } from "framer-motion";
import { useDaemonConnection } from "../hooks/useDaemonConnection";
import { useCursorEvents } from "../hooks/useCursorEvents";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";
import { Dashboard } from "./Dashboard";
import { TokenBars } from "./TokenBars";

// ─── AnimatedNumber ─────────────────────────────────────────────

function AnimatedNumber({ value, duration = 0.6 }: { value: number; duration?: number }) {
const count = useMotionValue(0);
const rounded = useTransform(() => Math.round(count.get()).toLocaleString());

useEffect(() => {
const controls = animate(count, value, { duration, ease: "easeOut" });
return controls.stop;
}, [value]);

return <motion.span>{rounded}</motion.span>;
}

// ─── AgentIcon ──────────────────────────────────────────────────

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

// ─── PhaseDot ───────────────────────────────────────────────────

function PhaseDot({ phase }: { phase: string }) {
const phaseColors: Record<string, string> = {
running: "#22c55e",
waiting_permission: "#f59e0b",
waiting_question: "#3b82f6",
completed: "#6b7280",
failed: "#ef4444",
paused: "#4b5563",
orphaned: "#6b7280",
};

const color = phaseColors[phase] ?? "#6b7280";
const isError = phase === "failed";
const isWarning = ["waiting_permission", "waiting_question"].includes(phase);

const pulseClass = isError
? "dot-pulse-error"
: isWarning
? "dot-pulse-warning"
: "dot-pulse-running";

return (
<div className="relative flex items-center justify-center w-2 h-2 shrink-0">
<div
className={`absolute inset-0 rounded-full ${pulseClass}`}
style={{ backgroundColor: color }}
/>
<div
className="w-2 h-2 rounded-full relative z-10"
style={{
backgroundColor: color,
boxShadow: `0 0 10px ${color}44`,
}}
/>
</div>
);
}

// ─── FloatingBar (Ripple-inspired Island) ───────────────────────

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

export function FloatingBar() {
const sessions = useSessionStore((s) => s.sessions);
const setExpanded = useSessionStore((s) => s.setExpanded);
const isExpanded = useSessionStore((s) => s.isExpanded);
const { connected } = useDaemonConnection();
const [isHovered, setIsHovered] = useState(false);
const [isFocused, setIsFocused] = useState(false);
const hoverTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

const { acquire, release } = useCursorEvents("floatingbar");

useEffect(() => {
  if (isExpanded || isFocused) {
    acquire();
  } else if (!isHovered) {
    release();
  }
}, [isExpanded, isFocused]);

useEffect(() => {
return () => {
if (hoverTimeoutRef.current) clearTimeout(hoverTimeoutRef.current);
};
}, []);

const activeSessions = Array.from(sessions.values()).filter((s) =>
["running", "waiting_permission", "waiting_question"].includes(s.phase)
);

const active = activeSessions[0];
const extraCount = activeSessions.length - 1;
const hasPermission = active?.permission != null;
const hasQuestion = active?.question != null;

const pillState: PillBorderState = !connected
? "offline"
: !active
? "idle"
: active.phase === "waiting_permission"
? "waiting_permission"
: active.phase === "waiting_question"
? "waiting_question"
: active.phase === "failed"
? "failed"
: "running";

const isWaiting = pillState === "waiting_permission" || pillState === "waiting_question";

const phaseLabel = !connected
? "Offline"
: !active
? "Orbitos Island"
: active.phase === "waiting_permission"
? "needs permission"
: active.phase === "waiting_question"
? "needs input"
: active.phase.replace("_", " ");

const targetWidth = isExpanded ? 720 : 340;

return (
<div className="fixed top-5 left-1/2 -translate-x-1/2 z-[100] pointer-events-none">
<motion.div
layout
tabIndex={0}
className="flex flex-col overflow-hidden pointer-events-auto border
backdrop-blur-xl bg-[#0C0C0E]/80 rounded-3xl focus:outline-none"
style={{
borderColor: isWaiting
? "rgba(245,158,11,0.2)"
: BORDER_COLORS[pillState],
boxShadow: isWaiting
? "0 0 24px rgba(245,158,11,0.15)"
: "0 12px 40px rgba(0,0,0,0.4)",
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
          }, HOVER_GRACE_MS);
        }}
onFocus={() => setIsFocused(true)}
onBlur={() => setIsFocused(false)}
>
{/* ── Main Content Row ── */}
<div className="flex items-center gap-3 px-5 py-3 min-h-[52px]">
{/* AgentIcon */}
<AnimatePresence mode="popLayout">
{active ? (
<AgentIcon key={active.agent} agent={active.agent} size={22} />
) : null}
</AnimatePresence>

{/* PhaseDot */}
<div className="flex-shrink-0">
<PhaseDot phase={active ? active.phase : connected ? "completed" : "failed"} />
</div>

{/* Agent Info */}
<div className="flex items-center gap-2 overflow-hidden min-w-0">
<motion.span
layout
className="text-[13px] font-semibold text-white/90 whitespace-nowrap"
>
{active ? getAgentDisplayName(active.agent) : "Orbitos Island"}
</motion.span>

{extraCount > 0 && (
<span className="text-[10px] font-semibold text-white/40 whitespace-nowrap">
+{extraCount}
</span>
)}

<AnimatePresence mode="wait">
<motion.span
key={active ? active.phase : connected ? "idle" : "offline"}
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

{/* TokenBars */}
{active && (
<AnimatePresence>
<TokenBars
key={`tb-${active.id}`}
tokensConsumed={active.tokens_input + active.tokens_output}
model={active.model}
/>
</AnimatePresence>
)}

{/* PermissionBadge */}
<AnimatePresence>
{(hasPermission || hasQuestion) && (
<motion.div
key="perm-badge"
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
</div>

{/* ── Dashboard extension (Large mode) ── */}
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
<div className="px-5 py-4 space-y-4">
{active ? (
<>
{active.phase === "waiting_permission" && (
<div className="flex items-start gap-2 px-3 py-2 rounded-lg bg-amber-500/10 border border-amber-500/20">
<span className="text-amber-400 text-[11px] mt-0.5 shrink-0">!</span>
<p className="text-[11px] text-amber-300/90 font-medium">
Waiting for permission to continue
</p>
</div>
)}
{active.phase === "waiting_question" && (
<div className="flex items-start gap-2 px-3 py-2 rounded-lg bg-blue-500/10 border border-blue-500/20">
<span className="text-blue-400 text-[11px] mt-0.5 shrink-0">?</span>
<p className="text-[11px] text-blue-300/90 font-medium">
Agent needs your input
</p>
</div>
)}

<div className="space-y-1">
<p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold">
Current Task
</p>
<p className="text-[12px] font-mono text-white/75 line-clamp-2 leading-relaxed">
{(active as any).current_action
? (active as any).current_action
: active.cwd
? `Working in ${active.cwd.split("/").pop()}/`
: "Processing\u2026"}
</p>
</div>

<div className="flex items-center gap-6 pt-1 font-mono">
<div className="space-y-0.5">
<p className="text-[9px] text-white/20 uppercase">Tokens</p>
<p className="text-[11px] text-white/60 tabular-nums">
<AnimatedNumber value={active.tokens_input + active.tokens_output} />
</p>
</div>
<div className="space-y-0.5">
<p className="text-[9px] text-white/20 uppercase">Duration</p>
<p className="text-[11px] text-white/60">
{(active.duration_ms / 1000).toFixed(1)}s
</p>
</div>
{active.model && (
<div className="space-y-0.5">
<p className="text-[9px] text-white/20 uppercase">Model</p>
<p className="text-[11px] text-white/60">
{active.model.split("-").pop()}
</p>
</div>
)}
{(active as any).pid && (
<div className="space-y-0.5">
<p className="text-[9px] text-white/20 uppercase">PID</p>
<p className="text-[11px] text-white/40 tabular-nums" title="OS process — tracked by Orbitos watcher">
{(active as any).pid}
</p>
</div>
)}
</div>

<div className="flex gap-2 pt-2">
<button
onClick={() => setExpanded(true)}
className="flex-1 px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 active:scale-[0.98] text-[11px] font-semibold text-white/70 transition-all border border-white/[0.06]"
>
Open Cockpit
</button>
<button
onClick={async () => {
if (!active) return;
try {
const { invoke } = await import("@tauri-apps/api/core");
await invoke("stop_agent", { sessionId: active.id });
} catch (e) {
console.error("stop failed:", e);
}
}}
className="flex-1 px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 active:scale-[0.98] text-[11px] font-semibold text-white/30 hover:text-red-400 transition-all border border-white/[0.06]"
>
Stop
</button>
</div>
</>
) : (
<div className="flex flex-col items-center py-4 space-y-3">
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
{connected ? "Ready \u2014 no active sessions" : "Daemon offline"}
</p>
<button
onClick={() => setExpanded(true)}
className="px-6 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-[11px] font-semibold text-white/70 transition-all border border-white/[0.06]"
>
View History
</button>
</div>
)}
</div>
</motion.div>
)}
</AnimatePresence>
</motion.div>
</div>
);
}
