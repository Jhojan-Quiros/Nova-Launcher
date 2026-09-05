import React, { useState } from "react";
import {
  User,
  Check,
  Plus,
  Trash2,
  Fingerprint,
  Clock,
  Calendar,
  ShieldCheck,
  AlertCircle,
  Gamepad2,
  ExternalLink,
  Copy,
  LogOut,
  Loader2,
} from "lucide-react";
import { GlassModal, GlassButton, GlassInput, GlassBadge } from "@/components/ui/glass";
import { useOfflineProfiles } from "../hooks/useOfflineProfiles";
import { useMicrosoftAuth } from "../hooks/useMicrosoftAuth";
import { useSettings } from "@/features/settings/hooks/useSettings";
import { authApi } from "@/services/tauri/authApi";
import { getErrorMessage } from "@/utils/errors";
import { formatDate } from "@/utils/formatters";
import type { DeviceCodeInfo } from "@/types/account";

interface AccountModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AccountModal: React.FC<AccountModalProps> = ({ isOpen, onClose }) => {
  const [tab, setTab] = useState<"offline" | "microsoft">("offline");
  const { settings } = useSettings();
  const activeMode = settings?.activeAuthMode ?? "offline";

  return (
    <GlassModal
      isOpen={isOpen}
      onClose={onClose}
      title="Account"
      description="Choose whether to play offline with a local nickname, or sign in with Microsoft."
      maxWidth="md"
    >
      <div className="space-y-5">
        <div className="flex items-center gap-2 p-1 rounded-2xl bg-white/[0.04] border border-white/10">
          <button
            onClick={() => setTab("offline")}
            className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl text-xs font-semibold transition-all ${
              tab === "offline" ? "bg-amber-500/20 text-amber-300 border border-amber-500/30" : "text-slate-400 hover:text-white"
            }`}
          >
            <ShieldCheck className="h-3.5 w-3.5" />
            Offline
            {activeMode === "offline" && <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />}
          </button>
          <button
            onClick={() => setTab("microsoft")}
            className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl text-xs font-semibold transition-all ${
              tab === "microsoft" ? "bg-blue-500/20 text-blue-300 border border-blue-500/30" : "text-slate-400 hover:text-white"
            }`}
          >
            <Gamepad2 className="h-3.5 w-3.5" />
            Microsoft
            {activeMode === "microsoft" && <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />}
          </button>
        </div>

        {tab === "offline" ? <OfflineTab /> : <MicrosoftTab clientId={settings?.microsoftClientId} />}

        <div className="flex justify-end pt-2 border-t border-white/10">
          <GlassButton variant="secondary" size="sm" onClick={onClose}>
            Close
          </GlassButton>
        </div>
      </div>
    </GlassModal>
  );
};

const OfflineTab: React.FC = () => {
  const {
    activeProfile,
    profiles,
    selectProfile,
    createProfile,
    deleteProfile,
    isCreating,
    isSelecting,
    isDeleting,
  } = useOfflineProfiles();

  const [newUsername, setNewUsername] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    const clean = newUsername.trim();

    if (clean.length < 3 || clean.length > 16) {
      setError("Username must be between 3 and 16 characters.");
      return;
    }

    if (!/^[a-zA-Z0-9_]+$/.test(clean)) {
      setError("Only letters, numbers, and underscores (_) are allowed.");
      return;
    }

    try {
      setError(null);
      await createProfile(clean);
      setNewUsername("");
      setIsAdding(false);
    } catch (err) {
      setError(getErrorMessage(err, "Failed to create offline profile"));
    }
  };

  const handleSelect = async (username: string) => {
    try {
      setError(null);
      await selectProfile(username);
    } catch (err) {
      setError(getErrorMessage(err, "Failed to select profile"));
    }
  };

  const handleDelete = async (username: string) => {
    try {
      setError(null);
      await deleteProfile(username);
    } catch (err) {
      setError(getErrorMessage(err, "Failed to delete profile"));
    }
  };

  return (
    <div className="space-y-5">
      <div className="flex items-center gap-3 p-3.5 rounded-2xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">
        <ShieldCheck className="h-4 w-4 shrink-0 text-amber-400" />
        <span>
          <strong>Offline Mode:</strong> Launches the game with a locally-generated nickname, no Microsoft account or ownership check required. Won't work on servers running in online-mode.
        </span>
      </div>

      {activeProfile && (
        <div className="p-4 rounded-2xl bg-white/[0.04] border border-white/10 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="h-10 w-10 rounded-xl bg-gradient-to-br from-blue-500/30 to-indigo-600/30 border border-blue-400/30 flex items-center justify-center text-blue-400 font-bold shadow-inner">
                {activeProfile.username.charAt(0).toUpperCase()}
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <span className="text-sm font-semibold text-white">{activeProfile.username}</span>
                  <GlassBadge variant="success" size="sm">Active Player</GlassBadge>
                </div>
                <p className="text-[11px] text-slate-400 font-mono flex items-center gap-1 mt-0.5">
                  <Fingerprint className="h-3 w-3 text-slate-500" />
                  {activeProfile.generatedLocalUuid}
                </p>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-4 text-[11px] text-slate-400 pt-1 border-t border-white/5">
            <span className="flex items-center gap-1">
              <Calendar className="h-3 w-3 text-slate-500" />
              Created: {formatDate(activeProfile.createdAt)}
            </span>
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3 text-slate-500" />
              Last Used: {formatDate(activeProfile.lastUsedAt)}
            </span>
          </div>
        </div>
      )}

      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs text-slate-400 font-medium px-1">
          <span>Available Profiles ({profiles.length})</span>
          {!isAdding && (
            <GlassButton
              variant="ghost"
              size="sm"
              onClick={() => setIsAdding(true)}
              className="h-7 text-xs text-blue-400 hover:text-blue-300"
            >
              <Plus className="h-3.5 w-3.5 mr-1" />
              New Profile
            </GlassButton>
          )}
        </div>

        {isAdding && (
          <form onSubmit={handleCreate} className="p-3 rounded-2xl bg-blue-500/[0.06] border border-blue-500/20 space-y-3">
            <div className="text-xs font-semibold text-blue-300">Create New Offline Profile</div>
            <GlassInput
              placeholder="Minecraft Username (e.g. Miner123)"
              value={newUsername}
              onChange={(e) => {
                setNewUsername(e.target.value);
                setError(null);
              }}
              autoFocus
            />
            {error && (
              <div className="flex items-center gap-1.5 text-xs text-rose-400">
                <AlertCircle className="h-3.5 w-3.5" />
                <span>{error}</span>
              </div>
            )}
            <div className="flex items-center justify-end gap-2 pt-1">
              <GlassButton
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => {
                  setIsAdding(false);
                  setError(null);
                }}
              >
                Cancel
              </GlassButton>
              <GlassButton
                type="submit"
                variant="primary"
                size="sm"
                disabled={isCreating || !newUsername.trim()}
              >
                {isCreating ? "Creating..." : "Save & Activate"}
              </GlassButton>
            </div>
          </form>
        )}

        {!isAdding && error && (
          <div className="flex items-center gap-1.5 text-xs text-rose-400 px-1">
            <AlertCircle className="h-3.5 w-3.5" />
            <span>{error}</span>
          </div>
        )}

        <div className="space-y-1.5 max-h-52 overflow-y-auto pr-1">
          {profiles.map((p) => {
            const isActive = activeProfile?.username === p.username;
            return (
              <div
                key={p.username}
                className={`flex items-center justify-between p-2.5 rounded-xl transition-all border ${
                  isActive
                    ? "bg-blue-500/10 border-blue-500/30 text-white"
                    : "bg-white/[0.02] border-white/5 text-slate-300 hover:bg-white/[0.05]"
                }`}
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <User className={`h-4 w-4 ${isActive ? "text-blue-400" : "text-slate-500"}`} />
                  <div className="min-w-0">
                    <div className="text-xs font-medium truncate">{p.username}</div>
                    <div className="text-[10px] text-slate-500 font-mono truncate max-w-[220px]">
                      {p.generatedLocalUuid}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-1.5 shrink-0">
                  {isActive ? (
                    <span className="flex items-center gap-1 text-[11px] font-semibold text-emerald-400 px-2 py-0.5 rounded-full bg-emerald-500/10">
                      <Check className="h-3 w-3" />
                      Selected
                    </span>
                  ) : (
                    <GlassButton
                      variant="ghost"
                      size="sm"
                      className="h-7 text-xs"
                      disabled={isSelecting}
                      onClick={() => handleSelect(p.username)}
                    >
                      Select
                    </GlassButton>
                  )}

                  {profiles.length > 1 && (
                    <GlassButton
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10"
                      title="Delete Profile"
                      disabled={isDeleting}
                      onClick={() => handleDelete(p.username)}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </GlassButton>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

const MicrosoftTab: React.FC<{ clientId?: string }> = ({ clientId }) => {
  const { account, isLoadingAccount, beginLogin, completeLogin, signOut, isSigningOut, activate, isActivating } =
    useMicrosoftAuth();
  const [deviceInfo, setDeviceInfo] = useState<DeviceCodeInfo | null>(null);
  const [isSigningIn, setIsSigningIn] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSignIn = async () => {
    setError(null);
    try {
      const info = await beginLogin();
      setDeviceInfo(info);
      setIsSigningIn(true);
      await completeLogin(info);
      setDeviceInfo(null);
    } catch (err) {
      setError(getErrorMessage(err, "Microsoft sign-in failed"));
      setDeviceInfo(null);
    } finally {
      setIsSigningIn(false);
    }
  };

  const handleActivate = async () => {
    setError(null);
    try {
      await activate();
    } catch (err) {
      setError(getErrorMessage(err, "Failed to switch to this account"));
    }
  };

  const handleSignOut = async () => {
    setError(null);
    try {
      await signOut();
    } catch (err) {
      setError(getErrorMessage(err, "Failed to sign out"));
    }
  };

  if (!clientId) {
    return (
      <div className="p-4 rounded-2xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs space-y-2">
        <div className="font-semibold flex items-center gap-1.5">
          <AlertCircle className="h-4 w-4" />
          Microsoft sign-in isn't set up yet
        </div>
        <p className="text-amber-300/80 leading-relaxed">
          Register a free app at{" "}
          <button
            className="underline hover:text-amber-200"
            onClick={() => authApi.openExternalUrl("https://portal.azure.com")}
          >
            portal.azure.com
          </button>{" "}
          and paste its Application (client) ID into Settings → Microsoft Account.
        </p>
      </div>
    );
  }

  if (isLoadingAccount) {
    return <div className="text-xs text-slate-400 text-center py-6">Loading account...</div>;
  }

  if (isSigningIn && deviceInfo) {
    return (
      <div className="p-5 rounded-2xl bg-blue-500/[0.06] border border-blue-500/20 space-y-4 text-center">
        <Loader2 className="h-6 w-6 mx-auto text-blue-400 animate-spin" />
        <div>
          <p className="text-xs text-slate-300 mb-2">
            Go to <span className="text-blue-300 font-semibold">{deviceInfo.verificationUri}</span> and enter this code:
          </p>
          <div className="flex items-center justify-center gap-2">
            <span className="text-2xl font-bold tracking-[0.3em] text-white font-mono">{deviceInfo.userCode}</span>
            <button
              title="Copy code"
              onClick={() => navigator.clipboard.writeText(deviceInfo.userCode)}
              className="p-1.5 rounded-lg hover:bg-white/10 text-slate-400 hover:text-white"
            >
              <Copy className="h-3.5 w-3.5" />
            </button>
          </div>
        </div>
        <GlassButton
          variant="primary"
          size="sm"
          onClick={() => authApi.openExternalUrl(deviceInfo.verificationUri)}
        >
          <ExternalLink className="h-3.5 w-3.5 mr-1.5" />
          Open in Browser
        </GlassButton>
        <p className="text-[11px] text-slate-500">Waiting for you to finish signing in...</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3 p-3.5 rounded-2xl bg-blue-500/10 border border-blue-500/20 text-blue-300 text-xs">
        <Gamepad2 className="h-4 w-4 shrink-0 text-blue-400" />
        <span>
          <strong>Microsoft Account:</strong> Uses your real, purchased Minecraft account. Required for servers running in online-mode.
        </span>
      </div>

      {error && (
        <div className="flex items-center gap-1.5 text-xs text-rose-400 px-1">
          <AlertCircle className="h-3.5 w-3.5" />
          <span>{error}</span>
        </div>
      )}

      {account ? (
        <div className="p-4 rounded-2xl bg-white/[0.04] border border-white/10 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="h-10 w-10 rounded-xl bg-gradient-to-br from-blue-500/30 to-indigo-600/30 border border-blue-400/30 flex items-center justify-center text-blue-400 font-bold shadow-inner">
                {account.username.charAt(0).toUpperCase()}
              </div>
              <div>
                <span className="text-sm font-semibold text-white">{account.username}</span>
                <p className="text-[11px] text-slate-400 font-mono flex items-center gap-1 mt-0.5">
                  <Fingerprint className="h-3 w-3 text-slate-500" />
                  {account.uuid}
                </p>
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2 pt-2 border-t border-white/5">
            <GlassButton variant="primary" size="sm" isLoading={isActivating} onClick={handleActivate}>
              <Check className="h-3.5 w-3.5 mr-1.5" />
              Use This Account
            </GlassButton>
            <GlassButton variant="ghost" size="sm" isLoading={isSigningOut} onClick={handleSignOut}>
              <LogOut className="h-3.5 w-3.5 mr-1.5" />
              Sign Out
            </GlassButton>
          </div>
        </div>
      ) : (
        <GlassButton variant="primary" size="md" className="w-full" onClick={handleSignIn}>
          <Gamepad2 className="h-4 w-4 mr-1.5" />
          Sign in with Microsoft
        </GlassButton>
      )}
    </div>
  );
};
