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
} from "lucide-react";
import { GlassPanel, GlassButton, GlassInput, GlassBadge } from "@/components/ui/glass";
import { useSettings } from "@/features/settings/hooks/useSettings";
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

  const [activeTab, setActiveTab] = useState<
    "general" | "minecraft" | "java" | "appearance" | "downloads" | "advanced"
  >("general");

  // Local form state
  const [closeOnLaunch, setCloseOnLaunch] = useState(false);
  const [defaultMinRam, setDefaultMinRam] = useState(2048);
  const [defaultMaxRam, setDefaultMaxRam] = useState(4096);
  const [defaultJavaPath, setDefaultJavaPath] = useState("");
  const [theme, setTheme] = useState("dark");
  const [blurIntensity, setBlurIntensity] = useState(100);
  const [maxConcurrentDownloads, setMaxConcurrentDownloads] = useState(8);
  const [savedSuccess, setSavedSuccess] = useState(false);

  useEffect(() => {
    if (settings) {
      setCloseOnLaunch(settings.closeOnLaunch);
      setDefaultMinRam(settings.defaultMinRam);
      setDefaultMaxRam(settings.defaultMaxRam);
      setDefaultJavaPath(settings.defaultJavaPath || "");
      setTheme(settings.theme);
      setBlurIntensity(settings.blurIntensity);
      setMaxConcurrentDownloads(settings.maxConcurrentDownloads);
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
    });
    setSavedSuccess(true);
    setTimeout(() => setSavedSuccess(false), 2500);
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
    </div>
  );
};