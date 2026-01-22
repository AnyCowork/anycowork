// Custom hooks to replace Convex hooks
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type Document } from "@/lib/api-client";
import { useSaveStatus } from "./useSaveStatus";

// Query hooks
export function useDocumentsSidebar(parentDocument?: string) {
  return useQuery({
    queryKey: ["documents", "sidebar", parentDocument],
    queryFn: () => api.documents.getSidebar(parentDocument),
  });
}

export function useDocumentById(documentId?: string) {
  return useQuery({
    queryKey: ["documents", documentId],
    queryFn: () => (documentId ? api.documents.getById(documentId) : null),
    enabled: !!documentId,
    staleTime: 1000, // Consider data stale after 1 second
    refetchOnMount: "always", // Always refetch when component mounts
  });
}

export function useDocumentsSearch() {
  return useQuery({
    queryKey: ["documents", "search"],
    queryFn: () => api.documents.getSearch(),
  });
}

export function useDocumentsTrash() {
  return useQuery({
    queryKey: ["documents", "trash"],
    queryFn: () => api.documents.getTrash(),
  });
}

// Mutation hooks
export function useCreateDocument() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      title,
      parentDocument,
    }: {
      title: string;
      parentDocument?: string;
    }) => api.documents.create(title, parentDocument),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useUpdateDocument() {
  const queryClient = useQueryClient();
  const { setSaving, setSaved, setError } = useSaveStatus();

  return useMutation({
    mutationFn: ({
      id,
      ...data
    }: {
      id: string;
      title?: string;
      content?: string;
      coverImage?: string;
      icon?: string;
      isPublished?: boolean;
    }) => api.documents.update(id, data),
    onMutate: () => {
      // Set saving status when mutation starts
      setSaving();
    },
    onSuccess: (updatedDoc, variables) => {
      // Set saved status when mutation succeeds
      setSaved();

      // Don't invalidate on content updates to avoid refetch loop
      // The editor already has the latest content

      // Invalidate if title, structure, icon, or coverImage changed
      if (
        variables.title !== undefined ||
        variables.isPublished !== undefined ||
        variables.icon !== undefined ||
        variables.coverImage !== undefined
      ) {
        queryClient.invalidateQueries({ queryKey: ["documents", "sidebar"] });
        queryClient.invalidateQueries({ queryKey: ["documents", "search"] });
        queryClient.invalidateQueries({
          queryKey: ["documents", variables.id],
        });
      }
    },
    onError: () => {
      // Set error status when mutation fails
      setError();
    },
  });
}

export function useArchiveDocument() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.documents.archive(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useRestoreDocument() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.documents.restore(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useRemoveDocument() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.documents.remove(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useRemoveIcon() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.documents.removeIcon(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ["documents", id] });
      queryClient.invalidateQueries({ queryKey: ["documents", "sidebar"] });
      queryClient.invalidateQueries({ queryKey: ["documents", "search"] });
    },
  });
}

export function useRemoveCoverImage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.documents.removeCoverImage(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ["documents", id] });
      queryClient.invalidateQueries({ queryKey: ["documents", "sidebar"] });
      queryClient.invalidateQueries({ queryKey: ["documents", "search"] });
    },
  });
}
