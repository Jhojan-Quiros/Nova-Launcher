export interface AppSettings {
  launcherDir: string;
  closeOnLaunch: boolean;
  defaultMinRam: number;
  defaultMaxRam: number;
  defaultJavaPath?: string;
  theme: "dark" | "light" | "system";
  blurIntensity: number;
  maxConcurrentDownloads: number;
  windowWidth: number;
  windowHeight: number;
  modpackCatalogUrl?: string;
  checkUpdatesOnStartup?: boolean;
  autoCheckIntervalMinutes?: number;
  allowAutomaticUpdates?: boolean;
  verifyFilesBeforeLaunch?: boolean;
  desktopNotifications?: boolean;
  strictModpackModeDefault?: boolean;
  activeAuthMode?: "offline" | "microsoft";
  microsoftClientId?: string;
}

export interface UpdateSettingsInput {
  launcherDir?: string;
  closeOnLaunch?: boolean;
  defaultMinRam?: number;
  defaultMaxRam?: number;
  defaultJavaPath?: string;
  theme?: string;
  blurIntensity?: number;
  maxConcurrentDownloads?: number;
  modpackCatalogUrl?: string;
  checkUpdatesOnStartup?: boolean;
  autoCheckIntervalMinutes?: number;
  allowAutomaticUpdates?: boolean;
  verifyFilesBeforeLaunch?: boolean;
  desktopNotifications?: boolean;
  strictModpackModeDefault?: boolean;
  microsoftClientId?: string;
}