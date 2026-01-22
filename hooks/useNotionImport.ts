import { useMutation, useQueryClient } from "@tanstack/react-query";
import { settingsApi } from "@/lib/settings-api";
import { toast } from "sonner";

export function useImportNotionFile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      file,
      parentId,
    }: {
      file: File;
      parentId?: string;
    }) => settingsApi.importNotionFile(file, parentId),
    onSuccess: (result) => {
      // Invalidate documents queries to refresh the sidebar
      queryClient.invalidateQueries({ queryKey: ["documents"] });
      queryClient.invalidateQueries({ queryKey: ["pages"] });
      
      toast.success(
        `Imported "${result.title}" with ${result.blocks_count} blocks`
      );
    },
    onError: (error: Error) => {
      toast.error(error.message || "Failed to import file");
    },
  });
}

export function useImportNotionZip() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      file,
      parentId,
    }: {
      file: File;
      parentId?: string;
    }) => settingsApi.importNotionZip(file, parentId),
    onSuccess: (result) => {
      // Invalidate documents queries to refresh the sidebar
      queryClient.invalidateQueries({ queryKey: ["documents"] });
      queryClient.invalidateQueries({ queryKey: ["pages"] });
      
      toast.success(
        `Imported ${result.imported} of ${result.total_files} pages`
      );
      
      if (result.failed > 0) {
        toast.warning(`${result.failed} pages failed to import`);
      }
    },
    onError: (error: Error) => {
      toast.error(error.message || "Failed to import ZIP file");
    },
  });
}
