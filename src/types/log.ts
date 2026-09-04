export interface LogEntry {
  timestamp: string;
  level: "ERROR" | "WARN" | "INFO" | "DEBUG" | string;
  target: string;
  message: string;
}

export interface GameLogEvent {
  instance_id: string;
  level: string;
  message: string;
}