import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { logsApi } from "@/services/tauri/logsApi";
import { onGameLog } from "@/services/tauri/events";
import type { LogEntry } from "@/types";

export function useLogs(instanceIdFilter?: string) {
  const [liveLogs, setLiveLogs] = useState<LogEntry[]>([]);

  const logsQuery = useQuery({
    queryKey: ["logs"],
    queryFn: async () => {
      const logs = await logsApi.get();
      setLiveLogs(logs);
      return logs;
    },
    refetchInterval: 5000,
  });

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    onGameLog((event) => {
      if (instanceIdFilter && event.instance_id !== instanceIdFilter) {
        return;
      }
      const newEntry: LogEntry = {
        timestamp: new Date().toLocaleTimeString(),
        level: event.level,
        target: event.instance_id,
        message: event.message,
      };
      setLiveLogs((prev) => [...prev.slice(-1000), newEntry]);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [instanceIdFilter]);

  const clear = async () => {
    await logsApi.clear();
    setLiveLogs([]);
  };

  const filteredLogs = instanceIdFilter
    ? liveLogs.filter((l) => l.target === instanceIdFilter)
    : liveLogs;

  return {
    logs: filteredLogs,
    isLoading: logsQuery.isLoading,
    clear,
    refetch: logsQuery.refetch,
  };
}