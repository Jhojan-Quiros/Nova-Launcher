import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloadProgressEvent, GameLogEvent } from "@/types";

export async function onDownloadProgress(
  callback: (payload: DownloadProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgressEvent>("download-progress", (event) => {
    callback(event.payload);
  });
}

export async function onGameLog(
  callback: (payload: GameLogEvent) => void
): Promise<UnlistenFn> {
  return listen<GameLogEvent>("game-log", (event) => {
    callback(event.payload);
  });
}

export async function onModpackProgress(
  callback: (payload: import("@/types").ModpackDownloadJobProgress) => void
): Promise<UnlistenFn> {
  return listen<import("@/types").ModpackDownloadJobProgress>(
    "modpack-update-progress",
    (event) => {
      callback(event.payload);
    }
  );
}

export async function onModpackUpdatesFound(
  callback: (count: number) => void
): Promise<UnlistenFn> {
  return listen<number>("modpack-updates-found", (event) => {
    callback(event.payload);
  });
}