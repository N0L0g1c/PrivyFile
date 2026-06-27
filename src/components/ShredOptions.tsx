import type { ShredMethod, ShredOptions } from "@/lib/types";

interface ShredOptionsProps {
  options: ShredOptions;
  onChange: (options: ShredOptions) => void;
  warning: string;
}

const METHODS: { value: ShredMethod; label: string }[] = [
  { value: "random1_pass", label: "Random (1 pass)" },
  { value: "dod5220", label: "DoD 5220.22-M (3 passes)" },
  { value: "seven_pass", label: "Secure wipe (7 passes)" },
  { value: "custom", label: "Custom passes" },
];

export default function ShredOptionsPanel({ options, onChange, warning }: ShredOptionsProps) {
  return (
    <div className="glass rounded-xl p-4 border border-amber-500/30 space-y-4">
      <div className="flex items-start gap-3">
        <span className="text-amber-400 text-lg">⚠️</span>
        <p className="text-sm text-amber-200/90">{warning}</p>
      </div>

      <div className="grid sm:grid-cols-2 gap-4">
        <label className="block text-sm">
          <span className="text-[var(--color-muted)] mb-1 block">Shred method</span>
          <select
            value={options.method}
            onChange={(event) =>
              onChange({ ...options, method: event.target.value as ShredMethod })
            }
            className="w-full rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] px-3 py-2"
          >
            {METHODS.map((method) => (
              <option key={method.value} value={method.value}>
                {method.label}
              </option>
            ))}
          </select>
        </label>

        {options.method === "custom" && (
          <label className="block text-sm">
            <span className="text-[var(--color-muted)] mb-1 block">Pass count</span>
            <input
              type="number"
              min={1}
              max={35}
              value={options.passes}
              onChange={(event) =>
                onChange({ ...options, passes: Number(event.target.value) || 1 })
              }
              className="w-full rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] px-3 py-2"
            />
          </label>
        )}
      </div>
    </div>
  );
}
