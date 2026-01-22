import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { settingsApi, type RAGFullConfig } from "@/lib/settings-api";
import { toast } from "sonner";

export function useRAGFullConfig() {
  return useQuery({
    queryKey: ["settings", "rag", "full"],
    queryFn: settingsApi.getFullConfig,
  });
}

export function useUpdateRAGFullConfig() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (config: RAGFullConfig) => settingsApi.updateFullConfig(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings", "rag"] });
      queryClient.invalidateQueries({ queryKey: ["rag", "config"] });
      toast.success("Configuration saved successfully");
    },
    onError: (error: Error) => {
      toast.error(error.message || "Failed to save configuration");
    },
  });
}
