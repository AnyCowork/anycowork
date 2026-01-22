import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ============================================================================
// TYPES
// ============================================================================

export interface SearchConfig {
  id: number;
  enabled: boolean;
  auto_index: boolean;
  index_pages: boolean;
  index_blocks: boolean;
  index_kb_files: boolean;
  llm_type: string;
  llm_model: string;
  updated_at: number;
}

export interface UpdateSearchConfig {
  enabled?: boolean;
  auto_index?: boolean;
  index_pages?: boolean;
  index_blocks?: boolean;
  index_kb_files?: boolean;
  llm_type?: string;
  llm_model?: string;
  updated_at: number;
}

export interface SearchStats {
  id: number;
  total_documents: number;
  total_pages: number;
  total_blocks: number;
  total_kb_files: number;
  last_rebuild_at?: number;
}

export interface SearchResult {
  id: string;
  result_type: string; // "page", "block", "kb_file"
  title: string;
  content: string;
  score: number;
  page_id?: string;
}

export interface SearchAnswerResponse {
  answer: string;
  sources: SearchResult[];
}

export interface SearchStreamEvent {
  event_type: string; // "sources", "chunk", "done", "error"
  data?: string;
  sources?: SearchResult[];
}

// ============================================================================
// SEARCH API
// ============================================================================

export const searchApi = {
  /**
   * Search content across pages, blocks, and knowledge base
   */
  search: async (query: string, topK: number = 10): Promise<SearchResult[]> => {
    return invoke<SearchResult[]>("search_content", { query, topK });
  },

  /**
   * Search and generate an answer using LLM
   */
  searchAndAnswer: async (question: string, mode: string = "default"): Promise<SearchAnswerResponse> => {
    return invoke<SearchAnswerResponse>("search_and_answer", { question, mode });
  },

  /**
   * Search and stream the answer in real-time
   * @param sessionId - Unique session ID for this stream
   * @param question - The question to answer
   * @param onEvent - Callback for stream events
   * @returns Unlisten function to stop listening
   */
  searchAndAnswerStream: async (
    sessionId: string,
    question: string,
    onEvent: (event: SearchStreamEvent) => void
  ): Promise<() => void> => {
    // Set up event listener
    const unlisten = await listen<SearchStreamEvent>(
      `search_stream:${sessionId}`,
      (event) => {
        onEvent(event.payload);
      }
    );

    // Start the stream
    await invoke("search_and_answer_stream", {
      sessionId,
      question,
    });

    return unlisten;
  },

  /**
   * Get related content for a specific block
   */
  getRelatedContent: async (blockId: string, topK: number = 5): Promise<SearchResult[]> => {
    return invoke<SearchResult[]>("get_related_content", { blockId, topK });
  },

  /**
   * Rebuild the search index
   * @param onProgress - Optional callback for rebuild progress
   * @returns Unlisten function if onProgress is provided
   */
  rebuildIndex: async (
    onProgress?: (data: { status: string; total_documents: number }) => void
  ): Promise<(() => void) | void> => {
    if (onProgress) {
      const unlisten = await listen<{ status: string; total_documents: number }>(
        "search_rebuild",
        (event) => {
          onProgress(event.payload);
        }
      );
      await invoke("rebuild_search_index");
      return unlisten;
    } else {
      await invoke("rebuild_search_index");
    }
  },

  /**
   * Get search statistics
   */
  getStats: async (): Promise<SearchStats> => {
    return invoke<SearchStats>("get_search_stats");
  },

  /**
   * Get search configuration
   */
  getConfig: async (): Promise<SearchConfig> => {
    const config = await invoke<any>("get_search_config");
    return {
      ...config,
      enabled: config.enabled === 1,
      auto_index: config.auto_index === 1,
      index_pages: config.index_pages === 1,
      index_blocks: config.index_blocks === 1,
      index_kb_files: config.index_kb_files === 1,
    };
  },

  /**
   * Update search configuration
   */
  updateConfig: async (config: Partial<UpdateSearchConfig>): Promise<SearchConfig> => {
    const updateData: any = {
      ...config,
      updated_at: new Date().toISOString(),
    };

    // Convert booleans to integers for Rust
    if (config.enabled !== undefined) updateData.enabled = config.enabled ? 1 : 0;
    if (config.auto_index !== undefined) updateData.auto_index = config.auto_index ? 1 : 0;
    if (config.index_pages !== undefined) updateData.index_pages = config.index_pages ? 1 : 0;
    if (config.index_blocks !== undefined) updateData.index_blocks = config.index_blocks ? 1 : 0;
    if (config.index_kb_files !== undefined) updateData.index_kb_files = config.index_kb_files ? 1 : 0;

    const result = await invoke<any>("update_search_config", { config: updateData });
    return {
      ...result,
      enabled: result.enabled === 1,
      auto_index: result.auto_index === 1,
      index_pages: result.index_pages === 1,
      index_blocks: result.index_blocks === 1,
      index_kb_files: result.index_kb_files === 1,
    };
  },
};

export default searchApi;
