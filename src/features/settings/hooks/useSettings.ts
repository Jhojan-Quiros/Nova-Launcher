import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { settingsApi } from "@/services/tauri/settingsApi";
import { javaApi } from "@/services/tauri/javaApi";
import type { UpdateSettingsInput } from "@/types";

export function useSettings() {
  const queryClient = useQueryClient();

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: () => settingsApi.get(),
  });

  const updateMutation = useMutation({
    mutationFn: (dto: UpdateSettingsInput) => settingsApi.update(dto),
    onSuccess: (data) => {
      queryClient.setQueryData(["settings"], data);
    },
  });

  const javaRuntimesQuery = useQuery({
    queryKey: ["java-runtimes"],
    queryFn: () => javaApi.detect(),
  });

  return {
    settings: settingsQuery.data,
    isLoading: settingsQuery.isLoading,
    updateSettings: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
    javaRuntimes: javaRuntimesQuery.data ?? [],
    isDetectingJava: javaRuntimesQuery.isLoading,
    detectJava: javaRuntimesQuery.refetch,
  };
}