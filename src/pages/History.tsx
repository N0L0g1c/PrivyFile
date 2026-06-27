import { useEffect, useState } from "react";
import PrivacyScore from "@/components/PrivacyScore";
import { runCommand } from "@/hooks/useTauriCommand";
import type { HistoryEntry } from "@/lib/types";

export default function HistoryPage() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);

  const refresh = async () => {
    try {
      const data = await runCommand<HistoryEntry[]>("load_history");
      setEntries(data.reverse());
    } catch {
      setEntries([]);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const clear = async () => {
    if (!confirm("Clear all local history?")) return;
    await runCommand("clear_history");
    setEntries([]);
  };

  return (
    <div className="max-w-4xl mx-auto px-4 py-8 space-y-6 animate-fade-in">
      <div className="flex items-center justify-between gap-4 flex-wrap">
        <h1 className="text-2xl font-bold">My Cleaned Files</h1>
        {entries.length > 0 && (
          <button
            type="button"
            onClick={() => void clear()}
            className="px-4 py-2 rounded-lg border border-[var(--color-border)] hover:border-red-500/40 text-red-300"
          >
            Clear history
          </button>
        )}
      </div>

      {entries.length === 0 ? (
        <p className="text-[var(--color-muted)]">No cleaned files recorded yet.</p>
      ) : (
        <ul className="space-y-3">
          {entries.map((entry) => (
            <li
              key={`${entry.timestamp}-${entry.source_path}`}
              className="glass rounded-xl p-4 border border-[var(--color-border)]"
            >
              <div className="flex flex-wrap items-center justify-between gap-3 mb-2">
                <span className="font-medium">{entry.source_path.split(/[\\/]/).pop()}</span>
                <span className="text-xs text-[var(--color-muted)]">{entry.timestamp}</span>
              </div>
              <p className="text-xs text-[var(--color-muted)] mb-2">{entry.source_path}</p>
              {entry.output_path && (
                <p className="text-xs text-emerald-400/80 mb-2">→ {entry.output_path}</p>
              )}
              <div className="flex items-center gap-4 text-sm">
                <PrivacyScore score={entry.privacy_score_before} label="Before" size="sm" />
                <span className="text-[var(--color-muted)]">→</span>
                <PrivacyScore score={entry.privacy_score_after} label="After" size="sm" />
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
