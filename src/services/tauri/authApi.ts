import { invoke } from "@tauri-apps/api/core";
import type { OfflineProfile } from "@/types/offlineProfile";

export const authApi = {
  async getActiveProfile(): Promise<OfflineProfile> {
    return invoke<OfflineProfile>("get_active_offline_profile");
  },

  async listProfiles(): Promise<OfflineProfile[]> {
    return invoke<OfflineProfile[]>("list_offline_profiles");
  },

  async createProfile(username: string): Promise<OfflineProfile> {
    return invoke<OfflineProfile>("create_offline_profile", { username });
  },

  async selectProfile(username: string): Promise<OfflineProfile> {
    return invoke<OfflineProfile>("select_offline_profile", { username });
  },

  async deleteProfile(username: string): Promise<void> {
    return invoke<void>("delete_offline_profile", { username });
  },
};
