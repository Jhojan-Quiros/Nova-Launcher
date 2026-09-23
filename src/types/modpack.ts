export type ModpackStatus =
  | "NotInstalled"
  | "Installed"
  | "UpdateAvailable"
  | "Updating"
  | "Corrupted"
  | "Installing";

export interface RemoteModpack {
  id: string;
  name: string;
  slug?: string | null;
  description?: string | null;
  author?: string | null;
  iconUrl?: string | null;
  bannerUrl?: string | null;
  latestVersion: string;
  minecraftVersion: string;
  loader: string;
  loaderVersion?: string | null;
  manifestUrl: string;
  tags?: string[];
  downloadsCount?: number;
}

export interface InstalledModpack {
  id: string;
  instanceId: string;
  modpackId: string;
  installedVersion: string;
  manifestUrl: string;
  installedAt: string;
  lastUpdatedAt: string;
  autoUpdate: boolean;
  strictMode: boolean;
  status: ModpackStatus;
  updateAvailable: boolean;
  latestKnownVersion?: string;
}

export interface CatalogItemWithStatus {
  modpack: RemoteModpack;
  status: ModpackStatus;
  installedVersion?: string;
  updateAvailable: boolean;
  associatedInstanceId?: string;
}

export interface ManifestFileEntry {
  path: string;
  fileName: string;
  category?: string;
  size: number;
  sha256: string;
  url: string;
  modId?: string | null;
}

export interface VersionStats {
  totalFiles: number;
  added: number;
  updated: number;
  removed: number;
  unchanged?: number;
  totalSize: number;
  downloadSize: number;
}

export interface ModpackVersionManifest {
  schemaVersion: number;
  packId: string;
  version: string;
  minecraftVersion: string;
  loader: string;
  loaderVersion?: string | null;
  releaseDate?: string | null;
  changelog: string[];
  stats?: VersionStats;
  files: ManifestFileEntry[];
  removedFiles: string[];
}

export interface ModpackMainManifestVersionRef {
  version: string;
  minecraftVersion: string;
  loader: string;
  releaseDate?: string | null;
  manifestUrl: string;
}

export interface ModpackMainManifest {
  schemaVersion: number;
  id: string;
  name: string;
  description?: string | null;
  author?: string | null;
  iconUrl?: string | null;
  bannerUrl?: string | null;
  latestVersion: string;
  minecraftVersion: string;
  loader: string;
  loaderVersion?: string | null;
  releaseDate?: string | null;
  manifestUrl: string;
  versions?: ModpackMainManifestVersionRef[];
}

export interface UpdatePlan {
  packId: string;
  fromVersion?: string;
  toVersion: string;
  downloads: ManifestFileEntry[];
  deletions: string[];
  unmodifiedCount: number;
  totalDownloadSize: number;
  totalFiles: number;
  changelog: string[];
}

export interface VerificationResult {
  totalChecked: number;
  valid: number;
  missing: string[];
  modified: string[];
  unmanagedUserFiles: string[];
  repairSize: number;
  filesToRepair: ManifestFileEntry[];
}

export interface ModpackUpdateHistoryRecord {
  id: string;
  instanceId: string;
  packId: string;
  fromVersion: string;
  toVersion: string;
  status: string;
  downloadSize: number;
  startedAt: string;
  completedAt?: string;
  errorCode?: string;
}

export interface ModpackDownloadJobProgress {
  instanceId: string;
  packId: string;
  version: string;
  downloadedBytes: number;
  totalBytes: number;
  percentage: number;
  currentFile: string;
  filesCompleted: number;
  totalFiles: number;
  stage: string;
}

export interface InstallModpackInput {
  catalogPack: RemoteModpack;
  instanceName?: string;
  customRam?: { minMb: number; maxMb: number };
  strictMode?: boolean;
  autoUpdate?: boolean;
}

export interface ModpackErrorPayload {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}
