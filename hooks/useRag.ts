import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ragApi } from "@/lib/rag-api";

export function useRAGSearch(query: string, enabled: boolean = true) {
  return useQuery({
    queryKey: ["rag", "search", query],
    queryFn: () => ragApi.search(query),
    enabled: enabled && !!query,
  });
}

export function useRAGChat() {
  return useMutation({
    mutationFn: ({ question, mode }: { question: string; mode?: string }) =>
      ragApi.chat(question, mode),
  });
}

export function useRAGRecommendations(blockId?: string) {
  return useQuery({
    queryKey: ["rag", "recommendations", blockId],
    queryFn: () => ragApi.getRecommendations(blockId!),
    enabled: !!blockId,
  });
}

export function useRAGConfig() {
  return useQuery({
    queryKey: ["rag", "config"],
    queryFn: ragApi.getConfig,
  });
}

export function useUpdateRAGConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ragApi.updateConfig,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["rag", "config"] });
    },
  });
}

export function useRebuildRAGIndex() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ragApi.rebuildIndex,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["rag"] });
    },
  });
}

export function useRAGStats() {
  return useQuery({
    queryKey: ["rag", "stats"],
    queryFn: ragApi.getStats,
  });
}
