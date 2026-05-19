interface SoundSettings {
  permission_request: boolean;
  task_error: boolean;
  task_completed: boolean;
}

interface SettingsData {
  notificationsEnabled: boolean;
  sounds: SoundSettings;
}

const STORAGE_KEY = "agentos_settings";

function load(): SettingsData {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as SettingsData;
  } catch { /* ignore */ }
  return {
    notificationsEnabled: true,
    sounds: { permission_request: true, task_error: true, task_completed: true },
  };
}

let data: SettingsData = load();

export function getSettings(): SettingsData {
  return { ...data, sounds: { ...data.sounds } };
}

export function updateSettings(partial: Partial<SettingsData>) {
  data = { ...data, ...partial };
  if (partial.sounds) data.sounds = { ...data.sounds, ...partial.sounds };
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  } catch { /* ignore */ }
}

export function shouldPlaySound(kind: keyof SoundSettings): boolean {
  return data.notificationsEnabled && data.sounds[kind];
}
