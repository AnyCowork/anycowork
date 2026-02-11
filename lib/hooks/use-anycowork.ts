/**
 * React Query hooks for AnyCowork API
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { anycoworkApi, AIConfig, MessagingConfig, Agent, AgentCreate, AgentUpdate, ExecutionMode, ExecutionSettingsUpdate, MailThread, MailMessage } from '../anycowork-api';
import { toast } from 'sonner';

// Query keys
export const queryKeys = {
  gatewayStatus: ['gateway', 'status'],
  sessions: ['sessions'],
  messagingStatus: ['messaging', 'status'],
  serverInfo: ['server', 'info'],
  aiConfig: ['config', 'ai'],
  messagingConfig: ['config', 'messaging'],
  executionSettings: ['config', 'execution'],
  availableModels: ['config', 'models'],
  agents: ['agents'],
  agent: (id: string) => ['agents', id],
  agentSkills: (id: string) => ['agents', id, 'skills'],
  agentMCP: (id: string) => ['agents', id, 'mcp'],
  agentMessaging: (id: string) => ['agents', id, 'messaging'],
  mailThreads: (accountId?: string, folder?: string, isArchived?: boolean) => ['mail', 'threads', accountId, folder, isArchived],
  mailMessages: (threadId: string) => ['mail', 'messages', threadId],
  unreadMailCount: (accountId?: string) => ['mail', 'unread', accountId],
};

// Gateway hooks
export function useGatewayStatus() {
  return useQuery({
    queryKey: queryKeys.gatewayStatus,
    queryFn: anycoworkApi.getGatewayStatus,
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

// Session hooks
export function useSessions() {
  return useQuery({
    queryKey: queryKeys.sessions,
    queryFn: anycoworkApi.listSessions,
    refetchInterval: 5000,
  });
}

export function useCreateSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: anycoworkApi.createSession,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
      toast.success('Session created successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      console.error('Create Session Error:', error);
      toast.error(`Failed to create session: ${msg}`);
    },
  });
}

export function useDeleteSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: anycoworkApi.deleteSession,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
      toast.success('Session deleted successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to delete session: ${msg}`);
    },
  });
}

// Messaging hooks
export function useMessagingStatus() {
  return useQuery({
    queryKey: queryKeys.messagingStatus,
    queryFn: anycoworkApi.getMessagingStatus,
    refetchInterval: 5000,
  });
}

// Server info hooks
export function useServerInfo() {
  return useQuery({
    queryKey: queryKeys.serverInfo,
    queryFn: anycoworkApi.getServerInfo,
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

// Configuration hooks
export function useAIConfig() {
  return useQuery({
    queryKey: queryKeys.aiConfig,
    queryFn: anycoworkApi.getAIConfig,
  });
}

export function useUpdateAIConfig() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (config: Partial<AIConfig>) => anycoworkApi.updateAIConfig(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.aiConfig });
      queryClient.invalidateQueries({ queryKey: queryKeys.serverInfo });
      toast.success('AI configuration updated successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to update AI configuration: ${msg}`);
    },
  });
}

export function useMessagingConfig() {
  return useQuery({
    queryKey: queryKeys.messagingConfig,
    queryFn: anycoworkApi.getMessagingConfig,
  });
}

export function useUpdateMessagingConfig() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (config: Partial<MessagingConfig>) => anycoworkApi.updateMessagingConfig(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.messagingConfig });
      queryClient.invalidateQueries({ queryKey: queryKeys.messagingStatus });
      toast.success('Messaging configuration updated successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to update messaging configuration: ${msg}`);
    },
  });
}

export function useTestTelegramConnection() {
  return useMutation({
    mutationFn: anycoworkApi.testTelegramConnection,
    onSuccess: (data) => {
      if (data.success) {
        toast.success(`Connected successfully! Bot: @${data.bot_username}`);
      } else {
        toast.error(data.error || 'Failed to connect');
      }
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Connection test failed: ${msg}`);
    },
  });
}

// Agent Definition hooks
export function useAgents(status?: 'active' | 'inactive' | 'error', limit?: number) {
  return useQuery({
    queryKey: [...queryKeys.agents, status, limit],
    queryFn: () => anycoworkApi.listAgents(status, limit),
  });
}

export function useAgent(agentId: string) {
  return useQuery({
    queryKey: queryKeys.agent(agentId),
    queryFn: () => anycoworkApi.getAgent(agentId),
    enabled: !!agentId,
  });
}

export function useCreateAgent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: AgentCreate) => anycoworkApi.createAgent(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.agents });
      toast.success('Character created successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to create character: ${msg}`);
    },
  });
}

export function useUpdateAgent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ agentId, data }: { agentId: string; data: AgentUpdate }) =>
      anycoworkApi.updateAgent(agentId, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.agents });
      queryClient.invalidateQueries({ queryKey: queryKeys.agent(variables.agentId) });
      toast.success('Character updated successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to update character: ${msg}`);
    },
  });
}

export function useDeleteAgent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: anycoworkApi.deleteAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.agents });
      toast.success('Character deleted successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to delete character: ${msg}`);
    },
  });
}

// Agent Skills hooks
export function useAgentSkills(agentId: string) {
  return useQuery({
    queryKey: queryKeys.agentSkills(agentId),
    queryFn: () => anycoworkApi.getAgentSkills(agentId),
    enabled: !!agentId,
  });
}

export function useAssignAgentSkills() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ agentId, skillIds }: { agentId: string; skillIds: string[] }) =>
      anycoworkApi.assignAgentSkills(agentId, skillIds),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.agentSkills(variables.agentId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.agent(variables.agentId) });
      toast.success('Skills assigned successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to assign skills: ${msg}`);
    },
  });
}

// Agent MCP Server hooks
export function useAgentMCPServers(agentId: string) {
  return useQuery({
    queryKey: queryKeys.agentMCP(agentId),
    queryFn: () => anycoworkApi.getAgentMCPServers(agentId),
    enabled: !!agentId,
  });
}

export function useAssignAgentMCPServers() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ agentId, mcpServerIds }: { agentId: string; mcpServerIds: string[] }) =>
      anycoworkApi.assignAgentMCPServers(agentId, mcpServerIds),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.agentMCP(variables.agentId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.agent(variables.agentId) });
      toast.success('MCP servers assigned successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to assign MCP servers: ${msg}`);
    },
  });
}

// Agent Messaging hooks
export function useAgentMessagingConnections(agentId: string) {
  return useQuery({
    queryKey: queryKeys.agentMessaging(agentId),
    queryFn: () => anycoworkApi.getAgentMessagingConnections(agentId),
    enabled: !!agentId,
  });
}

export function useConfigureAgentMessaging() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ agentId, connectionIds }: { agentId: string; connectionIds: string[] }) =>
      anycoworkApi.configureAgentMessaging(agentId, connectionIds),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.agentMessaging(variables.agentId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.agent(variables.agentId) });
      toast.success('Messaging configured successfully');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to configure messaging: ${msg}`);
    },
  });
}

// Messaging Connections hooks
export function useMessagingConnections() {
  return useQuery({
    queryKey: ['messaging', 'connections'],
    queryFn: () => anycoworkApi.listMessagingConnections(),
  });
}

// Execution Settings hooks
export function useExecutionSettings() {
  return useQuery({
    queryKey: queryKeys.executionSettings,
    queryFn: anycoworkApi.getExecutionSettings,
  });
}

export function useUpdateExecutionSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (settings: ExecutionSettingsUpdate) => anycoworkApi.updateExecutionSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionSettings });
      toast.success('Execution settings updated');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to update execution settings: ${msg}`);
    },
  });
}

export function useSetExecutionMode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (mode: ExecutionMode) => anycoworkApi.setExecutionMode(mode),
    onSuccess: (_, mode) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionSettings });
      toast.success(`Execution mode set to ${mode.replace('_', ' ')}`);
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to set execution mode: ${msg}`);
    },
  });
}

export function useAddWhitelistedCommand() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (pattern: string) => anycoworkApi.addWhitelistedCommand(pattern),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionSettings });
      toast.success('Command pattern added to whitelist');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to add command: ${msg}`);
    },
  });
}

export function useRemoveWhitelistedCommand() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (pattern: string) => anycoworkApi.removeWhitelistedCommand(pattern),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionSettings });
      toast.success('Command pattern removed from whitelist');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to remove command: ${msg}`);
    },
  });
}

export function useAddWhitelistedTool() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (tool: string) => anycoworkApi.addWhitelistedTool(tool),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionSettings });
      toast.success('Tool added to whitelist');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to add tool: ${msg}`);
    },
  });
}

export function useRemoveWhitelistedTool() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (tool: string) => anycoworkApi.removeWhitelistedTool(tool),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionSettings });
      toast.success('Tool removed from whitelist');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to remove tool: ${msg}`);
    },
  });
}

// Available Models hooks
export function useAvailableModels() {
  return useQuery({
    queryKey: queryKeys.availableModels,
    queryFn: anycoworkApi.getAvailableModels,
  });
}

// Mail hooks
export function useMailThreads(accountId?: string, folder?: string, isArchived?: boolean) {
  return useQuery({
    queryKey: queryKeys.mailThreads(accountId, folder, isArchived),
    queryFn: () => anycoworkApi.getMailThreads(accountId, folder, isArchived),
    refetchInterval: 5000,
  });
}

export function useMailMessages(threadId: string) {
  return useQuery({
    queryKey: queryKeys.mailMessages(threadId),
    queryFn: () => anycoworkApi.getMailThreadMessages(threadId),
    enabled: !!threadId,
    refetchInterval: 5000,
  });
}

export function useUnreadMailCount(accountId?: string) {
  return useQuery({
    queryKey: queryKeys.unreadMailCount(accountId),
    queryFn: () => anycoworkApi.getUnreadMailCount(accountId),
    refetchInterval: 10000,
  });
}

export function useSendMail() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ fromAgentId, toAgentId, subject, body }: { fromAgentId: string | null; toAgentId: string | null; subject: string; body: string }) =>
      anycoworkApi.sendMail(fromAgentId, toAgentId, subject, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mail'] });
      toast.success('Email sent');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to send email: ${msg}`);
    },
  });
}

export function useReplyToMail() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ threadId, fromAgentId, content }: { threadId: string; fromAgentId: string | null; content: string }) =>
      anycoworkApi.replyToMail(threadId, fromAgentId, content),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.mailMessages(variables.threadId) });
      queryClient.invalidateQueries({ queryKey: ['mail', 'threads'] });
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to send reply: ${msg}`);
    },
  });
}

export function useMarkThreadRead() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: anycoworkApi.markThreadRead,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mail'] });
    },
  });
}

export function useArchiveThread() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: anycoworkApi.archiveThread,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mail'] });
      toast.success('Thread archived');
    },
    onError: (error: Error | string) => {
      const msg = error instanceof Error ? error.message : String(error);
      toast.error(`Failed to archive thread: ${msg}`);
    },
  });
}

