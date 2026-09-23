import React, { useEffect, useState, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import {
  Package,
  Search,
  RefreshCw,
  Sparkles,
  AlertCircle,
  Settings,
} from "lucide-react";

import { GlassCard, GlassButton, GlassInput } from "@/components/ui/glass";
import { useModpackStore } from "../store/useModpackStore";
import { ModpackCard } from "../components/ModpackCard";
import { ModpackInstallModal } from "../components/ModpackInstallModal";
import { ModpackUpdateModal } from "../components/ModpackUpdateModal";
import type { CatalogItemWithStatus } from "@/types";

export const ModpacksPage: React.FC = () => {
  const navigate = useNavigate();
  const {
    catalog,
    isLoadingCatalog,
    catalogError,
    fetchCatalog,
    checkAllUpdates,
  } = useModpackStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedFilter, setSelectedFilter] = useState<"all" | "installed" | "updates" | "fabric" | "forge">("all");
  const [installModalItem, setInstallModalItem] = useState<CatalogItemWithStatus | null>(null);
  const [updateModalInstanceId, setUpdateModalInstanceId] = useState<string | null>(null);
  const [updateTargetVersion, setUpdateTargetVersion] = useState<string | undefined>(undefined);

  useEffect(() => {
    fetchCatalog();
  }, [fetchCatalog]);

  const filteredCatalog = useMemo(() => {
    return catalog.filter((item) => {
      const q = searchQuery.toLowerCase();
      const matchesSearch =
        !q ||
        item.modpack.name.toLowerCase().includes(q) ||
        (item.modpack.description?.toLowerCase().includes(q) ?? false) ||
        (item.modpack.author?.toLowerCase().includes(q) ?? false) ||
        (item.modpack.tags?.some((t) => t.toLowerCase().includes(q)) ?? false);

      if (!matchesSearch) return false;

      if (selectedFilter === "installed") {
        return item.status === "Installed";
      }
      if (selectedFilter === "updates") {
        return item.updateAvailable;
      }
      if (selectedFilter === "fabric") {
        return item.modpack.loader.toLowerCase() === "fabric";
      }
      if (selectedFilter === "forge") {
        return (
          item.modpack.loader.toLowerCase() === "forge" ||
          item.modpack.loader.toLowerCase() === "neoforge"
        );
      }
      return true;
    });
  }, [catalog, searchQuery, selectedFilter]);

  const updateCount = catalog.filter((i) => i.updateAvailable).length;

  return (
    <div className="space-y-6 pb-12">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2.5">
            <h1 className="text-2xl font-bold tracking-tight text-white">Modpacks</h1>
            {updateCount > 0 && (
              <span className="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-500/20 text-amber-300 border border-amber-500/30 shadow-glow-warning animate-pulse">
                <Sparkles className="h-3 w-3" />
                {updateCount} Update{updateCount > 1 ? "s" : ""} Available
              </span>
            )}
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Browse, install, and update curated modpacks managed remotely via Cloudflare R2 / CDN.
          </p>
        </div>

        <div className="flex items-center gap-2.5">
          <GlassButton
            variant="secondary"
            size="sm"
            onClick={() => checkAllUpdates()}
            title="Check for updates on all installed modpacks"
          >
            <Sparkles className="h-4 w-4 mr-1.5 text-amber-400" />
            Check Updates
          </GlassButton>
          <GlassButton
            variant="secondary"
            size="sm"
            isLoading={isLoadingCatalog}
            onClick={() => fetchCatalog(true)}
          >
            <RefreshCw className="h-4 w-4 mr-1.5" />
            Refresh Catalog
          </GlassButton>
        </div>
      </div>

      {/* Search & Filter Bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400 pointer-events-none" />
          <GlassInput
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search modpacks by name, category, author, or tags..."
            className="pl-10 h-10 w-full"
          />
        </div>

        {/* Filter Pills */}
        <div className="flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0">
          <button
            onClick={() => setSelectedFilter("all")}
            className={`px-3 py-1.5 rounded-xl text-xs font-medium transition-all ${
              selectedFilter === "all"
                ? "bg-blue-600/30 text-blue-300 border border-blue-500/40"
                : "bg-white/[0.04] text-slate-400 hover:text-slate-200 border border-white/10"
            }`}
          >
            All
          </button>
          <button
            onClick={() => setSelectedFilter("updates")}
            className={`px-3 py-1.5 rounded-xl text-xs font-medium transition-all flex items-center gap-1.5 ${
              selectedFilter === "updates"
                ? "bg-amber-500/30 text-amber-300 border border-amber-500/40"
                : "bg-white/[0.04] text-slate-400 hover:text-slate-200 border border-white/10"
            }`}
          >
            <Sparkles className="h-3 w-3" />
            Updates {updateCount > 0 && `(${updateCount})`}
          </button>
          <button
            onClick={() => setSelectedFilter("installed")}
            className={`px-3 py-1.5 rounded-xl text-xs font-medium transition-all ${
              selectedFilter === "installed"
                ? "bg-emerald-500/30 text-emerald-300 border border-emerald-500/40"
                : "bg-white/[0.04] text-slate-400 hover:text-slate-200 border border-white/10"
            }`}
          >
            Installed
          </button>
          <button
            onClick={() => setSelectedFilter("fabric")}
            className={`px-3 py-1.5 rounded-xl text-xs font-medium transition-all ${
              selectedFilter === "fabric"
                ? "bg-blue-600/30 text-blue-300 border border-blue-500/40"
                : "bg-white/[0.04] text-slate-400 hover:text-slate-200 border border-white/10"
            }`}
          >
            Fabric
          </button>
          <button
            onClick={() => setSelectedFilter("forge")}
            className={`px-3 py-1.5 rounded-xl text-xs font-medium transition-all ${
              selectedFilter === "forge"
                ? "bg-blue-600/30 text-blue-300 border border-blue-500/40"
                : "bg-white/[0.04] text-slate-400 hover:text-slate-200 border border-white/10"
            }`}
          >
            Forge
          </button>
        </div>
      </div>

      {/* Error state */}
      {catalogError && (
        <GlassCard className="p-6 border-red-500/30 bg-red-500/10 space-y-4">
          <div className="flex items-start gap-3">
            <AlertCircle className="h-6 w-6 text-red-400 shrink-0 mt-0.5" />
            <div className="space-y-1">
              <h3 className="text-sm font-semibold text-white">
                Unable to Load Remote Modpack Catalog
              </h3>
              <p className="text-xs text-red-300/80 leading-relaxed">
                {catalogError}
              </p>
              <p className="text-xs text-slate-400 pt-1">
                Make sure your Modpack Catalog URL is configured correctly in Settings.
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3 pt-2">
            <GlassButton
              variant="secondary"
              size="sm"
              onClick={() => fetchCatalog(true)}
            >
              <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
              Retry Connection
            </GlassButton>
            <GlassButton
              variant="ghost"
              size="sm"
              onClick={() => navigate("/settings")}
            >
              <Settings className="h-3.5 w-3.5 mr-1.5" />
              Open Settings
            </GlassButton>
          </div>
        </GlassCard>
      )}

      {/* Catalog Grid */}
      {filteredCatalog.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {filteredCatalog.map((item) => (
            <ModpackCard
              key={item.modpack.id}
              item={item}
              onInstall={(packItem) => setInstallModalItem(packItem)}
              onUpdate={(packItem) => {
                if (packItem.associatedInstanceId) {
                  setUpdateModalInstanceId(packItem.associatedInstanceId);
                  setUpdateTargetVersion(packItem.modpack.latestVersion);
                }
              }}
            />
          ))}
        </div>
      ) : !isLoadingCatalog && !catalogError ? (
        <div className="flex flex-col items-center justify-center py-20 text-center space-y-3">
          <div className="h-16 w-16 rounded-3xl bg-white/[0.04] border border-white/10 flex items-center justify-center text-slate-500">
            <Package className="h-8 w-8" />
          </div>
          <div className="space-y-1">
            <h3 className="text-base font-semibold text-white">No Modpacks Found</h3>
            <p className="text-xs text-slate-400 max-w-sm">
              {searchQuery
                ? `No modpacks matched "${searchQuery}". Try changing your search or filters.`
                : "No modpacks are available in the configured remote catalog."}
            </p>
          </div>
          {searchQuery && (
            <GlassButton
              variant="secondary"
              size="sm"
              onClick={() => setSearchQuery("")}
            >
              Clear Search
            </GlassButton>
          )}
        </div>
      ) : null}

      {/* Modals */}
      <ModpackInstallModal
        isOpen={Boolean(installModalItem)}
        onClose={() => setInstallModalItem(null)}
        item={installModalItem}
        onSuccess={(instanceId) => {
          navigate(`/instances/${instanceId}`);
        }}
      />

      <ModpackUpdateModal
        isOpen={Boolean(updateModalInstanceId)}
        onClose={() => {
          setUpdateModalInstanceId(null);
          setUpdateTargetVersion(undefined);
        }}
        instanceId={updateModalInstanceId}
        targetVersion={updateTargetVersion}
        onSuccess={() => {
          fetchCatalog(true);
        }}
      />
    </div>
  );
};
