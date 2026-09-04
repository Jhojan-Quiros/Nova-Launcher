import React, { useState } from "react";
import { Terminal, Copy, Trash2, Check, Search } from "lucide-react";
import { GlassPanel, GlassButton } from "@/components/ui/glass";
import { useLogs } from "@/features/logs/hooks/useLogs";

export const LogsPage: React.FC = () => {
  const { logs, clear } = useLogs();
  const [filterLevel, setFilterLevel] = useState<string>("ALL");
  const [search, setSearch] = useState("");
  const [copied, setCopied] = useState(false);

  const filteredLogs = logs.filter((log) => {
    const matchesLevel =
      filterLevel === "ALL" || log.level.toUpperCase() === filterLevel;
    const matchesSearch =
      search === "" ||
      log.message.toLowerCase().includes(search.toLowerCase()) ||
      log.target.toLowerCase().includes(search.toLowerCase());
    return matchesLevel && matchesSearch;
  });

  const handleCopy = () => {
    const text = filteredLogs
      .map((l) => `[${l.timestamp}] [${l.level}] [${l.target}] ${l.message}`)
      .join("\n");
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="space-y-4 max-w-6xl mx-auto h-[calc(100vh-100px)] flex flex-col">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2.5">
            <Terminal className="h-6 w-6 text-blue-400" />
            System & Game Logs
          </h1>
          <p className="text-xs text-slate-400 mt-0.5">
            Live output from the Nova Launcher core and running Minecraft instances.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <GlassButton variant="secondary" size="sm" onClick={handleCopy}>
            {copied ? (
              <>
                <Check className="h-3.5 w-3.5 mr-1 text-emerald-400" />
                Copied
              </>
            ) : (
              <>
                <Copy className="h-3.5 w-3.5 mr-1" />
                Copy
              </>
            )}
          </GlassButton>
          <GlassButton variant="ghost" size="sm" onClick={clear}>
            <Trash2 className="h-3.5 w-3.5 mr-1" />
            Clear
          </GlassButton>
        </div>
      </div>

      {/* Filter and search bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
        <div className="flex items-center gap-1.5 p-1 rounded-xl bg-white/[0.04] border border-white/10 text-xs">
          {["ALL", "ERROR", "WARN", "INFO", "DEBUG"].map((lvl) => (
            <button
              key={lvl}
              onClick={() => setFilterLevel(lvl)}
              className={`px-3 py-1 rounded-lg font-medium transition-all ${
                filterLevel === lvl
                  ? "bg-blue-600/30 text-blue-400 border border-blue-500/30 font-semibold"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              {lvl}
            </button>
          ))}
        </div>

        <div className="relative w-full sm:w-64">
          <Search className="absolute left-3 top-2.5 h-3.5 w-3.5 text-slate-500" />
          <input
            placeholder="Search logs..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-8 pr-3 py-1.5 rounded-xl text-xs bg-white/[0.04] border border-white/10 text-white placeholder:text-slate-500 outline-none focus:border-blue-500/60"
          />
        </div>
      </div>

      {/* Terminal View */}
      <GlassPanel
        variant="elevated"
        className="flex-1 overflow-hidden p-4 font-['JetBrains_Mono'] text-xs leading-relaxed bg-black/75 border-white/10 select-text flex flex-col"
      >
        <div className="flex-1 overflow-y-auto space-y-1 pr-2">
          {filteredLogs.length === 0 ? (
            <div className="text-slate-500 italic py-8 text-center">
              No log messages to display.
            </div>
          ) : (
            filteredLogs.map((log, idx) => (
              <div key={idx} className="flex items-start gap-2 hover:bg-white/[0.02] py-0.5 rounded px-1">
                <span className="text-slate-500 shrink-0 select-none">
                  [{log.timestamp}]
                </span>
                <span
                  className={`font-semibold shrink-0 select-none ${
                    log.level === "ERROR"
                      ? "text-rose-400"
                      : log.level === "WARN"
                      ? "text-amber-400"
                      : log.level === "DEBUG"
                      ? "text-purple-400"
                      : "text-blue-400"
                  }`}
                >
                  [{log.level}]
                </span>
                <span className="text-slate-400 font-semibold shrink-0">
                  [{log.target}]
                </span>
                <span className="text-slate-200 break-all">{log.message}</span>
              </div>
            ))
          )}
        </div>
      </GlassPanel>
    </div>
  );
};