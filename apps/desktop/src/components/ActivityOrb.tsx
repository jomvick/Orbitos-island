import { useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useSessionStore } from "../stores/sessionStore";

export function ActivityOrb() {
  const sessions = useSessionStore((s) => s.sessions);
  const isExpanded = useSessionStore((s) => s.isExpanded);
  const toggleExpanded = useSessionStore((s) => s.toggleExpanded);
  const setPendingOverlay = useSessionStore((s) => s.setPendingOverlay);

  const { activity, color, pulseSpeed, glowIntensity, hasUrgent } =
    useMemo(() => {
      const allSessions = Array.from(sessions.values());
      const totalTokens = allSessions.reduce(
        (sum, s) => sum + s.tokens_input + s.tokens_output,
        0
      );
      const activeCount = allSessions.filter((s) =>
        ["running", "waiting_permission", "waiting_question"].includes(s.phase)
      ).length;
      const hasPermission = allSessions.some(
        (s) => s.phase === "waiting_permission"
      );
      const hasError = allSessions.some((s) => s.phase === "failed");
      const hasQuestion = allSessions.some(
        (s) => s.phase === "waiting_question"
      );

      const urgent = hasPermission || hasError || hasQuestion;

      const stateColor = hasError
        ? "#ef4444"
        : hasPermission
        ? "#f59e0b"
        : hasQuestion
        ? "#f59e0b"
        : activeCount > 0
        ? "#6366f1"
        : "#22c55e";

      const tokensPerSec = totalTokens / Math.max(activeCount, 1);
      const speed = Math.min(0.5 + tokensPerSec / 10000, 2.0);

      return {
        activity: activeCount > 0 ? "active" : "idle",
        color: stateColor,
        pulseSpeed: speed,
        glowIntensity: urgent ? 1.0 : Math.min(0.3 + activeCount * 0.15, 1.0),
        hasUrgent: urgent,
      };
    }, [sessions]);

  if (isExpanded) return null;

  return (
    <AnimatePresence>
      <motion.button
        onClick={toggleExpanded}
        className="fixed bottom-6 right-6 z-50 cursor-pointer"
        initial={{ scale: 0, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0, opacity: 0 }}
        whileHover={{ scale: 1.15 }}
        whileTap={{ scale: 0.9 }}
      >
        <motion.div
          className="relative w-14 h-14 rounded-full flex items-center justify-center"
          animate={{
            boxShadow: [
              `0 0 ${20 * glowIntensity}px ${color}40`,
              `0 0 ${35 * glowIntensity}px ${color}60`,
              `0 0 ${20 * glowIntensity}px ${color}40`,
            ],
          }}
          transition={{
            duration: pulseSpeed,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        >
          <motion.div
            className="absolute inset-0 rounded-full opacity-30"
            style={{
              background: `radial-gradient(circle, ${color} 0%, transparent 70%)`,
            }}
            animate={{
              scale: [1, 1.3, 1],
              opacity: [0.2, 0.4, 0.2],
            }}
            transition={{
              duration: pulseSpeed * 1.5,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          />

          <motion.div
            className="absolute inset-1 rounded-full"
            style={{
              background: `conic-gradient(from 0deg, ${color}, transparent, ${color}88, transparent, ${color})`,
            }}
            animate={{ rotate: [0, 360] }}
            transition={{
              duration: 4 / pulseSpeed,
              repeat: Infinity,
              ease: "linear",
            }}
          />

          <div
            className="absolute inset-2 rounded-full"
            style={{
              backgroundColor: `${color}20`,
              backdropFilter: "blur(4px)",
              border: `1px solid ${color}40`,
            }}
          />

          <motion.div
            className="relative w-5 h-5 rounded-full"
            style={{ backgroundColor: color }}
            animate={{
              scale: hasUrgent ? [1, 1.2, 1] : [1, 1.05, 1],
            }}
            transition={{
              duration: hasUrgent ? 0.8 : pulseSpeed,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          />
        </motion.div>
      </motion.button>
    </AnimatePresence>
  );
}
