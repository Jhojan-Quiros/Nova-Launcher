import React, { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  Package,
  ArrowLeft,
  Download,
  Sparkles,
  ArrowUpCircle,
  CheckCircle2,
  Wrench,
  Layers,
  User,
  ShieldCheck,
  FileText,
  Loader2,
} from "lucide-react";
import { GlassCard, GlassButton, GlassBadge } from "@/components/ui/glass";
import { useModpackStore } from "../store/useModpackStore";
import { modpacksApi } from "@/services/tauri/modpacksApi";
import { ModpackInstallModal } from "../components/ModpackInstallModal";
import { ModpackUpdateModal } from "../components/ModpackUpdateModal";
import { ModpackRepairModal } from "../components/ModpackRepairModal";
import { formatBytes } from "@/utils/formatters";
import type {
  ModpackMainManifest,
  ModpackVersionManifest,
} from "@/types";


export const ModpackDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { catalog, fetchCatalog } = useModpackStore();
  const catalogItem = catalog.find(
    (i) => i.modpack && (i.modpack.id === id || i.modpack.slug === id)
  );

  const [mainManifest, setMainManifest] = useState<ModpackMainManifest | null>(null);
  const [versionManifest, setVersionManifest] = useState<ModpackVersionManifest | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);
  const [activeTab, setActiveTab] = useState<"overview" | "changelog" | "versions">("overview");

  const [installModalOpen, setInstallModalOpen] = useState(false);
  const [updateModalOpen, setUpdateModalOpen] = useState(false);
  const [repairModalOpen, setRepairModalOpen] = useState(false);

  useEffect(() => {
    if (catalog.length === 0) {
      fetchCatalog();
    }
  }, [catalog.length, fetchCatalog]);

  useEffect(() => {
    if (!catalogItem?.modpack.manifestUrl) return;

    setIsLoadingDetails(true);
    modpacksApi
      .getDetails(catalogItem.modpack.manifestUrl)
      .then(([main, ver]) => {
        setMainManifest(main);
        setVersionManifest(ver);
        setIsLoadingDetails(false);
      })
      .catch((err) => {
        console.error("Failed to load modpack details:", err);
        setIsLoadingDetails(false);
      });
  }, [catalogItem?.modpack.manifestUrl]);

  if (!catalogItem || !catalogItem.modpack) {
    return (
      <div className="flex flex-col items-center justify-center py-20 space-y-4">
        <Package className="h-12 w-12 text-slate-500" />
        <h2 className="text-lg font-semibold text-white">Modpack not found</h2>
        <GlassButton variant="secondary" onClick={() => navigate("/modpacks")}>
          <ArrowLeft className="h-4 w-4 mr-1.5" /> Back to Modpacks
        </GlassButton>
      </div>
    );
  }

  const { modpack, status, installedVersion, updateAvailable, associatedInstanceId } =
    catalogItem;

  return (
    <div className="space-y-6 pb-12">
      {/* Back button */}
      <div>
        <GlassButton
          variant="ghost"
          size="sm"
          onClick={() => navigate("/modpacks")}
          className="text-slate-400 hover:text-white"
        >
          <ArrowLeft className="h-4 w-4 mr-1.5" />
          Back to Modpacks
        </GlassButton>
      </div>

      {/* Hero Card */}
      <GlassCard className="p-0 overflow-hidden relative border-white/15 shadow-2xl">
        {/* Banner image or background gradient */}
        <div className="relative h-60 w-full overflow-hidden bg-gradient-to-r from-slate-900 via-indigo-950/60 to-slate-900">
          {modpack.bannerUrl ? (
            <img
              src={modpack.bannerUrl}
              alt={modpack.name}
              className="h-full w-full object-cover"
              onError={(e) => {
                (e.target as HTMLImageElement).style.display = "none";
              }}
            />
          ) : (
            <div className="absolute inset-0 bg-gradient-to-tr from-blue-900/40 via-indigo-900/30 to-purple-900/40" />
          )}

          {/* Gradient overlay */}
          <div className="absolute inset-0 bg-gradient-to-t from-[#11131c] via-[#11131c]/60 to-transparent" />

          {/* Content inside Banner */}
          <div className="absolute bottom-6 left-6 right-6 flex flex-col md:flex-row md:items-end justify-between gap-4">
            <div className="flex items-end gap-5">
              <div className="h-20 w-20 rounded-2xl overflow-hidden bg-black/60 border border-white/20 backdrop-blur-xl shadow-2xl shrink-0 flex items-center justify-center">
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
                  <Package className="h-10 w-10 text-blue-400" />
                )}
              </div>

              <div className="space-y-1">
                <div className="flex items-center gap-3">
                  <h1 className="text-2xl md:text-3xl font-bold text-white tracking-tight drop-shadow-md">
                    {modpack.name}
                  </h1>
                  {updateAvailable && (
                    <GlassBadge variant="warning" className="shadow-glow-sm">
                      <Sparkles className="h-3 w-3 mr-1" />
                      Update Available
                    </GlassBadge>
                  )}
                  {status === "Installed" && !updateAvailable && (
                    <GlassBadge variant="success">
                      <CheckCircle2 className="h-3 w-3 mr-1" />
                      Installed (v{installedVersion})
                    </GlassBadge>
                  )}
                </div>

                <div className="flex flex-wrap items-center gap-3 text-xs text-slate-300">
                  <span className="flex items-center gap-1.5">
                    <User className="h-3.5 w-3.5 text-slate-400" />
                    {modpack.author || "Community"}
                  </span>
                  <span>•</span>
                  <span className="flex items-center gap-1.5">
                    <Layers className="h-3.5 w-3.5 text-slate-400" />
                    MC {modpack.minecraftVersion}
                  </span>
                  <span>•</span>
                  <span className="capitalize font-medium text-blue-300">
                    {modpack.loader}
                  </span>
                  <span>•</span>
                  <span className="text-slate-400">
                    Latest v{modpack.latestVersion}
                  </span>
                </div>
              </div>
            </div>

            {/* Main Hero Action Buttons */}
            <div className="flex items-center gap-2.5">
              {updateAvailable ? (
                <GlassButton
                  variant="primary"
                  className="bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border-amber-500/40 shadow-glow-warning"
                  onClick={() => setUpdateModalOpen(true)}
                >
                  <ArrowUpCircle className="h-4 w-4 mr-2" />
                  Update to v{modpack.latestVersion}
                </GlassButton>
              ) : status === "Installed" ? (
                <>
                  {associatedInstanceId && (
                    <GlassButton
                      variant="secondary"
                      onClick={() => navigate(`/instances/${associatedInstanceId}`)}
                    >
                      Go to Instance
                    </GlassButton>
                  )}
                  <GlassButton
                    variant="secondary"
                    onClick={() => setRepairModalOpen(true)}
                    title="Scan and repair corrupted or missing files"
                  >
                    <Wrench className="h-4 w-4 mr-1.5 text-amber-400" />
                    Verify & Repair
                  </GlassButton>
                </>
              ) : (
                <GlassButton
                  variant="primary"
                  onClick={() => setInstallModalOpen(true)}
                >
                  <Download className="h-4 w-4 mr-2" />
                  Install Modpack
                </GlassButton>
              )}
            </div>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center gap-2 px-6 pt-3 border-t border-white/10 bg-[#11131c]/60 backdrop-blur-md">
          <button
            onClick={() => setActiveTab("overview")}
            className={`pb-3 text-xs font-semibold transition-colors border-b-2 px-1 ${
              activeTab === "overview"
                ? "text-blue-400 border-blue-500"
                : "text-slate-400 hover:text-slate-200 border-transparent"
            }`}
          >
            Overview
          </button>
          <button
            onClick={() => setActiveTab("changelog")}
            className={`pb-3 text-xs font-semibold transition-colors border-b-2 px-1 ${
              activeTab === "changelog"
                ? "text-blue-400 border-blue-500"
                : "text-slate-400 hover:text-slate-200 border-transparent"
            }`}
          >
            Changelog
          </button>
          <button
            onClick={() => setActiveTab("versions")}
            className={`pb-3 text-xs font-semibold transition-colors border-b-2 px-1 ${
              activeTab === "versions"
                ? "text-blue-400 border-blue-500"
                : "text-slate-400 hover:text-slate-200 border-transparent"
            }`}
          >
            Version History
          </button>
        </div>
      </GlassCard>

      {/* Tab Contents */}
      {isLoadingDetails ? (
        <div className="flex flex-col items-center justify-center py-16 space-y-3">
          <Loader2 className="h-8 w-8 text-blue-400 animate-spin" />
          <p className="text-xs text-slate-400">Loading modpack manifest...</p>
        </div>
      ) : activeTab === "overview" ? (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Main Content (2 cols) */}
          <div className="lg:col-span-2 space-y-6">
            <GlassCard className="p-6 space-y-4">
              <h3 className="text-sm font-semibold text-white">About this Modpack</h3>
              <p className="text-xs text-slate-300 leading-relaxed whitespace-pre-line">
                {modpack.description || "No description provided."}
              </p>

              {modpack.tags && modpack.tags.length > 0 && (
                <div className="pt-2 flex flex-wrap gap-2">
                  {modpack.tags.map((tag) => (
                    <GlassBadge key={tag} variant="default">
                      {tag}
                    </GlassBadge>
                  ))}
                </div>
              )}
            </GlassCard>

            {/* Version Stats */}
            {versionManifest?.stats && (
              <GlassCard className="p-6 space-y-4">
                <h3 className="text-sm font-semibold text-white">Package Information</h3>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 text-center">
                  <div className="p-3 rounded-xl bg-white/[0.03] border border-white/10">
                    <span className="text-[11px] text-slate-400 block mb-1">Total Files</span>
                    <span className="text-base font-bold text-white">
                      {versionManifest.stats.totalFiles}
                    </span>
                  </div>
                  <div className="p-3 rounded-xl bg-white/[0.03] border border-white/10">
                    <span className="text-[11px] text-slate-400 block mb-1">Total Size</span>
                    <span className="text-base font-bold text-white">
                      {formatBytes(versionManifest.stats.totalSize)}
                    </span>
                  </div>
                  <div className="p-3 rounded-xl bg-white/[0.03] border border-white/10">
                    <span className="text-[11px] text-slate-400 block mb-1">Added in v{versionManifest.version}</span>
                    <span className="text-base font-bold text-emerald-400">
                      +{versionManifest.stats.added}
                    </span>
                  </div>
                  <div className="p-3 rounded-xl bg-white/[0.03] border border-white/10">
                    <span className="text-[11px] text-slate-400 block mb-1">Updated</span>
                    <span className="text-base font-bold text-blue-400">
                      {versionManifest.stats.updated}
                    </span>
                  </div>
                </div>
              </GlassCard>
            )}
          </div>

          {/* Sidebar Specifications */}
          <div className="space-y-6">
            <GlassCard className="p-5 space-y-4">
              <h3 className="text-xs font-semibold text-slate-300 uppercase tracking-wider">
                Specifications
              </h3>

              <div className="space-y-3 text-xs">
                <div className="flex justify-between py-1.5 border-b border-white/[0.06]">
                  <span className="text-slate-400">Minecraft Version</span>
                  <span className="font-semibold text-white">{modpack.minecraftVersion}</span>
                </div>
                <div className="flex justify-between py-1.5 border-b border-white/[0.06]">
                  <span className="text-slate-400">Mod Loader</span>
                  <span className="font-semibold text-white capitalize">{modpack.loader}</span>
                </div>
                {modpack.loaderVersion && (
                  <div className="flex justify-between py-1.5 border-b border-white/[0.06]">
                    <span className="text-slate-400">Loader Version</span>
                    <span className="font-semibold text-white">{modpack.loaderVersion}</span>
                  </div>
                )}
                <div className="flex justify-between py-1.5 border-b border-white/[0.06]">
                  <span className="text-slate-400">Author</span>
                  <span className="font-semibold text-white">{modpack.author || "Community"}</span>
                </div>
                <div className="flex justify-between py-1.5 border-b border-white/[0.06]">
                  <span className="text-slate-400">Remote CDN</span>
                  <span className="font-semibold text-emerald-400 flex items-center gap-1">
                    <ShieldCheck className="h-3.5 w-3.5" />
                    Verified R2
                  </span>
                </div>
              </div>
            </GlassCard>
          </div>
        </div>
      ) : activeTab === "changelog" ? (
        <GlassCard className="p-6 space-y-4">
          <div className="flex items-center gap-2">
            <FileText className="h-4 w-4 text-blue-400" />
            <h3 className="text-sm font-semibold text-white">
              Release Notes (v{versionManifest?.version || modpack.latestVersion})
            </h3>
          </div>

          {versionManifest?.changelog && versionManifest.changelog.length > 0 ? (
            <div className="space-y-2 text-xs text-slate-300">
              {versionManifest.changelog.map((entry, idx) => (
                <div key={idx} className="flex items-start gap-2.5">
                  <span className="text-blue-400 font-bold">•</span>
                  <span>{entry}</span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-xs text-slate-400">
              No changelog provided for this release.
            </p>
          )}
        </GlassCard>
      ) : (
        <GlassCard className="p-6 space-y-4">
          <h3 className="text-sm font-semibold text-white">Available Versions</h3>
          <div className="divide-y divide-white/10">
            {mainManifest?.versions && mainManifest.versions.length > 0 ? (
              mainManifest.versions.map((ver) => (
                <div
                  key={ver.version}
                  className="py-3 flex items-center justify-between gap-4"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-sm text-white">
                        v{ver.version}
                      </span>
                      {ver.version === modpack.latestVersion && (
                        <GlassBadge variant="info">Latest</GlassBadge>
                      )}
                      {ver.version === installedVersion && (
                        <GlassBadge variant="success">Current</GlassBadge>
                      )}
                    </div>
                    <p className="text-xs text-slate-400">
                      Minecraft {ver.minecraftVersion} • {ver.loader}
                    </p>
                  </div>

                  <div>
                    {associatedInstanceId && ver.version !== installedVersion && (
                      <GlassButton
                        variant="secondary"
                        size="sm"
                        onClick={() => {
                          setUpdateModalOpen(true);
                        }}
                      >
                        Switch to v{ver.version}
                      </GlassButton>
                    )}
                  </div>
                </div>
              ))
            ) : (
              <div className="py-3 flex items-center justify-between gap-4">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-sm text-white">
                      v{mainManifest?.latestVersion || modpack.latestVersion}
                    </span>
                    <GlassBadge variant="info">Latest</GlassBadge>
                    {modpack.latestVersion === installedVersion && (
                      <GlassBadge variant="success">Current</GlassBadge>
                    )}
                  </div>
                  <p className="text-xs text-slate-400">
                    Minecraft {mainManifest?.minecraftVersion || modpack.minecraftVersion} • {mainManifest?.loader || modpack.loader}
                  </p>
                </div>
              </div>
            )}
          </div>
        </GlassCard>
      )}

      {/* Modals */}
      <ModpackInstallModal
        isOpen={installModalOpen}
        onClose={() => setInstallModalOpen(false)}
        item={catalogItem}
        onSuccess={(instanceId) => {
          navigate(`/instances/${instanceId}`);
        }}
      />

      {associatedInstanceId && (
        <ModpackUpdateModal
          isOpen={updateModalOpen}
          onClose={() => setUpdateModalOpen(false)}
          instanceId={associatedInstanceId}
          targetVersion={modpack.latestVersion}
          onSuccess={() => {
            fetchCatalog(true);
          }}
        />
      )}

      {associatedInstanceId && (
        <ModpackRepairModal
          isOpen={repairModalOpen}
          onClose={() => setRepairModalOpen(false)}
          instanceId={associatedInstanceId}
        />
      )}
    </div>
  );
};
