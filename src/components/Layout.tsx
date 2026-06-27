import { Link, useLocation } from "react-router-dom";
import { openUrl } from "@tauri-apps/plugin-opener";

const VASSDEV_URL = "https://vassbrekke.no/vassdev/";

const navItems = [
  { path: "/", label: "Workspace", icon: "🛡️" },
  { path: "/settings", label: "Settings", icon: "⚙️" },
  { path: "/history", label: "History", icon: "📁" },
];

async function openExternal(url: string) {
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

export default function Layout({ children }: { children: React.ReactNode }) {
  const location = useLocation();

  return (
    <div className="min-h-screen flex flex-col hero-glow">
      <header className="glass sticky top-0 z-50 border-b border-[var(--color-border)]">
        <div className="max-w-6xl mx-auto px-4 h-16 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-2.5 group">
            <div className="w-9 h-9 rounded-xl bg-emerald-500/20 flex items-center justify-center ring-1 ring-emerald-500/30 pulse-glow text-lg">
              🗂️
            </div>
            <div>
              <span className="font-bold text-lg tracking-tight group-hover:text-emerald-400 transition-colors">
                PrivyFile
              </span>
              <span className="block text-[10px] sm:text-xs text-[var(--color-muted)]">
                Sanitize before you share · by{" "}
                <button
                  type="button"
                  onClick={() => void openExternal(VASSDEV_URL)}
                  className="text-cyan-400/90 hover:text-cyan-300 transition-colors"
                >
                  VassDev
                </button>
              </span>
            </div>
          </Link>

          <nav className="flex items-center gap-1">
            {navItems.map((item) => {
              const active =
                item.path === "/"
                  ? location.pathname === "/"
                  : location.pathname.startsWith(item.path);
              return (
                <Link
                  key={item.path}
                  to={item.path}
                  className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-1.5 ${
                    active
                      ? "bg-emerald-500/15 text-emerald-400"
                      : "text-slate-400 hover:text-slate-200 hover:bg-white/5"
                  }`}
                >
                  <span className="hidden sm:inline">{item.icon}</span>
                  {item.label}
                </Link>
              );
            })}
          </nav>
        </div>
      </header>

      <main className="flex-1">{children}</main>

      <footer className="border-t border-[var(--color-border)] py-6 mt-auto">
        <div className="max-w-6xl mx-auto px-4 text-center text-sm text-[var(--color-muted)]">
          <p>Zero telemetry · All processing happens locally on your device</p>
        </div>
      </footer>
    </div>
  );
}
