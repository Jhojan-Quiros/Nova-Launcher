import { invoke } from "@tauri-apps/api/core";
import type { OfflineProfile } from "@/types/offlineProfile";
import type { DeviceCodeInfo, MicrosoftAccountInfo, MinecraftAccount } from "@/types/account";

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

  async beginMicrosoftLogin(): Promise<DeviceCodeInfo> {
    return invoke<DeviceCodeInfo>("begin_microsoft_login");
  },

  async completeMicrosoftLogin(deviceCode: string, interval: number, expiresIn: number): Promise<MinecraftAccount> {
    return invoke<MinecraftAccount>("complete_microsoft_login", { deviceCode, interval, expiresIn });
  },

  async getMicrosoftAccount(): Promise<MicrosoftAccountInfo | null> {
    return invoke<MicrosoftAccountInfo | null>("get_microsoft_account");
  },

  async signOutMicrosoft(): Promise<void> {
    return invoke<void>("sign_out_microsoft");
  },

  async activateMicrosoftAccount(): Promise<void> {
    return invoke<void>("activate_microsoft_account");
  },

  async openExternalUrl(url: string): Promise<void> {
    return invoke<void>("open_external_url", { url });
  },
};
