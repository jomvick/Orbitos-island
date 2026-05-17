import { motion } from "framer-motion";

interface TokenBarsProps {
  tokensConsumed: number;
  model?: string | null;
  maxTokens?: number;
}

const HEIGHTS = [4, 6, 8, 10, 12];

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

  const filled = Math.round(Math.min(tokensConsumed / dynamicMax, 1) * HEIGHTS.length);
  const clamped = Math.max(0, Math.min(filled, HEIGHTS.length));

  return (
    <motion.div
      initial={{ width: 0, opacity: 0 }}
      animate={{ width: "auto", opacity: 1 }}
      exit={{ width: 0, opacity: 0 }}
      transition={{ duration: 0.3, ease: "easeInOut" }}
      className="flex items-end gap-[2px] overflow-hidden"
    >
      {HEIGHTS.map((h, i) => (
        <div
          key={i}
          className="w-[3px] rounded-full overflow-hidden"
          style={{
            height: h,
            backgroundColor: i < clamped ? "rgba(255,255,255,0.6)" : "rgba(255,255,255,0.08)",
            transition: "background-color 0.4s ease",
            transitionDelay: `${i * 0.05}s`,
          }}
        />
      ))}
    </motion.div>
  );
}
