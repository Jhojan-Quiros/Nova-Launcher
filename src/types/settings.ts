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
}