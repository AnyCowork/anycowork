import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { anycoworkApi, Task, TaskCreate, TaskUpdate } from "@/lib/anycowork-api";

export function useTasks(sessionId?: string, status?: string) {
    return useQuery({
        queryKey: ["tasks", sessionId, status],
        queryFn: async () => {
            const response = await anycoworkApi.listTasks(sessionId, status);
            return response.tasks;
        },
        // Refresh every 10 seconds for real-time-like feel
        refetchInterval: 10000,
    });
}

export function useCreateTask() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: (data: TaskCreate) => anycoworkApi.createTask(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["tasks"] });
        },
    });
}

export function useUpdateTask() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ id, data }: { id: string; data: TaskUpdate }) =>
            anycoworkApi.updateTask(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["tasks"] });
        },
    });
}

export function useDeleteTask() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: (id: string) => anycoworkApi.deleteTask(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["tasks"] });
        },
    });
}
