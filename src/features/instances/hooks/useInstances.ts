import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { instancesApi } from "@/services/tauri/instancesApi";
import { minecraftApi } from "@/services/tauri/minecraftApi";
import type { CreateInstanceInput, UpdateInstanceInput } from "@/types";

export function useInstances() {
  const queryClient = useQueryClient();

  const instancesQuery = useQuery({
    queryKey: ["instances"],
    queryFn: () => instancesApi.list(),
    refetchInterval: 2500, // keep status (installing, running) updated
  });

  const createMutation = useMutation({
    mutationFn: (dto: CreateInstanceInput) => instancesApi.create(dto),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["instances"] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, dto }: { id: string; dto: UpdateInstanceInput }) =>
      instancesApi.update(id, dto),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["instances"] });
      queryClient.invalidateQueries({ queryKey: ["instance", variables.id] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => instancesApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["instances"] });
    },
  });

  const installMutation = useMutation({
    mutationFn: (instanceId: string) => minecraftApi.install(instanceId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["instances"] });
    },
  });

  const launchMutation = useMutation({
    mutationFn: (instanceId: string) => minecraftApi.launch(instanceId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["instances"] });
    },
  });

  return {
    instances: instancesQuery.data ?? [],
    isLoading: instancesQuery.isLoading,
    error: instancesQuery.error,
    createInstance: createMutation.mutateAsync,
    isCreating: createMutation.isPending,
    updateInstance: updateMutation.mutateAsync,
    deleteInstance: deleteMutation.mutateAsync,
    installInstance: installMutation.mutateAsync,
    isInstalling: installMutation.isPending,
    launchInstance: launchMutation.mutateAsync,
    isLaunching: launchMutation.isPending,
    refetch: instancesQuery.refetch,
  };
}

export function useInstance(id: string) {
  return useQuery({
    queryKey: ["instance", id],
    queryFn: () => instancesApi.get(id),
    enabled: Boolean(id),
    refetchInterval: 2000,
  });
}