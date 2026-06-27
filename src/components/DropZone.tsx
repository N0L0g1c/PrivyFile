import { useCallback, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface DropZoneProps {
  onFilesAdded: (paths: string[]) => void;
}

export default function DropZone({ onFilesAdded }: DropZoneProps) {
  const [dragging, setDragging] = useState(false);

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      setDragging(false);
      const paths = Array.from(event.dataTransfer.files).map((file) => {
        const withPath = file as File & { path?: string };
        return withPath.path ?? file.name;
      });
      if (paths.length) onFilesAdded(paths);
    },
    [onFilesAdded],
  );

  const pickFiles = async () => {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });
      if (selected) {
        onFilesAdded(Array.isArray(selected) ? selected : [selected]);
      }
    } catch {
      // Fallback not available outside Tauri
    }
  };

  const pickFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected && typeof selected === "string") {
        onFilesAdded([selected]);
      }
    } catch {
      // Fallback not available outside Tauri
    }
  };

  return (
    <div
      onDragOver={(event) => {
        event.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={handleDrop}
      className={`glass rounded-2xl border-2 border-dashed p-10 text-center transition-colors card-hover ${
        dragging ? "border-emerald-400 bg-emerald-500/5" : "border-[var(--color-border)]"
      }`}
    >
      <div className="text-4xl mb-4">📂</div>
      <h2 className="text-xl font-semibold mb-2">Drop files or folders here</h2>
      <p className="text-[var(--color-muted)] mb-6 max-w-md mx-auto">
        JPEG, PNG, PDF, Office docs, MP4/MOV, and more. Metadata is read locally — nothing leaves your device.
      </p>
      <div className="flex flex-wrap justify-center gap-3">
        <button
          type="button"
          onClick={() => void pickFiles()}
          className="px-5 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-900 font-semibold transition-colors"
        >
          Choose files
        </button>
        <button
          type="button"
          onClick={() => void pickFolder()}
          className="px-5 py-2.5 rounded-xl border border-[var(--color-border)] hover:border-emerald-500/40 transition-colors"
        >
          Choose folder
        </button>
      </div>
    </div>
  );
}
