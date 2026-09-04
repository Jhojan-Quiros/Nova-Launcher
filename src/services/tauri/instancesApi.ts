import { invoke } from "@tauri-apps/api/core";
import type { CreateInstanceInput, Instance, UpdateInstanceInput } from "@/types";

export const instancesApi = {
  async list(): Promise<Instance[]> {
    return invoke<Instance[]>("list_instances");
  },

  async get(id: string): Promise<Instance> {
    return invoke<Instance>("get_instance", { id });
  },

  async create(dto: CreateInstanceInput): Promise<Instance> {
    return invoke<Instance>("create_instance", { dto });
  },

  async update(id: string, dto: UpdateInstanceInput): Promise<Instance> {
    return invoke<Instance>("update_instance", { id, dto });
  },

  async delete(id: string): Promise<boolean> {
    return invoke<boolean>("delete_instance", { id });
  },

  async openFolder(path: string): Promise<void> {
    return invoke<void>("open_folder", { path });
  },
};