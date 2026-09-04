import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Play, FolderOpen, MoreVertical, Trash2, Settings, Box, Loader2 } from "lucide-react";
import { GlassCard, GlassButton, GlassBadge } from "@/components/ui/glass";
import { formatDate } from "@/utils/formatters";
import type { Instance } from "@/types";

interface InstanceCardProps {
  instance: Instance;
  onPlay: (id: string) => void;
  onInstall: (id: string) => void;
  onDelete: (id: string) => void;
  onOpenFolder: (path: string) => void;
  isLaunching?: boolean;
}

export const InstanceCard: React.FC<InstanceCardProps> = ({
  instance,
  onPlay,
  onInstall,
  onDelete,
  onOpenFolder,
  isLaunching = false,
}) => {
  const navigate = useNavigate();
  const [menuOpen, setMenuOpen] = useState(false);

  // Status computation
  const statusType =
    typeof instance.status === "string"
      ? instance.status
      : Object.keys(instance.status)[0];

  const isInstalling = statusType === "installing";
  const isRunning = statusType === "running";
  const isReady = statusType === "ready";
  const isIdle = statusType === "idle";

  const handlePrimaryAction = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isReady) {
      onPlay(instance.id);
    } else if (isIdle) {
      onInstall(instance.id);
    }
  };

  return (
    <GlassCard
      onClick={() => navigate(`/instances/${instance.id}`)}
      className="group flex flex-col justify-between h-[230px] p-5 cursor-pointer relative"
    >
      {/* Top row */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3.5 overflow-hidden">
          {/* Instance Icon Box */}
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500/20 to-indigo-600/20 border border-white/15 text-blue-400 shadow-inner group-hover:scale-105 transition-transform duration-200">
            <Box className="h-6 w-6" />
          </div>

          <div className="overflow-hidden">
            <h4 className="font-semibold text-white truncate text-base group-hover:text-blue-400 transition-colors">
              {instance.name}
            </h4>
            <div className="flex items-center gap-2 mt-1">
              <span className="text-xs font-medium text-slate-400">
                MC {instance.minecraftVersion}
              </span>
              <span className="h-1 w-1 rounded-full bg-slate-600" />
              <span className="text-xs font-medium text-slate-400 capitalize">
                {instance.loader}
              </span>
            </div>
          </div>
        </div>

        {/* Status badge */}
        <div className="relative">
          {isRunning && (
            <GlassBadge variant="success">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 mr-1.5 animate-ping" />
              Running
            </GlassBadge>
          )}
          {isInstalling && (
            <GlassBadge variant="info">
              <Loader2 className="h-3 w-3 mr-1 animate-spin" />
              Installing
            </GlassBadge>
          )}
          {isReady && <GlassBadge variant="default">Ready</GlassBadge>}
          {isIdle && <GlassBadge variant="warning">Not Installed</GlassBadge>}
          {statusType === "error" && <GlassBadge variant="danger">Error</GlassBadge>}
        </div>
      </div>

      {/* Middle info */}
      <div className="space-y-1.5 text-xs text-slate-400 pt-2 border-t border-white/[0.06]">
        <div className="flex justify-between">
          <span>Last played</span>
          <span className="text-slate-300 font-medium">{formatDate(instance.lastPlayedAt)}</span>
        </div>
        <div className="flex justify-between">
          <span>Memory</span>
          <span className="text-slate-300 font-medium">
            {instance.ram.minMb / 1024}GB - {instance.ram.maxMb / 1024}GB
          </span>
        </div>
      </div>

      {/* Bottom actions */}
      <div className="flex items-center justify-between pt-3 gap-2">
        <GlassButton
          variant={isReady ? "primary" : "secondary"}
          size="sm"
          disabled={isRunning || isInstalling || isLaunching}
          onClick={handlePrimaryAction}
          className="flex-1"
        >
          {isInstalling ? (
            <>
              <Loader2 className="h-3.5 w-3.5 animate-spin mr-1.5" />
              Installing...
            </>
          ) : isRunning ? (
            "In Game"
          ) : isReady ? (
            <>
              <Play className="h-3.5 w-3.5 mr-1.5 fill-current" />
              PLAY
            </>
          ) : (
            "INSTALL"
          )}
        </GlassButton>

        <div className="relative">
          <GlassButton
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-slate-400 hover:text-white"
            onClick={(e) => {
              e.stopPropagation();
              setMenuOpen(!menuOpen);
            }}
          >
            <MoreVertical className="h-4 w-4" />
          </GlassButton>

          {/* Context Dropdown */}
          {menuOpen && (
            <div
              className="absolute right-0 bottom-full mb-2 w-44 rounded-2xl bg-[#13151f]/95 border border-white/15 backdrop-blur-2xl shadow-2xl p-1.5 z-40 animate-in fade-in zoom-in-95 duration-150"
              onClick={(e) => e.stopPropagation()}
            >
              <button
                onClick={() => {
                  setMenuOpen(false);
                  navigate(`/instances/${instance.id}`);
                }}
                className="flex items-center gap-2.5 w-full px-3 py-2 text-xs font-medium text-slate-200 hover:bg-white/10 rounded-xl transition-colors text-left"
              >
                <Settings className="h-3.5 w-3.5 text-slate-400" />
                Configure
              </button>
              <button
                onClick={() => {
                  setMenuOpen(false);
                  onOpenFolder(instance.gameDirectory);
                }}
                className="flex items-center gap-2.5 w-full px-3 py-2 text-xs font-medium text-slate-200 hover:bg-white/10 rounded-xl transition-colors text-left"
              >
                <FolderOpen className="h-3.5 w-3.5 text-slate-400" />
                Open Game Folder
              </button>
              <div className="h-px bg-white/10 my-1" />
              <button
                disabled={isRunning}
                onClick={() => {
                  setMenuOpen(false);
                  onDelete(instance.id);
                }}
                className="flex items-center gap-2.5 w-full px-3 py-2 text-xs font-medium text-red-400 hover:bg-red-500/10 rounded-xl transition-colors text-left disabled:opacity-40"
              >
                <Trash2 className="h-3.5 w-3.5" />
                Delete Instance
              </button>
            </div>
          )}
        </div>
      </div>
    </GlassCard>
  );
};