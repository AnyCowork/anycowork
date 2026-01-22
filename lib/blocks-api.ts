/**
 * API client for block operations
 */

const API_BASE_URL = "/api";

export interface Block {
  id: string;
  page_id: string;
  type: string;
  content_json: {
    text?: string;
    checked?: boolean;
    headers?: string[];
    rows?: string[][];
    hasHeader?: boolean;
  };
  order_index: number;
  created_at: number;
  updated_at: number;
}

export interface BlockUpdate {
  type?: string;
  content_json?: any;
  order_index?: number;
}

/**
 * Get all blocks for a page
 */
export async function getPageBlocks(pageId: string): Promise<Block[]> {
  const response = await fetch(`${API_BASE_URL}/pages/${pageId}/blocks`);
  if (!response.ok) {
    throw new Error("Failed to fetch blocks");
  }
  return response.json();
}

/**
 * Create a new block
 */
export async function createBlock(
  pageId: string,
  type: string,
  content_json: any,
  order_index?: number
): Promise<Block> {
  const response = await fetch(`${API_BASE_URL}/pages/${pageId}/blocks`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      type,
      content_json,
      order_index,
    }),
  });

  if (!response.ok) {
    throw new Error("Failed to create block");
  }

  return response.json();
}

/**
 * Update a block
 */
export async function updateBlock(
  blockId: string,
  update: BlockUpdate
): Promise<Block> {
  const response = await fetch(`${API_BASE_URL}/blocks/${blockId}`, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(update),
  });

  if (!response.ok) {
    throw new Error("Failed to update block");
  }

  return response.json();
}

/**
 * Delete a block
 */
export async function deleteBlock(blockId: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/pages/blocks/${blockId}`, {
    method: "DELETE",
  });

  if (!response.ok) {
    throw new Error("Failed to delete block");
  }
}

/**
 * Get page attachments
 */
export async function getPageAttachments(pageId: string): Promise<any[]> {
  const response = await fetch(`${API_BASE_URL}/attachments/page/${pageId}`);
  if (!response.ok) {
    throw new Error("Failed to fetch attachments");
  }
  return response.json();
}
