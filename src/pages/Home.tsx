import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import DropZone from "@/components/DropZone";
import BatchProgress from "@/components/BatchProgress";
import MetadataTable from "@/components/MetadataTable";
import PrivacyScore from "@/components/PrivacyScore";
import ShredOptionsPanel from "@/components/ShredOptions";
import { runCommand } from "@/hooks/useTauriCommand";
import { useSettings } from "@/hooks/useSettings";
import type {
  BatchItem,
  BatchProgress as BatchProgressType,
  BatchResult,
  CleanOptions,
  CleanProfileId,
  MetadataReport,
  ProfileInfo,
  QueuedFile,
  ShredOptions,
} from "@/lib/types";

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export default function Home() {
  const { settings } = useSettings();
  const [queue, setQueue] = useState<QueuedFile[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [progress, setProgress] = useState<BatchProgressType | null>(null);
  const [shredOptions, setShredOptions] = useState<ShredOptions>({
    method: settings.default_shred_method,
    passes: settings.default_shred_passes,
  });
  const [shredWarning, setShredWarning] = useState("");
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<CleanProfileId>(
    settings.default_profile,
  );
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void runCommand<string>("shred_warning").then(setShredWarning).catch(() =>
      setShredWarning(
        "Secure deletion cannot be guaranteed on SSDs or flash storage due to wear leveling and TRIM.",
      ),
    );
    void runCommand<ProfileInfo[]>("list_profiles").then(setProfiles).catch(() => undefined);
  }, []);

  useEffect(() => {
    const unlisten = listen<BatchProgressType>("batch-progress", (event) => {
      setProgress(event.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const addFiles = useCallback((paths: string[]) => {
    setQueue((current) => [
      ...current,
      ...paths.map((path) => ({
        id: `${path}-${Date.now()}-${Math.random()}`,
        path,
        name: fileName(path),
        status: "pending" as const,
      })),
    ]);
  }, []);

  const loadMetadata = async (file: QueuedFile) => {
    setQueue((current) =>
      current.map((item) =>
        item.id === file.id ? { ...item, status: "processing", message: "Reading metadata" } : item,
      ),
    );
    try {
      const metadata = await runCommand<MetadataReport>("get_metadata", { path: file.path });
      setQueue((current) =>
        current.map((item) =>
          item.id === file.id ? { ...item, metadata, status: "done", message: undefined } : item,
        ),
      );
      setSelectedId(file.id);
    } catch (error) {
      setQueue((current) =>
        current.map((item) =>
          item.id === file.id
            ? { ...item, status: "error", message: String(error) }
            : item,
        ),
      );
    }
  };

  const runBatch = async (action: BatchItem["action"]) => {
    if (queue.length === 0) return;
    setBusy(true);
    setProgress(null);

    const items: BatchItem[] = queue.map((file) => ({ path: file.path, action }));
    const cleanOptions: CleanOptions = {
      preserve_original: settings.preserve_original,
      shred_original: action === "clean_and_shred",
      output_dir: settings.output_dir,
      profile: selectedProfile,
    };

    try {
      const result = await runCommand<BatchResult>("process_batch", {
        items,
        cleanOptions,
        shredOptions,
      });
      setQueue((current) =>
        current.map((file, index) => ({
          ...file,
          status: result.items[index]?.success ? "done" : "error",
          message: result.items[index]?.message,
        })),
      );
      if (result.report_path) {
        await runCommand("open_report", { path: result.report_path });
      }
    } catch (error) {
      alert(String(error));
    } finally {
      setBusy(false);
      setProgress(null);
    }
  };

  const selected = queue.find((file) => file.id === selectedId);

  return (
    <div className="max-w-6xl mx-auto px-4 py-8 space-y-6 animate-fade-in">
      <section className="text-center mb-2">
        <h1 className="text-3xl sm:text-4xl font-bold gradient-text mb-2">
          Sanitize files before you share or delete them
        </h1>
        <p className="text-[var(--color-muted)]">
          Remove metadata and securely shred sensitive files — 100% local processing
        </p>
      </section>

      <DropZone onFilesAdded={addFiles} />

      {profiles.length > 0 && (
        <div className="glass rounded-xl p-4 border border-[var(--color-border)]">
          <label className="text-sm text-[var(--color-muted)] block mb-2">Cleaning profile</label>
          <select
            value={selectedProfile}
            onChange={(event) => setSelectedProfile(event.target.value as CleanProfileId)}
            className="w-full rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] px-3 py-2"
          >
            {profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.label} — {profile.description}
              </option>
            ))}
          </select>
        </div>
      )}

      <ShredOptionsPanel
        options={shredOptions}
        onChange={setShredOptions}
        warning={shredWarning}
      />

      {queue.length > 0 && (
        <div className="glass rounded-2xl p-4 border border-[var(--color-border)] space-y-4">
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              disabled={busy}
              onClick={() => void runBatch("clean")}
              className="px-4 py-2 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-slate-900 font-semibold disabled:opacity-50"
            >
              Clean all
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                if (confirm("Permanently shred selected files? This cannot be undone.")) {
                  void runBatch("shred");
                }
              }}
              className="px-4 py-2 rounded-lg border border-red-500/40 text-red-300 hover:bg-red-500/10 disabled:opacity-50"
            >
              Shred all
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                if (
                  confirm(
                    "Clean copies will be saved, then originals will be shredded. Continue?",
                  )
                ) {
                  void runBatch("clean_and_shred");
                }
              }}
              className="px-4 py-2 rounded-lg border border-amber-500/40 text-amber-200 hover:bg-amber-500/10 disabled:opacity-50"
            >
              Clean + shred originals
            </button>
          </div>

          <BatchProgress progress={progress} />

          <ul className="divide-y divide-[var(--color-border)]/60">
            {queue.map((file) => (
              <li key={file.id} className="py-3 flex flex-wrap items-center gap-3 justify-between">
                <button
                  type="button"
                  onClick={() => setSelectedId(file.id)}
                  className={`text-left flex-1 min-w-0 ${selectedId === file.id ? "text-emerald-400" : ""}`}
                >
                  <span className="font-medium truncate block">{file.name}</span>
                  <span className="text-xs text-[var(--color-muted)] truncate block">{file.path}</span>
                </button>
                {file.metadata && <PrivacyScore score={file.metadata.privacy_score} size="sm" />}
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={() => void loadMetadata(file)}
                    className="text-xs px-3 py-1.5 rounded-lg border border-[var(--color-border)] hover:border-emerald-500/40"
                  >
                    Preview
                  </button>
                </div>
                {file.message && (
                  <span className="text-xs text-[var(--color-muted)] w-full">{file.message}</span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {selected?.metadata && (
        <div className="glass rounded-2xl p-4 border border-[var(--color-border)] space-y-3">
          <div className="flex items-center justify-between gap-4 flex-wrap">
            <h3 className="font-semibold">Metadata preview — {selected.name}</h3>
            <PrivacyScore score={selected.metadata.privacy_score} />
          </div>
          <MetadataTable tags={selected.metadata.tags} />
        </div>
      )}
    </div>
  );
}
