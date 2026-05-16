import { useState, useMemo, lazy, Suspense } from "react";
import { useSessionStore } from "../stores/sessionStore";
import { AgentPill } from "./AgentPill";
import { SearchBar } from "./SearchBar";
import { groupSessions, type GroupBy } from "../utils/sessionGrouping";
import { motion, AnimatePresence } from "framer-motion";
import { PlanDiffView } from "./PlanDiffView";

const Timeline = lazy(() => import("./Timeline").then(m => ({ default: m.Timeline })));
const AnalyticsPanel = lazy(() => import("./AnalyticsPanel").then(m => ({ default: m.AnalyticsPanel })));
const ActivityGraph = lazy(() => import("./ActivityGraph").then(m => ({ default: m.ActivityGraph })));

type Tab = "sessions" | "timeline" | "analytics" | "graphs" | "plans";

const tabVariants = {
  enter: (dir: number) => ({
    x: dir > 0 ? 120 : dir < 0 ? -120 : 0,
    opacity: 0,
    filter: "blur(6px)",
  }),
  center: {
    x: 0,
    opacity: 1,
    filter: "blur(0px)",
  },
  exit: (dir: number) => ({
    x: dir < 0 ? 120 : dir > 0 ? -120 : 0,
    opacity: 0,
    filter: "blur(6px)",
  }),
};

const TAB_LABELS: [Tab, string][] = [
  ["sessions", "Sessions"],
  ["timeline", "Timeline"],
  ["analytics", "Analytics"],
  ["graphs", "Graphs"],
  ["plans", "Plans"],
];

function TabPanel({ tab, tabDir, children }: { tab: Tab; tabDir: number; children: React.ReactNode }) {
  return (
    <motion.div
      key={tab}
      custom={tabDir}
      variants={tabVariants}
      initial="enter"
      animate="center"
      exit="exit"
      transition={{ duration: 0.18, ease: [0.4, 0, 0.2, 1] }}
    >
      {children}
    </motion.div>
  );
}

function TabFallback() {
  return (
    <div className="flex items-center justify-center py-20">
      <div className="w-5 h-5 rounded-full border-2 border-white/20 border-t-white/60 animate-spin" />
    </div>
  );
}

export function Dashboard({ embedded }: { embedded?: boolean }) {
  const sessions = useSessionStore((s) => s.sessions);
  const setExpanded = useSessionStore((s) => s.setExpanded);
  const [tab, setTab] = useState<Tab>("sessions");
  const [tabDir, setTabDir] = useState(0);
  const [searchQuery, setSearchQuery] = useState("");
  const [groupBy, setGroupBy] = useState<GroupBy>("none");

  const switchTab = (id: Tab) => {
    const idx = TAB_LABELS.findIndex(([t]) => t === tab);
    const nextIdx = TAB_LABELS.findIndex(([t]) => t === id);
    setTabDir(nextIdx > idx ? 1 : -1);
    setTab(id);
  };

  const allSessions = useMemo(
    () =>
      Array.from(sessions.values()).sort(
        (a, b) =>
          new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
      ),
    [sessions]
  );

  const filteredSessions = useMemo(() => {
    if (!searchQuery.trim()) return allSessions;
    const q = searchQuery.toLowerCase();
    return allSessions.filter(
      (s) =>
        s.agent.toLowerCase().includes(q) ||
        s.id.toLowerCase().includes(q) ||
        s.cwd?.toLowerCase().includes(q) ||
        s.branch?.toLowerCase().includes(q) ||
        s.model?.toLowerCase().includes(q)
    );
  }, [allSessions, searchQuery]);

  const groupedData = useMemo(() => {
    if (tab !== "sessions") return null;
    if (groupBy === "none")
      return [{ key: "all", label: "", sessions: filteredSessions, count: filteredSessions.length }];
    return groupSessions(filteredSessions, groupBy);
  }, [filteredSessions, groupBy, tab]);

  const content = (
    <>
      <div
        className="flex items-center justify-between px-6 py-4
          border-b border-white/5 shrink-0"
      >
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="w-2.5 h-2.5 rounded-full bg-accent-blue shadow-[0_0_10px_rgba(59,130,246,0.5)]" />
            <h2 className="text-[11px] font-bold text-white/50 tracking-[0.2em] uppercase font-mono">
              AgentOS / Cockpit
            </h2>
          </div>
          
          <nav className="flex gap-1 bg-white/[0.03] p-1 rounded-xl border border-white/5">
            {TAB_LABELS.map(([id, label]) => (
              <button
                key={id}
                onClick={() => switchTab(id)}
                className={`text-[11px] px-3 py-1.5 rounded-lg transition-all duration-200 font-medium ${
                  tab === id
                    ? "bg-white/10 text-white shadow-sm"
                    : "text-white/30 hover:text-white/60 hover:bg-white/[0.02]"
                }`}
              >
                {label}
              </button>
            ))}
          </nav>
        </div>
        
        <button
          onClick={() => setExpanded(false)}
          className="w-8 h-8 flex items-center justify-center rounded-full
            bg-white/5 hover:bg-white/10 text-white/40 hover:text-white
            transition-all duration-200"
        >
          {"\u2715"}
        </button>
      </div>

      <div className="px-6 py-4 shrink-0 bg-white/[0.01]">
        <SearchBar value={searchQuery} onChange={setSearchQuery} />
      </div>

      <div className="flex-1 overflow-y-auto px-6 pb-6 custom-scrollbar">
        <Suspense fallback={<TabFallback />}>
          <AnimatePresence mode="wait" custom={tabDir}>
            {tab === "timeline" && (
              <TabPanel tab={tab} tabDir={tabDir}>
                <Timeline sessions={filteredSessions} />
              </TabPanel>
            )}
            {tab === "analytics" && (
              <TabPanel tab={tab} tabDir={tabDir}>
                <AnalyticsPanel sessions={filteredSessions} />
              </TabPanel>
            )}
            {tab === "graphs" && (
              <TabPanel tab={tab} tabDir={tabDir}>
                <ActivityGraph sessions={filteredSessions} />
              </TabPanel>
            )}
            {tab === "plans" && (
              <TabPanel tab={tab} tabDir={tabDir}>
                <PlanDiffView sessions={filteredSessions} />
              </TabPanel>
            )}
            {tab === "sessions" && (
              <TabPanel tab={tab} tabDir={tabDir}>
                <div className="space-y-6">
                  <div className="flex items-center gap-2 pb-2 border-b border-white/[0.02]">
                    {(
                      [
                        ["none", "All"],
                        ["agent", "Agent"],
                        ["project", "Project"],
                        ["state", "State"],
                      ] as [GroupBy, string][]
                    ).map(([key, label]) => (
                      <button
                        key={key}
                        onClick={() => setGroupBy(key)}
                        className={`text-[10px] px-2.5 py-1 rounded-md transition-all font-bold uppercase tracking-wider ${
                          groupBy === key
                            ? "text-accent-blue bg-accent-blue/10"
                            : "text-white/20 hover:text-white/40"
                        }`}
                      >
                        {label}
                      </button>
                    ))}
                  </div>

                  <AnimatePresence mode="popLayout">
                    {groupedData?.map((group) => {
                      const active = group.sessions.filter((s) =>
                        ["running", "waiting_permission", "waiting_question"].includes(s.phase)
                      );
                      const completed = group.sessions.filter((s) =>
                        ["completed", "failed"].includes(s.phase)
                      );

                      return (
                        <motion.div 
                          key={group.key}
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          layout
                        >
                          {groupBy !== "none" && (
                            <h3 className="text-[10px] font-bold text-white/20 uppercase tracking-[0.2em] mb-3 px-1">
                              {group.label} <span className="ml-1 text-white/10">({group.count})</span>
                            </h3>
                          )}
                          
                          <div className="space-y-1">
                            {active.map((session) => (
                              <DashboardRow key={session.id} session={session} />
                            ))}
                            {completed.slice(0, 15).map((session) => (
                              <DashboardRow key={session.id} session={session} compact />
                            ))}
                          </div>
                        </motion.div>
                      );
                    })}
                  </AnimatePresence>

                  {filteredSessions.length === 0 && (
                    <motion.div 
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      className="flex flex-col items-center justify-center py-20 opacity-20"
                    >
                      <div className="w-12 h-12 rounded-full border-2 border-dashed border-white mb-4" />
                      <p className="text-sm font-medium">
                        {searchQuery ? "No matches found" : "Waiting for agents..."}
                      </p>
                    </motion.div>
                  )}
                </div>
              </TabPanel>
            )}
          </AnimatePresence>
        </Suspense>
      </div>
    </>
  );

  if (embedded) {
    return (
      <motion.div
        initial={{ height: 0, opacity: 0 }}
        animate={{ height: "auto", opacity: 1 }}
        exit={{ height: 0, opacity: 0 }}
        transition={{ duration: 0.25, ease: "easeInOut" }}
        className="overflow-hidden border-t border-white/5"
      >
        <div className="flex flex-col max-h-[75vh] overflow-hidden">
          {content}
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ scale: 0.95, opacity: 0, y: -20 }}
      animate={{ scale: 1, opacity: 1, y: 0 }}
      exit={{ scale: 0.95, opacity: 0, y: -20 }}
      className="fixed top-6 left-1/2 -translate-x-1/2 z-[150]
        w-[720px] max-h-[85vh] overflow-hidden
        rounded-[28px] glass-card-heavy
        flex flex-col font-sans shadow-2xl"
    >
      {content}
    </motion.div>
  );
}

interface DashboardRowProps {
  session: {
    id: string;
    agent: string;
    phase: string;
    tokens_input: number;
    tokens_output: number;
    model?: string;
    cwd?: string;
    branch?: string;
    duration_ms: number;
  };
  compact?: boolean;
}

function DashboardRow({ session, compact }: DashboardRowProps) {
  const duration = session.duration_ms
    ? `${(session.duration_ms / 1000).toFixed(1)}s`
    : "--";

  return (
    <div
      className="flex items-center gap-3 px-3 py-2 rounded-lg
        hover:bg-white/[0.04] transition-colors cursor-pointer group"
    >
      <AgentPill session={session as any} compact />
      <div className="flex-1 min-w-0">
        {!compact && session.cwd && (
          <p className="text-[11px] text-text-dim truncate font-mono">{session.cwd}</p>
        )}
      </div>
      <div className="flex items-center gap-3 text-[11px] text-text-dim font-mono">
        <span>{session.tokens_input + session.tokens_output} tok</span>
        <span>{duration}</span>
      </div>
    </div>
  );
}
