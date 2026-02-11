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
        <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
          <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <div className="mx-auto max-w-7xl px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2.5">
                  <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                    <Globe className="h-4 w-4 text-primary-foreground" />
                  </div>
                  <div>
                    <h1 className="text-xl font-bold">Connections</h1>
                    <p className="text-xs text-muted-foreground">
                      Manage distributed AnyCowork nodes and federation
                    </p>
                  </div>
                </div>
                <Dialog open={isOpen} onOpenChange={setIsOpen}>
                    <DialogTrigger asChild>
                        <Button size="sm" className="gap-1.5">
                            <Plus className="h-3.5 w-3.5" /> Add Connection
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
            </div>
          </div>

          <div className="mx-auto max-w-7xl p-6">
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
                            <div className="text-lg font-bold flex items-center gap-2">
                                <Globe className="h-4 w-4 text-muted-foreground" />
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
                        <Globe className="mx-auto h-8 w-8 opacity-50 mb-3" />
                        <h3 className="text-lg font-medium">No connections yet</h3>
                        <p>Add a connection to start federating with other nodes.</p>
                    </div>
                )}
            </div>
          </div>
        </div>
    );
}
