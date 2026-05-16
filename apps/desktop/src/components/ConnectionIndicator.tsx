interface ConnectionIndicatorProps {
  connected: boolean;
}

export function ConnectionIndicator({ connected }: ConnectionIndicatorProps) {
  if (connected) return null;

  return (
    <div
      className="fixed bottom-6 right-6 z-[200]
        flex items-center gap-2.5 px-4 py-2
        rounded-2xl glass-card border-accent-red/20
        text-[11px] font-bold text-accent-red uppercase tracking-wider"
    >
      <div className="w-2 h-2 rounded-full bg-accent-red shadow-[0_0_10px_rgba(239,68,68,0.5)] animate-pulse" />
      Daemon Disconnected
    </div>
  );
}
