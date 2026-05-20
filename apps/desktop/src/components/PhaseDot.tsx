export function PhaseDot({ phase }: { phase: string }) {
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