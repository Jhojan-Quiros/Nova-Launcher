import React, { useState, useEffect } from "react";
import {
  Settings,
  Cpu,
  Monitor,
  HardDrive,
  Download,
  FolderOpen,
  RefreshCw,
  Check,
  Save,
  ShieldCheck,
  Fingerprint,
  User,
  Calendar,
  Clock,
  Package,
} from "lucide-react";

import { GlassPanel, GlassButton, GlassInput, GlassBadge } from "@/components/ui/glass";
import { useSettings } from "@/features/settings/hooks/useSettings";
import { useOfflineProfiles } from "@/features/auth/hooks/useOfflineProfiles";
import { useModpackStore } from "@/features/modpacks/store/useModpackStore";
import { AccountModal } from "@/features/auth/components/AccountModal";
import { authApi } from "@/services/tauri/authApi";
import { formatDate } from "@/utils/formatters";
import { instancesApi } from "@/services/tauri/instancesApi";

export const SettingsPage: React.FC = () => {
  const {
    settings,
    isLoading,
    updateSettings,
    isUpdating,
    javaRuntimes,
    isDetectingJava,
    detectJava,
  } = useSettings();

  const { activeProfile, profiles, selectProfile } = useOfflineProfiles();
  const [isProfileModalOpen, setIsProfileModalOpen] = useState(false);

  const [activeTab, setActiveTab] = useState<
    "general" | "modpacks" | "offline" | "minecraft" | "java" | "appearance" | "downloads" | "advanced"
  >("general");

  // Local form state
  const [closeOnLaunch, setCloseOnLaunch] = useState(false);
  const [defaultMinRam, setDefaultMinRam] = useState(2048);
  const [defaultMaxRam, setDefaultMaxRam] = useState(4096);
  const [defaultJavaPath, setDefaultJavaPath] = useState("");
  const [theme, setTheme] = useState("dark");
  const [blurIntensity, setBlurIntensity] = useState(100);
  const [maxConcurrentDownloads, setMaxConcurrentDownloads] = useState(8);
  const [microsoftClientId, setMicrosoftClientId] = useState("");
  const [savedSuccess, setSavedSuccess] = useState(false);

  // Modpack settings state
  const [modpackCatalogUrl, setModpackCatalogUrl] = useState("");
  const [checkUpdatesOnStartup, setCheckUpdatesOnStartup] = useState(true);
  const [autoCheckIntervalMinutes, setAutoCheckIntervalMinutes] = useState(60);
  const [strictModpackModeDefault, setStrictModpackModeDefault] = useState(false);
  const [verifyFilesBeforeLaunch, setVerifyFilesBeforeLaunch] = useState(false);
  const [isTestingCatalog, setIsTestingCatalog] = useState(false);
  const [catalogTestResult, setCatalogTestResult] = useState<string | null>(null);

  useEffect(() => {
    if (settings) {
      setCloseOnLaunch(settings.closeOnLaunch);
      setDefaultMinRam(settings.defaultMinRam);
      setDefaultMaxRam(settings.defaultMaxRam);
      setDefaultJavaPath(settings.defaultJavaPath || "");
      setTheme(settings.theme);
      setBlurIntensity(settings.blurIntensity);
      setMaxConcurrentDownloads(settings.maxConcurrentDownloads);
      setMicrosoftClientId(settings.microsoftClientId || "");
      setModpackCatalogUrl(settings.modpackCatalogUrl || "https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/catalog.json");
      setCheckUpdatesOnStartup(settings.checkUpdatesOnStartup ?? true);
      setAutoCheckIntervalMinutes(settings.autoCheckIntervalMinutes ?? 60);
      setStrictModpackModeDefault(settings.strictModpackModeDefault ?? false);
      setVerifyFilesBeforeLaunch(settings.verifyFilesBeforeLaunch ?? false);
    }
  }, [settings]);

  const handleSave = async () => {
    await updateSettings({
      closeOnLaunch,
      defaultMinRam,
      defaultMaxRam,
      defaultJavaPath,
      theme,
      blurIntensity,
      maxConcurrentDownloads,
      modpackCatalogUrl,
      checkUpdatesOnStartup,
      autoCheckIntervalMinutes,
      strictModpackModeDefault,
      verifyFilesBeforeLaunch,
      microsoftClientId,
    });
    setSavedSuccess(true);
    setTimeout(() => setSavedSuccess(false), 2500);
  };

  const handleTestCatalog = async () => {
    setIsTestingCatalog(true);
    setCatalogTestResult(null);
    try {
      await updateSettings({ modpackCatalogUrl });
      await useModpackStore.getState().fetchCatalog(true);
      setCatalogTestResult("Successfully connected and synced modpack catalog!");
    } catch (e: any) {
      setCatalogTestResult(`Failed: ${e?.message || "Could not reach catalog URL"}`);
    } finally {
      setIsTestingCatalog(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex h-96 items-center justify-center text-xs text-slate-400">
        Loading settings...
      </div>
    );
  }

  const tabs = [
    { id: "general", label: "General", icon: Settings },
    { id: "modpacks", label: "Modpacks", icon: Package },
    { id: "offline", label: "Account", icon: ShieldCheck },
    { id: "minecraft", label: "Minecraft", icon: Monitor },
    { id: "java", label: "Java Runtime", icon: Cpu },
    { id: "appearance", label: "Appearance", icon: Monitor },
    { id: "downloads", label: "Downloads", icon: Download },
    { id: "advanced", label: "Advanced", icon: HardDrive },
  ];


  return (
    <div className="space-y-6 max-w-5xl mx-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2.5">
            <Settings className="h-6 w-6 text-blue-400" />
            Launcher Settings
          </h1>
          <p className="text-xs text-slate-400 mt-0.5">
            Configure system defaults, Java runtime environments, and visual appearance.
          </p>
        </div>

        <div className="flex items-center gap-3">
          {savedSuccess && (
            <span className="text-xs text-emerald-400 font-medium flex items-center gap-1 animate-in fade-in">
              <Check className="h-4 w-4" />
              Settings saved!
            </span>
          )}
          <GlassButton variant="primary" onClick={handleSave} isLoading={isUpdating}>
            <Save className="h-4 w-4 mr-1.5" />
            Save Settings
          </GlassButton>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        {/* Navigation tabs */}
        <div className="space-y-1">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`flex items-center gap-3 w-full px-4 py-2.5 rounded-xl text-xs font-medium transition-all text-left ${
                  isActive
                    ? "bg-blue-600/20 text-blue-400 border border-blue-500/30 shadow-inner"
                    : "text-slate-400 hover:text-white hover:bg-white/5 border border-transparent"
                }`}
              >
                <Icon className="h-4 w-4 shrink-0" />
                {tab.label}
              </button>
            );
          })}
        </div>

        {/* Content area */}
        <div className="md:col-span-3">
          <GlassPanel variant="elevated" className="p-6 space-y-6">
            {/* General */}
            {activeTab === "general" && (
              <div className="space-y-4">
                <h3 className="text-sm font-semibold text-white">General Settings</h3>

                <div className="space-y-3 pt-2">
                  <label className="flex items-center justify-between p-3.5 rounded-xl bg-white/[0.03] border border-white/5 cursor-pointer">
                    <div className="space-y-0.5">
                      <span className="text-xs font-medium text-white">Close launcher on start</span>
                      <p className="text-[11px] text-slate-400">
                        Automatically close or minimize the launcher window when Minecraft launches.
                      </p>
                    </div>
                    <input
                      type="checkbox"
                      checked={closeOnLaunch}
                      onChange={(e) => setCloseOnLaunch(e.target.checked)}
                      className="rounded bg-white/10 border-white/20 text-blue-600 focus:ring-0 h-4 w-4"
                    />
                  </label>
                </div>
              </div>
            )}

            {/* Modpacks Tab */}
            {activeTab === "modpacks" && (
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-semibold text-white">
                    Remote Modpacks & CDN Distribution
                  </h3>
                  <p className="text-xs text-slate-400">
                    Configure the remote JSON catalog endpoint hosted on Cloudflare R2 or custom CDN.
                  </p>
                </div>

                {/* Catalog URL */}
                <div className="space-y-2 p-4 rounded-2xl bg-white/[0.03] border border-white/10">
                  <label className="text-xs font-semibold text-slate-200 block">
                    Modpack Catalog URL (catalog.json)
                  </label>
                  <div className="flex gap-2">
                    <GlassInput
                      value={modpackCatalogUrl}
                      onChange={(e) => setModpackCatalogUrl(e.target.value)}
                      placeholder="https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/catalog.json"
                      className="flex-1 font-mono text-xs"
                    />
                    <GlassButton
                      variant="secondary"
                      size="md"
                      isLoading={isTestingCatalog}
                      onClick={handleTestCatalog}
                    >
                      <RefreshCw className="h-4 w-4 mr-1.5" />
                      Test & Refresh
                    </GlassButton>
                  </div>
                  <p className="text-[11px] text-slate-400">
                    Direct public URL to your <code className="text-blue-300">catalog.json</code> file.
                    The launcher will pull modpack manifests and differential update plans from this domain.
                  </p>
                  {catalogTestResult && (
                    <div
                      className={`p-2.5 rounded-xl text-xs ${
                        catalogTestResult.startsWith("Successfully")
                          ? "bg-emerald-500/15 border border-emerald-500/30 text-emerald-300"
                          : "bg-red-500/15 border border-red-500/30 text-red-300"
                      }`}
                    >
                      {catalogTestResult}
                    </div>
                  )}
                </div>

                {/* Automatic Checks */}
                <div className="space-y-4">
                  <h4 className="text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Update Checks & Notifications
                  </h4>

                  <label className="flex items-center justify-between p-4 rounded-2xl bg-white/[0.03] border border-white/10 cursor-pointer hover:bg-white/[0.05] transition-colors">
                    <div>
                      <span className="text-xs font-medium text-white block">
                        Check for Updates on Startup
                      </span>
                      <span className="text-[11px] text-slate-400">
                        Automatically checks remote manifests 4 seconds after Nova Launcher starts.
                      </span>
                    </div>
                    <input
                      type="checkbox"
                      checked={checkUpdatesOnStartup}
                      onChange={(e) => setCheckUpdatesOnStartup(e.target.checked)}
                      className="rounded bg-white/10 border-white/20 text-blue-600 focus:ring-0 h-4 w-4"
                    />
                  </label>

                  <div className="flex items-center justify-between p-4 rounded-2xl bg-white/[0.03] border border-white/10">
                    <div>
                      <span className="text-xs font-medium text-white block">
                        Background Check Interval (Minutes)
                      </span>
                      <span className="text-[11px] text-slate-400">
                        How often to query the remote CDN for new modpack releases while running.
                      </span>
                    </div>
                    <input
                      type="number"
                      min={10}
                      max={720}
                      value={autoCheckIntervalMinutes}
                      onChange={(e) => setAutoCheckIntervalMinutes(Number(e.target.value))}
                      className="w-24 px-3 py-1.5 rounded-xl bg-white/10 border border-white/15 text-xs text-white text-right focus:outline-none focus:border-blue-500"
                    />
                  </div>
                </div>

                {/* Strict Mode & Verification Defaults */}
                <div className="space-y-4">
                  <h4 className="text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Safety & Integrity
                  </h4>

                  <label className="flex items-center justify-between p-4 rounded-2xl bg-white/[0.03] border border-white/10 cursor-pointer hover:bg-white/[0.05] transition-colors">
                    <div className="max-w-md">
                      <span className="text-xs font-medium text-white block">
                        Strict Modpack Mode Default
                      </span>
                      <span className="text-[11px] text-slate-400">
                        When enabled by default on new modpack installs, unmanaged mods in <code className="text-blue-300">mods/</code> are purged during updates to guarantee parity with the remote manifest.
                      </span>
                    </div>
                    <input
                      type="checkbox"
                      checked={strictModpackModeDefault}
                      onChange={(e) => setStrictModpackModeDefault(e.target.checked)}
                      className="rounded bg-white/10 border-white/20 text-blue-600 focus:ring-0 h-4 w-4"
                    />
                  </label>

                  <label className="flex items-center justify-between p-4 rounded-2xl bg-white/[0.03] border border-white/10 cursor-pointer hover:bg-white/[0.05] transition-colors">
                    <div className="max-w-md">
                      <span className="text-xs font-medium text-white block">
                        Verify Files Before Launch
                      </span>
                      <span className="text-[11px] text-slate-400">
                        Performs a fast SHA-256 integrity check against the installed manifest before launching Minecraft.
                      </span>
                    </div>
                    <input
                      type="checkbox"
                      checked={verifyFilesBeforeLaunch}
                      onChange={(e) => setVerifyFilesBeforeLaunch(e.target.checked)}
                      className="rounded bg-white/10 border-white/20 text-blue-600 focus:ring-0 h-4 w-4"
                    />
                  </label>
                </div>
              </div>
            )}

            {/* Account */}
            {activeTab === "offline" && (

              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-semibold text-white">Account</h3>
                    <p className="text-xs text-slate-400">
                      Play offline with a local nickname, or sign in with a Microsoft account for online-mode servers.
                    </p>
                  </div>
                  <GlassButton
                    variant="primary"
                    size="sm"
                    onClick={() => setIsProfileModalOpen(true)}
                  >
                    <User className="h-3.5 w-3.5 mr-1.5" />
                    Manage Account
                  </GlassButton>
                </div>

                {/* Microsoft Client ID configuration */}
                <div className="space-y-2 p-4 rounded-2xl bg-white/[0.03] border border-white/10">
                  <label className="text-xs font-semibold text-slate-200 block">
                    Microsoft Application Client ID
                  </label>
                  <div className="flex gap-2">
                    <GlassInput
                      value={microsoftClientId}
                      onChange={(e) => setMicrosoftClientId(e.target.value)}
                      placeholder="e.g. 3f2504e0-4f89-11d3-9a0c-0305e82c3301"
                      className="flex-1 font-mono text-xs"
                    />
                    <GlassButton
                      variant="secondary"
                      size="md"
                      onClick={() => authApi.openExternalUrl("https://portal.azure.com")}
                    >
                      Open Azure Portal
                    </GlassButton>
                  </div>
                  <p className="text-[11px] text-slate-400">
                    Required to enable "Sign in with Microsoft". Register a free public-client app at{" "}
                    <span className="text-blue-300">portal.azure.com</span> → App registrations → New registration
                    (Personal Microsoft accounts only, allow public client flows), then paste its Application (client) ID here.
                  </p>
                </div>

                <div className="p-4 rounded-2xl bg-amber-500/[0.08] border border-amber-500/20 text-xs text-amber-300 space-y-1">
                  <div className="font-semibold flex items-center gap-1.5">
                    <ShieldCheck className="h-4 w-4 text-amber-400" />
                    Offline Mode is Active (Zero Credentials / Offline Token)
                  </div>
                  <p className="text-[11px] text-amber-300/80 leading-relaxed">
                    The launcher passes an offline session to Minecraft without contacting Microsoft or Mojang authentication servers. Player UUIDs are generated locally and deterministically using Mojang's offline MD5 algorithm.
                  </p>
                </div>

                {/* Active Player Card */}
                {activeProfile && (
                  <div className="p-5 rounded-2xl bg-white/[0.03] border border-white/10 space-y-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3.5">
                        <div className="h-12 w-12 rounded-2xl bg-blue-500/20 border border-blue-400/30 flex items-center justify-center text-blue-400 font-bold text-lg shadow-inner">
                          {activeProfile.username.charAt(0).toUpperCase()}
                        </div>
                        <div>
                          <div className="flex items-center gap-2">
                            <span className="text-base font-semibold text-white">{activeProfile.username}</span>
                            <GlassBadge variant="success" size="sm">Active Offline Player</GlassBadge>
                          </div>
                          <div className="text-xs text-slate-400 font-mono flex items-center gap-1.5 mt-1">
                            <Fingerprint className="h-3.5 w-3.5 text-slate-500" />
                            <span>{activeProfile.generatedLocalUuid}</span>
                          </div>
                        </div>
                      </div>

                      <GlassButton
                        variant="secondary"
                        size="sm"
                        onClick={() => setIsProfileModalOpen(true)}
                      >
                        Switch Player
                      </GlassButton>
                    </div>

                    <div className="grid grid-cols-2 gap-4 pt-3 border-t border-white/5 text-xs text-slate-400">
                      <div className="flex items-center gap-2">
                        <Calendar className="h-3.5 w-3.5 text-slate-500" />
                        <span>Created: {formatDate(activeProfile.createdAt)}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Clock className="h-3.5 w-3.5 text-slate-500" />
                        <span>Last Used: {formatDate(activeProfile.lastUsedAt)}</span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Saved Profiles Quick List */}
                <div className="space-y-2 pt-2">
                  <span className="text-xs font-medium text-slate-300">
                    Saved Offline Profiles ({profiles.length})
                  </span>
                  <div className="space-y-2">
                    {profiles.map((p) => {
                      const isActive = activeProfile?.username === p.username;
                      return (
                        <div
                          key={p.username}
                          className={`flex items-center justify-between p-3 rounded-xl border text-xs transition-all ${
                            isActive
                              ? "bg-blue-600/15 border-blue-500/30 text-white"
                              : "bg-white/[0.02] border-white/5 text-slate-300 hover:bg-white/5"
                          }`}
                        >
                          <div className="flex items-center gap-2.5">
                            <User className={`h-4 w-4 ${isActive ? "text-blue-400" : "text-slate-500"}`} />
                            <div>
                              <span className="font-medium text-white">{p.username}</span>
                              <span className="text-[10px] text-slate-500 font-mono ml-2">
                                {p.generatedLocalUuid}
                              </span>
                            </div>
                          </div>

                          {isActive ? (
                            <span className="text-[11px] font-semibold text-emerald-400 flex items-center gap-1">
                              <Check className="h-3 w-3" /> Selected
                            </span>
                          ) : (
                            <GlassButton
                              variant="ghost"
                              size="sm"
                              className="h-7 text-xs"
                              onClick={() => selectProfile(p.username)}
                            >
                              Select
                            </GlassButton>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>
            )}

            {/* Minecraft */}
            {activeTab === "minecraft" && (
              <div className="space-y-6">
                <h3 className="text-sm font-semibold text-white">Default Minecraft Options</h3>

                <div className="space-y-4">
                  <div>
                    <div className="flex justify-between text-xs mb-2">
                      <span className="text-slate-300 font-medium">Default Maximum Memory (RAM)</span>
                      <span className="text-blue-400 font-mono font-semibold">{defaultMaxRam / 1024} GB</span>
                    </div>
                    <input
                      type="range"
                      min={1024}
                      max={16384}
                      step={1024}
                      value={defaultMaxRam}
                      onChange={(e) => setDefaultMaxRam(Number(e.target.value))}
                      className="w-full accent-blue-500 cursor-pointer"
                    />
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-2">
                      <span className="text-slate-300 font-medium">Default Minimum Memory (RAM)</span>
                      <span className="text-slate-400 font-mono font-semibold">{defaultMinRam / 1024} GB</span>
                    </div>
                    <input
                      type="range"
                      min={512}
                      max={defaultMaxRam}
                      step={512}
                      value={defaultMinRam}
                      onChange={(e) => setDefaultMinRam(Number(e.target.value))}
                      className="w-full accent-blue-500 cursor-pointer"
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Java Runtime */}
            {activeTab === "java" && (
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-semibold text-white">Java Runtime Manager</h3>
                    <p className="text-xs text-slate-400">
                      Minecraft requires Java 21+ for modern versions (1.20.5+) and Java 17 for 1.18 - 1.20.4.
                    </p>
                  </div>
                  <GlassButton
                    variant="secondary"
                    size="sm"
                    onClick={() => detectJava()}
                    isLoading={isDetectingJava}
                  >
                    <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
                    Detect Local Java
                  </GlassButton>
                </div>

                <div className="space-y-3">
                  <GlassInput
                    label="Default Java Executable Path (Optional override)"
                    placeholder="Leave blank to use auto-detected runtime"
                    value={defaultJavaPath}
                    onChange={(e) => setDefaultJavaPath(e.target.value)}
                  />

                  <div className="space-y-2 pt-2">
                    <span className="text-xs font-medium text-slate-300">
                      Detected Java Installations ({javaRuntimes.length})
                    </span>

                    <div className="space-y-2">
                      {javaRuntimes.length === 0 ? (
                        <div className="p-4 rounded-xl bg-white/[0.03] border border-white/5 text-xs text-slate-400 text-center">
                          No Java installations detected automatically.
                        </div>
                      ) : (
                        javaRuntimes.map((r) => (
                          <div
                            key={r.id}
                            onClick={() => setDefaultJavaPath(r.path)}
                            className={`p-3 rounded-xl border text-xs cursor-pointer transition-all ${
                              defaultJavaPath === r.path
                                ? "bg-blue-600/20 border-blue-500/40 text-white"
                                : "bg-white/[0.02] border-white/10 text-slate-300 hover:bg-white/5"
                            }`}
                          >
                            <div className="flex items-center justify-between">
                              <span className="font-semibold text-white">{r.name}</span>
                              <GlassBadge variant={r.majorVersion >= 21 ? "success" : "default"}>
                                Java {r.majorVersion}
                              </GlassBadge>
                            </div>
                            <p className="text-[11px] text-slate-400 font-mono truncate mt-1">
                              {r.path}
                            </p>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Appearance */}
            {activeTab === "appearance" && (
              <div className="space-y-6">
                <h3 className="text-sm font-semibold text-white">Visual & Theme</h3>

                <div className="space-y-4">
                  <div className="space-y-2">
                    <span className="text-xs font-medium text-slate-300">Color Theme</span>
                    <div className="grid grid-cols-3 gap-3">
                      {["dark", "light", "system"].map((t) => (
                        <button
                          key={t}
                          onClick={() => setTheme(t)}
                          className={`p-3 rounded-xl text-xs font-medium capitalize border transition-all ${
                            theme === t
                              ? "bg-blue-600/20 text-blue-400 border-blue-500/40"
                              : "bg-white/[0.02] text-slate-400 border-white/10 hover:bg-white/5"
                          }`}
                        >
                          {t}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div>
                    <div className="flex justify-between text-xs mb-2">
                      <span className="text-slate-300 font-medium">Liquid Glass Blur Intensity</span>
                      <span className="text-blue-400 font-mono">{blurIntensity}%</span>
                    </div>
                    <input
                      type="range"
                      min={0}
                      max={100}
                      value={blurIntensity}
                      onChange={(e) => setBlurIntensity(Number(e.target.value))}
                      className="w-full accent-blue-500 cursor-pointer"
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Downloads */}
            {activeTab === "downloads" && (
              <div className="space-y-4">
                <h3 className="text-sm font-semibold text-white">Download Settings</h3>

                <div className="space-y-2">
                  <div className="flex justify-between text-xs mb-2">
                    <span className="text-slate-300 font-medium">Max Concurrent File Downloads</span>
                    <span className="text-blue-400 font-mono font-semibold">
                      {maxConcurrentDownloads} files
                    </span>
                  </div>
                  <input
                    type="range"
                    min={2}
                    max={16}
                    step={1}
                    value={maxConcurrentDownloads}
                    onChange={(e) => setMaxConcurrentDownloads(Number(e.target.value))}
                    className="w-full accent-blue-500 cursor-pointer"
                  />
                  <span className="text-[11px] text-slate-400 block mt-1">
                    Limits simultaneous connections to prevent network saturation. Recommended: 6 to 10.
                  </span>
                </div>
              </div>
            )}

            {/* Advanced */}
            {activeTab === "advanced" && (
              <div className="space-y-4">
                <h3 className="text-sm font-semibold text-white">Advanced & Diagnostics</h3>

                <div className="space-y-3 pt-2">
                  <div className="flex items-center justify-between p-3.5 rounded-xl bg-white/[0.03] border border-white/5">
                    <div>
                      <span className="text-xs font-medium text-white block">Launcher Data Directory</span>
                      <span className="text-[11px] text-slate-400">Contains instances, libraries, and SQLite database</span>
                    </div>
                    <GlassButton
                      variant="secondary"
                      size="sm"
                      onClick={() => instancesApi.openFolder("launcher-data")}
                    >
                      <FolderOpen className="h-3.5 w-3.5 mr-1.5" />
                      Open Folder
                    </GlassButton>
                  </div>

                  <div className="flex items-center justify-between p-3.5 rounded-xl bg-white/[0.03] border border-white/5">
                    <div>
                      <span className="text-xs font-medium text-white block">Logs Directory</span>
                      <span className="text-[11px] text-slate-400">View file logs for troubleshooting</span>
                    </div>
                    <GlassButton
                      variant="secondary"
                      size="sm"
                      onClick={() => instancesApi.openFolder("launcher-data/logs")}
                    >
                      <FolderOpen className="h-3.5 w-3.5 mr-1.5" />
                      Open Logs
                    </GlassButton>
                  </div>
                </div>
              </div>
            )}
          </GlassPanel>
        </div>
      </div>

      <AccountModal
        isOpen={isProfileModalOpen}
        onClose={() => setIsProfileModalOpen(false)}
      />
    </div>
  );
};