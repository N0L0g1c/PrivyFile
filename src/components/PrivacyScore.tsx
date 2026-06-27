import type { MetadataReport } from "@/lib/types";

interface PrivacyScoreProps {
  score: number;
  label?: string;
  size?: "sm" | "md" | "lg";
}

/** Higher score = cleaner file (100 = no sensitive metadata). */
export function privacyScoreLabel(score: number): string {
  if (score >= 100) return "Fully clean";
  if (score >= 80) return "Well sanitized";
  if (score >= 60) return "Some metadata remains";
  if (score > 0) return "High exposure risk";
  return "Maximum exposure";
}

function scoreColor(score: number): string {
  if (score >= 80) return "text-emerald-400 border-emerald-500/40 bg-emerald-500/10";
  if (score >= 60) return "text-amber-400 border-amber-500/40 bg-amber-500/10";
  return "text-red-400 border-red-500/40 bg-red-500/10";
}

export default function PrivacyScore({ score, label, size = "md" }: PrivacyScoreProps) {
  const sizeClasses = {
    sm: "text-xs px-2 py-1",
    md: "text-sm px-3 py-1.5",
    lg: "text-base px-4 py-2",
  };

  const normalized = Math.max(0, Math.min(100, Math.round(score)));

  return (
    <div className="flex flex-col gap-1">
      {label && <span className="text-xs text-[var(--color-muted)]">{label}</span>}
      <span
        className={`inline-flex items-center gap-2 rounded-full border font-semibold ${sizeClasses[size]} ${scoreColor(normalized)}`}
        title={privacyScoreLabel(normalized)}
      >
        <span>{normalized}/100</span>
        <span className="font-normal opacity-80">{privacyScoreLabel(normalized)}</span>
      </span>
    </div>
  );
}

export function PrivacyScoreFromReport({ report }: { report: MetadataReport }) {
  return <PrivacyScore score={report.privacy_score} label="Privacy score" />;
}
