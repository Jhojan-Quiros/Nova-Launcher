import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { authApi } from "@/services/tauri/authApi";
import type { DeviceCodeInfo, MicrosoftAccountInfo } from "@/types/account";

export const MICROSOFT_ACCOUNT_QUERY_KEY = ["microsoft-account"];

export const useMicrosoftAuth = () => {
  const queryClient = useQueryClient();

  const accountQuery = useQuery<MicrosoftAccountInfo | null>({
    queryKey: MICROSOFT_ACCOUNT_QUERY_KEY,
    queryFn: authApi.getMicrosoftAccount,
    staleTime: 60_000,
  });

  const invalidateAfterAuthChange = () => {
    queryClient.invalidateQueries({ queryKey: MICROSOFT_ACCOUNT_QUERY_KEY });
    queryClient.invalidateQueries({ queryKey: ["settings"] });
  };

  const beginLoginMutation = useMutation({
    mutationFn: (): Promise<DeviceCodeInfo> => authApi.beginMicrosoftLogin(),
  });

  const completeLoginMutation = useMutation({
    mutationFn: (info: DeviceCodeInfo) =>
      authApi.completeMicrosoftLogin(info.deviceCode, info.interval, info.expiresIn),
    onSuccess: invalidateAfterAuthChange,
  });

  const signOutMutation = useMutation({
    mutationFn: () => authApi.signOutMicrosoft(),
    onSuccess: invalidateAfterAuthChange,
  });

  const activateMutation = useMutation({
    mutationFn: () => authApi.activateMicrosoftAccount(),
    onSuccess: invalidateAfterAuthChange,
  });

  return {
    account: accountQuery.data ?? null,
    isLoadingAccount: accountQuery.isLoading,
    beginLogin: beginLoginMutation.mutateAsync,
    isBeginningLogin: beginLoginMutation.isPending,
    completeLogin: completeLoginMutation.mutateAsync,
    isCompletingLogin: completeLoginMutation.isPending,
    signOut: signOutMutation.mutateAsync,
    isSigningOut: signOutMutation.isPending,
    activate: activateMutation.mutateAsync,
    isActivating: activateMutation.isPending,
  };
};
