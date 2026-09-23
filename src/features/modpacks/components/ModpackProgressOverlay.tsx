import React from "react";
import { X, Loader2, ShieldCheck } from "lucide-react";

import { formatBytes } from "@/utils/formatters";
import { useModpackStore } from "../store/useModpackStore";
import { modpacksApi } from "@/services/tauri/modpacksApi";

export const ModpackProgressOverlay: React.FC = () => {
  const activeJobs = useModpackStore((s) => s.activeJobs);
  const jobs = Object.values(activeJobs);

  if (jobs.length === 0) return null;

  const currentJob = jobs[0];

  const handleCancel = async () => {
    try {
      await modpacksApi.cancel(currentJob.instanceId);
    } catch (e) {
      console.error("Failed to cancel modpack operation:", e);
    }
  };

  return (
    <div className="fixed bottom-6 right-6 z-50 w-96 rounded-2xl bg-[#11131c]/90 border border-blue-500/30 backdrop-blur-2xl shadow-2xl p-4 animate-in slide-in-from-bottom-5 duration-300">
      <div className="flex items-start justify-between gap-3 mb-2">
        <div className="flex items-center gap-2.5 overflow-hidden">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-blue-500/20 text-blue-400 border border-blue-500/30">
            {currentJob.stage === "Verifying Checksums" ? (
              <ShieldCheck className="h-4 w-4 text-emerald-400" />
            ) : (
              <Loader2 className="h-4 w-4 animate-spin text-blue-400" />
            )}
          </div>
          <div className="overflow-hidden">
            <h5 className="text-xs font-bold text-white truncate">
              {currentJob.stage || "Updating Modpack"}
            </h5>
            <p className="text-[11px] text-slate-400 truncate">
              {currentJob.currentFile || "Preparing..."}
            </p>
          </div>
        </div>

        <button
          onClick={handleCancel}
          title="Cancel Operation"
          className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      {/* Progress bar */}
      <div className="space-y-1.5 mt-3">
        <div className="flex justify-between text-[11px] font-medium text-slate-300">
          <span>
            {currentJob.filesCompleted} / {currentJob.totalFiles} files
          </span>
          <span>
            {formatBytes(currentJob.downloadedBytes)} /{" "}
            {formatBytes(currentJob.totalBytes)} ({Math.round(currentJob.percentage)}%)
          </span>
        </div>

        <div className="w-full h-2 bg-black/50 rounded-full overflow-hidden border border-white/5">
          <div
            className="h-full bg-gradient-to-r from-blue-500 to-indigo-500 rounded-full transition-all duration-200"
            style={{ width: `${Math.min(100, Math.max(0, currentJob.percentage))}%` }}
          />
        </div>
      </div>
    </div>
  );
};
