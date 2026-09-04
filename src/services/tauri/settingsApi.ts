import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, UpdateSettingsInput } from "@/types";

export const settingsApi = {
  async get(): Promise<AppSettings> {
    return invoke<AppSettings>("get_settings");
  },

  async update(dto: UpdateSettingsInput): Promise<AppSettings> {
    return invoke<AppSettings>("update_settings", { dto });
  },
};