import { fetchWithRetry } from "@/lib/api-client";

export interface RAGSearchResult {
  document_id: string;
  score: number;
  text: string;
  snippet?: string;
  metadata: Record<string, any>;
  page_id: string;
  block_id?: string;
}

export interface RAGChatResponse {
  question: string;
  answer: string;
  sources: RAGSearchResult[];
  mode: string;
  backend: string;
}

export interface RAGConfig {
  enabled: boolean;
  backend: string;
  auto_index: boolean;
}

export const ragApi = {
  search: async (
    query: string,
    topK: number = 10
  ): Promise<RAGSearchResult[]> => {
    const response = await fetchWithRetry("/api/rag/search", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query, top_k: topK }),
    });
    const data = await response.json();
    return data.results;
  },

  chat: async (
    question: string,
    mode: string = "hybrid"
  ): Promise<RAGChatResponse> => {
    const response = await fetchWithRetry("/api/rag/chat", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ question, mode, top_k: 5 }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail || "Chat request failed");
    }

    return await response.json();
  },

  chatStream: async (
    question: string,
    mode: string = "hybrid",
    onChunk: (chunk: string) => void,
    onSources?: (sources: RAGSearchResult[]) => void,
    onComplete?: (backend: string) => void,
    onError?: (error: string) => void
  ): Promise<void> => {
    try {
      const response = await fetch("/api/rag/chat/stream", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ question, mode, top_k: 5 }),
      });

      if (!response.ok) {
        throw new Error("Stream request failed");
      }

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      if (!reader) {
        throw new Error("No response body");
      }

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split("\n");

        for (const line of lines) {
          if (line.startsWith("data: ")) {
            try {
              const data = JSON.parse(line.slice(6));

              if (data.type === "sources" && onSources) {
                onSources(data.sources);
              } else if (data.type === "chunk") {
                onChunk(data.content);
              } else if (data.type === "answer") {
                onChunk(data.content);
              } else if (data.type === "done" && onComplete) {
                onComplete(data.backend);
              } else if (data.type === "error" && onError) {
                onError(data.error);
              }
            } catch (e) {
              console.error("Failed to parse SSE data:", e);
            }
          }
        }
      }
    } catch (error: any) {
      if (onError) {
        onError(error.message || "Stream failed");
      }
      throw error;
    }
  },

  getRecommendations: async (
    blockId: string,
    topK: number = 5
  ): Promise<RAGSearchResult[]> => {
    const response = await fetchWithRetry(
      `/api/rag/recommendations/${blockId}?top_k=${topK}`
    );
    const data = await response.json();
    return data.recommendations;
  },

  getConfig: async (): Promise<RAGConfig> => {
    const response = await fetchWithRetry("/api/rag/config");
    return await response.json();
  },

  updateConfig: async (
    config: Partial<RAGConfig>
  ): Promise<{ success: boolean }> => {
    const response = await fetchWithRetry("/api/rag/config", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(config),
    });
    return await response.json();
  },

  rebuildIndex: async (): Promise<{
    success: boolean;
    indexed_count: number;
  }> => {
    const response = await fetchWithRetry("/api/rag/rebuild", {
      method: "POST",
    });
    return await response.json();
  },

  getStats: async (): Promise<Record<string, any>> => {
    const response = await fetchWithRetry("/api/rag/stats");
    return await response.json();
  },
};
