import { useMemo, useState } from "react";
import type { TagEntry } from "@/lib/types";
import { CATEGORY_LABELS } from "@/lib/types";

interface MetadataTableProps {
  tags: TagEntry[];
  removedOnly?: boolean;
  removedNames?: Set<string>;
}

type SortKey = "name" | "value" | "category";

export default function MetadataTable({
  tags,
  removedOnly = false,
  removedNames,
}: MetadataTableProps) {
  const [sortKey, setSortKey] = useState<SortKey>("category");
  const [ascending, setAscending] = useState(true);

  const rows = useMemo(() => {
    let filtered = tags;
    if (removedOnly && removedNames) {
      filtered = tags.filter((tag) => removedNames.has(tag.name));
    }
    return [...filtered].sort((a, b) => {
      const left = a[sortKey];
      const right = b[sortKey];
      const cmp = left.localeCompare(right);
      return ascending ? cmp : -cmp;
    });
  }, [tags, removedOnly, removedNames, sortKey, ascending]);

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setAscending(!ascending);
    } else {
      setSortKey(key);
      setAscending(true);
    }
  };

  if (rows.length === 0) {
    return (
      <p className="text-sm text-[var(--color-muted)] py-4 text-center">
        No metadata tags found.
      </p>
    );
  }

  return (
    <div className="overflow-x-auto rounded-xl border border-[var(--color-border)]">
      <table className="w-full text-sm">
        <thead className="bg-[var(--color-surface-raised)] text-left">
          <tr>
            {(["name", "value", "category"] as SortKey[]).map((key) => (
              <th key={key} className="px-4 py-3 font-medium text-emerald-400/90">
                <button type="button" onClick={() => toggleSort(key)} className="hover:text-emerald-300">
                  {key === "name" ? "Tag" : key === "value" ? "Value" : "Category"}
                  {sortKey === key ? (ascending ? " ↑" : " ↓") : ""}
                </button>
              </th>
            ))}
            {removedOnly && <th className="px-4 py-3 font-medium text-emerald-400/90">Removed</th>}
          </tr>
        </thead>
        <tbody>
          {rows.map((tag) => (
            <tr key={`${tag.name}-${tag.value}`} className="border-t border-[var(--color-border)]/60">
              <td className="px-4 py-2 font-mono text-xs">{tag.name}</td>
              <td className="px-4 py-2 break-all">{tag.value}</td>
              <td className="px-4 py-2">{CATEGORY_LABELS[tag.category]}</td>
              {removedOnly && (
                <td className="px-4 py-2 text-emerald-400">
                  {removedNames?.has(tag.name) ? "Yes" : "No"}
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
