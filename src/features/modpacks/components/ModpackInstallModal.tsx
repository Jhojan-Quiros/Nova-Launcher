import React, { useState } from "react";
import { GlassModal, GlassButton, GlassInput } from "@/components/ui/glass";
import { Download, ShieldCheck, Cpu } from "lucide-react";

import { modpacksApi } from "@/services/tauri/modpacksApi";
import { useModpackStore } from "../store/useModpackStore";
import type { CatalogItemWithStatus } from "@/types";

interface ModpackInstallModalProps {
  isOpen: boolean;
  onClose: () => void;
  item: CatalogItemWithStatus | null;
  onSuccess?: (instanceId: string) => void;
}

export const ModpackInstallModal: React.FC<ModpackInstallModalProps> = ({
  isOpen,
  onClose,
  item,
  onSuccess,
}) => {
  if (!item) return null;

  const { modpack } = item;
  const [instanceName, setInstanceName] = useState(modpack.name);
  const [minRamGb, setMinRamGb] = useState(2);
  const [maxRamGb, setMaxRamGb] = useState(4);
  const [strictMode, setStrictMode] = useState(false);
  const [autoUpdate, setAutoUpdate] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const fetchCatalog = useModpackStore((s) => s.fetchCatalog);

  const handleInstall = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setErrorMessage(null);

    try {
      const installed = await modpacksApi.install({
        catalogPack: modpack,
        instanceName: instanceName.trim() || modpack.name,
        customRam: {
          minMb: minRamGb * 1024,
          maxMb: maxRamGb * 1024,
        },
        strictMode,
        autoUpdate,
      });

      await fetchCatalog(true);
      setIsSubmitting(false);
      onClose();
      if (onSuccess) {
        onSuccess(installed.instanceId);
      }
    } catch (err: any) {
      console.error("Installation failed:", err);
      setErrorMessage(
        err?.message || "Failed to install modpack. Please verify network connection."
      );
      setIsSubmitting(false);
    }
  };

  return (
    <GlassModal
      isOpen={isOpen}
      onClose={onClose}
      title={`Install ${modpack.name}`}
      description={`Version ${modpack.latestVersion} • Minecraft ${modpack.minecraftVersion} (${modpack.loader})`}
      maxWidth="md"
    >
      <form onSubmit={handleInstall} className="space-y-5">
        {errorMessage && (
          <div className="p-3 rounded-xl bg-red-500/15 border border-red-500/30 text-xs text-red-300">
            {errorMessage}
          </div>
        )}

        {/* Instance Name */}
        <div className="space-y-1.5">
          <label className="text-xs font-semibold text-slate-300">
            Instance Name
          </label>
          <GlassInput
            value={instanceName}
            onChange={(e) => setInstanceName(e.target.value)}
            placeholder="e.g. My Modpack World"
            required
          />
        </div>

        {/* Memory Allocation */}
        <div className="space-y-3 p-3.5 rounded-2xl bg-white/[0.03] border border-white/10">
          <div className="flex items-center gap-2 text-xs font-semibold text-slate-300">
            <Cpu className="h-4 w-4 text-blue-400" />
            <span>RAM Allocation</span>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Minimum RAM: {minRamGb} GB
              </label>
              <input
                type="range"
                min={1}
                max={16}
                step={1}
                value={minRamGb}
                onChange={(e) => {
                  const val = Number(e.target.value);
                  setMinRamGb(val);
                  if (val > maxRamGb) setMaxRamGb(val);
                }}
                className="w-full accent-blue-500 cursor-pointer"
              />
            </div>
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Maximum RAM: {maxRamGb} GB
              </label>
              <input
                type="range"
                min={2}
                max={32}
                step={1}
                value={maxRamGb}
                onChange={(e) => {
                  const val = Number(e.target.value);
                  setMaxRamGb(val);
                  if (val < minRamGb) setMinRamGb(val);
                }}
                className="w-full accent-blue-500 cursor-pointer"
              />
            </div>
          </div>
        </div>

        {/* Strict Modpack Mode Switch */}
        <div className="flex items-start justify-between gap-3 p-3.5 rounded-2xl bg-white/[0.03] border border-white/10">
          <div className="space-y-0.5">
            <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-200">
              <ShieldCheck className="h-4 w-4 text-amber-400" />
              <span>Strict Modpack Mode</span>
            </div>
            <p className="text-[11px] text-slate-400 leading-normal">
              When enabled, only files defined in the official manifest are kept.
              Any custom mods added manually to <code className="text-blue-300">mods/</code> will be cleaned up on update.
            </p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={strictMode}
            onClick={() => setStrictMode(!strictMode)}
            className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
              strictMode ? "bg-blue-600" : "bg-white/10"
            }`}
          >
            <span
              className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out ${
                strictMode ? "translate-x-5" : "translate-x-0"
              }`}
            />
          </button>
        </div>

        {/* Auto Update Check Switch */}
        <div className="flex items-start justify-between gap-3 p-3.5 rounded-2xl bg-white/[0.03] border border-white/10">
          <div className="space-y-0.5">
            <span className="text-xs font-semibold text-slate-200 block">
              Enable Update Checks
            </span>
            <p className="text-[11px] text-slate-400 leading-normal">
              Automatically check remote manifests and notify when new versions are available.
            </p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={autoUpdate}
            onClick={() => setAutoUpdate(!autoUpdate)}
            className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
              autoUpdate ? "bg-blue-600" : "bg-white/10"
            }`}
          >
            <span
              className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out ${
                autoUpdate ? "translate-x-5" : "translate-x-0"
              }`}
            />
          </button>
        </div>

        {/* Modal Actions */}
        <div className="flex items-center justify-end gap-3 pt-2">

          <GlassButton
            type="button"
            variant="ghost"
            onClick={onClose}
            disabled={isSubmitting}
          >
            Cancel
          </GlassButton>
          <GlassButton
            type="submit"
            variant="primary"
            isLoading={isSubmitting}
          >
            <Download className="h-4 w-4 mr-1.5" />
            Start Installation
          </GlassButton>
        </div>
      </form>
    </GlassModal>
  );
};
