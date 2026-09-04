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
      return {
        isDownloading: !isComplete,
        currentFile: progress.file,
        currentInstanceId: progress.instanceId,
        downloadedBytes: progress.downloadedBytes,
        totalBytes: progress.totalBytes,
        percentage: progress.percentage,
        speed: progress.speedBytesPerSec,
        history: [progress, ...state.history.slice(0, 19)],
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