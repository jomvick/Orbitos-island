import { motion } from "framer-motion";

interface TokenBarsProps {
  tokensConsumed: number;
  model?: string | null;
  maxTokens?: number;
}

const TOKEN_LIMITS: Record<string, number> = {
  "opus-4": 1_000_000,
  "sonnet": 200_000,
  "haiku": 200_000,
  default: 50_000,
};

export function TokenBars({ tokensConsumed, model, maxTokens }: TokenBarsProps) {
  const dynamicMax =
    maxTokens ??
    TOKEN_LIMITS[
      Object.keys(TOKEN_LIMITS).find((k) => model?.toLowerCase().includes(k)) ?? "default"
    ];

  const ratio = Math.min(tokensConsumed / dynamicMax, 1);
  const clamped = Math.max(0, ratio);

  return (
    <motion.div
      initial={{ opacity: 0, scaleX: 0 }}
      animate={{ opacity: 1, scaleX: 1 }}
      exit={{ opacity: 0, scaleX: 0 }}
      transition={{ duration: 0.3, ease: "easeInOut" }}
      style={{ transformOrigin: "right center" }}
      className="w-12 h-1 rounded-full bg-white/[0.08] overflow-hidden shrink-0"
    >
      <motion.div
        className="h-full rounded-full bg-gradient-to-r from-white/40 to-white/70"
        initial={{ width: 0 }}
        animate={{ width: `${clamped * 100}%` }}
        transition={{ duration: 0.5, ease: "easeOut" }}
        style={{
          boxShadow: "0 0 6px rgba(255,255,255,0.15)",
        }}
      />
    </motion.div>
  );
}
