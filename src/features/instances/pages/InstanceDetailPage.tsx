import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  Play,
  FolderOpen,
  Box,
  Settings,
  Terminal,
  Puzzle,
  Info,
  HardDrive,
  Cpu,
  Save,
  Loader2,
  Trash2,
} from "lucide-react";
import { GlassPanel, GlassCard, GlassButton, GlassBadge, GlassInput } from "@/components/ui/glass";
import { useInstance, useInstances } from "@/features/instances/hooks/useInstances";
import { useLogs } from "@/features/logs/hooks/useLogs";
import { formatDate, formatDuration } from "@/utils/formatters";
import { instancesApi } from "@/services/tauri/instancesApi";

export const InstanceDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: instance, isLoading, refetch } = useInstance(id || "");
  const { updateInstance, deleteInstance, launchInstance, installInstance } = useInstances();
  const { logs, clear } = useLogs(id);

  const [activeTab, setActiveTab] = useState<"overview" | "mods" | "settings" | "logs">("overview");

  // Settings form state
  const [editName, setEditName] = useState("");
  const [editJavaPath, setEditJavaPath] = useState("");
  const [editMinRam, setEditMinRam] = useState(2048);
  const [editMaxRam, setEditMaxRam] = useState(4096);
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    if (instance) {
      setEditName(instance.name);
      setEditJavaPath(instance.javaPath || "");
      setEditMinRam(instance.ram.minMb);
      setEditMaxRam(instance.ram.maxMb);
    }
  }, [instance]);

  if (isLoading) {
    return (
      <div className="flex h-96 items-center justify-center text-xs text-slate-400">
        Loading instance details...
      </div>
    );
  }

  if (!instance) {
    return (
      <div className="py-20 text-center space-y-3">
        <p className="text-sm text-slate-400">Instance not found</p>
        <GlassButton variant="secondary" onClick={() => navigate("/instances")}>
          Back to Instances
        </GlassButton>
      </div>
    );
  }

  const statusType =
    typeof instance.status === "string"
      ? instance.status
      : Object.keys(instance.status)[0];

  const isRunning = statusType === "running";
  const isInstalling = statusType === "installing";
  const isReady = statusType === "ready";

  const handleSaveSettings = async () => {
    setIsSaving(true);
    try {
      await updateInstance({
        id: instance.id,
        dto: {
          name: editName,
          javaPath: editJavaPath,
          minRam: editMinRam,
          maxRam: editMaxRam,
        },
      });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2500);
      refetch();
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async () => {
    if (confirm(`Are you sure you want to delete instance "${instance.name}"? This cannot be undone.`)) {
      await deleteInstance(instance.id);
      navigate("/instances");
    }
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto">
      {/* Back button */}
      <button
        onClick={() => navigate("/instances")}
        className="flex items-center gap-2 text-xs font-medium text-slate-400 hover:text-white transition-colors"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Instances
      </button>

      {/* Header Panel */}
      <GlassPanel variant="elevated" className="p-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-6">
          <div className="flex items-center gap-4">
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-tr from-blue-600/30 to-indigo-600/30 border border-white/20 text-blue-400 shadow-inner">
              <Box className="h-8 w-8" />
            </div>

            <div className="space-y-1">
              <div className="flex items-center gap-2.5">
                <h1 className="text-2xl font-bold text-white tracking-tight">{instance.name}</h1>
                <GlassBadge variant={isRunning ? "success" : isReady ? "default" : "warning"}>
                  {isRunning ? "Running" : isReady ? "Ready" : isInstalling ? "Installing" : "Idle"}
                </GlassBadge>
              </div>

              <div className="flex items-center gap-3 text-xs text-slate-400">
                <span>MC {instance.minecraftVersion}</span>
                <span>•</span>
                <span className="capitalize">{instance.loader}</span>
                <span>•</span>
                <span>ID: {instance.id}</span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <GlassButton
              variant="secondary"
              size="md"
              onClick={() => instancesApi.openFolder(instance.gameDirectory)}
              title="Open Folder in Explorer"
            >
              <FolderOpen className="h-4 w-4" />
              Folder
            </GlassButton>

            <GlassButton
              variant={isReady ? "primary" : "secondary"}
              size="md"
              disabled={isRunning || isInstalling}
              onClick={() => {
                if (isReady) launchInstance(instance.id);
                else installInstance(instance.id);
              }}
              className="px-6"
            >
              {isInstalling ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin mr-1.5" />
                  Installing...
                </>
              ) : isRunning ? (
                "In Game"
              ) : isReady ? (
                <>
                  <Play className="h-4 w-4 fill-current mr-1.5" />
                  Play
                </>
              ) : (
                "Install"
              )}
            </GlassButton>
          </div>
        </div>

        {/* Tab selector */}
        <div className="flex items-center gap-2 mt-6 pt-4 border-t border-white/10 text-xs">
          {[
            { id: "overview", label: "Overview", icon: Info },
            { id: "mods", label: "Mods", icon: Puzzle, badge: "Soon" },
            { id: "settings", label: "Settings", icon: Settings },
            { id: "logs", label: "Logs", icon: Terminal },
          ].map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`flex items-center gap-2 px-4 py-2 rounded-xl font-medium transition-all ${
                  isActive
                    ? "bg-blue-600/20 text-blue-400 border border-blue-500/30 shadow-inner"
                    : "text-slate-400 hover:text-white hover:bg-white/5 border border-transparent"
                }`}
              >
                <Icon className="h-3.5 w-3.5" />
                {tab.label}
                {tab.badge && (
                  <span className="text-[10px] px-1.5 py-0.2 rounded bg-white/10 text-slate-400">
                    {tab.badge}
                  </span>
                )}
              </button>
            );
          })}
        </div>
      </GlassPanel>

      {/* Tab: Overview */}
      {activeTab === "overview" && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <GlassCard interactive={false} className="p-5 space-y-4">
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <Cpu className="h-4 w-4 text-blue-400" />
              Runtime Configuration
            </h3>
            <div className="space-y-2.5 text-xs text-slate-300">
              <div className="flex justify-between py-1.5 border-b border-white/5">
                <span className="text-slate-400">Minecraft Version</span>
                <span className="font-mono font-medium text-white">{instance.minecraftVersion}</span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-white/5">
                <span className="text-slate-400">Mod Loader</span>
                <span className="font-medium text-white capitalize">{instance.loader}</span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-white/5">
                <span className="text-slate-400">Memory Allocation</span>
                <span className="font-medium text-white">
                  {instance.ram.minMb / 1024} GB - {instance.ram.maxMb / 1024} GB
                </span>
              </div>
              <div className="flex justify-between py-1.5">
                <span className="text-slate-400">Java Executable</span>
                <span className="font-mono text-slate-400 truncate max-w-[200px]">
                  {instance.javaPath || "Auto-detected"}
                </span>
              </div>
            </div>
          </GlassCard>

          <GlassCard interactive={false} className="p-5 space-y-4">
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <HardDrive className="h-4 w-4 text-indigo-400" />
              Storage & Isolation
            </h3>
            <div className="space-y-2.5 text-xs text-slate-300">
              <div className="flex justify-between py-1.5 border-b border-white/5">
                <span className="text-slate-400">Game Directory</span>
                <span className="font-mono text-slate-400 truncate max-w-[220px]" title={instance.gameDirectory}>
                  {instance.gameDirectory}
                </span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-white/5">
                <span className="text-slate-400">Total Play Time</span>
                <span className="font-medium text-white">
                  {formatDuration(instance.totalPlayTimeSeconds)}
                </span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-white/5">
                <span className="text-slate-400">Last Played</span>
                <span className="font-medium text-white">{formatDate(instance.lastPlayedAt)}</span>
              </div>
              <div className="flex justify-between py-1.5">
                <span className="text-slate-400">Created At</span>
                <span className="font-medium text-slate-400">{formatDate(instance.createdAt)}</span>
              </div>
            </div>
          </GlassCard>
        </div>
      )}

      {/* Tab: Mods (Coming Soon) */}
      {activeTab === "mods" && (
        <GlassPanel variant="subtle" className="p-12 text-center space-y-4">
          <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-amber-500/10 border border-amber-500/20 text-amber-400">
            <Puzzle className="h-8 w-8" />
          </div>
          <div className="max-w-md mx-auto space-y-2">
            <h3 className="text-lg font-bold text-white">Mod Management — Coming Soon</h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              In subsequent phases, you will be able to search and install mods and modpacks directly from Modrinth and CurseForge.
            </p>
            <p className="text-xs text-slate-500 bg-white/5 p-3 rounded-xl border border-white/10 font-mono">
              Architecture ready: ModProviderPort & ModLoaderInstallerPort are fully defined in the Rust core.
            </p>
          </div>
        </GlassPanel>
      )}

      {/* Tab: Settings */}
      {activeTab === "settings" && (
        <GlassCard interactive={false} className="p-6 space-y-6">
          <h3 className="text-base font-semibold text-white">Instance Configuration</h3>

          <div className="space-y-4 max-w-xl">
            <GlassInput
              label="Instance Display Name"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
            />

            <GlassInput
              label="Custom Java Path (Leave blank for default auto-detection)"
              placeholder="e.g. C:\Program Files\Java\jdk-21\bin\java.exe"
              value={editJavaPath}
              onChange={(e) => setEditJavaPath(e.target.value)}
            />

            <div>
              <div className="flex justify-between text-xs mb-2">
                <span className="text-slate-300 font-medium">Maximum Memory (RAM)</span>
                <span className="text-blue-400 font-mono font-semibold">{editMaxRam / 1024} GB</span>
              </div>
              <input
                type="range"
                min={1024}
                max={16384}
                step={1024}
                value={editMaxRam}
                onChange={(e) => setEditMaxRam(Number(e.target.value))}
                className="w-full accent-blue-500 cursor-pointer"
              />
            </div>

            <div className="flex items-center gap-3 pt-2">
              <GlassButton
                variant="primary"
                onClick={handleSaveSettings}
                isLoading={isSaving}
              >
                <Save className="h-4 w-4 mr-1.5" />
                Save Changes
              </GlassButton>
              {saveSuccess && (
                <span className="text-xs text-emerald-400 font-medium animate-in fade-in">
                  Saved successfully!
                </span>
              )}
            </div>
          </div>

          <div className="pt-6 border-t border-white/10">
            <h4 className="text-xs font-semibold text-red-400 uppercase tracking-wider mb-2">
              Danger Zone
            </h4>
            <p className="text-xs text-slate-400 mb-3">
              Deleting this instance will permanently remove all isolated worlds, options, and data.
            </p>
            <GlassButton variant="danger" size="sm" onClick={handleDelete}>
              <Trash2 className="h-3.5 w-3.5 mr-1.5" />
              Delete Instance
            </GlassButton>
          </div>
        </GlassCard>
      )}

      {/* Tab: Logs */}
      {activeTab === "logs" && (
        <GlassCard interactive={false} className="p-4 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-slate-300">
              Game Console Logs ({logs.length} entries)
            </span>
            <GlassButton variant="ghost" size="sm" onClick={clear}>
              Clear
            </GlassButton>
          </div>

          <div className="h-96 overflow-y-auto rounded-2xl bg-black/60 border border-white/10 p-4 font-['JetBrains_Mono'] text-xs leading-relaxed space-y-1 select-text">
            {logs.length === 0 ? (
              <div className="text-slate-500 italic">No logs recorded yet. Launch the game to see live logs.</div>
            ) : (
              logs.map((l, i) => (
                <div key={i} className="flex items-start gap-2">
                  <span className="text-slate-500 shrink-0 select-none">[{l.timestamp}]</span>
                  <span
                    className={`font-semibold shrink-0 select-none ${
                      l.level === "ERROR"
                        ? "text-red-400"
                        : l.level === "WARN"
                        ? "text-amber-400"
                        : "text-blue-400"
                    }`}
                  >
                    [{l.level}]
                  </span>
                  <span className="text-slate-200 break-all">{l.message}</span>
                </div>
              ))
            )}
          </div>
        </GlassCard>
      )}
    </div>
  );
};