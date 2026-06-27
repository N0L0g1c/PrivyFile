import type { BatchProgress as BatchProgressType } from "@/lib/types";

interface BatchProgressProps {
  progress: BatchProgressType | null;
}

export default function BatchProgress({ progress }: BatchProgressProps) {
  if (!progress) return null;

  return (
    <div className="glass rounded-xl p-4 border border-[var(--color-border)] animate-fade-in">
      <div className="flex items-center justify-between mb-2 text-sm">
        <span>
          Processing {progress.current}/{progress.total}
        </span>
        <span className="text-emerald-400">{progress.percent}%</span>
      </div>
      <div className="h-2 rounded-full bg-slate-800 overflow-hidden mb-2">
        <div
          className="h-full bg-emerald-500 transition-all duration-300"
          style={{ width: `${progress.percent}%` }}
        />
      </div>
      <p className="text-xs text-[var(--color-muted)] truncate">{progress.file_path}</p>
      <p className="text-xs text-slate-400">{progress.status}</p>
    </div>
  );
}
