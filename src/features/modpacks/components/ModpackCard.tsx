import React from "react";
import { useNavigate } from "react-router-dom";
import {
  Download,
  Sparkles,
  ArrowUpCircle,
  CheckCircle2,
  Package,
  Layers,
  User,
} from "lucide-react";

import { GlassCard, GlassButton, GlassBadge } from "@/components/ui/glass";
import { useModpackStore } from "../store/useModpackStore";
import type { CatalogItemWithStatus } from "@/types";

interface ModpackCardProps {
  item: CatalogItemWithStatus;
  onInstall: (item: CatalogItemWithStatus) => void;
  onUpdate: (item: CatalogItemWithStatus) => void;
}

export const ModpackCard: React.FC<ModpackCardProps> = ({
  item,
  onInstall,
  onUpdate,
}) => {
  const navigate = useNavigate();
  const { modpack, status, installedVersion, updateAvailable, associatedInstanceId } = item;

  const activeJob = useModpackStore((s) =>
    associatedInstanceId ? s.activeJobs[associatedInstanceId] : undefined
  );

  const isUpdatingOrInstalling = Boolean(activeJob);

  if (!modpack) return null;

  return (
    <GlassCard
      onClick={() => navigate(`/modpacks/${modpack.id}`)}
      className="group flex flex-col justify-between h-[360px] p-0 cursor-pointer overflow-hidden border border-white/10 hover:border-blue-500/40 hover:shadow-glow transition-all duration-300"
    >
      {/* Top Banner / Header */}
      <div className="relative h-36 w-full overflow-hidden bg-gradient-to-br from-slate-900 via-indigo-950/40 to-slate-900">
        {modpack.bannerUrl ? (
          <img
            src={modpack.bannerUrl}
            alt={modpack.name}
            className="h-full w-full object-cover group-hover:scale-105 transition-transform duration-500"
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = "none";
            }}
          />
        ) : (
          <div className="absolute inset-0 bg-gradient-to-tr from-blue-900/30 via-indigo-800/20 to-purple-900/30 flex items-center justify-center">
            <Package className="h-12 w-12 text-blue-400/30" />
          </div>
        )}

        {/* Gradient shadow overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-[#11131c] via-[#11131c]/50 to-transparent" />

        {/* Icon & Version Badges Overlay */}
        <div className="absolute bottom-3 left-4 right-4 flex items-end justify-between">
          <div className="flex items-center gap-3">
            {/* Pack Icon */}
            <div className="h-12 w-12 rounded-xl overflow-hidden bg-black/50 border border-white/20 backdrop-blur-md shadow-lg shrink-0 flex items-center justify-center">
              {modpack.iconUrl ? (
                <img
                  src={modpack.iconUrl}
                  alt={modpack.name}
                  className="h-full w-full object-cover"
                  onError={(e) => {
                    (e.target as HTMLImageElement).style.display = "none";
                  }}
                />
              ) : (
                <Package className="h-6 w-6 text-blue-400" />
              )}
            </div>

            <div className="overflow-hidden">
              <h3 className="text-base font-bold text-white truncate drop-shadow-md group-hover:text-blue-400 transition-colors">
                {modpack.name}
              </h3>
              <div className="flex items-center gap-1.5 text-xs text-slate-300">
                <User className="h-3 w-3 text-slate-400" />
                <span className="truncate">{modpack.author || "Community"}</span>
              </div>
            </div>
          </div>

          {/* Status Badge */}
          <div>
            {updateAvailable && (
              <GlassBadge variant="warning" className="animate-pulse shadow-glow-sm">
                <Sparkles className="h-3 w-3 mr-1" />
                Update
              </GlassBadge>
            )}
            {!updateAvailable && status === "Installed" && (
              <GlassBadge variant="success">
                <CheckCircle2 className="h-3 w-3 mr-1" />
                Installed
              </GlassBadge>
            )}
          </div>
        </div>
      </div>

      {/* Middle Body */}
      <div className="flex-1 p-4 flex flex-col justify-between space-y-3">
        {/* Description */}
        <p className="text-xs text-slate-400 line-clamp-2 leading-relaxed">
          {modpack.description || "No description provided for this modpack."}
        </p>

        {/* Tags */}
        {modpack.tags && modpack.tags.length > 0 && (
          <div className="flex flex-wrap gap-1.5 max-h-6 overflow-hidden">
            {modpack.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className="text-[10px] px-2 py-0.5 rounded-full bg-white/[0.06] text-slate-300 border border-white/10 font-medium"
              >
                {tag}
              </span>
            ))}
            {modpack.tags.length > 3 && (
              <span className="text-[10px] px-1.5 py-0.5 text-slate-500">
                +{modpack.tags.length - 3}
              </span>
            )}
          </div>
        )}

        {/* Info Grid */}
        <div className="grid grid-cols-2 gap-2 pt-2 border-t border-white/[0.08] text-[11px]">
          <div className="flex items-center gap-1.5 text-slate-400">
            <Layers className="h-3.5 w-3.5 text-slate-500" />
            <span>MC {modpack.minecraftVersion}</span>
          </div>
          <div className="flex items-center gap-1.5 text-slate-400 capitalize">
            <span className="h-1.5 w-1.5 rounded-full bg-blue-400" />
            <span>{modpack.loader}</span>
          </div>
        </div>
      </div>

      {/* Footer / Active Job or Action Buttons */}
      <div className="p-4 pt-0">
        {isUpdatingOrInstalling && activeJob ? (
          <div className="space-y-1.5 p-2 rounded-xl bg-blue-500/10 border border-blue-500/20">
            <div className="flex items-center justify-between text-[11px] text-blue-300">
              <span className="truncate max-w-[180px] font-medium">
                {activeJob.stage}: {activeJob.currentFile || "Processing"}
              </span>
              <span className="font-semibold">{Math.round(activeJob.percentage)}%</span>
            </div>
            <div className="w-full h-1.5 bg-black/40 rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-blue-500 to-indigo-500 rounded-full transition-all duration-200"
                style={{ width: `${Math.min(100, Math.max(0, activeJob.percentage))}%` }}
              />
            </div>
          </div>
        ) : updateAvailable ? (
          <div className="flex gap-2">
            <GlassButton
              variant="primary"
              size="sm"
              className="flex-1 bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border-amber-500/40 shadow-glow-warning"
              onClick={(e) => {
                e.stopPropagation();
                onUpdate(item);
              }}
            >
              <ArrowUpCircle className="h-3.5 w-3.5 mr-1.5" />
              Update to v{modpack.latestVersion}
            </GlassButton>
            {associatedInstanceId && (
              <GlassButton
                variant="secondary"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  navigate(`/instances/${associatedInstanceId}`);
                }}
              >
                Instance
              </GlassButton>
            )}
          </div>
        ) : status === "Installed" ? (
          <div className="flex gap-2">
            <GlassButton
              variant="secondary"
              size="sm"
              className="flex-1 text-slate-300"
              onClick={(e) => {
                e.stopPropagation();
                if (associatedInstanceId) {
                  navigate(`/instances/${associatedInstanceId}`);
                } else {
                  navigate(`/modpacks/${modpack.id}`);
                }
              }}
            >
              <CheckCircle2 className="h-3.5 w-3.5 mr-1.5 text-emerald-400" />
              Installed (v{installedVersion})
            </GlassButton>
          </div>
        ) : (
          <GlassButton
            variant="primary"
            size="sm"
            className="w-full"
            onClick={(e) => {
              e.stopPropagation();
              onInstall(item);
            }}
          >
            <Download className="h-3.5 w-3.5 mr-1.5" />
            Install Modpack
          </GlassButton>
        )}
      </div>
    </GlassCard>
  );
};
