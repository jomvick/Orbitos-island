import { invoke } from "@tauri-apps/api/core";
import { useSessionStore } from "../stores/sessionStore";
import { getAgentColor, getAgentDisplayName } from "@agentos/shared-schema";

export function Overlay() {
  const pendingOverlay = useSessionStore((s) => s.pendingOverlay);
  const setPendingOverlay = useSessionStore((s) => s.setPendingOverlay);

  if (!pendingOverlay) return null;

  const color = getAgentColor(pendingOverlay.agent);
  const name = getAgentDisplayName(pendingOverlay.agent);

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

  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center
        bg-[#050505]/60 backdrop-blur-sm font-sans"
      onClick={() => setPendingOverlay(null)}
    >
      <div
        className="w-[440px] rounded-[24px] bg-[#0C0C0E]/90 border border-white/5
          p-8 shadow-[0_40px_80px_rgba(0,0,0,0.6)]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-8">
          <div
            className="w-2.5 h-2.5 rounded-full"
            style={{ backgroundColor: color, boxShadow: `0 0 12px ${color}40` }}
          />
          <span className="text-[15px] font-semibold text-white/90">{name}</span>
          <div className="ml-auto px-2.5 py-1 rounded-full bg-white/[0.03] border border-white/5">
            <span className="text-[10px] font-bold text-white/30 uppercase tracking-[0.1em]">
              Security Check
            </span>
          </div>
        </div>

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
            
            <div className="flex gap-3">
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
                  className="w-full px-5 py-4 rounded-xl text-[13px] font-medium text-left
                    bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 text-white/70
                    transition-all cursor-pointer"
                  onClick={() => handleAnswer(option)}
                >
                  {option}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
