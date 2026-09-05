import { useQuery } from "@tanstack/react-query";
import { minecraftApi } from "@/services/tauri/minecraftApi";

export function useForgeVersions(mcVersion: string, enabled: boolean) {
  return useQuery({
    queryKey: ["forge-versions", mcVersion],
    queryFn: () => minecraftApi.getForgeVersions(mcVersion),
    staleTime: 1000 * 60 * 30,
    enabled,
    retry: false,
  });
}
