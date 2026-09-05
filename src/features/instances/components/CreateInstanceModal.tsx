import React, { useEffect, useState } from "react";
import { Check, Sparkles, Box, HardDrive, Cpu, Search, Loader2 } from "lucide-react";
import { GlassModal, GlassButton, GlassInput, GlassCard } from "@/components/ui/glass";
import { useMinecraftVersions } from "@/features/minecraft/hooks/useMinecraftVersions";
import { useForgeVersions } from "@/features/minecraft/hooks/useForgeVersions";
import type { CreateInstanceInput, ModLoader } from "@/types";

interface CreateInstanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (data: CreateInstanceInput) => Promise<void>;
}

export const CreateInstanceModal: React.FC<CreateInstanceModalProps> = ({
  isOpen,
  onClose,
  onCreate,
}) => {
  const [step, setStep] = useState<number>(1);
  const [name, setName] = useState("");
  const [selectedVersion, setSelectedVersion] = useState("1.21.1");
  const [loader, setLoader] = useState<ModLoader>("vanilla");
  const [loaderVersion, setLoaderVersion] = useState<string | undefined>(undefined);
  const [showSnapshots, setShowSnapshots] = useState(false);
  const [versionSearch, setVersionSearch] = useState("");
  const [minRam, setMinRam] = useState(2048);
  const [maxRam, setMaxRam] = useState(4096);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { data: versionData, isLoading: versionsLoading } = useMinecraftVersions({
    showReleases: true,
    showSnapshots,
    showOldBeta: false,
    showOldAlpha: false,
  });

  const versions = versionData?.versions ?? [];
  const filteredVersions = versions.filter((v) =>
    v.id.toLowerCase().includes(versionSearch.toLowerCase())
  );

  const {
    data: forgeVersions,
    isLoading: forgeVersionsLoading,
    isError: forgeVersionsErrored,
  } = useForgeVersions(selectedVersion, loader === "forge");

  // Reset the picked Forge build whenever the loader or MC version changes,
  // and auto-pick the recommended build once options come back.
  useEffect(() => {
    if (loader !== "forge") {
      setLoaderVersion(undefined);
      return;
    }
    if (forgeVersions && forgeVersions.length > 0) {
      const recommended = forgeVersions.find((v) => v.label === "recommended");
      setLoaderVersion(recommended?.version ?? forgeVersions[0].version);
    } else {
      setLoaderVersion(undefined);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loader, selectedVersion, forgeVersions]);

  const handleCreate = async () => {
    if (!name.trim()) {
      setError("Please enter a name for your instance");
      setStep(1);
      return;
    }
    if (loader === "forge" && !loaderVersion) {
      setError("Please select a Forge build for this Minecraft version");
      setStep(3);
      return;
    }
    setError(null);
    setIsSubmitting(true);
    try {
      await onCreate({
        name: name.trim(),
        minecraftVersion: selectedVersion,
        loader,
        loaderVersion: loader === "forge" ? loaderVersion : undefined,
        minRam,
        maxRam,
      });
      // reset
      setName("");
      setStep(1);
      onClose();
    } catch (err: any) {
      setError(err?.message || "Failed to create instance");
    } finally {
      setIsSubmitting(false);
    }
  };

  const stepTitles = [
    "Instance Name",
    "Select Minecraft Version",
    "Choose Mod Loader",
    "Memory (RAM) Allocation",
    "Ready to Create",
  ];

  return (
    <GlassModal
      isOpen={isOpen}
      onClose={onClose}
      title={stepTitles[step - 1]}
      description={`Step ${step} of 5`}
      maxWidth="lg"
    >
      {/* Wizard stepper tabs */}
      <div className="flex items-center justify-between mb-6 px-2">
        {[1, 2, 3, 4, 5].map((s) => (
          <div key={s} className="flex items-center">
            <div
              className={`flex h-8 w-8 items-center justify-center rounded-full text-xs font-semibold transition-all ${
                s === step
                  ? "bg-blue-600 text-white shadow-glow-accent ring-2 ring-blue-400/40"
                  : s < step
                  ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/40"
                  : "bg-white/5 text-slate-400 border border-white/10"
              }`}
            >
              {s < step ? <Check className="h-4 w-4" /> : s}
            </div>
            {s < 5 && (
              <div
                className={`h-0.5 w-10 sm:w-16 mx-1 transition-colors ${
                  s < step ? "bg-emerald-500/40" : "bg-white/10"
                }`}
              />
            )}
          </div>
        ))}
      </div>

      {error && (
        <div className="mb-4 p-3 rounded-xl bg-red-500/15 border border-red-500/30 text-red-300 text-xs font-medium">
          {error}
        </div>
      )}

      {/* Step 1: Name */}
      {step === 1 && (
        <div className="space-y-4 py-2">
          <GlassInput
            label="Instance Name"
            placeholder="e.g. Vanilla Survival 1.21"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />
          <div className="space-y-2 pt-2">
            <span className="text-xs text-slate-400">Suggestions:</span>
            <div className="flex flex-wrap gap-2">
              {["Survival World", "Creative Build", "Hardcore 1.21", "Redstone Testing"].map((s) => (
                <button
                  key={s}
                  type="button"
                  onClick={() => setName(s)}
                  className="px-2.5 py-1 rounded-lg text-xs bg-white/5 hover:bg-white/10 text-slate-300 border border-white/10 transition-colors"
                >
                  {s}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Step 2: Minecraft Version */}
      {step === 2 && (
        <div className="space-y-3 py-1">
          <div className="flex items-center justify-between gap-3">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
              <input
                placeholder="Search versions..."
                value={versionSearch}
                onChange={(e) => setVersionSearch(e.target.value)}
                className="w-full pl-9 pr-3 py-2 rounded-xl text-xs bg-white/5 border border-white/10 text-white placeholder:text-slate-500 outline-none focus:border-blue-500/60"
              />
            </div>
            <label className="flex items-center gap-2 text-xs text-slate-300 cursor-pointer select-none">
              <input
                type="checkbox"
                checked={showSnapshots}
                onChange={(e) => setShowSnapshots(e.target.checked)}
                className="rounded border-white/20 bg-white/10 text-blue-600 focus:ring-0"
              />
              Show Snapshots
            </label>
          </div>

          <div className="h-60 overflow-y-auto rounded-2xl border border-white/10 bg-white/[0.02] p-1.5 space-y-1">
            {versionsLoading ? (
              <div className="flex items-center justify-center h-full text-xs text-slate-400">
                Loading official Minecraft versions...
              </div>
            ) : filteredVersions.length === 0 ? (
              <div className="flex items-center justify-center h-full text-xs text-slate-500">
                No versions found
              </div>
            ) : (
              filteredVersions.map((v) => (
                <div
                  key={v.id}
                  onClick={() => setSelectedVersion(v.id)}
                  className={`flex items-center justify-between px-3 py-2 rounded-xl text-xs cursor-pointer transition-all ${
                    selectedVersion === v.id
                      ? "bg-blue-600/30 text-white border border-blue-400/40 font-semibold shadow-inner"
                      : "text-slate-300 hover:bg-white/[0.06] hover:text-white"
                  }`}
                >
                  <span className="font-mono">{v.id}</span>
                  <span className="text-[10px] px-2 py-0.5 rounded-full uppercase tracking-wider bg-white/5 text-slate-400">
                    {v.versionType}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* Step 3: Mod Loader */}
      {step === 3 && (
        <div className="space-y-3 py-2">
          <div className="grid grid-cols-2 gap-3">
            {/* Vanilla */}
            <GlassCard
              active={loader === "vanilla"}
              onClick={() => setLoader("vanilla")}
              className="p-4"
            >
              <div className="flex items-center gap-3 mb-2">
                <Box className="h-5 w-5 text-blue-400" />
                <span className="font-semibold text-sm text-white">Vanilla</span>
              </div>
              <p className="text-xs text-slate-400">
                Official unmodified Minecraft. Fast, stable, and pure.
              </p>
            </GlassCard>

            {/* Forge */}
            <GlassCard
              active={loader === "forge"}
              onClick={() => setLoader("forge")}
              className="p-4"
            >
              <div className="flex items-center gap-3 mb-2">
                <HardDrive className="h-5 w-5 text-orange-400" />
                <span className="font-semibold text-sm text-white">Forge</span>
              </div>
              <p className="text-xs text-slate-400">Classic modding ecosystem.</p>
            </GlassCard>

            {/* Fabric */}
            <GlassCard
              interactive={false}
              className="p-4 opacity-50 cursor-not-allowed border-dashed"
            >
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-3">
                  <Sparkles className="h-5 w-5 text-amber-400" />
                  <span className="font-semibold text-sm text-white">Fabric</span>
                </div>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">Coming Soon</span>
              </div>
              <p className="text-xs text-slate-400">Lightweight modular mod loader.</p>
            </GlassCard>

            {/* NeoForge */}
            <GlassCard
              interactive={false}
              className="p-4 opacity-50 cursor-not-allowed border-dashed"
            >
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-3">
                  <Cpu className="h-5 w-5 text-purple-400" />
                  <span className="font-semibold text-sm text-white">NeoForge</span>
                </div>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-white/10 text-slate-400">Coming Soon</span>
              </div>
              <p className="text-xs text-slate-400">Modern fork of Forge for 1.20.2+.</p>
            </GlassCard>
          </div>

          {loader === "forge" && (
            <div className="rounded-2xl border border-white/10 bg-white/[0.02] p-3 space-y-2">
              <span className="text-xs text-slate-400">Forge build for {selectedVersion}</span>
              {forgeVersionsLoading ? (
                <div className="flex items-center gap-2 text-xs text-slate-400 py-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Checking available Forge builds...
                </div>
              ) : forgeVersionsErrored || !forgeVersions || forgeVersions.length === 0 ? (
                <p className="text-xs text-red-300 py-1">
                  No Forge build is published for Minecraft {selectedVersion}. Pick a different version in step 2.
                </p>
              ) : (
                <div className="flex flex-wrap gap-2">
                  {forgeVersions.map((v) => (
                    <button
                      key={v.version}
                      type="button"
                      onClick={() => setLoaderVersion(v.version)}
                      className={`px-3 py-1.5 rounded-xl text-xs border transition-colors ${
                        loaderVersion === v.version
                          ? "bg-orange-500/20 border-orange-400/50 text-white font-semibold"
                          : "bg-white/5 border-white/10 text-slate-300 hover:bg-white/10"
                      }`}
                    >
                      {v.version} <span className="text-slate-400 capitalize">({v.label})</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Step 4: Memory RAM */}
      {step === 4 && (
        <div className="space-y-6 py-2">
          <div>
            <div className="flex justify-between text-xs mb-2">
              <span className="text-slate-300 font-medium">Maximum Memory (RAM)</span>
              <span className="text-blue-400 font-semibold font-mono">{maxRam / 1024} GB ({maxRam} MB)</span>
            </div>
            <input
              type="range"
              min={1024}
              max={16384}
              step={1024}
              value={maxRam}
              onChange={(e) => setMaxRam(Number(e.target.value))}
              className="w-full accent-blue-500 cursor-pointer"
            />
            <div className="flex justify-between text-[10px] text-slate-500 mt-1">
              <span>1 GB</span>
              <span>4 GB (Recommended)</span>
              <span>8 GB</span>
              <span>16 GB</span>
            </div>
          </div>

          <div>
            <div className="flex justify-between text-xs mb-2">
              <span className="text-slate-300 font-medium">Minimum Memory (RAM)</span>
              <span className="text-slate-400 font-semibold font-mono">{minRam / 1024} GB ({minRam} MB)</span>
            </div>
            <input
              type="range"
              min={512}
              max={maxRam}
              step={512}
              value={minRam}
              onChange={(e) => setMinRam(Number(e.target.value))}
              className="w-full accent-blue-500 cursor-pointer"
            />
          </div>
        </div>
      )}

      {/* Step 5: Summary */}
      {step === 5 && (
        <div className="space-y-3 py-2">
          <div className="rounded-2xl bg-white/[0.04] border border-white/10 p-4 space-y-3 text-xs">
            <div className="flex justify-between border-b border-white/5 pb-2">
              <span className="text-slate-400">Name</span>
              <span className="text-white font-semibold">{name || "Untitled Instance"}</span>
            </div>
            <div className="flex justify-between border-b border-white/5 pb-2">
              <span className="text-slate-400">Minecraft Version</span>
              <span className="text-white font-semibold font-mono">{selectedVersion}</span>
            </div>
            <div className="flex justify-between border-b border-white/5 pb-2">
              <span className="text-slate-400">Loader</span>
              <span className="text-white font-semibold capitalize">
                {loader}
                {loader === "forge" && loaderVersion ? ` ${loaderVersion}` : ""}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-400">Allocated RAM</span>
              <span className="text-white font-semibold">{minRam / 1024}GB - {maxRam / 1024}GB</span>
            </div>
          </div>
        </div>
      )}

      {/* Modal footer navigation */}
      <div className="flex items-center justify-between mt-6 pt-4 border-t border-white/10">
        {step > 1 ? (
          <GlassButton variant="ghost" onClick={() => setStep(step - 1)}>
            Back
          </GlassButton>
        ) : (
          <div />
        )}

        {step < 5 ? (
          <GlassButton
            variant="primary"
            onClick={() => {
              if (step === 1 && !name.trim()) {
                setError("Please enter a name");
                return;
              }
              if (step === 3 && loader === "forge" && !loaderVersion) {
                setError("Please select a Forge build for this Minecraft version");
                return;
              }
              setError(null);
              setStep(step + 1);
            }}
          >
            Continue
          </GlassButton>
        ) : (
          <GlassButton
            variant="primary"
            onClick={handleCreate}
            isLoading={isSubmitting}
          >
            Create & Finish
          </GlassButton>
        )}
      </div>
    </GlassModal>
  );
};