import { invoke } from "@tauri-apps/api/core";
import type { ForgeVersionOption, VersionFilter, VersionListResponse } from "@/types";

export const minecraftApi = {
  async getVersions(filter?: VersionFilter): Promise<VersionListResponse> {
    return invoke<VersionListResponse>("get_minecraft_versions", { filter });
  },

  async getForgeVersions(mcVersion: string): Promise<ForgeVersionOption[]> {
    return invoke<ForgeVersionOption[]>("get_forge_versions", { mcVersion });
  },

  async install(instanceId: string): Promise<void> {
    return invoke<void>("install_instance", { instanceId });
  },

  async launch(instanceId: string): Promise<void> {
    return invoke<void>("launch_instance", { instanceId });
  },
};