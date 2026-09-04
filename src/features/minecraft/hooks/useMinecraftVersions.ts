import { useQuery } from "@tanstack/react-query";
import { minecraftApi } from "@/services/tauri/minecraftApi";
import type { VersionFilter } from "@/types";

export function useMinecraftVersions(filter?: VersionFilter) {
  return useQuery({
    queryKey: ["minecraft-versions", filter],
    queryFn: () => minecraftApi.getVersions(filter),
    staleTime: 1000 * 60 * 30, // 30 minutes cache
  });
}