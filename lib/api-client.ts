// API Client for FastAPI Backend
// This adapts the Convex API structure to our FastAPI endpoints

const API_BASE_URL = import.meta.env.VITE_API_URL || "/api";

// Retry helper for handling backend reloads
export async function fetchWithRetry(
  url: string,
  options?: RequestInit,
  retries = 3,
  delay = 1000
): Promise<Response> {
  for (let i = 0; i < retries; i++) {
    try {
      const response = await fetch(url, options);
      return response;
    } catch (error) {
      // If it's the last retry, throw the error
      if (i === retries - 1) throw error;

      // Wait before retrying (exponential backoff)
      await new Promise((resolve) =>
        setTimeout(resolve, delay * Math.pow(2, i))
      );
    }
  }
  throw new Error("Max retries reached");
}

export interface Document {
  _id: string;
  title: string;
  parentDocument?: string;
  userId?: string;
  isArchived: boolean;
  isPublished: boolean;
  content?: string;
  coverImage?: string;
  icon?: string;
  _creationTime: number;
}

interface Page {
  id: string;
  title: string;
  type: string;
  parent_id: string | null;
  day_date: string | null;
  icon: string | null;
  cover_image: string | null;
  created_at: number;
  updated_at: number;
}

interface Block {
  id: string;
  page_id: string;
  type: string;
  content_json: {
    text?: string;
    richContent?: any;
    checked?: boolean;
    // Media block properties
    type?: string;
    url?: string;
    caption?: string;
    width?: number;
    name?: string;
    [key: string]: any; // Allow additional properties
  };
  order_index: number;
  created_at: number;
  updated_at: number;
}

// Helper to convert Page to Document
function pageToDocument(page: Page, blocks?: Block[]): Document {
  // Aggregate block content into a single content string
  const content = blocks
    ? blocks.map((b) => b.content_json.text || "").join("\n")
    : "";

  return {
    _id: page.id,
    title: page.title,
    parentDocument: page.parent_id || undefined,
    isArchived: false,
    isPublished: false,
    content,
    coverImage: page.cover_image || undefined,
    icon: page.icon || undefined,
    _creationTime: page.created_at * 1000, // Convert to ms
  };
}

// API Functions matching Convex structure
export const api = {
  documents: {
    // Get sidebar documents
    getSidebar: async (parentDocument?: string): Promise<Document[]> => {
      let url = `${API_BASE_URL}/pages`;

      // Add parent_id query parameter if provided
      if (parentDocument) {
        url += `?parent_id=${encodeURIComponent(parentDocument)}`;
      }

      const response = await fetchWithRetry(url);
      if (!response.ok) throw new Error("Failed to fetch pages");

      const pages: Page[] = await response.json();

      // Filter out daily pages and apply additional client-side filtering if needed
      const filtered = pages.filter((p) => p.type !== "daily");

      // Convert to documents
      return filtered.map((p) => pageToDocument(p));
    },

    // Create new document
    create: async (title: string, parentDocument?: string): Promise<string> => {
      const response = await fetchWithRetry(`${API_BASE_URL}/pages`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          title,
          type: "page",
          parent_id: parentDocument || null,
        }),
      });

      if (!response.ok) throw new Error("Failed to create page");

      const page: Page = await response.json();
      return page.id;
    },

    // Get document by ID
    getById: async (documentId: string): Promise<Document | null> => {
      try {
        const [pageResponse, blocksResponse] = await Promise.all([
          fetchWithRetry(`${API_BASE_URL}/pages/${documentId}`),
          fetchWithRetry(`${API_BASE_URL}/pages/${documentId}/blocks`),
        ]);

        if (!pageResponse.ok) return null;

        const page: Page = await pageResponse.json();
        const blocks: Block[] = blocksResponse.ok
          ? await blocksResponse.json()
          : [];

        // Convert blocks to BlockNote format
        const blockNoteBlocks = blocks.map((block) => {
          let bnType = "paragraph";
          let props: any = {};
          let content: any[] = [];

          if (block.type === "image") {
            // Handle image blocks
            bnType = "image";
            props = {
              url: block.content_json.url || "",
              caption: block.content_json.caption || "",
              width: block.content_json.width,
              ...block.content_json,
            };
            content = []; // Images don't have text content
          } else if (block.type === "file") {
            // Handle file blocks (PDFs, documents, etc.)
            bnType = "file";
            props = {
              url: block.content_json.url || "",
              name: block.content_json.name || "",
              caption: block.content_json.caption || "",
              ...block.content_json,
            };
            content = []; // Files don't have text content
          } else if (block.type === "video") {
            // Handle video blocks
            bnType = "video";
            props = {
              url: block.content_json.url || "",
              caption: block.content_json.caption || "",
              width: block.content_json.width,
              ...block.content_json,
            };
            content = []; // Videos don't have text content
          } else if (block.type === "audio") {
            // Handle audio blocks
            bnType = "audio";
            props = {
              url: block.content_json.url || "",
              caption: block.content_json.caption || "",
              ...block.content_json,
            };
            content = []; // Audio doesn't have text content
          } else if (block.type.startsWith("h")) {
            bnType = "heading";
            props.level = parseInt(block.type.substring(1));
            // Use richContent if available, otherwise fall back to text
            content = block.content_json.richContent || [
              { type: "text", text: block.content_json.text || "", styles: {} },
            ];
          } else if (block.type === "bullet" || block.type === "bullet_list") {
            bnType = "bulletListItem";
            // Use richContent if available, otherwise fall back to text
            content = block.content_json.richContent || [
              { type: "text", text: block.content_json.text || "", styles: {} },
            ];
          } else if (block.type === "checkbox") {
            bnType = "checkListItem";
            props.checked = block.content_json.checked || false;
            // Use richContent if available, otherwise fall back to text
            content = block.content_json.richContent || [
              { type: "text", text: block.content_json.text || "", styles: {} },
            ];
          } else {
            // Default paragraph
            // Use richContent if available, otherwise fall back to text
            content = block.content_json.richContent || [
              { type: "text", text: block.content_json.text || "", styles: {} },
            ];
          }

          return {
            id: block.id,
            type: bnType,
            props,
            content,
            children: [],
          };
        });

        const content =
          blockNoteBlocks.length > 0
            ? JSON.stringify(blockNoteBlocks)
            : undefined;

        return {
          _id: page.id,
          title: page.title,
          parentDocument: page.parent_id || undefined,
          isArchived: false,
          isPublished: false,
          content,
          coverImage: page.cover_image || undefined,
          icon: page.icon || undefined,
          _creationTime: page.created_at * 1000,
        };
      } catch {
        return null;
      }
    },

    // Update document
    update: async (
      id: string,
      data: {
        title?: string;
        content?: string;
        coverImage?: string;
        icon?: string;
        isPublished?: boolean;
      }
    ): Promise<Document> => {
      const updateData: any = {};

      if (data.title !== undefined) updateData.title = data.title;
      if (data.icon !== undefined) updateData.icon = data.icon;
      if (data.coverImage !== undefined)
        updateData.cover_image = data.coverImage;

      // Update page metadata
      const response = await fetch(`${API_BASE_URL}/pages/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(updateData),
      });

      if (!response.ok) throw new Error("Failed to update page");

      const page: Page = await response.json();

      // If content is being updated, calculate diff and send only changes
      if (data.content !== undefined) {
        try {
          const blockNoteBlocks = JSON.parse(data.content);

          // Get existing blocks
          const blocksResponse = await fetch(
            `${API_BASE_URL}/pages/${id}/blocks`
          );
          const existingBlocks: Block[] = blocksResponse.ok
            ? await blocksResponse.json()
            : [];

          // Create a map of existing blocks by order_index for quick lookup
          const existingBlocksMap = new Map(
            existingBlocks.map((b) => [b.order_index, b])
          );

          // Track which blocks to keep
          const processedIndices = new Set<number>();

          // Update or create blocks
          const updatePromises = blockNoteBlocks.map(
            async (bnBlock: any, index: number) => {
              processedIndices.add(index);

              // Map BlockNote block types to our backend types
              let blockType = "text";
              if (bnBlock.type === "heading") {
                blockType = `h${bnBlock.props?.level || 1}`;
              } else if (bnBlock.type === "bulletListItem") {
                blockType = "bullet";
              } else if (bnBlock.type === "checkListItem") {
                blockType = "checkbox";
              } else if (bnBlock.type === "paragraph") {
                blockType = "text";
              } else if (bnBlock.type === "image") {
                blockType = "image";
              } else if (bnBlock.type === "file") {
                blockType = "file";
              } else if (bnBlock.type === "video") {
                blockType = "video";
              } else if (bnBlock.type === "audio") {
                blockType = "audio";
              }

              // Build content_json based on block type
              let content_json: any = {};

              if (bnBlock.type === "image") {
                // For image blocks, save the full block structure
                content_json = {
                  type: "image",
                  url: bnBlock.props?.url || "",
                  caption: bnBlock.props?.caption || "",
                  width: bnBlock.props?.width,
                  ...bnBlock.props,
                };
              } else if (bnBlock.type === "file") {
                // For file blocks (PDFs, documents, etc.)
                content_json = {
                  type: "file",
                  url: bnBlock.props?.url || "",
                  name: bnBlock.props?.name || "",
                  caption: bnBlock.props?.caption || "",
                  ...bnBlock.props,
                };
              } else if (bnBlock.type === "video") {
                // For video blocks
                content_json = {
                  type: "video",
                  url: bnBlock.props?.url || "",
                  caption: bnBlock.props?.caption || "",
                  width: bnBlock.props?.width,
                  ...bnBlock.props,
                };
              } else if (bnBlock.type === "audio") {
                // For audio blocks
                content_json = {
                  type: "audio",
                  url: bnBlock.props?.url || "",
                  caption: bnBlock.props?.caption || "",
                  ...bnBlock.props,
                };
              } else {
                // For text blocks, extract text content
                let text = "";
                if (bnBlock.content && Array.isArray(bnBlock.content)) {
                  text = bnBlock.content
                    .map((c: any) => (typeof c === "string" ? c : c.text || ""))
                    .join("");
                }
                content_json = {
                  text,
                  checked: bnBlock.props?.checked,
                };
              }

              const existingBlock = existingBlocksMap.get(index);

              // Check if block needs updating
              if (existingBlock) {
                const needsUpdate =
                  existingBlock.type !== blockType ||
                  JSON.stringify(existingBlock.content_json) !==
                    JSON.stringify(content_json);

                if (needsUpdate) {
                  // Update existing block
                  return fetch(`${API_BASE_URL}/blocks/${existingBlock.id}`, {
                    method: "PATCH",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                      type: blockType,
                      content_json,
                      order_index: index,
                    }),
                  });
                }
                return null; // No update needed
              } else {
                // Create new block
                return fetch(`${API_BASE_URL}/pages/${id}/blocks`, {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                  body: JSON.stringify({
                    type: blockType,
                    content_json,
                    order_index: index,
                  }),
                });
              }
            }
          );

          // Delete blocks that no longer exist
          const deletePromises = existingBlocks
            .filter((block) => !processedIndices.has(block.order_index))
            .map((block) =>
              fetch(`${API_BASE_URL}/blocks/${block.id}`, { method: "DELETE" })
            );

          await Promise.all([...updatePromises, ...deletePromises]);
        } catch (error) {
          console.error("Failed to update blocks:", error);
        }
      }

      return pageToDocument(page);
    },

    // Archive document (soft delete)
    archive: async (id: string): Promise<Document> => {
      const response = await fetch(`${API_BASE_URL}/pages/${id}/archive`, {
        method: "POST",
      });

      if (!response.ok) throw new Error("Failed to archive page");

      const page: Page = await response.json();
      return pageToDocument(page);
    },

    // Get trash (archived documents)
    getTrash: async (): Promise<Document[]> => {
      const response = await fetch(`${API_BASE_URL}/pages/trash/list`);
      if (!response.ok) throw new Error("Failed to fetch trash");

      const pages: Page[] = await response.json();
      return pages.map((p) => pageToDocument(p));
    },

    // Restore from trash
    restore: async (id: string): Promise<Document> => {
      const response = await fetch(`${API_BASE_URL}/pages/${id}/restore`, {
        method: "POST",
      });

      if (!response.ok) throw new Error("Failed to restore page");

      const page: Page = await response.json();
      return pageToDocument(page);
    },

    // Remove permanently
    remove: async (id: string): Promise<void> => {
      const response = await fetch(`${API_BASE_URL}/pages/${id}`, {
        method: "DELETE",
      });

      if (!response.ok) throw new Error("Failed to delete page");
    },

    // Search documents
    getSearch: async (): Promise<Document[]> => {
      const response = await fetch(`${API_BASE_URL}/pages`);
      if (!response.ok) throw new Error("Failed to fetch pages");

      const pages: Page[] = await response.json();
      return pages
        .filter((p) => p.type === "page")
        .map((p) => pageToDocument(p));
    },

    // Remove icon
    removeIcon: async (id: string): Promise<Document> => {
      const response = await fetchWithRetry(`${API_BASE_URL}/pages/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ icon: null }),
      });

      if (!response.ok) throw new Error("Failed to remove icon");

      const page: Page = await response.json();
      return pageToDocument(page);
    },

    // Remove cover image
    removeCoverImage: async (id: string): Promise<Document> => {
      const response = await fetchWithRetry(`${API_BASE_URL}/pages/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ cover_image: null }),
      });

      if (!response.ok) throw new Error("Failed to remove cover image");

      const page: Page = await response.json();
      return pageToDocument(page);
    },
  },
};
