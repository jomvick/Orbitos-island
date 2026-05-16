import { useState, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { AgentSession, PlanProposal, DiffPayload, FileDiff } from "@agentos/shared-schema";
import { getAgentColor } from "@agentos/shared-schema";

type SubTab = "plans" | "diffs";

function parseDiffLines(diff: string): { type: "same" | "add" | "remove" | "hunk"; content: string }[] {
  return diff.split("\n").map((line) => {
    if (line.startsWith("@@")) return { type: "hunk" as const, content: line };
    if (line.startsWith("+")) return { type: "add" as const, content: line };
    if (line.startsWith("-")) return { type: "remove" as const, content: line };
    return { type: "same" as const, content: line };
  });
}

function DiffLine({ line }: { line: { type: string; content: string } }) {
  const bg =
    line.type === "add"
      ? "bg-green-500/10"
      : line.type === "remove"
      ? "bg-red-500/10"
      : line.type === "hunk"
      ? "bg-accent-blue/10"
      : "bg-transparent";
  const text =
    line.type === "add"
      ? "text-green-300"
      : line.type === "remove"
      ? "text-red-300"
      : line.type === "hunk"
      ? "text-accent-blue"
      : "text-white/40";

  return (
    <div className={`flex font-mono text-[11px] leading-[18px] ${bg} px-4`}>
      <span className={`shrink-0 w-5 select-none text-white/20`}>
        {line.type === "add" ? "+" : line.type === "remove" ? "-" : line.type === "hunk" ? "@" : " "}
      </span>
      <span className={`whitespace-pre ${text}`}>{line.content}</span>
    </div>
  );
}

function DiffFileCard({ file, agentColor }: { file: FileDiff; agentColor: string }) {
  const [expanded, setExpanded] = useState(true);
  const lines = useMemo(() => parseDiffLines(file.diff_content), [file.diff_content]);
  const statusColor =
    file.status === "created" ? "text-green-400" : file.status === "deleted" ? "text-red-400" : "text-accent-blue";

  return (
    <motion.div
      layout
      className="rounded-xl border border-white/[0.06] overflow-hidden bg-white/[0.02]"
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/[0.02] transition-colors text-left"
      >
        <div className="w-1.5 h-1.5 rounded-full shrink-0" style={{ backgroundColor: agentColor }} />
        <span className={`text-[12px] font-mono font-medium ${statusColor}`}>
          {file.status ? `${file.status} ` : ""}
        </span>
        <span className="text-[12px] font-mono text-white/80">{file.file_path}</span>
        <div className="flex-1" />
        <span className="text-[10px] text-white/30">{lines.filter(l => l.type === "add").length} additions, {lines.filter(l => l.type === "remove").length} deletions</span>
        <motion.svg
          animate={{ rotate: expanded ? 180 : 0 }}
          className="w-3 h-3 text-white/30"
          fill="none" viewBox="0 0 24 24" stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </motion.svg>
      </button>

      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: "auto" }}
            exit={{ height: 0 }}
            className="overflow-hidden"
          >
            <div className="border-t border-white/[0.04] divide-y divide-white/[0.02]">
              {lines.map((line, i) => (
                <DiffLine key={i} line={line} />
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

function PlanCard({ plan, agentColor }: { plan: PlanProposal; agentColor: string }) {
  const [expanded, setExpanded] = useState(true);

  return (
    <motion.div
      layout
      className="rounded-xl border border-white/[0.06] overflow-hidden bg-white/[0.02]"
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/[0.02] transition-colors text-left"
      >
        <div className="w-1.5 h-1.5 rounded-full shrink-0" style={{ backgroundColor: agentColor }} />
        <span className="text-[12px] font-semibold text-white/80">{plan.summary}</span>
        <span className="text-[10px] text-white/30">({plan.items.length} steps)</span>
        <div className="flex-1" />
        <motion.svg
          animate={{ rotate: expanded ? 180 : 0 }}
          className="w-3 h-3 text-white/30"
          fill="none" viewBox="0 0 24 24" stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </motion.svg>
      </button>

      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: "auto" }}
            exit={{ height: 0 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-3 space-y-1.5">
              {plan.reasoning && (
                <p className="text-[11px] text-white/40 italic mb-2 mt-1">{plan.reasoning}</p>
              )}
              {plan.items.map((item, i) => (
                <div key={i} className="flex items-start gap-3 px-3 py-2 rounded-lg bg-white/[0.03]">
                  <span className="w-5 h-5 rounded-full bg-accent-blue/20 text-accent-blue text-[10px] font-bold flex items-center justify-center shrink-0 mt-0.5">
                    {i + 1}
                  </span>
                  <div className="min-w-0">
                    <p className="text-[12px] font-medium text-white/80">{item.action}</p>
                    {item.file && (
                      <p className="text-[10px] font-mono text-white/40 mt-0.5">{item.file}</p>
                    )}
                    {item.details && (
                      <p className="text-[10px] text-white/50 mt-0.5 leading-relaxed">{item.details}</p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

export function PlanDiffView({ sessions }: { sessions: AgentSession[] }) {
  const [subTab, setSubTab] = useState<SubTab>("plans");

  const plans = useMemo(
    () => sessions.filter((s): s is AgentSession & { plan: PlanProposal } => s.plan != null),
    [sessions]
  );

  const diffs = useMemo(
    () => sessions.filter((s): s is AgentSession & { diff: DiffPayload } => s.diff != null),
    [sessions]
  );

  return (
    <div className="space-y-4">
      <div className="flex gap-1 bg-white/[0.03] p-1 rounded-xl border border-white/5 w-fit">
        <button
          onClick={() => setSubTab("plans")}
          className={`text-[11px] px-3 py-1.5 rounded-lg transition-all duration-200 font-medium ${
            subTab === "plans"
              ? "bg-white/10 text-white shadow-sm"
              : "text-white/30 hover:text-white/60"
          }`}
        >
          Plans {plans.length > 0 && `(${plans.length})`}
        </button>
        <button
          onClick={() => setSubTab("diffs")}
          className={`text-[11px] px-3 py-1.5 rounded-lg transition-all duration-200 font-medium ${
            subTab === "diffs"
              ? "bg-white/10 text-white shadow-sm"
              : "text-white/30 hover:text-white/60"
          }`}
        >
          Diffs {diffs.length > 0 && `(${diffs.length})`}
        </button>
      </div>

      <AnimatePresence mode="wait">
        {subTab === "plans" ? (
          <motion.div
            key="plans"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-2"
          >
            {plans.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 opacity-20">
                <div className="w-10 h-10 rounded-full border-2 border-dashed border-white mb-3 flex items-center justify-center">
                  <span className="text-white text-[16px] font-bold">P</span>
                </div>
                <p className="text-[12px] font-medium">No plans proposed</p>
              </div>
            ) : (
              plans.map((session) => (
                <PlanCard
                  key={session.id + "-plan"}
                  plan={session.plan}
                  agentColor={getAgentColor(session.agent)}
                />
              ))
            )}
          </motion.div>
        ) : (
          <motion.div
            key="diffs"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-2"
          >
            {diffs.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 opacity-20">
                <div className="w-10 h-10 rounded-full border-2 border-dashed border-white mb-3 flex items-center justify-center">
                  <span className="text-white text-[16px] font-bold">~</span>
                </div>
                <p className="text-[12px] font-medium">No diffs available</p>
              </div>
            ) : (
              diffs.map((session) =>
                session.diff.files.map((file) => (
                  <DiffFileCard
                    key={session.diff.id + file.file_path}
                    file={file}
                    agentColor={getAgentColor(session.agent)}
                  />
                ))
              )
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}