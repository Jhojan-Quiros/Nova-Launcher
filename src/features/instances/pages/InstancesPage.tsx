import React, { useState } from "react";
import { Plus, Search, Layers, RefreshCw } from "lucide-react";
import { GlassButton } from "@/components/ui/glass";
import { useInstances } from "@/features/instances/hooks/useInstances";
import { InstanceCard } from "@/features/instances/components/InstanceCard";
import { CreateInstanceModal } from "@/features/instances/components/CreateInstanceModal";
import { instancesApi } from "@/services/tauri/instancesApi";
import { getErrorMessage } from "@/utils/errors";

export const InstancesPage: React.FC = () => {
  const {
    instances,
    isLoading,
    refetch,
    createInstance,
    deleteInstance,
    installInstance,
    launchInstance,
  } = useInstances();

  const [search, setSearch] = useState("");
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);

  const filtered = instances.filter((i) =>
    i.name.toLowerCase().includes(search.toLowerCase()) ||
    i.minecraftVersion.toLowerCase().includes(search.toLowerCase())
  );

  const runAction = async (action: () => Promise<unknown>, fallback: string) => {
    setActionError(null);
    try {
      await action();
    } catch (err) {
      setActionError(getErrorMessage(err, fallback));
    }
  };

  return (
    <div className="space-y-6 max-w-6xl mx-auto">
      {/* Top bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2.5">
            <Layers className="h-6 w-6 text-blue-400" />
            Instances
          </h1>
          <p className="text-xs text-slate-400 mt-0.5">
            Manage your completely isolated Minecraft installations and worlds.
          </p>
        </div>

        <div className="flex items-center gap-2.5">
          <GlassButton variant="ghost" size="icon" onClick={() => refetch()} title="Refresh">
            <RefreshCw className="h-4 w-4" />
          </GlassButton>
          <GlassButton variant="primary" size="md" onClick={() => setIsCreateOpen(true)}>
            <Plus className="h-4 w-4" />
            New Instance
          </GlassButton>
        </div>
      </div>

      {actionError && (
        <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/30 text-red-300 text-xs font-mono break-all">
          {actionError}
        </div>
      )}

      {/* Search and filters */}
      <div className="relative max-w-md">
        <Search className="absolute left-3.5 top-3 h-4 w-4 text-slate-400" />
        <input
          placeholder="Search instances by name or version..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full pl-10 pr-4 py-2.5 rounded-2xl text-xs bg-white/[0.05] border border-white/10 text-white placeholder:text-slate-500 outline-none focus:border-blue-500/80 transition-all backdrop-blur-md"
        />
      </div>

      {/* Grid */}
      {isLoading ? (
        <div className="py-20 text-center text-xs text-slate-400">
          Loading instances...
        </div>
      ) : filtered.length === 0 ? (
        <div className="py-20 text-center space-y-3">
          <p className="text-sm text-slate-400">
            {search ? "No instances match your search query." : "No instances yet."}
          </p>
          <GlassButton variant="primary" size="sm" onClick={() => setIsCreateOpen(true)}>
            <Plus className="h-4 w-4" />
            Create One Now
          </GlassButton>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {filtered.map((inst) => (
            <InstanceCard
              key={inst.id}
              instance={inst}
              onPlay={(id) => runAction(() => launchInstance(id), "Failed to launch the instance")}
              onInstall={(id) => runAction(() => installInstance(id), "Failed to install the instance")}
              onDelete={(id) => deleteInstance(id)}
              onOpenFolder={(path) => instancesApi.openFolder(path)}
            />
          ))}
        </div>
      )}

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