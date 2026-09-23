import { invoke } from "@tauri-apps/api/core";
import type {
  CatalogItemWithStatus,
  InstalledModpack,
  InstallModpackInput,
  ModpackMainManifest,
  ModpackUpdateHistoryRecord,
  ModpackVersionManifest,
  UpdatePlan,
  VerificationResult,
} from "@/types";

export const modpacksApi = {
  async getCatalog(forceRefresh: boolean = false): Promise<CatalogItemWithStatus[]> {
    return invoke<CatalogItemWithStatus[]>("get_modpack_catalog", {
      forceRefresh,
    });
  },

  async getDetails(
    manifestUrl: string
  ): Promise<[ModpackMainManifest, ModpackVersionManifest]> {
    return invoke<[ModpackMainManifest, ModpackVersionManifest]>(
      "get_modpack_details",
      { manifestUrl }
    );
  },

  async checkUpdate(instanceId: string): Promise<string | null> {
    return invoke<string | null>("check_modpack_update", { instanceId });
  },

  async checkAllUpdates(): Promise<number> {
    return invoke<number>("check_all_modpack_updates");
  },

  async prepareUpdate(
    instanceId: string,
    targetVersion?: string
  ): Promise<UpdatePlan> {
    return invoke<UpdatePlan>("prepare_modpack_update", {
      instanceId,
      targetVersion: targetVersion ?? null,
    });
  },

  async install(dto: InstallModpackInput): Promise<InstalledModpack> {
    return invoke<InstalledModpack>("install_modpack", { dto });
  },

  async update(
    instanceId: string,
    targetVersion?: string
  ): Promise<InstalledModpack> {
    return invoke<InstalledModpack>("update_modpack", {
      instanceId,
      targetVersion: targetVersion ?? null,
    });
  },

  async verify(instanceId: string): Promise<VerificationResult> {
    return invoke<VerificationResult>("verify_modpack", { instanceId });
  },

  async repair(instanceId: string): Promise<VerificationResult> {
    return invoke<VerificationResult>("repair_modpack", { instanceId });
  },

  async cancel(instanceId: string): Promise<boolean> {
    return invoke<boolean>("cancel_modpack_operation", { instanceId });
  },

  async getHistory(
    instanceId: string
  ): Promise<ModpackUpdateHistoryRecord[]> {
    return invoke<ModpackUpdateHistoryRecord[]>("get_modpack_update_history", {
      instanceId,
    });
  },

  async getInstalledForInstance(
    instanceId: string
  ): Promise<InstalledModpack | null> {
    return invoke<InstalledModpack | null>("get_installed_modpack", {
      instanceId,
    });
  },

  async listInstalled(): Promise<InstalledModpack[]> {
    return invoke<InstalledModpack[]>("list_installed_modpacks");
  },
};
