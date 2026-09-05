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
import { ModpacksPage } from "@/features/modpacks/pages/ModpacksPage";
import { ModpackDetailPage } from "@/features/modpacks/pages/ModpackDetailPage";
import { ModpackProgressOverlay } from "@/features/modpacks/components/ModpackProgressOverlay";
import {
  onDownloadProgress,
  onModpackProgress,
  onModpackUpdatesFound,
} from "@/services/tauri/events";
import { useDownloadStore } from "@/store/useDownloadStore";
import { useModpackStore } from "@/features/modpacks/store/useModpackStore";
import { ErrorBoundary } from "@/components/common/ErrorBoundary";

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
    let unlistenDl: (() => void) | undefined;
    let unlistenModpack: (() => void) | undefined;
    let unlistenUpdates: (() => void) | undefined;

    onDownloadProgress((payload) => {
      updateProgress(payload);
    }).then((fn) => {
      unlistenDl = fn;
    });

    onModpackProgress((payload) => {
      useModpackStore.getState().updateJobProgress(payload);
    }).then((fn) => {
      unlistenModpack = fn;
    });

    onModpackUpdatesFound((count) => {
      useModpackStore.getState().setAvailableUpdatesCount(count);
    }).then((fn) => {
      unlistenUpdates = fn;
    });

    // Initial catalog fetch
    useModpackStore.getState().fetchCatalog();

    return () => {
      if (unlistenDl) unlistenDl();
      if (unlistenModpack) unlistenModpack();
      if (unlistenUpdates) unlistenUpdates();
    };
  }, [updateProgress]);

  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <ErrorBoundary>
          <Routes>
            <Route path="/" element={<MainLayout />}>
              <Route index element={<HomePage />} />
              <Route path="instances" element={<InstancesPage />} />
              <Route path="instances/:id" element={<InstanceDetailPage />} />
              <Route path="modpacks" element={<ModpacksPage />} />
              <Route path="modpacks/:id" element={<ModpackDetailPage />} />
              <Route path="downloads" element={<DownloadsPage />} />
              <Route path="settings" element={<SettingsPage />} />
              <Route path="logs" element={<LogsPage />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Route>
          </Routes>
          <ModpackProgressOverlay />
        </ErrorBoundary>
      </HashRouter>
    </QueryClientProvider>
  );
};