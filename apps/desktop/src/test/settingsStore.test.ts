import { describe, it, expect, beforeEach } from "vitest";
import { getSettings, updateSettings, shouldPlaySound } from "../stores/settingsStore";

function reset() {
  updateSettings({
    notificationsEnabled: true,
    sounds: { permission_request: true, task_error: true, task_completed: true },
  });
  localStorage.setItem(
    "agentos_settings",
    JSON.stringify({
      notificationsEnabled: true,
      sounds: { permission_request: true, task_error: true, task_completed: true },
    })
  );
}

describe("settingsStore", () => {
  beforeEach(() => {
    reset();
  });

  it("returns default settings when nothing is stored", () => {
    localStorage.clear();
    updateSettings({
      notificationsEnabled: true,
      sounds: { permission_request: true, task_error: true, task_completed: true },
    });
    const settings = getSettings();
    expect(settings.notificationsEnabled).toBe(true);
    expect(settings.sounds.permission_request).toBe(true);
    expect(settings.sounds.task_error).toBe(true);
    expect(settings.sounds.task_completed).toBe(true);
  });

  it("updates a single field", () => {
    updateSettings({ notificationsEnabled: false });
    expect(getSettings().notificationsEnabled).toBe(false);
  });

  it("updates sound settings", () => {
    updateSettings({ sounds: { permission_request: false, task_error: true, task_completed: true } });
    expect(getSettings().sounds.permission_request).toBe(false);
  });

  it("persists settings to localStorage", () => {
    updateSettings({ notificationsEnabled: false });
    const raw = localStorage.getItem("agentos_settings");
    expect(raw).toBeTruthy();
    expect(JSON.parse(raw!).notificationsEnabled).toBe(false);
  });

  it("shouldPlaySound returns false when notifications disabled", () => {
    updateSettings({ notificationsEnabled: false });
    expect(shouldPlaySound("permission_request")).toBe(false);
  });

  it("shouldPlaySound returns false when sound type disabled", () => {
    updateSettings({ sounds: { permission_request: false, task_error: true, task_completed: true } });
    expect(shouldPlaySound("permission_request")).toBe(false);
  });

  it("shouldPlaySound returns true when both enabled", () => {
    expect(shouldPlaySound("task_error")).toBe(true);
  });
});
