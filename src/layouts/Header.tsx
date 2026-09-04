import React from "react";
import { Activity } from "lucide-react";
import { useDownloadStore } from "@/store/useDownloadStore";
import { formatSpeed } from "@/utils/formatters";

export const Header: React.FC = () => {
  const { isDownloading, currentFile, percentage, speed } = useDownloadStore();

  return (
    <header
      data-tauri-drag-region
      className="h-12 w-full px-6 flex items-center justify-between border-b border-white/[0.08] bg-[#090a0f]/60 backdrop-blur-xl z-20 select-none"
    >
      <div className="flex items-center gap-3">
        <span className="text-xs font-semibold text-slate-400 tracking-wide">
          NOVA LAUNCHER
        </span>
      </div>

      {/* Right status */}
      <div className="flex items-center gap-4">
        {isDownloading && (
          <div className="flex items-center gap-3 px-3 py-1 rounded-full bg-blue-500/10 border border-blue-500/20 text-xs">
            <Activity className="h-3.5 w-3.5 text-blue-400 animate-spin" />
            <span className="text-slate-300 truncate max-w-[140px]">{currentFile || "Downloading..."}</span>
            <span className="font-semibold text-blue-400">{percentage.toFixed(0)}%</span>
            <span className="text-[10px] text-slate-400">({formatSpeed(speed)})</span>
          </div>
        )}
      </div>
    </header>
  );
};