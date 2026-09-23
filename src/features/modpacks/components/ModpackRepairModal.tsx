import React, { useState, useEffect } from "react";
import { GlassModal, GlassButton, GlassBadge } from "@/components/ui/glass";
import {
  CheckCircle2,
  AlertTriangle,

  FileX,
  FileWarning,
  Loader2,
  Download,
} from "lucide-react";
import { modpacksApi } from "@/services/tauri/modpacksApi";
import { formatBytes } from "@/utils/formatters";
import type { VerificationResult } from "@/types";

interface ModpackRepairModalProps {
  isOpen: boolean;
  onClose: () => void;
  instanceId: string | null;
  onSuccess?: () => void;
}

export const ModpackRepairModal: React.FC<ModpackRepairModalProps> = ({
  isOpen,
  onClose,
  instanceId,
  onSuccess,
}) => {
  const [result, setResult] = useState<VerificationResult | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);
  const [isRepairing, setIsRepairing] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const runVerification = () => {
    if (!instanceId) return;

    setIsVerifying(true);
    setErrorMessage(null);
    setResult(null);

    modpacksApi
      .verify(instanceId)
      .then((res) => {
        setResult(res);
        setIsVerifying(false);
      })
      .catch((err: any) => {
        setErrorMessage(err?.message || "Failed to verify files integrity.");
        setIsVerifying(false);
      });
  };

  useEffect(() => {
    if (isOpen && instanceId) {
      runVerification();
    } else {
      setResult(null);
      setErrorMessage(null);
    }
  }, [isOpen, instanceId]);

  const handleRepair = async () => {
    if (!instanceId) return;

    setIsRepairing(true);
    setErrorMessage(null);

    try {
      const fixed = await modpacksApi.repair(instanceId);
      setResult(fixed);
      setIsRepairing(false);
      if (onSuccess) {
        onSuccess();
      }
    } catch (err: any) {
      console.error("Repair failed:", err);
      setErrorMessage(err?.message || "Failed to repair files.");
      setIsRepairing(false);
    }
  };

  const hasIssues = result && (result.missing.length > 0 || result.modified.length > 0);

  return (
    <GlassModal
      isOpen={isOpen}
      onClose={onClose}
      title="Verify & Repair Modpack Files"
      description="Check all installed mods and configs against SHA-256 checksums."
      maxWidth="lg"
    >
      {isVerifying ? (
        <div className="flex flex-col items-center justify-center py-12 space-y-3">
          <Loader2 className="h-8 w-8 text-blue-400 animate-spin" />
          <p className="text-xs text-slate-400">
            Scanning files and calculating SHA-256 checksums...
          </p>
        </div>
      ) : errorMessage ? (
        <div className="space-y-4 py-2">
          <div className="flex items-start gap-3 p-4 rounded-2xl bg-red-500/15 border border-red-500/30 text-red-300">
            <AlertTriangle className="h-5 w-5 shrink-0 text-red-400 mt-0.5" />
            <div className="text-xs space-y-1">
              <span className="font-semibold block">Integrity Check Error</span>
              <p>{errorMessage}</p>
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <GlassButton variant="secondary" onClick={runVerification}>
              Retry Check
            </GlassButton>
            <GlassButton variant="ghost" onClick={onClose}>
              Close
            </GlassButton>
          </div>
        </div>
      ) : result ? (
        <div className="space-y-5">
          {/* Summary Status */}
          <div className="flex items-center justify-between p-4 rounded-2xl bg-white/[0.04] border border-white/10">
            <div className="flex items-center gap-3">
              {hasIssues ? (
                <div className="h-10 w-10 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/30 flex items-center justify-center">
                  <AlertTriangle className="h-5 w-5" />
                </div>
              ) : (
                <div className="h-10 w-10 rounded-xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center justify-center">
                  <CheckCircle2 className="h-5 w-5" />
                </div>
              )}
              <div>
                <h4 className="text-sm font-semibold text-white">
                  {hasIssues ? "Integrity Issues Detected" : "All Files Verified"}
                </h4>
                <p className="text-xs text-slate-400">
                  {hasIssues
                    ? `${result.missing.length} missing, ${result.modified.length} modified files.`
                    : `All ${result.valid} files match their manifest checksums.`}
                </p>
              </div>
            </div>
            {hasIssues && (
              <GlassBadge variant="warning">
                Repair Size: {formatBytes(result.repairSize)}
              </GlassBadge>
            )}
          </div>

          {/* Stats Breakdown */}
          <div className="grid grid-cols-3 gap-3">
            <div className="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-center">
              <div className="text-xs text-emerald-400 font-semibold mb-1">
                Valid Files
              </div>
              <span className="text-lg font-bold text-white">{result.valid}</span>
            </div>

            <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-center">
              <div className="text-xs text-red-400 font-semibold mb-1">
                Missing
              </div>
              <span className="text-lg font-bold text-white">
                {result.missing.length}
              </span>
            </div>

            <div className="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-center">
              <div className="text-xs text-amber-400 font-semibold mb-1">
                Modified / Corrupt
              </div>
              <span className="text-lg font-bold text-white">
                {result.modified.length}
              </span>
            </div>
          </div>

          {/* Files List if Issues Exist */}
          {hasIssues && (
            <div className="space-y-2">
              <span className="text-xs font-semibold text-slate-300">
                Files Scheduled for Redownload:
              </span>
              <div className="p-3 rounded-xl bg-black/40 border border-white/10 max-h-40 overflow-y-auto space-y-1.5 text-xs">
                {result.missing.map((file) => (
                  <div
                    key={file}
                    className="flex items-center gap-2 text-red-400"
                  >
                    <FileX className="h-3.5 w-3.5 shrink-0" />
                    <span className="truncate font-mono">{file} (Missing)</span>
                  </div>
                ))}
                {result.modified.map((file) => (
                  <div
                    key={file}
                    className="flex items-center gap-2 text-amber-400"
                  >
                    <FileWarning className="h-3.5 w-3.5 shrink-0" />
                    <span className="truncate font-mono">{file} (Hash Mismatch)</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex items-center justify-end gap-3 pt-3 border-t border-white/10">
            <GlassButton
              type="button"
              variant="ghost"
              onClick={onClose}
              disabled={isRepairing}
            >
              {hasIssues ? "Cancel" : "Done"}
            </GlassButton>
            {hasIssues && (
              <GlassButton
                type="button"
                variant="primary"
                isLoading={isRepairing}
                onClick={handleRepair}
                className="bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border-amber-500/40"
              >
                <Download className="h-4 w-4 mr-1.5" />
                Repair Files ({formatBytes(result.repairSize)})
              </GlassButton>
            )}
          </div>
        </div>
      ) : null}
    </GlassModal>
  );
};
