import React, { useState } from "react";
import { Activity, Zap, Gamepad2 } from "lucide-react";
import { useDownloadStore } from "@/store/useDownloadStore";
import { formatSpeed } from "@/utils/formatters";
import { useOfflineProfiles } from "@/features/auth/hooks/useOfflineProfiles";
import { useMicrosoftAuth } from "@/features/auth/hooks/useMicrosoftAuth";
import { useSettings } from "@/features/settings/hooks/useSettings";
import { AccountModal } from "@/features/auth/components/AccountModal";

export const Header: React.FC = () => {
  const { isDownloading, currentFile, percentage, speed } = useDownloadStore();
  const { activeProfile } = useOfflineProfiles();
  const { account: microsoftAccount } = useMicrosoftAuth();
  const { settings } = useSettings();
  const isMicrosoftActive = settings?.activeAuthMode === "microsoft";
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

          {/* Active Account Pill */}
          <button
            onClick={() => setIsProfileModalOpen(true)}
            className={`flex items-center gap-2 px-3 py-1 rounded-full bg-white/[0.05] hover:bg-white/[0.1] border border-white/10 text-xs transition-all cursor-pointer group shadow-sm ${
              isMicrosoftActive ? "hover:border-blue-400/40" : "hover:border-amber-400/40"
            }`}
            title="Manage Account"
          >
            <div
              className={`relative flex items-center justify-center h-5 w-5 rounded-full border ${
                isMicrosoftActive
                  ? "bg-blue-500/20 border-blue-400/30 text-blue-300"
                  : "bg-amber-500/20 border-amber-400/30 text-amber-300"
              }`}
            >
              {isMicrosoftActive ? <Gamepad2 className="h-3 w-3" /> : <Zap className="h-3 w-3" />}
              <span className="absolute -bottom-0.5 -right-0.5 h-1.5 w-1.5 rounded-full bg-emerald-400 ring-2 ring-[#090a0f]" />
            </div>
            <span className="text-slate-300 font-medium group-hover:text-white transition-colors">
              {isMicrosoftActive ? microsoftAccount?.username ?? "Microsoft" : activeProfile?.username ?? "Offline"}
            </span>
          </button>
        </div>
      </header>

      <AccountModal
        isOpen={isProfileModalOpen}
        onClose={() => setIsProfileModalOpen(false)}
      />
    </>
  );
};