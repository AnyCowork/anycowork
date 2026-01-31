/**
 * AnyCowork API Client (Tauri IPC)
 */
import { invoke } from '@tauri-apps/api/core';

// Types (Mirrors Rust structs)
export enum ExecutionMode {
  Autopilot = 'autopilot',
  RequireApproval = 'require_approval',
  SmartApproval = 'smart_approval'
}

export interface Agent {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  created_at: string;
  updated_at: string;
  // Extras for UI compatibility
  avatar?: string;
  status?: 'active' | 'inactive' | 'error';
  ai_config?: AIConfig;
  characteristics?: AgentCharacteristics;
  skills?: string[];
  mcp_servers?: string[];
  messaging_connections?: string[];
  platform_configs?: Record<string, any>;
  working_directories?: string[];
  permissions?: AgentPermissions;
  api_keys?: Record<string, string>;
  execution_settings?: ExecutionSettings;
  workspace_path?: string;
  scope_type?: string;
}

export interface AgentCharacteristics {
  personality?: string;
  expertise?: string[];
  communication_style?: string;
  tone?: string;
}

export interface AgentPermissions {
  can_execute_commands?: boolean;
  can_access_files?: boolean;
  can_access_network?: boolean;
}

export interface ExecutionSettings {
  mode: ExecutionMode | string;
  sandbox_mode?: 'sandbox' | 'direct' | 'flexible';
  whitelisted_commands?: string[];
  whitelisted_tools?: string[];
  blacklisted_commands?: string[];
}

export interface AgentCreate {
  name: string;
  description: string;
  system_prompt: string;
  avatar?: string;
  characteristics?: AgentCharacteristics;
  ai_config?: AIConfig;
}

export interface AgentUpdate {
  name?: string;
  description?: string;
  system_prompt?: string;
  avatar?: string;
  characteristics?: AgentCharacteristics;
  ai_config?: AIConfig;
  skills?: string[];
  mcp_servers?: string[];
  execution_settings?: ExecutionSettings;
  platform_configs?: Record<string, any>;
}

export interface AIConfig {
  provider?: string;
  model?: string;
  max_tokens?: number;
  temperature?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  anthropic_api_key?: string;
  anthropic_model?: string;
  openai_api_key?: string;
  openai_model?: string;
  gemini_api_key?: string;
  gemini_model?: string;
}

export interface MessagingConfig {
  telegram?: {
    enabled: boolean;
    bot_token: string;
    allowed_users?: string[];
  };
}

export interface ExecutionSettingsUpdate {
  mode?: ExecutionMode | string;
  whitelisted_commands?: string[];
  whitelisted_tools?: string[];
  blacklisted_commands?: string[];
}

export interface Task {
  id: string;
  title: string;
  description?: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  priority?: 'low' | 'medium' | 'high' | number;
  agent_id?: string;
  created_at: string;
  updated_at: string;
}

export interface TaskCreate {
  title: string;
  description?: string;
  agent_id?: string;
  session_id?: string;
  priority?: 'low' | 'medium' | 'high' | number;
}

export interface TaskUpdate {
  title?: string;
  description?: string;
  status?: Task['status'];
  priority?: Task['priority'];
}

export interface FederationNode {
  id: string;
  name: string;
  node_name?: string;
  url: string;
  host?: string;
  port?: number;
  gateway_port?: number;
  status: 'online' | 'offline';
  capabilities?: string[];
}

// Plan & Task State (for Scratchpad)
export interface TaskState {
  id: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: string;
}

export interface PlanUpdate {
  tasks: TaskState[];
}

export interface Session {
  id: string;
  agent_config: any;
  created_at: number;
}

// Telegram Config types (for new Telegram integration)
export interface TelegramConfig {
  id: string;
  bot_token: string;
  agent_id: string;
  is_active: number;
  allowed_chat_ids?: string;
  created_at: string;
  updated_at: string;
}

export interface TelegramBotStatus {
  config_id: string;
  is_running: boolean;
}

// Skill types
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
  source_path?: string;
  category?: string;
  requires_sandbox: number;
  sandbox_config?: string;
  execution_mode: string; // "sandbox", "direct", "flexible"
}

export interface MarketplaceSkill {
  id: string;
  name: string;
  display_title: string;
  description: string;
  category?: string;
  dir_name: string;
  is_installed: boolean;
}

export interface SkillFile {
  id: string;
  skill_id: string;
  relative_path: string;
  content: string;
  file_type: string;
  created_at: string;
}

// MCP Types
export interface McpServer {
  id: string;
  name: string;
  server_type: 'stdio' | 'sse';
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  is_enabled: boolean;
  template_id?: string;
  created_at: number;
  updated_at: number;
  status: 'connected' | 'online' | 'offline' | 'error'; // Frontend only
}

export interface McpServerUpdate {
  name?: string;
  server_type?: 'stdio' | 'sse';
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  is_enabled?: boolean;
}

export interface McpTemplate {
  id: string;
  name: string;
  description: string;
  server_type: 'stdio' | 'sse';
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
}

// API Methods
export const anycoworkApi = {
  // Agents
  listAgents: async (_status?: 'active' | 'inactive' | 'error', _limit?: number) => {
    // Status filtering and limiting done client-side for now
    return invoke<Agent[]>('get_agents');
  },

  createAgent: async (data: AgentCreate) => {
    return invoke<Agent>('create_agent', {
      name: data.name,
      description: data.description,
      systemPrompt: data.system_prompt,
    });
  },

  // Chat
  // Note: Rust 'chat' command returns string, not stream yet.
  sendMessage: async (sessionId: string, message: string, mode?: string, model?: string) => {
    return invoke<string>('chat', { sessionId: sessionId, message, mode, model });
  },

  approveAction: async (stepId: string) => {
    console.log("Approving action with stepId:", stepId);
    return invoke('approve_action', { stepId: stepId, step_id: stepId });
  },

  rejectAction: async (stepId: string) => {
    console.log("Rejecting action with stepId:", stepId);
    return invoke('reject_action', { stepId: stepId, step_id: stepId });
  },

  // Stubs for other methods to prevent compilation errors in UI
  getServerInfo: async () => ({
    workspace_path: 'local',
    api: { host: 'localhost', port: 0 },
    ai_provider: 'rig',
    model: 'gpt-4',
    messaging: { telegram: { enabled: false } },
    gateway: { host: 'localhost', port: 0, url: '' }
  }),
  // Sessions
  listSessions: async () => {
    try {
      const sessions = await invoke<any[]>('get_sessions');
      return { sessions, count: sessions.length };
    } catch (e) {
      console.error("Failed to list sessions:", e);
      return { sessions: [], count: 0 };
    }
  },

  createSession: async (agentId: string) => {
    return invoke<Session>('create_session', { agentId: agentId });
  },

  deleteSession: async (id: string) => {
    return invoke('delete_session', { sessionId: id });
  },

  getSessionMessages: async (sessionId: string) => {
    return invoke<any[]>('get_session_messages', { sessionId: sessionId });
  },

  getGatewayStatus: async () => ({ status: 'ok', connected_clients: 0, uptime: 0 }),
  // Messaging
  getMessagingStatus: async () => ({
    telegram: {
      connected: false,
      active_sessions: 0,
      bot_username: ''
    }
  }),
  healthCheck: async () => ({ status: 'ok' }),

  // Tasks
  listTasks: async (_sessionId?: string, _status?: string) => ({ tasks: [] as Task[], count: 0 }),
  createTask: async (_data: TaskCreate) => ({} as Task),
  updateTask: async (_taskId: string, _data: TaskUpdate) => ({} as Task),
  deleteTask: async (_taskId: string) => ({ success: true }),

  // Configuration
  getAIConfig: async () => ({
    max_tokens: 4000,
    temperature: 0.7,
    top_p: 1.0,
    frequency_penalty: 0.0,
    presence_penalty: 0.0,
    provider: 'gemini',
    anthropic_api_key: '',
    anthropic_model: 'claude-3-opus',
    openai_api_key: '',
    openai_model: 'gpt-4',
    gemini_api_key: '',
    gemini_model: 'gemini-3-flash-preview'
  }),
  updateAIConfig: async (config: any) => ({ success: true }),
  // Messaging (Bridge for UI single-config view)
  getMessagingConfig: async () => {
    const configs = await invoke<any[]>('get_telegram_configs');
    const config = configs[0]; // Use first one for now
    return {
      telegram: {
        enabled: config ? config.is_active === 1 : false,
        bot_token: config ? config.bot_token : '',
        allowed_users: [],
        config_id: config ? config.id : undefined // Store ID for updates
      }
    };
  },

  updateMessagingConfig: async (config: any) => {
    // 1. Get existing configs to find ID
    const configs = await invoke<any[]>('get_telegram_configs');
    const existing = configs[0];

    const telegram = config.telegram;
    const isActive = telegram.enabled ? 1 : 0;

    if (existing) {
      // Update
      await invoke('update_telegram_config', {
        config_id: existing.id,
        new_bot_token: telegram.bot_token,
        new_is_active: isActive
      });

      // Handle Start/Stop
      if (existing.is_active !== isActive) {
        if (isActive) {
          await invoke('start_telegram_bot', { config_id: existing.id });
        } else {
          await invoke('stop_telegram_bot', { config_id: existing.id });
        }
      }
      // If token changed and it was active, restart might be needed (simplified here)

    } else {
      // Create New
      // Need an agent ID. Fetch agents and pick first one.
      const agents = await invoke<any[]>('get_agents');
      if (agents.length === 0) throw new Error("No agents available to attach bot to");

      const newConfig = await invoke<any>('create_telegram_config', {
        bot_token: telegram.bot_token,
        agent_id: agents[0].id,
        allowed_chat_ids: null
      });

      if (isActive) {
        await invoke('start_telegram_bot', { config_id: newConfig.id });
      }
    }
    return { success: true };
  },

  testTelegramConnection: async (botToken: string) => {
    return invoke<{ success: boolean; bot_username?: string; error?: string }>('test_telegram_bot', {
      botToken: botToken
    });
  },

  // Telegram Bot Config (Tauri commands)
  listTelegramConfigs: async () => {
    return invoke<TelegramConfig[]>('get_telegram_configs');
  },
  getTelegramConfig: async (configId: string) => {
    return invoke<TelegramConfig>('get_telegram_config', { config_id: configId });
  },
  createTelegramConfig: async (botToken: string, agentId: string, allowedChatIds?: string) => {
    return invoke<TelegramConfig>('create_telegram_config', {
      bot_token: botToken,
      agent_id: agentId,
      allowed_chat_ids: allowedChatIds
    });
  },
  updateTelegramConfig: async (configId: string, data: {
    new_bot_token?: string;
    new_agent_id?: string;
    new_is_active?: number;
    new_allowed_chat_ids?: string;
  }) => {
    return invoke<TelegramConfig>('update_telegram_config', {
      config_id: configId,
      new_bot_token: data.new_bot_token,
      new_agent_id: data.new_agent_id,
      new_is_active: data.new_is_active,
      new_allowed_chat_ids: data.new_allowed_chat_ids,
    });
  },
  deleteTelegramConfig: async (configId: string) => {
    return invoke('delete_telegram_config', { config_id: configId });
  },
  startTelegramBot: async (configId: string) => {
    return invoke('start_telegram_bot', { config_id: configId });
  },
  stopTelegramBot: async (configId: string) => {
    return invoke('stop_telegram_bot', { config_id: configId });
  },
  getTelegramBotStatus: async (configId: string) => {
    return invoke<TelegramBotStatus>('get_telegram_bot_status', { config_id: configId });
  },
  getRunningTelegramBots: async () => {
    return invoke<string[]>('get_running_telegram_bots');
  },

  // Agent Definitions
  getAgent: async (id: string) => ({ id, name: 'Agent', description: '', system_prompt: '' }),
  updateAgent: async (agentId: string, data: any) => {
    // Serialize platform_configs if present (backend expects JSON string)
    if (data.platform_configs && typeof data.platform_configs === 'object') {
      data = {
        ...data,
        platform_configs: JSON.stringify(data.platform_configs)
      };
    }

    return invoke<Agent>('update_agent', {
      agentId: agentId,
      data,
    });
  },
  deleteAgent: async (agentId: string) => ({ success: true }),

  // Agent Skills
  getAgentSkills: async (agentId: string) => {
    return invoke<AgentSkill[]>('get_agent_skills', { agentId });
  },
  assignSkillToAgent: async (agentId: string, skillId: string) => {
    return invoke<void>('assign_skill_to_agent', { agentId, skillId });
  },
  unassignSkillFromAgent: async (agentId: string, skillId: string) => {
    return invoke<void>('unassign_skill_from_agent', { agentId, skillId });
  },
  assignAgentSkills: async (agentId: string, skillIds: string[]) => ({ success: true }),
  getAgentMCPServers: async (agentId: string) => ([]),
  assignAgentMCPServers: async (agentId: string, mcpServerIds: string[]) => ({ success: true }),
  getAgentMessagingConnections: async (agentId: string) => ([]),
  configureAgentMessaging: async (agentId: string, connectionIds: string[]) => ({ success: true }),

  // Messaging Connections
  listMessagingConnections: async () => ([]),

  // Skills
  listSkills: async (enabledOnly?: boolean) => {
    return invoke<AgentSkill[]>('get_skills', { enabledOnly });
  },
  getSkill: async (skillId: string) => {
    return invoke<AgentSkill>('get_skill', { skillId });
  },
  createSkill: async (name: string, displayTitle: string, description: string, skillContent: string, additionalFilesJson?: string) => {
    return invoke<AgentSkill>('create_skill', {
      nameParam: name,
      displayTitle,
      description,
      skillContent,
      additionalFilesJson
    });
  },
  updateSkill: async (skillId: string, data: {
    name?: string;
    displayTitle?: string;
    description?: string;
    skillContent?: string;
    additionalFilesJson?: string;
    enabled?: boolean;
    category?: string;
    requiresSandbox?: boolean;
    sandboxConfig?: string;
  }) => {
    return invoke<AgentSkill>('update_skill', {
      skillId,
      nameParam: data.name,
      displayTitle: data.displayTitle,
      description: data.description,
      skillContent: data.skillContent,
      additionalFilesJson: data.additionalFilesJson,
      enabledParam: data.enabled,
      category: data.category,
      requiresSandbox: data.requiresSandbox,
      sandboxConfig: data.sandboxConfig,
    });
  },
  deleteSkill: async (skillId: string) => {
    return invoke<void>('delete_skill', { skillId });
  },
  toggleSkill: async (skillId: string) => {
    return invoke<AgentSkill>('toggle_skill', { skillId });
  },
  getSkillFiles: async (skillId: string) => {
    return invoke<SkillFile[]>('get_skill_files', { skillId });
  },

  // Skill Import/Marketplace
  listMarketplaceSkills: async () => {
    return invoke<MarketplaceSkill[]>('list_marketplace_skills', {});
  },
  installSkill: async (skillDirName: string) => {
    return invoke<AgentSkill>('install_marketplace_skill', { skillDirName });
  },
  importSkillFromDirectory: async (directoryPath: string) => {
    return invoke<AgentSkill>('import_skill_from_directory', { directoryPath });
  },
  importSkillFromZip: async (zipPath: string) => {
    return invoke<AgentSkill>('import_skill_from_zip', { zipPath });
  },

  // Docker Sandbox
  checkDockerAvailable: async () => {
    return invoke<boolean>('check_docker_available', {});
  },

  // Agent Scope
  updateAgentScope: async (agentId: string, scopeType: string, workspacePath?: string) => {
    return invoke<void>('update_agent_scope', { agentId, scopeType, workspacePath });
  },

  // MCP
  // MCP
  listMCPServers: async () => {
    const servers = await invoke<McpServer[]>('get_mcp_servers');
    // Map status for frontend since backend doesn't return it yet (placeholder)
    return servers.map(s => ({ ...s, status: s.is_enabled ? 'connected' : 'offline' as any }));
  },
  createMCPServer: async (data: McpServerUpdate, templateId?: string) => {
    return invoke<McpServer>('create_mcp_server', { data, templateId });
  },
  updateMCPServer: async (id: string, data: McpServerUpdate) => {
    return invoke<McpServer>('update_mcp_server', { id, data });
  },
  deleteMCPServer: async (id: string) => {
    return invoke<void>('delete_mcp_server', { id });
  },
  getMCPTemplates: async () => {
    return invoke<McpTemplate[]>('get_mcp_templates');
  },
  addMCPToAgent: async (agentId: string, mcpServerId: string) => {
    return invoke<Agent>('add_mcp_to_agent', { agentId, mcpServerId });
  },
  removeMCPFromAgent: async (agentId: string, mcpServerId: string) => {
    return invoke<Agent>('remove_mcp_from_agent', { agentId, mcpServerId });
  },

  // Federation
  listNodes: async () => ([]),
  registerNode: async (node: any) => ({ success: true }),

  // Execution
  getExecutionSettings: async () => ({
    mode: 'autopilot',
    whitelisted_commands: [],
    whitelisted_tools: [],
    blacklisted_commands: [],
    available_modes: ['autopilot', 'require_approval', 'smart_approval']
  }),
  updateExecutionSettings: async (settings: any) => ({ status: 'ok', settings: {} }),
  setExecutionMode: async (mode: any) => ({ status: 'ok', mode }),
  addWhitelistedCommand: async (pattern: string) => ({ status: 'ok', pattern }),
  removeWhitelistedCommand: async (pattern: string) => ({ status: 'ok', pattern }),
  addWhitelistedTool: async (tool: string) => ({ status: 'ok', tool }),
  removeWhitelistedTool: async (tool: string) => ({ status: 'ok', tool }),

  getAvailableModels: async () => ({
    providers: {
      anthropic: { display_name: 'Anthropic', available: true, default: 'claude-3-opus', models: [] },
      openai: { display_name: 'OpenAI', available: true, default: 'gpt-4', models: [] },
      gemini: { display_name: 'Google', available: true, default: 'gemini-3-flash-preview', models: [] }
    },
    defaults: {}
  }),

  // Window commands
  toggleDevtools: async () => invoke<void>('toggle_devtools'),
  isDevMode: async () => invoke<boolean>('is_dev_mode'),
  getCurrentWorkingDirectory: async () => invoke<string>('get_current_working_directory'),
};
