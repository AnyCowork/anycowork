/**
 * Connections Page - Manage configured federation nodes
 */

import { useState } from "react";
import { Plus, Trash2, Globe, Server, CheckCircle2, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { anycoworkApi, FederationNode } from "@/lib/anycowork-api";

export default function ConnectionsPage() {
    const [isOpen, setIsOpen] = useState(false);
    const queryClient = useQueryClient();

    const { data: nodes = [], isLoading } = useQuery({
        queryKey: ["federation-nodes"],
        queryFn: anycoworkApi.listNodes,
    });

    const registerMutation = useMutation({
        mutationFn: anycoworkApi.registerNode,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["federation-nodes"] });
            setIsOpen(false);
            resetForm();
        },
    });

    const [formData, setFormData] = useState<Partial<FederationNode>>({
        node_name: "",
        host: "",
        port: 8080,
        gateway_port: 8081,
        capabilities: ["chat", "agent"],
    });

    const resetForm = () => {
        setFormData({
            node_name: "",
            host: "",
            port: 8080,
            gateway_port: 8081,
            capabilities: ["chat", "agent"],
        });
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        registerMutation.mutate({
            ...formData,
            node_id: crypto.randomUUID(), // Generate client-side for now or let server handle? Server handles if not provided, usually.
        });
    };

    return (
        <div className="container mx-auto p-6 space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Connections</h1>
                    <p className="text-muted-foreground">
                        Manage distributed AnyCowork nodes and federation.
                    </p>
                </div>
                <Dialog open={isOpen} onOpenChange={setIsOpen}>
                    <DialogTrigger asChild>
                        <Button className="gap-2">
                            <Plus className="h-4 w-4" /> Add Connection
                        </Button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Add New Connection</DialogTitle>
                            <DialogDescription>
                                Connect to another AnyCowork Desktop or Mobile node.
                            </DialogDescription>
                        </DialogHeader>
                        <form onSubmit={handleSubmit} className="space-y-4">
                            <div className="space-y-2">
                                <Label htmlFor="name">Node Name</Label>
                                <Input
                                    id="name"
                                    placeholder="e.g. My Mac Mini"
                                    required
                                    value={formData.node_name}
                                    onChange={(e) =>
                                        setFormData({ ...formData, node_name: e.target.value })
                                    }
                                />
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label htmlFor="host">Host / IP</Label>
                                    <Input
                                        id="host"
                                        placeholder="192.168.1.x"
                                        required
                                        value={formData.host}
                                        onChange={(e) =>
                                            setFormData({ ...formData, host: e.target.value })
                                        }
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="port">API Port</Label>
                                    <Input
                                        id="port"
                                        type="number"
                                        required
                                        value={formData.port}
                                        onChange={(e) =>
                                            setFormData({ ...formData, port: parseInt(e.target.value) })
                                        }
                                    />
                                </div>
                            </div>
                            <DialogFooter>
                                <Button
                                    type="button"
                                    variant="outline"
                                    onClick={() => setIsOpen(false)}
                                >
                                    Cancel
                                </Button>
                                <Button type="submit" disabled={registerMutation.isPending}>
                                    {registerMutation.isPending ? "Adding..." : "Add Node"}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
            </div>

            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                {nodes.map((node) => (
                    <Card key={node.node_id}>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">
                                {node.node_name}
                            </CardTitle>
                            {node.status === "active" ? (
                                <CheckCircle2 className="h-4 w-4 text-green-500" />
                            ) : (
                                <XCircle className="h-4 w-4 text-muted-foreground" />
                            )}
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold flex items-center gap-2">
                                <Globe className="h-6 w-6 text-muted-foreground" />
                                {node.host}
                            </div>
                            <div className="text-xs text-muted-foreground mt-1">
                                Port: {node.port} | Gateway: {node.gateway_port}
                            </div>
                            <div className="flex flex-wrap gap-2 mt-4">
                                {node.capabilities.map((cap) => (
                                    <Badge key={cap} variant="secondary">
                                        {cap}
                                    </Badge>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                ))}
                {nodes.length === 0 && !isLoading && (
                    <div className="col-span-full text-center p-12 text-muted-foreground border rounded-lg border-dashed">
                        <Globe className="mx-auto h-12 w-12 opacity-50 mb-4" />
                        <h3 className="text-lg font-medium">No connections yet</h3>
                        <p>Add a connection to start federating with other nodes.</p>
                    </div>
                )}
            </div>
        </div>
    );
}
