export type VersionType = "release" | "snapshot" | "old_beta" | "old_alpha";

export interface MinecraftVersion {
  id: string;
  versionType: VersionType;
  url: string;
  releaseTime: string;
  sha1?: string;
}

export interface VersionListResponse {
  latestRelease: string;
  latestSnapshot: string;
  versions: MinecraftVersion[];
}

export interface VersionFilter {
  showReleases: boolean;
  showSnapshots: boolean;
  showOldBeta: boolean;
  showOldAlpha: boolean;
}

export interface ForgeVersionOption {
  version: string;
  label: "recommended" | "latest";
}