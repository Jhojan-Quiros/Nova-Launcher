export interface DownloadProgressEvent {
  instanceId?: string;
  file: string;
  downloadedBytes: number;
  totalBytes: number;
  percentage: number;
  speedBytesPerSec: number;
}