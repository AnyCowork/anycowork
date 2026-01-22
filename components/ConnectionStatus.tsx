import React from "react";
import { Badge } from "@/components/ui/badge";
import { CheckCircle2, XCircle } from "lucide-react";

interface ConnectionStatusProps {
    name: string;
    connected: boolean;
}

export const ConnectionStatus: React.FC<ConnectionStatusProps> = ({ name, connected }) => {
    return (
        <div className="flex items-center gap-2">
            {connected ? (
                <CheckCircle2 className="h-4 w-4 text-green-500" />
            ) : (
                <XCircle className="h-4 w-4 text-red-500" />
            )}
            <Badge variant={connected ? "default" : "destructive"}>{name}</Badge>
        </div>
    );
};
