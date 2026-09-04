import { invoke } from "@tauri-apps/api/core";
import type { LogEntry } from "@/types";

export const logsApi = {
  async get(): Promise<LogEntry[]> {
    return invoke<LogEntry[]>("get_logs");
  },

  async clear(): Promise<void> {
    return invoke<void>("clear_logs");
  },
};