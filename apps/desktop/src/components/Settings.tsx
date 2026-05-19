import { useState, useSyncExternalStore } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getSettings, updateSettings } from "../stores/settingsStore";

type SettingsTab = "general" | "plugins" | "terminals" | "notifications" | "about";

interface SettingRowProps {
  label: string;
  description?: string;
  children: React.ReactNode;
}

function SettingRow({ label, description, children }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between py-3 px-4 rounded-xl bg-white/[0.03] border border-white/[0.06]">
      <div className="space-y-0.5">
        <p className="text-[13px] font-medium text-white/80">{label}</p>
        {description && (
          <p className="text-[11px] text-white/40">{description}</p>
        )}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ enabled, onChange }: { enabled: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!enabled)}
      className={`relative w-10 h-6 rounded-full transition-colors ${
        enabled ? "bg-indigo-500" : "bg-white/10"
      }`}
    >
      <motion.div
        layout
        className="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white"
        animate={{ x: enabled ? 16 : 0 }}
        transition={{ type: "spring", stiffness: 500, damping: 30 }}
      />
    </button>
  );
}

export function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");

  const settings = useSyncExternalStore(
    (cb) => {
      window.addEventListener("storage", cb);
      return () => window.removeEventListener("storage", cb);
    },
    () => getSettings(),
  );

  const [autoStart, setAutoStart] = useState(false);
  const [terminalJump, setTerminalJump] = useState(true);
  const [analyticsEnabled, setAnalyticsEnabled] = useState(true);

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "general", label: "General" },
    { id: "plugins", label: "Plugins" },
    { id: "terminals", label: "Terminals" },
    { id: "notifications", label: "Notifications" },
    { id: "about", label: "About" },
  ];

  const pluginList = [
    { name: "opencode", desc: "OpenCode CLI", version: "0.1.0", active: true },
    { name: "claude", desc: "Claude Code", version: "0.1.0", active: true },
    { name: "codex", desc: "Codex CLI", version: "0.1.0", active: true },
    { name: "antigravity", desc: "Antigravity", version: "0.1.0", active: true },
    { name: "aider", desc: "Aider AI", version: "0.1.0", active: true },
    { name: "gemini", desc: "Gemini CLI", version: "0.1.0", active: true },
  ];

  const terminalList = [
    { name: "tmux", desc: "Terminal multiplexer", active: true },
    { name: "zellij", desc: "Terminal workspace", active: true },
    { name: "ghostty", desc: "GPU-accelerated terminal", active: false },
    { name: "wezterm", desc: "GPU-accelerated terminal", active: false },
    { name: "kitty", desc: "GPU-accelerated terminal", active: false },
  ];

  return (
    <div className="flex flex-col w-full h-full bg-[#0C0C0E] text-white">
      {/* Header */}
      <div className="flex items-center gap-3 px-5 py-4 border-b border-white/[0.06]">
        <div className="w-8 h-8 rounded-lg bg-indigo-500/20 flex items-center justify-center">
          <svg className="w-4 h-4 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </div>
        <div>
          <h1 className="text-[15px] font-bold text-white/90">Preferences</h1>
          <p className="text-[11px] text-white/40">Configure Orbitos Island behavior</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 px-4 py-3 border-b border-white/[0.06] overflow-x-auto">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-3 py-1.5 rounded-lg text-[12px] font-medium transition-all ${
              activeTab === tab.id
                ? "bg-white/10 text-white/90"
                : "text-white/40 hover:text-white/60 hover:bg-white/5"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-3">
        <AnimatePresence mode="wait">
          {activeTab === "general" && (
            <motion.div
              key="general"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="space-y-2"
            >
              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1">Startup</p>
              <SettingRow label="Launch at login" description="Automatically start Orbitos Island on login">
                <Toggle enabled={autoStart} onChange={setAutoStart} />
              </SettingRow>

              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1 mt-4">Behavior</p>
              <SettingRow label="Terminal jump" description="Jump to terminal pane on session click">
                <Toggle enabled={terminalJump} onChange={setTerminalJump} />
              </SettingRow>
              <SettingRow label="Collect analytics" description="Track token usage and session duration">
                <Toggle enabled={analyticsEnabled} onChange={setAnalyticsEnabled} />
              </SettingRow>

              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1 mt-4">Daemon</p>
              <SettingRow label="Socket path" description="Unix socket for IPC">
                <input
                  type="text"
                  defaultValue="~/.agentos/run/agentosd.sock"
                  className="w-44 px-2 py-1 rounded-lg bg-white/5 border border-white/10 text-[11px] font-mono text-white/60 text-right focus:outline-none focus:border-indigo-500/50"
                />
              </SettingRow>
            </motion.div>
          )}

          {activeTab === "plugins" && (
            <motion.div
              key="plugins"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="space-y-2"
            >
              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1">Registered Plugins</p>
              {pluginList.map((p) => (
                <div
                  key={p.name}
                  className="flex items-center justify-between py-2.5 px-4 rounded-xl bg-white/[0.03] border border-white/[0.06]"
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-2 h-2 rounded-full ${
                        p.active ? "bg-green-500" : "bg-white/20"
                      }`}
                    />
                    <div>
                      <p className="text-[13px] font-medium text-white/80">{p.name}</p>
                      <p className="text-[11px] text-white/40">{p.desc} · v{p.version}</p>
                    </div>
                  </div>
                  <Toggle enabled={p.active} onChange={() => {}} />
                </div>
              ))}
            </motion.div>
          )}

          {activeTab === "terminals" && (
            <motion.div
              key="terminals"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="space-y-2"
            >
              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1">Terminal Integrations</p>
              {terminalList.map((t) => (
                <div
                  key={t.name}
                  className="flex items-center justify-between py-2.5 px-4 rounded-xl bg-white/[0.03] border border-white/[0.06]"
                >
                  <div>
                    <p className="text-[13px] font-medium text-white/80">{t.name}</p>
                    <p className="text-[11px] text-white/40">{t.desc}</p>
                  </div>
                  <span
                    className={`text-[11px] font-medium ${
                      t.active ? "text-green-500" : "text-white/30"
                    }`}
                  >
                    {t.active ? "Available" : "Not detected"}
                  </span>
                </div>
              ))}
            </motion.div>
          )}

          {activeTab === "notifications" && (
            <motion.div
              key="notifications"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="space-y-2"
            >
              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1">Notifications</p>
              <SettingRow label="Enable notifications" description="Show desktop notifications">
                <Toggle enabled={settings.notificationsEnabled} onChange={(v) => updateSettings({ notificationsEnabled: v })} />
              </SettingRow>

              <p className="text-[11px] text-white/30 uppercase tracking-[0.1em] font-bold px-1 mt-4">Sound Alerts</p>
              <SettingRow label="Permission request" description="Urgent double-pulse tone when agent needs approval">
                <Toggle enabled={settings.sounds.permission_request} onChange={(v) => updateSettings({ sounds: { ...settings.sounds, permission_request: v } })} />
              </SettingRow>
              <SettingRow label="Task failed" description="Warning tone when a session errors out">
                <Toggle enabled={settings.sounds.task_error} onChange={(v) => updateSettings({ sounds: { ...settings.sounds, task_error: v } })} />
              </SettingRow>
              <SettingRow label="Task completed" description="Subtle chime when a session completes">
                <Toggle enabled={settings.sounds.task_completed} onChange={(v) => updateSettings({ sounds: { ...settings.sounds, task_completed: v } })} />
              </SettingRow>
            </motion.div>
          )}

          {activeTab === "about" && (
            <motion.div
              key="about"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="flex flex-col items-center py-8 space-y-4"
            >
              <div className="w-16 h-16 rounded-2xl bg-indigo-500/20 flex items-center justify-center">
                <span className="text-2xl font-bold text-indigo-400">A</span>
              </div>
              <div className="text-center">
                <h2 className="text-[15px] font-bold text-white/90">Orbitos Island</h2>
                <p className="text-[12px] text-white/40">v0.1.0</p>
              </div>
              <p className="text-[12px] text-white/40 text-center max-w-xs leading-relaxed">
                Linux-native cockpit for AI coding agents.
                Monitor, manage, and orchestrate your development agents.
              </p>
              <div className="flex gap-3 pt-4">
                <span className="px-3 py-1 rounded-lg bg-white/5 text-[11px] text-white/40 font-mono">
                  6 plugins
                </span>
                <span className="px-3 py-1 rounded-lg bg-white/5 text-[11px] text-white/40 font-mono">
                  5 terminals
                </span>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
