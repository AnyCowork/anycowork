import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { anycoworkApi } from "@/lib/anycowork-api";

// --- Agents ---

export function useAgents(status?: 'active' | 'inactive' | 'error') {
    return useQuery({
        queryKey: ["agents", status],
        queryFn: () => anycoworkApi.listAgents(status),
    });
}

export function useAgent(agentId: string) {
    return useQuery({
        queryKey: ["agent", agentId],
        queryFn: () => anycoworkApi.getAgent(agentId),
        enabled: !!agentId,
    });
}

// --- Skills ---

export function useAgentSkills(agentId: string) {
    return useQuery({
        queryKey: ["agent-skills", agentId],
        queryFn: () => anycoworkApi.getAgentSkills(agentId),
        enabled: !!agentId,
    });
}

export function useAssignAgentSkills() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ agentId, skillIds }: { agentId: string; skillIds: string[] }) =>
            anycoworkApi.assignAgentSkills(agentId, skillIds),
        onSuccess: (_, { agentId }) => {
            queryClient.invalidateQueries({ queryKey: ["agent-skills", agentId] });
        },
    });
}

// --- MCP Servers ---

export function useAgentMCPServers(agentId: string) {
    return useQuery({
        queryKey: ["agent-mcp", agentId],
        queryFn: () => anycoworkApi.getAgentMCPServers(agentId),
        enabled: !!agentId,
    });
}

export function useAssignAgentMCPServers() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ agentId, serverIds }: { agentId: string; serverIds: string[] }) =>
            anycoworkApi.assignAgentMCPServers(agentId, serverIds),
        onSuccess: (_, { agentId }) => {
            queryClient.invalidateQueries({ queryKey: ["agent-mcp", agentId] });
        },
    });
}
