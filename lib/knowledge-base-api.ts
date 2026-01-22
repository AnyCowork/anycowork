/**
 * Knowledge Base API Client (Tauri IPC)
 * Handles knowledge base file uploads and indexing
 */
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ============================================================================
// TYPES
// ============================================================================

export interface KnowledgeBaseItem {
  id: string;
  type: string;
  name: string;
  path?: string;
  pattern?: string;
  link_type?: string;
  indexing_status: string;
  indexing_error?: string;
  total_files?: number;
  indexed_files: number;
  retry_count: number;
  last_indexed_at?: string;
  created_at: string;
  updated_at: string;
}

export interface KBFileUpload {
  file_name: string;
  file_data: number[];
}

export interface IndexingProgress {
  item_id: string;
  status: string;
  indexed_files: number;
  total_files?: number;
  error?: string;
}

// ============================================================================
// KNOWLEDGE BASE API
// ============================================================================

export const knowledgeBaseApi = {
  /**
   * Upload files to knowledge base
   */
  upload: async (files: File[]): Promise<KnowledgeBaseItem> => {
    const fileUploads: KBFileUpload[] = await Promise.all(
      files.map(async (file) => {
        const arrayBuffer = await file.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        return {
          file_name: file.name,
          file_data: Array.from(uint8Array),
        };
      })
    );

    return invoke<KnowledgeBaseItem>('upload_kb_files', { files: fileUploads });
  },

  /**
   * Add a link (directory or pattern) to knowledge base
   */
  addLink: async (path: string, linkType: string): Promise<KnowledgeBaseItem> => {
    return invoke<KnowledgeBaseItem>('add_kb_link', {
      path,
      linkType,
    });
  },

  /**
   * Get all knowledge base items
   */
  getAll: async (): Promise<KnowledgeBaseItem[]> => {
    return invoke<KnowledgeBaseItem[]>('get_kb_items');
  },

  /**
   * Remove a knowledge base item
   */
  remove: async (itemId: string): Promise<void> => {
    return invoke<void>('remove_kb_item', { itemId });
  },

  /**
   * Retry indexing a failed item
   */
  retry: async (itemId: string): Promise<KnowledgeBaseItem> => {
    return invoke<KnowledgeBaseItem>('retry_kb_indexing', { itemId });
  },

  /**
   * Listen to indexing progress events
   */
  onIndexingProgress: (
    callback: (progress: IndexingProgress) => void
  ): Promise<() => void> => {
    return listen<IndexingProgress>('kb_indexing', (event) => {
      callback(event.payload);
    });
  },
};

// Export default for convenience
export default knowledgeBaseApi;
