import { motion } from "framer-motion";

interface TokenBarsProps {
  tokensConsumed: number;
  maxTokens?: number;
}

const HEIGHTS = [4, 6, 8, 10, 12];

export function TokenBars({ tokensConsumed, maxTokens = 50000 }: TokenBarsProps) {
  const filled = Math.round(Math.min(tokensConsumed / maxTokens, 1) * HEIGHTS.length);
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
