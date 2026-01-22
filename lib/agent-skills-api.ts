const API_BASE_URL = import.meta.env.VITE_API_URL || "/api";

export interface AgentSkill {
  id: string;
  name: string;
  displayTitle: string;
  description: string;
  skillContent: string;
  additionalFiles?: Record<string, string>;
  enabled: boolean;
  version: number;
  createdAt: number;
  updatedAt: number;
}

export interface CreateAgentSkillData {
  name: string;
  displayTitle: string;
  description: string;
  skillContent: string;
  additionalFiles?: Record<string, string>;
}

export interface UpdateAgentSkillData {
  name?: string;
  displayTitle?: string;
  description?: string;
  skillContent?: string;
  additionalFiles?: Record<string, string>;
  enabled?: boolean;
  version?: number;
}

export const agentSkillsApi = {
  // Get all skills
  getAll: async (enabledOnly: boolean = false): Promise<AgentSkill[]> => {
    const url = enabledOnly
      ? `${API_BASE_URL}/agent-skills?enabled_only=true`
      : `${API_BASE_URL}/agent-skills`;

    const response = await fetch(url);
    if (!response.ok) throw new Error("Failed to fetch skills");
    return response.json();
  },

  // Get skill by ID
  getById: async (id: string): Promise<AgentSkill> => {
    const response = await fetch(`${API_BASE_URL}/agent-skills/${id}`);
    if (!response.ok) throw new Error("Failed to fetch skill");
    return response.json();
  },

  // Create skill
  create: async (data: CreateAgentSkillData): Promise<AgentSkill> => {
    const response = await fetch(`${API_BASE_URL}/agent-skills`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail || "Failed to create skill");
    }
    return response.json();
  },

  // Update skill
  update: async (
    id: string,
    data: UpdateAgentSkillData
  ): Promise<AgentSkill> => {
    const response = await fetch(`${API_BASE_URL}/agent-skills/${id}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail || "Failed to update skill");
    }
    return response.json();
  },

  // Delete skill
  delete: async (id: string): Promise<void> => {
    const response = await fetch(`${API_BASE_URL}/agent-skills/${id}`, {
      method: "DELETE",
    });

    if (!response.ok) throw new Error("Failed to delete skill");
  },

  // Toggle skill enabled/disabled
  toggle: async (id: string): Promise<AgentSkill> => {
    const response = await fetch(`${API_BASE_URL}/agent-skills/${id}/toggle`, {
      method: "POST",
    });

    if (!response.ok) throw new Error("Failed to toggle skill");
    return response.json();
  },
};
