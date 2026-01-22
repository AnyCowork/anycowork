// Chat API Client for conversation management

const API_BASE_URL = import.meta.env.VITE_API_URL || "/api";

export interface Conversation {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  archived: boolean;
  pinned: boolean;
}

export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: "user" | "assistant" | "system";
  content: string;
  metadata_json?: Record<string, any>;
  tokens?: number;
  created_at: number;
}

export interface ConversationWithMessages extends Conversation {
  messages: ChatMessage[];
}

export interface ConversationStats {
  conversation_id: string;
  message_count: number;
  total_tokens: number;
  created_at: number;
  updated_at: number;
}

export const chatApi = {
  // Conversation management
  conversations: {
    create: async (title: string): Promise<Conversation> => {
      const response = await fetch(`${API_BASE_URL}/chat/conversations`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title }),
      });
      if (!response.ok) throw new Error("Failed to create conversation");
      return response.json();
    },

    list: async (params?: {
      archived?: boolean;
      limit?: number;
      offset?: number;
    }): Promise<Conversation[]> => {
      const queryParams = new URLSearchParams();
      if (params?.archived !== undefined)
        queryParams.set("archived", String(params.archived));
      if (params?.limit) queryParams.set("limit", String(params.limit));
      if (params?.offset) queryParams.set("offset", String(params.offset));

      const url = `${API_BASE_URL}/chat/conversations${queryParams.toString() ? `?${queryParams}` : ""}`;
      const response = await fetch(url);
      if (!response.ok) throw new Error("Failed to fetch conversations");
      return response.json();
    },

    get: async (conversationId: string): Promise<ConversationWithMessages> => {
      const response = await fetch(
        `${API_BASE_URL}/chat/conversations/${conversationId}`
      );
      if (!response.ok) throw new Error("Failed to fetch conversation");
      return response.json();
    },

    update: async (
      conversationId: string,
      data: {
        title?: string;
        archived?: boolean;
        pinned?: boolean;
      }
    ): Promise<Conversation> => {
      const response = await fetch(
        `${API_BASE_URL}/chat/conversations/${conversationId}`,
        {
          method: "PATCH",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        }
      );
      if (!response.ok) throw new Error("Failed to update conversation");
      return response.json();
    },

    delete: async (conversationId: string): Promise<void> => {
      const response = await fetch(
        `${API_BASE_URL}/chat/conversations/${conversationId}`,
        {
          method: "DELETE",
        }
      );
      if (!response.ok) throw new Error("Failed to delete conversation");
    },

    getStats: async (conversationId: string): Promise<ConversationStats> => {
      const response = await fetch(
        `${API_BASE_URL}/chat/conversations/${conversationId}/stats`
      );
      if (!response.ok) throw new Error("Failed to fetch stats");
      return response.json();
    },
  },

  // Message management
  messages: {
    add: async (
      conversationId: string,
      message: {
        role: "user" | "assistant" | "system";
        content: string;
        metadata_json?: Record<string, any>;
        tokens?: number;
      }
    ): Promise<ChatMessage> => {
      const response = await fetch(
        `${API_BASE_URL}/chat/conversations/${conversationId}/messages`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(message),
        }
      );
      if (!response.ok) throw new Error("Failed to add message");
      return response.json();
    },

    list: async (
      conversationId: string,
      params?: { limit?: number; offset?: number }
    ): Promise<ChatMessage[]> => {
      const queryParams = new URLSearchParams();
      if (params?.limit) queryParams.set("limit", String(params.limit));
      if (params?.offset) queryParams.set("offset", String(params.offset));

      const url = `${API_BASE_URL}/chat/conversations/${conversationId}/messages${queryParams.toString() ? `?${queryParams}` : ""}`;
      const response = await fetch(url);
      if (!response.ok) throw new Error("Failed to fetch messages");
      return response.json();
    },

    delete: async (
      conversationId: string,
      messageId: string
    ): Promise<void> => {
      const response = await fetch(
        `${API_BASE_URL}/chat/conversations/${conversationId}/messages/${messageId}`,
        {
          method: "DELETE",
        }
      );
      if (!response.ok) throw new Error("Failed to delete message");
    },
  },
};
