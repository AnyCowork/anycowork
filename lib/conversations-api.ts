/**
 * Conversations API Client (Tauri IPC)
 * Handles sessions and messages with extended features
 */
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPES
// ============================================================================

export interface Session {
  id: string;
  agent_id: string;
  title?: string;
  created_at: string;
  updated_at: string;
  archived: number;
  pinned: number;
}

export interface Message {
  id: string;
  role: string;
  content: string;
  session_id: string;
  created_at: string;
  metadata_json?: string;
  tokens?: number;
}

export interface SessionWithMessages {
  session: Session;
  messages: Message[];
}

export interface SessionStats {
  session_id: string;
  message_count: number;
  total_tokens?: number;
}

// ============================================================================
// CONVERSATIONS API
// ============================================================================

export const conversationsApi = {
  /**
   * Create a new conversation session
   */
  create: async (agentId: string): Promise<Session> => {
    return invoke<Session>('create_session', { agentId });
  },

  /**
   * List all conversations with optional filters
   */
  list: async (params?: {
    archived?: boolean;
    limit?: number;
    offset?: number;
  }): Promise<Session[]> => {
    return invoke<Session[]>('get_sessions', {
      archivedParam: params?.archived,
      limit: params?.limit,
      offset: params?.offset,
    });
  },

  /**
   * Get a single conversation (session only)
   */
  get: async (sessionId: string): Promise<SessionWithMessages> => {
    return invoke<SessionWithMessages>('get_session_with_messages', {
      sessionId,
    });
  },

  /**
   * Update conversation metadata
   */
  update: async (
    sessionId: string,
    data: {
      title?: string;
      archived?: boolean;
      pinned?: boolean;
    }
  ): Promise<Session> => {
    return invoke<Session>('update_session', {
      sessionId,
      title: data.title,
      archivedParam: data.archived,
      pinnedParam: data.pinned,
    });
  },

  /**
   * Delete a conversation
   */
  delete: async (sessionId: string): Promise<void> => {
    return invoke<void>('delete_session', { sessionId });
  },

  /**
   * Get conversation statistics
   */
  getStats: async (sessionId: string): Promise<SessionStats> => {
    return invoke<SessionStats>('get_session_stats', { sessionId });
  },
};

// ============================================================================
// MESSAGES API
// ============================================================================

export const messagesApi = {
  /**
   * Add a message to a conversation
   */
  add: async (
    sessionId: string,
    role: string,
    content: string,
    metadata?: Record<string, any>,
    tokens?: number
  ): Promise<Message> => {
    return invoke<Message>('add_message', {
      sessionId,
      role,
      content,
      metadataJson: metadata ? JSON.stringify(metadata) : undefined,
      tokens,
    });
  },

  /**
   * List messages in a conversation
   */
  list: async (sessionId: string): Promise<Message[]> => {
    return invoke<Message[]>('get_session_messages', {
      sessionId,
    });
  },

  /**
   * Delete a message
   */
  delete: async (sessionId: string, messageId: string): Promise<void> => {
    return invoke<void>('delete_message', { messageId });
  },
};

// ============================================================================
// COMBINED API (backward compatibility with chat-api.ts)
// ============================================================================

/**
 * Combined API matching the legacy chat-api.ts interface
 */
export const chatApi = {
  conversations: {
    create: conversationsApi.create,
    list: conversationsApi.list,
    get: conversationsApi.get,
    update: conversationsApi.update,
    delete: conversationsApi.delete,
    getStats: conversationsApi.getStats,
  },

  messages: {
    add: messagesApi.add,
    list: messagesApi.list,
    delete: messagesApi.delete,
  },
};

// Export everything for convenience
export default chatApi;
