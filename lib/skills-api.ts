/**
 * Skills API Client (Tauri IPC)
 * Handles agent skills and assignments
 */
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPES
// ============================================================================

export interface AgentSkill {
  id: string;
  name: string;
  display_title: string;
  description: string;
  skill_content: string;
  additional_files_json?: string;
  enabled: number;
  version: number;
  created_at: string;
  updated_at: string;
}

// ============================================================================
// SKILLS API
// ============================================================================

export const skillsApi = {
  /**
   * Get all skills (optionally filter by enabled status)
   */
  getAll: async (enabledOnly?: boolean): Promise<AgentSkill[]> => {
    return invoke<AgentSkill[]>('get_skills', { enabledOnly });
  },

  /**
   * Get a single skill by ID
   */
  getById: async (skillId: string): Promise<AgentSkill> => {
    return invoke<AgentSkill>('get_skill', { skillId });
  },

  /**
   * Create a new skill
   */
  create: async (data: {
    name: string;
    display_title: string;
    description: string;
    skill_content: string;
    additional_files_json?: string;
  }): Promise<AgentSkill> => {
    return invoke<AgentSkill>('create_skill', {
      nameParam: data.name,
      displayTitle: data.display_title,
      description: data.description,
      skillContent: data.skill_content,
      additionalFilesJson: data.additional_files_json,
    });
  },

  /**
   * Update a skill
   */
  update: async (
    skillId: string,
    data: {
      name?: string;
      display_title?: string;
      description?: string;
      skill_content?: string;
      additional_files_json?: string;
      enabled?: boolean;
    }
  ): Promise<AgentSkill> => {
    return invoke<AgentSkill>('update_skill', {
      skillId,
      nameParam: data.name,
      displayTitle: data.display_title,
      description: data.description,
      skillContent: data.skill_content,
      additionalFilesJson: data.additional_files_json,
      enabledParam: data.enabled,
    });
  },

  /**
   * Delete a skill
   */
  delete: async (skillId: string): Promise<void> => {
    return invoke<void>('delete_skill', { skillId });
  },

  /**
   * Toggle a skill's enabled status
   */
  toggle: async (skillId: string): Promise<AgentSkill> => {
    return invoke<AgentSkill>('toggle_skill', { skillId });
  },
};

// Export default for convenience
export default skillsApi;
