import React from "react";
import { Download, CheckCircle2, Activity } from "lucide-react";
import { GlassPanel, GlassCard, GlassButton, GlassProgress, GlassBadge } from "@/components/ui/glass";
import { useDownloadStore } from "@/store/useDownloadStore";
import { formatBytes, formatSpeed } from "@/utils/formatters";

export const DownloadsPage: React.FC = () => {
  const {
    isDownloading,
    currentFile,
    currentInstanceId,
    downloadedBytes,
    totalBytes,
    percentage,
    speed,
    history,
    resetProgress,
  } = useDownloadStore();

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2.5">
          <Download className="h-6 w-6 text-blue-400" />
          Download Manager
        </h1>
        <p className="text-xs text-slate-400 mt-0.5">
          Real-time downloads of Minecraft version assets, libraries, and client files.
        </p>
      </div>

      {/* Active Download Card */}
      {isDownloading ? (
        <GlassPanel variant="elevated" className="p-6 space-y-4 border-blue-500/30">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-blue-600/20 text-blue-400">
                <Activity className="h-5 w-5 animate-spin" />
              </div>
              <div>
                <h3 className="text-sm font-semibold text-white truncate max-w-md">
                  {currentFile || "Downloading assets and libraries..."}
                </h3>
                <span className="text-xs text-slate-400">
                  {currentInstanceId ? `Instance: ${currentInstanceId}` : "Shared Libraries"}
                </span>
              </div>
            </div>
            <GlassBadge variant="info">Active</GlassBadge>
          </div>

          <GlassProgress value={percentage} height="md" />

          <div className="flex items-center justify-between text-xs text-slate-400">
            <span>
              {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)} ({percentage.toFixed(1)}%)
            </span>
            <span className="font-semibold text-blue-400">{formatSpeed(speed)}</span>
          </div>
        </GlassPanel>
      ) : (
        <GlassCard interactive={false} className="p-8 text-center space-y-2">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-xl bg-white/5 border border-white/10 text-emerald-400">
            <CheckCircle2 className="h-6 w-6" />
          </div>
          <h3 className="text-sm font-semibold text-white">No active downloads</h3>
          <p className="text-xs text-slate-400">All instance files and caches are up to date.</p>
        </GlassCard>
      )}

      {/* Recent downloads log */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-white">Recent Activity</h3>
          {history.length > 0 && (
            <GlassButton variant="ghost" size="sm" onClick={resetProgress}>
              Clear
            </GlassButton>
          )}
        </div>

        <div className="space-y-2">
          {history.length === 0 ? (
            <div className="p-6 text-center text-xs text-slate-500 bg-white/[0.02] rounded-2xl border border-white/5">
              Activity log is empty.
            </div>
          ) : (
            history.map((item, idx) => (
              <div
                key={idx}
                className="flex items-center justify-between p-3.5 rounded-xl bg-white/[0.03] border border-white/5 text-xs text-slate-300"
              >
                <div className="flex items-center gap-3 overflow-hidden">
                  <CheckCircle2 className="h-4 w-4 text-emerald-400 shrink-0" />
                  <span className="truncate max-w-sm">{item.file}</span>
                </div>
                <div className="flex items-center gap-4 shrink-0 text-slate-400">
                  <span>{formatBytes(item.totalBytes)}</span>
                  <span className="font-mono text-emerald-400">100%</span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};