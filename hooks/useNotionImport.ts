import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";

export const useImportNotionFile = () => {
    return useMutation({
        mutationFn: async (file: File) => {
            // TODO: Implement backend command
            console.log("Mock importing Notion file:", file.name);
            await new Promise((resolve) => setTimeout(resolve, 1000));
            return { success: true };
        },
        onSuccess: () => {
            toast.success("Notion import (Mock) successful!");
        },
        onError: (error) => {
            toast.error("Failed to import Notion file");
            console.error(error);
        }
    });
};

export const useImportNotionZip = () => {
    return useMutation({
        mutationFn: async (file: File) => {
            // TODO: Implement backend command
            console.log("Mock importing Notion zip:", file.name);
            await new Promise((resolve) => setTimeout(resolve, 2000));
            return { success: true };
        },
        onSuccess: () => {
            toast.success("Notion Zip import (Mock) successful!");
        },
        onError: (error) => {
            toast.error("Failed to import Notion zip");
            console.error(error);
        }
    });
};
