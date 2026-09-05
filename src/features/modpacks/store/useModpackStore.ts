import { create } from "zustand";
import { modpacksApi } from "@/services/tauri/modpacksApi";
import type {
  CatalogItemWithStatus,
  ModpackDownloadJobProgress,
  UpdatePlan,
  VerificationResult,
} from "@/types";

interface ModpackState {
  catalog: CatalogItemWithStatus[];
  isLoadingCatalog: boolean;
  catalogError: string | null;
  availableUpdatesCount: number;
  activeJobs: Record<string, ModpackDownloadJobProgress>;
  activeUpdatePlan: UpdatePlan | null;
  activeVerificationResult: VerificationResult | null;
  activeInstanceForModal: string | null;

  fetchCatalog: (forceRefresh?: boolean) => Promise<void>;
  checkAllUpdates: () => Promise<number>;
  updateJobProgress: (progress: ModpackDownloadJobProgress) => void;
  clearJob: (instanceId: string) => void;
  setActiveUpdatePlan: (plan: UpdatePlan | null, instanceId?: string) => void;
  setActiveVerificationResult: (
    res: VerificationResult | null,
    instanceId?: string
  ) => void;
  setAvailableUpdatesCount: (count: number) => void;
}

export const useModpackStore = create<ModpackState>((set, get) => ({
  catalog: [],
  isLoadingCatalog: false,
  catalogError: null,
  availableUpdatesCount: 0,
  activeJobs: {},
  activeUpdatePlan: null,
  activeVerificationResult: null,
  activeInstanceForModal: null,

  fetchCatalog: async (forceRefresh = false) => {
    set({ isLoadingCatalog: true, catalogError: null });
    try {
      const rawItems = await modpacksApi.getCatalog(forceRefresh);
      const items: CatalogItemWithStatus[] = (rawItems || []).map((i: any) => {
        const modpack = i.modpack || i.remote;
        return {
          ...i,
          modpack,
          associatedInstanceId: i.associatedInstanceId || i.installed?.instanceId,
        };
      });
      const updatesCount = items.filter((i) => i.updateAvailable).length;
      set({
        catalog: items,
        isLoadingCatalog: false,
        availableUpdatesCount: updatesCount,
      });
    } catch (err: any) {
      const message =
        err?.message || "Failed to load modpack catalog from remote CDN.";
      set({
        catalogError: message,
        isLoadingCatalog: false,
      });
    }
  },

  checkAllUpdates: async () => {
    try {
      const count = await modpacksApi.checkAllUpdates();
      set({ availableUpdatesCount: count });
      // Refresh catalog status
      get().fetchCatalog(true);
      return count;
    } catch (err) {
      console.error("Failed to check modpack updates:", err);
      return 0;
    }
  },

  updateJobProgress: (progress: ModpackDownloadJobProgress) => {
    set((state) => {
      const isComplete =
        progress.percentage >= 100 &&
        (progress.stage === "Completed" || progress.stage === "done");
      if (isComplete) {
        const next = { ...state.activeJobs };
        delete next[progress.instanceId];
        return { activeJobs: next };
      }
      return {
        activeJobs: {
          ...state.activeJobs,
          [progress.instanceId]: progress,
        },
      };
    });
  },

  clearJob: (instanceId: string) => {
    set((state) => {
      const next = { ...state.activeJobs };
      delete next[instanceId];
      return { activeJobs: next };
    });
  },

  setActiveUpdatePlan: (plan: UpdatePlan | null, instanceId?: string) => {
    set({
      activeUpdatePlan: plan,
      activeInstanceForModal: instanceId ?? null,
    });
  },

  setActiveVerificationResult: (
    res: VerificationResult | null,
    instanceId?: string
  ) => {
    set({
      activeVerificationResult: res,
      activeInstanceForModal: instanceId ?? null,
    });
  },

  setAvailableUpdatesCount: (count: number) => {
    set({ availableUpdatesCount: count });
  },
}));
