export type ModLoader = "vanilla" | "fabric" | "forge" | "neoforge";

export type InstanceStatus =
  | { idle: null }
  | { installing: null }
  | { ready: null }
  | { running: null }
  | { error: string }
  | "idle"
  | "installing"
  | "ready"
  | "running";

export interface RamConfig {
  minMb: number;
  maxMb: number;
}

export interface Instance {
  id: string;
  name: string;
  minecraftVersion: string;
  loader: ModLoader;
  loaderVersion?: string;
  gameDirectory: string;
  javaPath?: string;
  javaVersion?: number;
  ram: RamConfig;
  icon?: string;
  status: InstanceStatus;
  totalPlayTimeSeconds: number;
  lastPlayedAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateInstanceInput {
  name: string;
  minecraftVersion: string;
  loader: ModLoader;
  loaderVersion?: string;
  minRam?: number;
  maxRam?: number;
}

export interface UpdateInstanceInput {
  name?: string;
  javaPath?: string;
  minRam?: number;
  maxRam?: number;
}