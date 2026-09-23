import React, { useState, useEffect } from "react";
import { GlassModal, GlassButton, GlassBadge } from "@/components/ui/glass";
import {
  ArrowUpCircle,
  Download,
  Trash2,
  CheckCircle2,
  FileText,
  Shield,
  Loader2,
  AlertTriangle,
} from "lucide-react";
import { modpacksApi } from "@/services/tauri/modpacksApi";
import { formatBytes } from "@/utils/formatters";
import { useModpackStore } from "../store/useModpackStore";
import type { UpdatePlan } from "@/types";

interface ModpackUpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
  instanceId: string | null;
  targetVersion?: string;
  onSuccess?: () => void;
}

export const ModpackUpdateModal: React.FC<ModpackUpdateModalProps> = ({
  isOpen,
  onClose,
  instanceId,
  targetVersion,
  onSuccess,
}) => {
  const [plan, setPlan] = useState<UpdatePlan | null>(null);
  const [isLoadingPlan, setIsLoadingPlan] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const fetchCatalog = useModpackStore((s) => s.fetchCatalog);

  useEffect(() => {
    if (!isOpen || !instanceId) {
      setPlan(null);
      setErrorMessage(null);
      return;
    }

    let isMounted = true;
    setIsLoadingPlan(true);
    setErrorMessage(null);

    modpacksApi
      .prepareUpdate(instanceId, targetVersion)
      .then((p) => {
        if (isMounted) {
          setPlan(p);
          setIsLoadingPlan(false);
        }
      })
      .catch((err: any) => {
        if (isMounted) {
          setErrorMessage(err?.message || "Failed to calculate update diff plan.");
          setIsLoadingPlan(false);
        }
      });

    return () => {
      isMounted = false;
    };
  }, [isOpen, instanceId, targetVersion]);

  const handleApplyUpdate = async () => {
    if (!instanceId) return;

    setIsUpdating(true);
    setErrorMessage(null);

    try {
      await modpacksApi.update(instanceId, targetVersion);
      await fetchCatalog(true);
      setIsUpdating(false);
      onClose();
      if (onSuccess) {
        onSuccess();
      }
    } catch (err: any) {
      console.error("Update failed:", err);
      setErrorMessage(
        err?.message || "Failed to complete modpack update. Changes have been rolled back safely."
      );
      setIsUpdating(false);
    }
  };

  return (
    <GlassModal
      isOpen={isOpen}
      onClose={onClose}
      title="Modpack Update Available"
      description="Review changes and download differences before applying."
      maxWidth="lg"
    >
      {isLoadingPlan ? (
        <div className="flex flex-col items-center justify-center py-12 space-y-3">
          <Loader2 className="h-8 w-8 text-blue-400 animate-spin" />
          <p className="text-xs text-slate-400">
            Analyzing local files and computing download diff...
          </p>
        </div>
      ) : errorMessage && !plan ? (
        <div className="space-y-4 py-4">
          <div className="flex items-start gap-3 p-4 rounded-2xl bg-red-500/15 border border-red-500/30 text-red-300">
            <AlertTriangle className="h-5 w-5 shrink-0 text-red-400 mt-0.5" />
            <div className="text-xs space-y-1">
              <span className="font-semibold block">Update Error</span>
              <p>{errorMessage}</p>
            </div>
          </div>
          <div className="flex justify-end">
            <GlassButton variant="secondary" onClick={onClose}>
              Close
            </GlassButton>
          </div>
        </div>
      ) : plan ? (
        <div className="space-y-5">
          {errorMessage && (
            <div className="p-3 rounded-xl bg-red-500/15 border border-red-500/30 text-xs text-red-300">
              {errorMessage}
            </div>
          )}

          {/* Version Header Diff */}
          <div className="flex items-center justify-between p-4 rounded-2xl bg-white/[0.04] border border-white/10">
            <div className="flex items-center gap-3">
              <span className="text-sm text-slate-300">Target Version</span>
              <div className="flex items-center gap-2">
                <GlassBadge variant="default">
                  {plan.fromVersion ? `v${plan.fromVersion}` : "Current"}
                </GlassBadge>
                <span className="text-slate-500">→</span>
                <GlassBadge variant="warning" className="font-semibold shadow-glow-sm">
                  v{plan.toVersion}
                </GlassBadge>
              </div>
            </div>
            <div className="text-xs text-slate-400">
              Differential Download:{" "}
              <span className="text-white font-semibold">
                {formatBytes(plan.totalDownloadSize)}
              </span>
            </div>
          </div>

          {/* Diff Metrics Grid */}
          <div className="grid grid-cols-3 gap-3">
            <div className="p-3.5 rounded-xl bg-blue-500/10 border border-blue-500/20 text-center space-y-1">
              <div className="flex items-center justify-center gap-1.5 text-blue-400 text-xs font-semibold">
                <Download className="h-3.5 w-3.5" />
                <span>To Download</span>
              </div>
              <p className="text-lg font-bold text-white">
                {plan.downloads.length}{" "}
                <span className="text-xs font-normal text-slate-400">
                  files ({formatBytes(plan.totalDownloadSize)})
                </span>
              </p>
            </div>

            <div className="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-center space-y-1">
              <div className="flex items-center justify-center gap-1.5 text-emerald-400 text-xs font-semibold">
                <CheckCircle2 className="h-3.5 w-3.5" />
                <span>Unmodified</span>
              </div>
              <p className="text-lg font-bold text-white">
                {plan.unmodifiedCount}{" "}
                <span className="text-xs font-normal text-slate-400">
                  files (Kept)
                </span>
              </p>
            </div>

            <div className="p-3.5 rounded-xl bg-red-500/10 border border-red-500/20 text-center space-y-1">
              <div className="flex items-center justify-center gap-1.5 text-red-400 text-xs font-semibold">
                <Trash2 className="h-3.5 w-3.5" />
                <span>Obsolete / Removed</span>
              </div>
              <p className="text-lg font-bold text-white">
                {plan.deletions.length}{" "}
                <span className="text-xs font-normal text-slate-400">
                  files
                </span>
              </p>
            </div>
          </div>

          {/* Changelog */}
          {plan.changelog && plan.changelog.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
                <FileText className="h-4 w-4 text-blue-400" />
                <span>What's New in v{plan.toVersion}</span>
              </div>
              <div className="p-3.5 rounded-xl bg-white/[0.03] border border-white/10 max-h-36 overflow-y-auto space-y-1 text-xs text-slate-300">
                {plan.changelog.map((item, idx) => (
                  <div key={idx} className="flex items-start gap-2">
                    <span className="text-blue-400 font-bold">•</span>
                    <span>{item}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Safety Guarantee */}
          <div className="flex items-center gap-2.5 p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-xs text-emerald-300">
            <Shield className="h-4 w-4 shrink-0 text-emerald-400" />
            <span>
              Protected Paths: Your worlds (<code>saves/</code>), screenshots, and settings (<code>options.txt</code>) are strictly shielded and will not be touched.
            </span>
          </div>

          {/* Modal Actions */}
          <div className="flex items-center justify-end gap-3 pt-3 border-t border-white/10">
            <GlassButton
              type="button"
              variant="ghost"
              onClick={onClose}
              disabled={isUpdating}
            >
              Cancel
            </GlassButton>
            <GlassButton
              type="button"
              variant="primary"
              isLoading={isUpdating}
              onClick={handleApplyUpdate}
              className="bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border-amber-500/40 shadow-glow-warning"
            >
              <ArrowUpCircle className="h-4 w-4 mr-1.5" />
              Update Now
            </GlassButton>
          </div>
        </div>
      ) : null}
    </GlassModal>
  );
};
