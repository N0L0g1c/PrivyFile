import { useCallback, useEffect, useState } from "react";
import { runCommand } from "@/hooks/useTauriCommand";
import type { AppSettings } from "@/lib/types";
import { DEFAULT_SETTINGS } from "@/lib/types";

const STORAGE_KEY = "privyfile-settings";

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    void (async () => {
      try {
        const remote = await runCommand<AppSettings>("load_settings");
        setSettings({ ...DEFAULT_SETTINGS, ...remote });
      } catch {
        const local = localStorage.getItem(STORAGE_KEY);
        if (local) {
          setSettings({ ...DEFAULT_SETTINGS, ...JSON.parse(local) });
        }
      } finally {
        setLoaded(true);
      }
    })();
  }, []);

  const saveSettings = useCallback(async (next: AppSettings) => {
    setSettings(next);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    try {
      await runCommand("save_settings", { settings: next });
    } catch {
      // Browser preview fallback
    }
  }, []);

  return { settings, saveSettings, loaded };
}
