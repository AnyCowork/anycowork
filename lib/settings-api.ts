import { fetchWithRetry } from "@/lib/api-client";

export interface RAGFullConfig {
  enabled: boolean;
  auto_index: boolean;
  leann: {
    backend_name: string;
    embedding_model: string;
    embedding_mode: string;
    is_compact: boolean;
    is_recompute: boolean;
    distance_metric: string;
    llm_type: string;
    llm_model: string;
  };
}

export interface ImportResult {
  page_id: string;
  title: string;
  blocks_count: number;
}

export interface ImportZipResult {
  total_files: number;
  imported: number;
  failed: number;
  results: Array<{
    success: boolean;
    filename: string;
    page_id?: string;
    title?: string;
    blocks_count?: number;
    error?: string;
  }>;
}

export const settingsApi = {
  // RAG Configuration
  getFullConfig: async (): Promise<RAGFullConfig> => {
    const response = await fetchWithRetry("/api/rag/config/full");
    return await response.json();
  },

  updateFullConfig: async (
    config: RAGFullConfig
  ): Promise<{ success: boolean }> => {
    const response = await fetchWithRetry("/api/rag/config/full", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(config),
    });
    return await response.json();
  },

  // Notion Import
  importNotionFile: async (
    file: File,
    parentId?: string
  ): Promise<ImportResult> => {
    const formData = new FormData();
    formData.append("file", file);
    if (parentId) {
      formData.append("parent_id", parentId);
    }

    const response = await fetch("/api/settings/import/notion/file", {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail || "Import failed");
    }

    return await response.json();
  },

  importNotionZip: async (
    file: File,
    parentId?: string
  ): Promise<ImportZipResult> => {
    const formData = new FormData();
    formData.append("file", file);
    if (parentId) {
      formData.append("parent_id", parentId);
    }

    const response = await fetch("/api/settings/import/notion/zip", {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail || "Import failed");
    }

    return await response.json();
  },
};
