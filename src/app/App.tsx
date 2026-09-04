import React, { useEffect } from "react";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MainLayout } from "@/layouts/MainLayout";
import { HomePage } from "@/features/home/pages/HomePage";
import { InstancesPage } from "@/features/instances/pages/InstancesPage";
import { InstanceDetailPage } from "@/features/instances/pages/InstanceDetailPage";
import { DownloadsPage } from "@/features/downloads/pages/DownloadsPage";
import { SettingsPage } from "@/features/settings/pages/SettingsPage";
import { LogsPage } from "@/features/logs/pages/LogsPage";
import { onDownloadProgress } from "@/services/tauri/events";
import { useDownloadStore } from "@/store/useDownloadStore";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

export const App: React.FC = () => {
  const updateProgress = useDownloadStore((s) => s.updateProgress);

  useEffect(() => {
    // Wire global Tauri event listener for download progress
    let unlisten: (() => void) | undefined;
    onDownloadProgress((payload) => {
      updateProgress(payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [updateProgress]);

  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route path="/" element={<MainLayout />}>
            <Route index element={<HomePage />} />
            <Route path="instances" element={<InstancesPage />} />
            <Route path="instances/:id" element={<InstanceDetailPage />} />
            <Route path="downloads" element={<DownloadsPage />} />
            <Route path="settings" element={<SettingsPage />} />
            <Route path="logs" element={<LogsPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  );
};