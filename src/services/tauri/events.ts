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