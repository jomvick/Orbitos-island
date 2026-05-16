export class NotificationThrottle {
  private lastNotified = new Map<string, number>();
  private readonly cooldownMs: number;

  constructor(cooldownMs = 5000) {
    this.cooldownMs = cooldownMs;
  }

  shouldNotify(key: string): boolean {
    const now = Date.now();
    const last = this.lastNotified.get(key) ?? 0;
    if (now - last >= this.cooldownMs) {
      this.lastNotified.set(key, now);
      return true;
    }
    return false;
  }

  clear(key: string): void {
    this.lastNotified.delete(key);
  }

  reset(): void {
    this.lastNotified.clear();
  }
}
