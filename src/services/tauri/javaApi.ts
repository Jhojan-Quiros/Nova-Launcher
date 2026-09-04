import { invoke } from "@tauri-apps/api/core";
import type { JavaRuntime } from "@/types";

export const javaApi = {
  async detect(): Promise<JavaRuntime[]> {
    return invoke<JavaRuntime[]>("detect_java");
  },

  async probe(path: string): Promise<JavaRuntime> {
    return invoke<JavaRuntime>("probe_java", { path });
  },
};