import React, { useState } from "react";
import { Activity, Zap } from "lucide-react";
import { useDownloadStore } from "@/store/useDownloadStore";
import { formatSpeed } from "@/utils/formatters";
import { useOfflineProfiles } from "@/features/auth/hooks/useOfflineProfiles";
import { ProfileModal } from "@/features/auth/components/ProfileModal";

export const Header: React.FC = () => {
  const { isDownloading, currentFile, percentage, speed } = useDownloadStore();
  const { activeProfile } = useOfflineProfiles();
  const [isProfileModalOpen, setIsProfileModalOpen] = useState(false);

  return (
    <>
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
        <div className="flex items-center gap-3">
          {isDownloading && (
            <div className="flex items-center gap-3 px-3 py-1 rounded-full bg-blue-500/10 border border-blue-500/20 text-xs">
              <Activity className="h-3.5 w-3.5 text-blue-400 animate-spin" />
              <span className="text-slate-300 truncate max-w-[140px]">{currentFile || "Downloading..."}</span>
              <span className="font-semibold text-blue-400">{percentage.toFixed(0)}%</span>
              <span className="text-[10px] text-slate-400">({formatSpeed(speed)})</span>
            </div>
          )}

          {/* Offline Mode Profile Pill */}
          <button
            onClick={() => setIsProfileModalOpen(true)}
            className="flex items-center gap-2 px-3 py-1 rounded-full bg-white/[0.05] hover:bg-white/[0.1] border border-white/10 hover:border-amber-400/40 text-xs transition-all cursor-pointer group shadow-sm"
            title="Manage Offline Profile"
          >
            <div className="relative flex items-center justify-center h-5 w-5 rounded-full bg-amber-500/20 border border-amber-400/30 text-amber-300">
              <Zap className="h-3 w-3" />
              <span className="absolute -bottom-0.5 -right-0.5 h-1.5 w-1.5 rounded-full bg-emerald-400 ring-2 ring-[#090a0f]" />
            </div>
            <span className="text-slate-300 font-medium group-hover:text-white transition-colors">
              {activeProfile?.username ?? "Offline"}
            </span>
          </button>
        </div>
      </header>

      <ProfileModal
        isOpen={isProfileModalOpen}
        onClose={() => setIsProfileModalOpen(false)}
      />
    </>
  );
};