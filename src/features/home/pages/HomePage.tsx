import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, Play, Sparkles, Box, Clock, HardDrive, ArrowRight, ArrowUpCircle } from "lucide-react";

import { GlassPanel, GlassCard, GlassButton, GlassBadge } from "@/components/ui/glass";
import { useInstances } from "@/features/instances/hooks/useInstances";
import { useModpackStore } from "@/features/modpacks/store/useModpackStore";
import { CreateInstanceModal } from "@/features/instances/components/CreateInstanceModal";
import { formatDate } from "@/utils/formatters";
import { getErrorMessage } from "@/utils/errors";

export const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const { instances, launchInstance, installInstance, createInstance } = useInstances();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const catalog = useModpackStore((s) => s.catalog);
  const pendingUpdates = catalog.filter((c) => c.updateAvailable);

  // Find most recently played or first available instance for Hero Card
  const heroInstance = instances.length > 0 ? instances[0] : null;

  const runAction = async (action: () => Promise<unknown>, fallback: string) => {
    setActionError(null);
    try {
      await action();
    } catch (err) {
      setActionError(getErrorMessage(err, fallback));
    }
  };

  const handleHeroAction = () => {
    if (!heroInstance) return;
    const statusType =
      typeof heroInstance.status === "string"
        ? heroInstance.status
        : Object.keys(heroInstance.status)[0];

    if (statusType === "ready") {
      runAction(() => launchInstance(heroInstance.id), "Failed to launch the instance");
    } else if (statusType === "idle" || statusType === "error") {
      runAction(() => installInstance(heroInstance.id), "Failed to install the instance");
    }
  };

  return (
    <div className="space-y-8 max-w-6xl mx-auto">
      {/* Welcome header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight text-white flex items-center gap-3">
            Welcome back <Sparkles className="h-6 w-6 text-blue-400" />
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Manage your Minecraft instances with isolated workspaces and liquid glass aesthetics.
          </p>
        </div>

        <GlassButton
          variant="primary"
          size="md"
          onClick={() => setIsCreateOpen(true)}
          className="self-start sm:self-auto"
        >
          <Plus className="h-4 w-4" />
          New Instance
        </GlassButton>
      </div>

      {actionError && (
        <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/30 text-red-300 text-xs font-mono break-all">
          {actionError}
        </div>
      )}

      {/* Hero Card */}
      {heroInstance ? (
        <GlassPanel
          variant="elevated"
          className="relative overflow-hidden p-8 border-white/15 bg-gradient-to-br from-blue-900/30 via-surface/90 to-indigo-950/20"
        >
          <div className="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
            <div className="space-y-3 max-w-xl">
              <div className="flex items-center gap-2.5">
                <GlassBadge variant="info">Featured Instance</GlassBadge>
                <span className="text-xs text-slate-400">
                  Last played: {formatDate(heroInstance.lastPlayedAt)}
                </span>
              </div>

              <h2 className="text-3xl font-bold text-white tracking-tight">
                {heroInstance.name}
              </h2>

              <p className="text-sm text-slate-300">
                Minecraft <span className="font-semibold text-white">{heroInstance.minecraftVersion}</span> (Vanilla)
                with isolated world saves, mods, and options.
              </p>

              <div className="flex items-center gap-4 text-xs text-slate-400 pt-1">
                <span className="flex items-center gap-1.5">
                  <HardDrive className="h-3.5 w-3.5 text-blue-400" />
                  {heroInstance.ram.maxMb / 1024} GB RAM
                </span>
                <span className="flex items-center gap-1.5">
                  <Clock className="h-3.5 w-3.5 text-indigo-400" />
                  {Math.round(heroInstance.totalPlayTimeSeconds / 60)} mins played
                </span>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <GlassButton
                variant="primary"
                size="lg"
                onClick={handleHeroAction}
                className="px-8 shadow-glow-accent"
              >
                <Play className="h-5 w-5 fill-current" />
                PLAY NOW
              </GlassButton>
              <GlassButton
                variant="secondary"
                size="lg"
                onClick={() => navigate(`/instances/${heroInstance.id}`)}
              >
                Details
              </GlassButton>
            </div>
          </div>

          {/* Background subtle watermark glow */}
          <div className="absolute -right-10 -bottom-10 opacity-10 pointer-events-none">
            <Box className="h-72 w-72 text-blue-400" />
          </div>
        </GlassPanel>
      ) : (
        <GlassPanel variant="subtle" className="p-10 text-center space-y-4">
          <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-white/5 border border-white/10 text-blue-400">
            <Box className="h-7 w-7" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white">No instances created yet</h3>
            <p className="text-xs text-slate-400 mt-1 max-w-sm mx-auto">
              Create your first isolated Minecraft instance to get started.
            </p>
          </div>
          <GlassButton variant="primary" onClick={() => setIsCreateOpen(true)}>
            <Plus className="h-4 w-4" />
            Create Instance
          </GlassButton>
        </GlassPanel>
      )}

      {/* Modpack Updates Available Banner */}
      {pendingUpdates.length > 0 && (
        <GlassCard className="p-5 border-amber-500/30 bg-amber-500/10 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="h-10 w-10 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/30 flex items-center justify-center">
                <Sparkles className="h-5 w-5" />
              </div>
              <div>
                <h4 className="text-sm font-bold text-white">
                  {pendingUpdates.length} Modpack Update{pendingUpdates.length > 1 ? "s" : ""} Ready
                </h4>
                <p className="text-xs text-amber-200/80">
                  New versions are available with updated mods and performance fixes.
                </p>
              </div>
            </div>

            <GlassButton
              variant="primary"
              size="sm"
              onClick={() => navigate("/modpacks")}
              className="bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border-amber-500/40 shadow-glow-warning"
            >
              <ArrowUpCircle className="h-4 w-4 mr-1.5" />
              View Updates
            </GlassButton>
          </div>
        </GlassCard>
      )}

      {/* Recent Instances Section */}
      <div className="space-y-4">

        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-white tracking-tight">
            Your Instances ({instances.length})
          </h3>
          {instances.length > 0 && (
            <button
              onClick={() => navigate("/instances")}
              className="text-xs text-blue-400 hover:text-blue-300 font-medium flex items-center gap-1 transition-colors"
            >
              View all <ArrowRight className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        {instances.length > 0 ? (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {instances.slice(0, 3).map((inst) => (
              <GlassCard
                key={inst.id}
                onClick={() => navigate(`/instances/${inst.id}`)}
                className="p-5 flex flex-col justify-between h-44 cursor-pointer"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-blue-500/10 border border-white/10 text-blue-400">
                      <Box className="h-5 w-5" />
                    </div>
                    <div>
                      <h4 className="font-semibold text-white text-sm truncate max-w-[140px]">
                        {inst.name}
                      </h4>
                      <span className="text-xs text-slate-400">MC {inst.minecraftVersion}</span>
                    </div>
                  </div>
                  <GlassBadge variant="default">{inst.loader}</GlassBadge>
                </div>

                <div className="flex items-center justify-between text-xs text-slate-400 pt-3 border-t border-white/5">
                  <span>{formatDate(inst.lastPlayedAt)}</span>
                  <GlassButton
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      const statusType =
                        typeof inst.status === "string"
                          ? inst.status
                          : Object.keys(inst.status)[0];
                      if (statusType === "ready") {
                        runAction(() => launchInstance(inst.id), "Failed to launch the instance");
                      } else {
                        runAction(() => installInstance(inst.id), "Failed to install the instance");
                      }
                    }}
                    className="text-blue-400 hover:text-blue-300"
                  >
                    <Play className="h-3.5 w-3.5 mr-1 fill-current" />
                    Play
                  </GlassButton>
                </div>
              </GlassCard>
            ))}
          </div>
        ) : null}
      </div>

      <CreateInstanceModal
        isOpen={isCreateOpen}
        onClose={() => setIsCreateOpen(false)}
        onCreate={async (dto) => {
          const created = await createInstance(dto);
          installInstance(created.id);
        }}
      />
    </div>
  );
};