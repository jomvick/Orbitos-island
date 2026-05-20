import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";

interface AgentInfo {
  name: string;
  binary: string | null;
  installed: boolean;
  hooks_supported: boolean;
  hooks_installed: boolean;
  config_path: string | null;
  message: string;
}

interface DiscoverResponse {
  data: {
    agents: AgentInfo[];
    total_agents: number;
    installed_count: number;
    hooks_installed_count: number;
  };
}

export function DiscoveryWizard() {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const scan = async () => {
    setLoading(true);
    setError(null);
    try {
      const result: DiscoverResponse = await invoke("discover_agents");
      setAgents(result?.data?.agents ?? []);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    scan();
  }, []);

  const installed = agents.filter((a) => a.installed).length;
  const withHooks = agents.filter((a) => a.hooks_installed).length;

  return (
    <div className="space-y-3">
      {/* Summary bar */}
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-4 text-[10px] text-white/30">
          <span>
            {installed}/{agents.length} installed
          </span>
          <span>{withHooks} hooks configured</span>
        </div>
        <button
          onClick={scan}
          disabled={loading}
          className="px-3 py-1 rounded-lg bg-white/5 hover:bg-white/10
            text-[11px] font-semibold text-white/60 hover:text-white/80
            transition-all border border-white/[0.06] disabled:opacity-30"
        >
          {loading ? "Scanning..." : "Scan"}
        </button>
      </div>

      {error && (
        <div className="px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-[11px] text-red-400">
          {error}
        </div>
      )}

      {/* Agent list */}
      <div className="space-y-1">
        {agents.map((agent) => (
          <motion.div
            key={agent.name}
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center justify-between py-2.5 px-4 rounded-xl bg-white/[0.03] border border-white/[0.06]"
          >
            <div className="flex items-center gap-3">
              <div
                className={`w-2 h-2 rounded-full ${
                  agent.hooks_installed
                    ? "bg-green-500"
                    : agent.installed
                      ? "bg-amber-500"
                      : "bg-white/20"
                }`}
              />
              <div>
                <p className="text-[13px] font-medium text-white/80 capitalize">
                  {agent.name}
                </p>
                <p className="text-[11px] text-white/40">{agent.message}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              {agent.hooks_supported && !agent.hooks_installed && agent.installed && (
                <button
                  onClick={scan}
                  className="px-2 py-1 rounded-md bg-indigo-500/20 hover:bg-indigo-500/30
                    text-[10px] font-semibold text-indigo-400 transition-all"
                >
                  Install Hooks
                </button>
              )}
              <span
                className={`text-[11px] font-medium ${
                  agent.hooks_installed
                    ? "text-green-500"
                    : agent.installed
                      ? "text-amber-500/70"
                      : "text-white/20"
                }`}
              >
                {agent.hooks_installed
                  ? "Active"
                  : agent.installed
                    ? "Detected"
                    : "Not found"}
              </span>
            </div>
          </motion.div>
        ))}

        {agents.length === 0 && !loading && !error && (
          <div className="flex flex-col items-center py-10 opacity-20">
            <p className="text-xs font-medium">No agents detected</p>
          </div>
        )}
      </div>
    </div>
  );
}