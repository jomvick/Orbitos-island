/**
 * AgentOS OpenCode Plugin
 *
 * Pipes OpenCode lifecycle events to the agentos-hook CLI.
 * Install: copy to ~/.config/opencode/plugins/agentos.js
 *          and add to ~/.config/opencode/config.json "plugin" array.
 *
 * Communication flow:
 *   OpenCode SDK → this plugin → agentos-hook (stdio) → agentosd (Unix socket)
 */

const { execSync, spawn } = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");

const HOOK_BINARY = process.env.AGENTOS_HOOK || "agentos-hook";
const SOCKET_PATH = "/tmp/agentosd.sock";

let currentSessionId = null;
let activeRequestIds = new Set();

function generateId() {
  return (
    Math.random().toString(36).substring(2, 15) +
    Math.random().toString(36).substring(2, 15)
  );
}

function getCwd() {
  try {
    return process.cwd();
  } catch {
    return "";
  }
}

function getBranch() {
  try {
    const branch = execSync("git rev-parse --abbrev-ref HEAD", {
      encoding: "utf8",
      timeout: 2000,
      stdio: ["pipe", "pipe", "ignore"],
    }).trim();
    return branch || undefined;
  } catch {
    return undefined;
  }
}

function getTerminal() {
  if (process.env.TMUX) return "tmux";
  if (process.env.ZELLIJ) return "zellij";
  if (process.env.TERM_PROGRAM) return process.env.TERM_PROGRAM;
  return undefined;
}

function sendToHook(type, extra = {}) {
  const payload = JSON.stringify({
    type,
    session_id: currentSessionId || generateId(),
    cwd: getCwd(),
    branch: getBranch(),
    terminal: getTerminal(),
    timestamp: new Date().toISOString(),
    ...extra,
  });

  try {
    const result = execSync(`${HOOK_BINARY} --source opencode`, {
      input: payload,
      encoding: "utf8",
      timeout: 2000,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return result.trim();
  } catch (e) {
    // fail-open: agent should never be blocked
    return "";
  }
}

function handleBlockingEvent(eventName, payload, timeoutMs = 30000) {
  const payloadStr = JSON.stringify(payload);
  try {
    const result = execSync(`${HOOK_BINARY} --source opencode`, {
      input: payloadStr,
      encoding: "utf8",
      timeout: timeoutMs,
      stdio: ["pipe", "pipe", "pipe"],
    });
    const trimmed = result.trim();
    if (trimmed) {
      try {
        return JSON.parse(trimmed);
      } catch {
        return { type: trimmed };
      }
    }
  } catch {
    // timeout or error — default to deny/reject
  }
  return { type: "deny", reason: "hook timeout" };
}

const plugin = {
  name: "agentos",
  description: "AgentOS integration plugin — forwards events to agentosd",
  version: "0.1.0",

  async initialize(config) {
    if (config?.hookBinary) {
      process.env.AGENTOS_HOOK = config.hookBinary;
    }
  },

  async onEvent(event) {
    if (!event || !event.type) return;

    switch (event.type) {
      case "session.created": {
        currentSessionId = event.session?.id || generateId();
        sendToHook("session_start", {
          session_id: currentSessionId,
          model: event.session?.model,
        });
        break;
      }

      case "session.status": {
        if (event.status === "deleted" && currentSessionId) {
          sendToHook("session_complete", {
            session_id: currentSessionId,
            duration_ms: event.session?.elapsedTime,
          });
          currentSessionId = null;
        }
        break;
      }

      case "message.part.updated": {
        if (event.message?.role === "assistant") {
          sendToHook("activity", {
            session_id: currentSessionId,
            metadata: {
              tokens: event.message?.usage?.tokens,
              model: event.message?.model,
            },
          });
        }
        break;
      }

      case "tool.use": {
        sendToHook("activity", {
          session_id: currentSessionId,
          metadata: {
            tool: event.tool?.name,
            input: event.tool?.input,
          },
        });
        break;
      }

      case "permission.asked": {
        const directive = handleBlockingEvent("permission_request", {
          type: "permission_request",
          session_id: currentSessionId,
          permission: {
            id: event.permission?.id,
            command: event.permission?.command,
            description:
              event.permission?.description || event.permission?.title,
          },
        });

        if (directive.type === "allow" && event.permission?.id) {
          try {
            await fetch(
              `http://localhost:17900/permission/${event.permission.id}/reply`,
              {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ reply: "once" }),
              }
            );
          } catch {}
        }
        break;
      }

      case "question.asked": {
        const directive = handleBlockingEvent("question", {
          type: "question",
          session_id: currentSessionId,
          question: {
            id: event.question?.id,
            text: event.question?.text,
            options: event.question?.options?.map((o) => o.label || o),
          },
        });

        if (directive.answer && event.question?.id) {
          try {
            await fetch(
              `http://localhost:17900/question/${event.question.id}/reply`,
              {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ answers: [[directive.answer]] }),
              }
            );
          } catch {}
        }
        break;
      }

      case "error": {
        sendToHook("session_failed", {
          session_id: currentSessionId,
          error: event.error?.message || "Unknown error",
        });
        break;
      }

      case "heartbeat": {
        sendToHook("heartbeat", {
          session_id: currentSessionId,
        });
        break;
      }
    }
  },

  async shutdown() {
    if (currentSessionId) {
      sendToHook("session_complete", {
        session_id: currentSessionId,
        metadata: { reason: "plugin_shutdown" },
      });
    }
  },
};

module.exports = plugin;
