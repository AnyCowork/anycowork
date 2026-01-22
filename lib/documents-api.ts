/**
 * Documents API Client (Tauri IPC)
 * Handles pages, blocks, and attachments
 */
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPES
// ============================================================================

export interface Page {
  id: string;
  title: string;
  type: string;
  parent_id?: string;
  day_date?: string;
  icon?: string;
  cover_image?: string;
  is_archived: number;
  is_published: number;
  created_at: string;
  updated_at: string;
}

export interface Block {
  id: string;
  page_id: string;
  type: string;
  content_json: string;
  order_index: number;
  created_at: string;
  updated_at: string;
}

export interface Attachment {
  id: string;
  page_id: string;
  block_id?: string;
  file_path: string;
  file_name: string;
  file_type: string;
  file_size: number;
  created_at: string;
}

export interface BlockUpdate {
  id: string;
  block_type?: string;
  content_json?: string;
  order_index?: number;
}

// ============================================================================
// PAGE API
// ============================================================================

export const pagesApi = {
  /**
   * Create a new page
   */
  create: async (title: string, pageType: string = 'page', parentId?: string): Promise<Page> => {
    return invoke<Page>('create_page', {
      title,
      pageType,
      parentId,
    });
  },

  /**
   * Get pages (optionally filtered by parent and archived status)
   */
  getAll: async (parentId?: string, archived?: boolean): Promise<Page[]> => {
    return invoke<Page[]>('get_pages', {
      parentIdParam: parentId,
      archived,
    });
  },

  /**
   * Get a single page by ID
   */
  getById: async (pageId: string): Promise<Page> => {
    return invoke<Page>('get_page', { pageId });
  },

  /**
   * Update a page
   */
  update: async (
    pageId: string,
    data: {
      title?: string;
      icon?: string;
      cover_image?: string;
      is_published?: boolean;
    }
  ): Promise<Page> => {
    return invoke<Page>('update_page', {
      pageId,
      titleParam: data.title,
      iconParam: data.icon,
      coverImageParam: data.cover_image,
      isPublishedParam: data.is_published,
    });
  },

  /**
   * Archive a page
   */
  archive: async (pageId: string): Promise<Page> => {
    return invoke<Page>('archive_page', { pageId });
  },

  /**
   * Restore an archived page
   */
  restore: async (pageId: string): Promise<Page> => {
    return invoke<Page>('restore_page', { pageId });
  },

  /**
   * Permanently delete a page
   */
  delete: async (pageId: string): Promise<void> => {
    return invoke<void>('delete_page', { pageId });
  },
};

// ============================================================================
// BLOCK API
// ============================================================================

export const blocksApi = {
  /**
   * Get all blocks for a page
   */
  getPageBlocks: async (pageId: string): Promise<Block[]> => {
    return invoke<Block[]>('get_page_blocks', {
      pageIdParam: pageId,
    });
  },

  /**
   * Create a new block
   */
  create: async (
    pageId: string,
    blockType: string,
    contentJson: string,
    orderIndex?: number
  ): Promise<Block> => {
    return invoke<Block>('create_block', {
      pageIdParam: pageId,
      blockType,
      contentJsonParam: contentJson,
      orderIndexParam: orderIndex,
    });
  },

  /**
   * Update a block
   */
  update: async (
    blockId: string,
    data: {
      block_type?: string;
      content_json?: string;
      order_index?: number;
    }
  ): Promise<Block> => {
    return invoke<Block>('update_block', {
      blockId,
      blockType: data.block_type,
      contentJsonParam: data.content_json,
      orderIndexParam: data.order_index,
    });
  },

  /**
   * Delete a block
   */
  delete: async (blockId: string): Promise<void> => {
    return invoke<void>('delete_block', { blockId });
  },

  /**
   * Batch update multiple blocks
   */
  batchUpdate: async (pageId: string, updates: BlockUpdate[]): Promise<Block[]> => {
    return invoke<Block[]>('batch_update_blocks', {
      pageIdParam: pageId,
      updates,
    });
  },
};

// ============================================================================
// ATTACHMENT API
// ============================================================================

export const attachmentsApi = {
  /**
   * Upload an attachment
   */
  upload: async (pageId: string, fileName: string, fileData: Uint8Array): Promise<Attachment> => {
    return invoke<Attachment>('upload_attachment', {
      pageIdParam: pageId,
      fileName,
      fileData: Array.from(fileData),
    });
  },

  /**
   * Get all attachments for a page
   */
  getPageAttachments: async (pageId: string): Promise<Attachment[]> => {
    return invoke<Attachment[]>('get_page_attachments', {
      pageIdParam: pageId,
    });
  },

  /**
   * Delete an attachment
   */
  delete: async (attachmentId: string): Promise<void> => {
    return invoke<void>('delete_attachment', { attachmentId });
  },
};

// ============================================================================
// COMBINED API (for backward compatibility with existing Document interface)
// ============================================================================

/**
 * Combined API that provides a unified interface similar to the old api-client.ts
 */
export const documentsApi = {
  /**
   * Get sidebar (root-level pages)
   */
  getSidebar: async (): Promise<Page[]> => {
    return pagesApi.getAll(undefined, false);
  },

  /**
   * Create a page
   */
  create: async (title: string, parentId?: string): Promise<string> => {
    const page = await pagesApi.create(title, 'page', parentId);
    return page.id;
  },

  /**
   * Get a page with its blocks
   */
  getById: async (pageId: string) => {
    const [page, blocks] = await Promise.all([
      pagesApi.getById(pageId),
      blocksApi.getPageBlocks(pageId),
    ]);

    return {
      ...page,
      blocks: blocks.map((b) => ({
        ...b,
        content: JSON.parse(b.content_json),
      })),
    };
  },

  /**
   * Update a page and its content
   */
  update: async (
    id: string,
    data: {
      title?: string;
      icon?: string;
      cover_image?: string;
      content?: any; // BlockNote content
    }
  ) => {
    // Update page metadata
    const updates: any = {};
    if (data.title !== undefined) updates.title = data.title;
    if (data.icon !== undefined) updates.icon = data.icon;
    if (data.cover_image !== undefined) updates.cover_image = data.cover_image;

    if (Object.keys(updates).length > 0) {
      await pagesApi.update(id, updates);
    }

    // If content is provided, update blocks
    if (data.content) {
      // For now, we'll just return the page
      // Full BlockNote integration would require parsing and updating blocks
      console.warn('Block content updates not yet fully implemented');
    }

    return documentsApi.getById(id);
  },

  /**
   * Archive a page
   */
  archive: async (pageId: string): Promise<void> => {
    await pagesApi.archive(pageId);
  },

  /**
   * Get trash (archived pages)
   */
  getTrash: async (): Promise<Page[]> => {
    return pagesApi.getAll(undefined, true);
  },

  /**
   * Restore a page from trash
   */
  restore: async (pageId: string): Promise<void> => {
    await pagesApi.restore(pageId);
  },

  /**
   * Permanently delete a page
   */
  remove: async (pageId: string): Promise<void> => {
    await pagesApi.delete(pageId);
  },

  /**
   * Search pages (TODO: implement full-text search in Phase 5)
   */
  getSearch: async (query: string): Promise<Page[]> => {
    // For now, get all non-archived pages and filter client-side
    const pages = await pagesApi.getAll(undefined, false);
    const lowerQuery = query.toLowerCase();
    return pages.filter((p) => p.title.toLowerCase().includes(lowerQuery));
  },

  /**
   * Remove icon from page
   */
  removeIcon: async (pageId: string): Promise<void> => {
    await pagesApi.update(pageId, { icon: '' });
  },

  /**
   * Remove cover image from page
   */
  removeCoverImage: async (pageId: string): Promise<void> => {
    await pagesApi.update(pageId, { cover_image: '' });
  },
};

// Export everything for convenience
export default documentsApi;
