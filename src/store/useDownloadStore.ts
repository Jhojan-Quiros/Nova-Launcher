import { create } from "zustand";
import type { DownloadProgressEvent } from "@/types";

interface DownloadStore {
  isDownloading: boolean;
  currentFile: string;
  currentInstanceId?: string;
  downloadedBytes: number;
  totalBytes: number;
  percentage: number;
  speed: number;
  history: DownloadProgressEvent[];
  updateProgress: (progress: DownloadProgressEvent) => void;
  resetProgress: () => void;
}

export const useDownloadStore = create<DownloadStore>((set) => ({
  isDownloading: false,
  currentFile: "",
  currentInstanceId: undefined,
  downloadedBytes: 0,
  totalBytes: 0,
  percentage: 0,
  speed: 0,
  history: [],
  updateProgress: (progress) =>
    set((state) => {
      const isComplete = progress.percentage >= 100;

      let newHistory = state.history;
      if (
        state.currentFile &&
        state.currentFile !== progress.file &&
        !state.history.some((h) => h.file === state.currentFile)
      ) {
        newHistory = [
          {
            instanceId: state.currentInstanceId,
            file: state.currentFile,
            downloadedBytes: progress.downloadedBytes,
            totalBytes: progress.totalBytes,
            percentage: 100,
            speedBytesPerSec: progress.speedBytesPerSec,
          },
          ...state.history.slice(0, 19),
        ];
      }

      return {
        isDownloading: !isComplete,
        currentFile: isComplete ? "" : progress.file,
        currentInstanceId: progress.instanceId,
        downloadedBytes: progress.downloadedBytes,
        totalBytes: progress.totalBytes,
        percentage: progress.percentage,
        speed: progress.speedBytesPerSec,
        history: newHistory,
      };
    }),
  resetProgress: () =>
    set({
      isDownloading: false,
      currentFile: "",
      currentInstanceId: undefined,
      downloadedBytes: 0,
      totalBytes: 0,
      percentage: 0,
      speed: 0,
    }),
}));