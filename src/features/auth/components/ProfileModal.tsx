import React, { useState } from "react";
import { User, Check, Plus, Trash2, Fingerprint, Clock, Calendar, ShieldCheck, AlertCircle } from "lucide-react";
import { GlassModal, GlassButton, GlassInput, GlassBadge } from "@/components/ui/glass";
import { useOfflineProfiles } from "../hooks/useOfflineProfiles";
import { formatDate } from "@/utils/formatters";

interface ProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ProfileModal: React.FC<ProfileModalProps> = ({ isOpen, onClose }) => {
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
    } catch (err: any) {
      setError(err?.toString() || "Failed to create offline profile");
    }
  };

  const handleSelect = async (username: string) => {
    try {
      setError(null);
      await selectProfile(username);
    } catch (err: any) {
      setError(err?.toString() || "Failed to select profile");
    }
  };

  const handleDelete = async (username: string) => {
    try {
      setError(null);
      await deleteProfile(username);
    } catch (err: any) {
      setError(err?.toString() || "Failed to delete profile");
    }
  };

  return (
    <GlassModal
      isOpen={isOpen}
      onClose={onClose}
      title="Offline Mode Profiles"
      description="Manage local Minecraft profiles without tokens or credential validation"
      maxWidth="md"
    >
      <div className="space-y-6">
        {/* Offline Mode Banner */}
        <div className="flex items-center gap-3 p-3.5 rounded-2xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">
          <ShieldCheck className="h-4 w-4 shrink-0 text-amber-400" />
          <span>
            <strong>Offline Mode active:</strong> Launches the game without account authentication or tokens. Player UUIDs are generated deterministically from your player name.
          </span>
        </div>

        {/* Active Profile Card */}
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

        {/* Profiles List */}
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

          {/* New Profile Form */}
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

          {/* List items */}
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

        <div className="flex justify-end pt-2 border-t border-white/10">
          <GlassButton variant="secondary" size="sm" onClick={onClose}>
            Close
          </GlassButton>
        </div>
      </div>
    </GlassModal>
  );
};
