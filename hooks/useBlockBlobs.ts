/**
 * Hook for managing blobs (images/files) associated with blocks
 */
import { useState, useCallback } from "react";

interface Blob {
  id: string;
  filename: string;
  size: number;
  mime_type: string;
  url: string;
  block_id?: string;
  created_at: number;
}

export const useBlockBlobs = () => {
  const [blobs, setBlobs] = useState<Blob[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const apiBase = import.meta.env.VITE_API_URL || "/api";

  /**
   * Upload a file and associate it with a block
   */
  const uploadBlob = useCallback(
    async (file: File, blockId?: string): Promise<Blob> => {
      setLoading(true);
      setError(null);

      try {
        const formData = new FormData();
        formData.append("file", file);
        if (blockId) {
          formData.append("block_id", blockId);
        }

        const response = await fetch(`${apiBase}/blobs/upload`, {
          method: "POST",
          body: formData,
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.detail || "Upload failed");
        }

        const blob = await response.json();
        return blob;
      } catch (err) {
        const message = err instanceof Error ? err.message : "Upload failed";
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [apiBase]
  );

  /**
   * Get all blobs for a specific block
   */
  const fetchBlockBlobs = useCallback(
    async (blockId: string): Promise<Blob[]> => {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch(`${apiBase}/blobs/block/${blockId}/list`);

        if (!response.ok) {
          throw new Error("Failed to fetch blobs");
        }

        const data = await response.json();
        setBlobs(data);
        return data;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to fetch blobs";
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [apiBase]
  );

  /**
   * Delete a blob
   */
  const deleteBlob = useCallback(
    async (blobId: string, deleteFile: boolean = false): Promise<void> => {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch(
          `${apiBase}/blobs/${blobId}?delete_file=${deleteFile}`,
          {
            method: "DELETE",
          }
        );

        if (!response.ok) {
          throw new Error("Failed to delete blob");
        }

        // Remove from local state
        setBlobs((prev) => prev.filter((b) => b.id !== blobId));
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to delete blob";
        setError(message);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [apiBase]
  );

  /**
   * Get blob metadata
   */
  const getBlob = useCallback(
    async (blobId: string): Promise<Blob> => {
      const response = await fetch(`${apiBase}/blobs/${blobId}`);

      if (!response.ok) {
        throw new Error("Failed to fetch blob");
      }

      return response.json();
    },
    [apiBase]
  );

  return {
    blobs,
    loading,
    error,
    uploadBlob,
    fetchBlockBlobs,
    deleteBlob,
    getBlob,
  };
};
