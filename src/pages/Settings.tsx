import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import ShredOptionsPanel from "@/components/ShredOptions";
import { runCommand } from "@/hooks/useTauriCommand";
import { useSettings } from "@/hooks/useSettings";
import type { CleanProfileId, ShredOptions } from "@/lib/types";

export default function SettingsPage() {
  const { settings, saveSettings, loaded } = useSettings();
  const [draft, setDraft] = useState(settings);
  const [shredWarning, setShredWarning] = useState("");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEffect(() => {
    void runCommand<string>("shred_warning").then(setShredWarning).catch(() => undefined);
  }, []);

  if (!loaded) return null;

  const pickOutputDir = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      setDraft({ ...draft, output_dir: selected });
    }
  };

  const pickWatchFolder = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      setDraft({ ...draft, watch_folder_path: selected });
    }
  };

  const save = async () => {
    await saveSettings(draft);
    if (draft.watch_folder_enabled && draft.watch_folder_path) {
      await runCommand("start_watch_folder", {
        folder: draft.watch_folder_path,
        options: {
          preserve_original: draft.preserve_original,
          shred_original: false,
          output_dir: draft.output_dir,
          profile: draft.default_profile,
        },
      });
    } else {
      await runCommand("stop_watch_folder").catch(() => undefined);
    }
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="max-w-3xl mx-auto px-4 py-8 space-y-6 animate-fade-in">
      <h1 className="text-2xl font-bold">Settings</h1>

      <div className="glass rounded-2xl p-5 border border-[var(--color-border)] space-y-5">
        <div>
          <label className="text-sm text-[var(--color-muted)] block mb-2">Output folder</label>
          <div className="flex gap-2">
            <input
              readOnly
              value={draft.output_dir ?? "Same folder as source file"}
              className="flex-1 rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] px-3 py-2 text-sm"
            />
            <button
              type="button"
              onClick={() => void pickOutputDir()}
              className="px-4 py-2 rounded-lg border border-[var(--color-border)] hover:border-emerald-500/40"
            >
              Browse
            </button>
          </div>
        </div>

        <label className="flex items-center gap-3 text-sm">
          <input
            type="checkbox"
            checked={draft.preserve_original}
            onChange={(event) =>
              setDraft({ ...draft, preserve_original: event.target.checked })
            }
          />
          Keep original files (non-destructive by default)
        </label>

        <label className="flex items-center gap-3 text-sm">
          <input
            type="checkbox"
            checked={draft.enable_history}
            onChange={(event) =>
              setDraft({ ...draft, enable_history: event.target.checked })
            }
          />
          Enable local cleaned-files history
        </label>

        <label className="block text-sm">
          <span className="text-[var(--color-muted)] mb-1 block">Default cleaning profile</span>
          <select
            value={draft.default_profile}
            onChange={(event) =>
              setDraft({ ...draft, default_profile: event.target.value as CleanProfileId })
            }
            className="w-full rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] px-3 py-2"
          >
            <option value="remove_all">Remove all metadata</option>
            <option value="social_media_share">Social media share</option>
            <option value="legal_document">Legal document</option>
            <option value="photo_backup">Photo backup</option>
          </select>
        </label>

        <ShredOptionsPanel
          options={{
            method: draft.default_shred_method,
            passes: draft.default_shred_passes,
          }}
          onChange={(options: ShredOptions) =>
            setDraft({
              ...draft,
              default_shred_method: options.method,
              default_shred_passes: options.passes,
            })
          }
          warning={shredWarning}
        />

        <div className="border-t border-[var(--color-border)] pt-4 space-y-3">
          <label className="flex items-center gap-3 text-sm">
            <input
              type="checkbox"
              checked={draft.watch_folder_enabled}
              onChange={(event) =>
                setDraft({ ...draft, watch_folder_enabled: event.target.checked })
              }
            />
            Watch folder — auto-clean new files
          </label>
          {draft.watch_folder_enabled && (
            <div className="flex gap-2">
              <input
                readOnly
                value={draft.watch_folder_path ?? "No folder selected"}
                className="flex-1 rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] px-3 py-2 text-sm"
              />
              <button
                type="button"
                onClick={() => void pickWatchFolder()}
                className="px-4 py-2 rounded-lg border border-[var(--color-border)] hover:border-emerald-500/40"
              >
                Browse
              </button>
            </div>
          )}
        </div>

        <button
          type="button"
          onClick={() => void save()}
          className="px-5 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-900 font-semibold"
        >
          {saved ? "Saved!" : "Save settings"}
        </button>
      </div>
    </div>
  );
}
