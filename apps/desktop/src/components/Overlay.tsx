import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../stores/sessionStore";
import { useCursorEvents } from "../hooks/useCursorEvents";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";
import { PlanDiffView } from "./PlanDiffView";
import { AnimatePresence, motion } from "framer-motion";

const PERMISSION_TIMEOUT_MS = 5 * 60 * 1000;

type OverlayTab = "permission" | "review";

export function Overlay() {
  const pendingOverlay = useSessionStore((s) => s.pendingOverlay);
  const setPendingOverlay = useSessionStore((s) => s.setPendingOverlay);

  const { acquire, release } = useCursorEvents("overlay");
  const [tab, setTab] = useState<OverlayTab>("permission");

  useEffect(() => {
    if (pendingOverlay) {
      acquire();
    } else {
      release();
    }
  }, [pendingOverlay]);

  useEffect(() => {
    if (!pendingOverlay) return;
    const timer = setTimeout(() => {
      setPendingOverlay(null);
    }, PERMISSION_TIMEOUT_MS);
    return () => clearTimeout(timer);
  }, [pendingOverlay, setPendingOverlay]);

  useEffect(() => {
    setTab("permission");
  }, [pendingOverlay]);

  if (!pendingOverlay) return null;

  const color = getAgentColor(pendingOverlay.agent);
  const name = getAgentDisplayName(pendingOverlay.agent);
  const hasPlansOrDiffs = pendingOverlay.plan != null || pendingOverlay.diff != null;

  const handleApprove = async () => {
    if (!pendingOverlay.permission) return;
    try {
      await invoke("resolve_permission", {
        permissionId: pendingOverlay.permission.id,
        approved: true,
        response: null as string | null,
      });
    } catch (e) {
      console.error("approve failed:", e);
    }
    setPendingOverlay(null);
  };

  const handleReject = async () => {
    if (!pendingOverlay.permission) return;
    try {
      await invoke("resolve_permission", {
        permissionId: pendingOverlay.permission.id,
        approved: false,
        response: null as string | null,
      });
    } catch (e) {
      console.error("reject failed:", e);
    }
    setPendingOverlay(null);
  };

  const handleAnswer = async (answer: string) => {
    if (!pendingOverlay.question) return;
    try {
      await invoke("answer_question", {
        questionId: pendingOverlay.question.id,
        answer,
      });
    } catch (e) {
      console.error("answer failed:", e);
    }
    setPendingOverlay(null);
  };

  const switchTab = (id: OverlayTab) => {
    setTab(id);
  };

  const isWide = tab === "review";

  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center
      bg-[#050505]/60 backdrop-blur-sm font-sans"
      onClick={() => setPendingOverlay(null)}
    >
      <div
        className={`rounded-[24px] bg-[#0C0C0E]/90 border border-white/5
        p-8 shadow-[0_40px_80px_rgba(0,0,0,0.6)] flex flex-col ${
          isWide ? "w-[680px]" : "w-[440px]"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-6">
          <div
            className="w-2.5 h-2.5 rounded-full"
            style={{ backgroundColor: color, boxShadow: `0 0 12px ${color}40` }}
          />
          <span className="text-[15px] font-semibold text-white/90">{name}</span>
          <div className="ml-auto px-2.5 py-1 rounded-full bg-white/[0.03] border border-white/5">
            <span className="text-[10px] font-bold text-white/30 uppercase tracking-[0.1em]">
              {tab === "review" ? "Review Changes" : "Security Check"}
            </span>
          </div>
        </div>

        {hasPlansOrDiffs && (
          <div className="flex gap-1 bg-white/[0.03] p-1 rounded-xl border border-white/5 w-fit mb-6">
            <button
              onClick={() => switchTab("permission")}
              className={`text-[11px] px-3 py-1.5 rounded-lg transition-all duration-200 font-medium ${
                tab === "permission"
                  ? "bg-white/10 text-white shadow-sm"
                  : "text-white/30 hover:text-white/60"
              }`}
            >
              Permission
            </button>
            <button
              onClick={() => switchTab("review")}
              className={`text-[11px] px-3 py-1.5 rounded-lg transition-all duration-200 font-medium ${
                tab === "review"
                  ? "bg-white/10 text-white shadow-sm"
                  : "text-white/30 hover:text-white/60"
              }`}
            >
              Review Changes
            </button>
          </div>
        )}

        <AnimatePresence mode="wait">
          {tab === "permission" ? (
            <motion.div
              key="permission"
              initial={{ opacity: 0, x: -8 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 8 }}
              transition={{ duration: 0.15 }}
            >
              {pendingOverlay.permission && (
                <div className="space-y-8">
                  <div className="space-y-3">
                    <p className="text-[14px] text-white/60 leading-relaxed">
                      {pendingOverlay.permission.description}
                    </p>
                    <div className="bg-black/20 rounded-xl p-4 border border-white/5 font-mono text-[12px] text-accent-blue/70 overflow-x-auto custom-scrollbar">
                      {pendingOverlay.permission.command}
                    </div>
                  </div>
                </div>
              )}

              {pendingOverlay.question && (
                <div className="space-y-4">
                  <p className="text-[14px] text-white/60 leading-relaxed mb-4">
                    {pendingOverlay.question.question}
                  </p>
                  <div className="space-y-2">
                    {pendingOverlay.question.options.map((option, i) => (
                      <button
                        key={i}
                        className="w-full flex items-center gap-3 px-5 py-4 rounded-xl text-[13px] font-medium text-left
                        bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 text-white/70
                        transition-all cursor-pointer"
                        onClick={() => handleAnswer(option)}
                      >
                        <span className="w-5 h-5 rounded-md bg-white/[0.06] flex items-center justify-center text-[10px] font-bold text-white/30 shrink-0">
                          {i + 1}
                        </span>
                        {option}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </motion.div>
          ) : (
            <motion.div
              key="review"
              initial={{ opacity: 0, x: 8 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -8 }}
              transition={{ duration: 0.15 }}
              className="max-h-[420px] overflow-y-auto custom-scrollbar -mx-2 px-2"
            >
              <PlanDiffView sessions={[pendingOverlay]} />
            </motion.div>
          )}
        </AnimatePresence>

        {pendingOverlay.permission && (
          <div className="flex gap-3 mt-8">
            <button
              className="flex-1 px-4 py-3 rounded-xl text-[13px] font-semibold
              bg-white/[0.03] hover:bg-white/5 text-white/40 hover:text-red-400 transition-all cursor-pointer border border-white/5"
              onClick={handleReject}
            >
              Reject
            </button>
            <button
              className="flex-1 px-4 py-3 rounded-xl text-[13px] font-semibold
              text-white transition-all cursor-pointer shadow-lg
              hover:brightness-110 active:scale-[0.98] border border-white/10"
              style={{ backgroundColor: color }}
              onClick={handleApprove}
            >
              Approve Action
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
