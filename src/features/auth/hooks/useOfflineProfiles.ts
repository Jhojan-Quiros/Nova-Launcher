import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { authApi } from "@/services/tauri/authApi";
import type { OfflineProfile } from "@/types/offlineProfile";

export const OFFLINE_PROFILE_QUERY_KEY = ["offline-profile", "active"];
export const OFFLINE_PROFILES_LIST_KEY = ["offline-profile", "list"];

export const useOfflineProfiles = () => {
  const queryClient = useQueryClient();

  const activeProfileQuery = useQuery<OfflineProfile>({
    queryKey: OFFLINE_PROFILE_QUERY_KEY,
    queryFn: authApi.getActiveProfile,
    staleTime: 60_000,
  });

  const profilesListQuery = useQuery<OfflineProfile[]>({
    queryKey: OFFLINE_PROFILES_LIST_KEY,
    queryFn: authApi.listProfiles,
    staleTime: 60_000,
  });

  const selectMutation = useMutation({
    mutationFn: (username: string) => authApi.selectProfile(username),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OFFLINE_PROFILE_QUERY_KEY });
      queryClient.invalidateQueries({ queryKey: OFFLINE_PROFILES_LIST_KEY });
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });

  const createMutation = useMutation({
    mutationFn: (username: string) => authApi.createProfile(username),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OFFLINE_PROFILE_QUERY_KEY });
      queryClient.invalidateQueries({ queryKey: OFFLINE_PROFILES_LIST_KEY });
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (username: string) => authApi.deleteProfile(username),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OFFLINE_PROFILE_QUERY_KEY });
      queryClient.invalidateQueries({ queryKey: OFFLINE_PROFILES_LIST_KEY });
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });

  return {
    activeProfile: activeProfileQuery.data,
    profiles: profilesListQuery.data ?? [],
    isLoading: activeProfileQuery.isLoading || profilesListQuery.isLoading,
    selectProfile: selectMutation.mutateAsync,
    createProfile: createMutation.mutateAsync,
    deleteProfile: deleteMutation.mutateAsync,
    isSelecting: selectMutation.isPending,
    isCreating: createMutation.isPending,
    isDeleting: deleteMutation.isPending,
  };
};
