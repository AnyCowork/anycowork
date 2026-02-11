/**
 * MCP Servers Management Page - Manage global Model Context Protocol servers
 * MCP Servers are global components that can be used by any agent
 */

import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Server, Plus, CheckCircle2, XCircle, AlertCircle, Loader2, Info, Trash2, Edit, Terminal, Globe } from "lucide-react";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { anycoworkApi, type McpServer, type McpTemplate, type McpServerUpdate } from "@/lib/anycowork-api";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { toast } from "sonner";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { KeyValueEditor } from "@/components/KeyValueEditor";
import { StringListEditor } from "@/components/StringListEditor";

export default function MCPServersPage() {
  const queryClient = useQueryClient();
  const { confirm, ConfirmDialog } = useConfirm();
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [editingServer, setEditingServer] = useState<McpServer | null>(null);

  const { data: mcpServers, isLoading } = useQuery({
    queryKey: ['mcp-servers'],
    queryFn: () => anycoworkApi.listMCPServers()
  });

  const { data: templates = [] } = useQuery({
    queryKey: ['mcp-templates'],
    queryFn: () => anycoworkApi.getMCPTemplates(),
    enabled: isAddDialogOpen
  });

  const createServerMutation = useMutation({
    mutationFn: (data: { server: McpServerUpdate, templateId?: string }) =>
      anycoworkApi.createMCPServer(data.server, data.templateId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mcp-servers'] });
      setIsAddDialogOpen(false);
      toast.success("MCP Server added successfully");
    },
    onError: (error) => {
      toast.error(`Failed to add server: ${error}`);
    }
  });

  const updateServerMutation = useMutation({
    mutationFn: (data: { id: string, server: McpServerUpdate }) =>
      anycoworkApi.updateMCPServer(data.id, data.server),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mcp-servers'] });
      setEditingServer(null);
      toast.success("MCP Server updated successfully");
    },
    onError: (error) => {
      toast.error(`Failed to update server: ${error}`);
    }
  });

  const deleteServerMutation = useMutation({
    mutationFn: (id: string) => anycoworkApi.deleteMCPServer(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mcp-servers'] });
      toast.success("MCP Server deleted successfully");
    },
    onError: (error) => {
      toast.error(`Failed to delete server: ${error}`);
    }
  });

  const getStatusInfo = (status: string) => {
    switch (status) {
      case "connected":
      case "online":
        return { icon: <CheckCircle2 className="h-4 w-4 text-green-500" />, label: "Connected", color: "text-green-500" };
      case "offline":
        return { icon: <XCircle className="h-4 w-4 text-slate-500" />, label: "Offline", color: "text-slate-500" };
      case "error":
        return { icon: <AlertCircle className="h-4 w-4 text-red-500" />, label: "Error", color: "text-red-500" };
      default:
        return { icon: <Info className="h-4 w-4 text-slate-500" />, label: "Unknown", color: "text-slate-500" };
    }
  };

  const [deletingId, setDeletingId] = useState<string | null>(null);

  const handleDeleteWithLoading = async (id: string) => {
    if (await confirm("Are you sure you want to delete this MCP Server?", { variant: "destructive" })) {
      setDeletingId(id);
      deleteServerMutation.mutate(id, {
        onSettled: () => setDeletingId(null)
      });
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      <ConfirmDialog />

      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto max-w-7xl px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                <Server className="h-4 w-4 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-xl font-bold">Connectors - MCP Servers</h1>
                <p className="text-xs text-muted-foreground">
                  Manage global Model Context Protocol servers available to all agents
                </p>
              </div>
            </div>
            <Button size="sm" className="gap-1.5" onClick={() => setIsAddDialogOpen(true)}>
              <Plus className="h-3.5 w-3.5" />
              Add MCP Server
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="mx-auto max-w-7xl p-6">
        {isLoading ? (
          <div className="flex justify-center p-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <TooltipProvider>
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {mcpServers?.map((server) => {
                const statusInfo = getStatusInfo(server.status);
                const commandDisplay = server.server_type === 'stdio'
                  ? `${server.command} ${server.args?.join(' ') || ''}`.trim()
                  : server.url || '';

                return (
                  <Card key={server.id} className="group relative overflow-hidden transition-all hover:shadow-lg flex flex-col">
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-2.5">
                          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary/20 to-primary/10">
                            {server.server_type === 'stdio' ? (
                              <Terminal className="h-4 w-4 text-primary" />
                            ) : (
                              <Globe className="h-4 w-4 text-primary" />
                            )}
                          </div>
                          <div className="flex-1 min-w-0">
                            <CardTitle className="text-lg truncate">{server.name}</CardTitle>
                            <div className="mt-1 flex items-center gap-2">
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <span className="flex items-center gap-1 cursor-help">
                                    {statusInfo.icon}
                                    <span className={`text-xs ${statusInfo.color}`}>{statusInfo.label}</span>
                                  </span>
                                </TooltipTrigger>
                                <TooltipContent>
                                  <p>Server status: {statusInfo.label}</p>
                                </TooltipContent>
                              </Tooltip>
                              <Badge variant="outline" className="text-xs capitalize">
                                {server.server_type}
                              </Badge>
                            </div>
                          </div>
                        </div>
                      </div>
                    </CardHeader>

                    <CardContent className="flex-1 pt-0">
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <div className="bg-muted/50 p-3 rounded-lg font-mono text-xs overflow-hidden text-ellipsis whitespace-nowrap cursor-help border border-muted">
                            {server.server_type === 'stdio' ? (
                              <span className="flex items-center gap-2">
                                <span className="text-muted-foreground">$</span>
                                <span className="truncate">{commandDisplay}</span>
                              </span>
                            ) : (
                              <span className="truncate text-blue-600 dark:text-blue-400">{server.url}</span>
                            )}
                          </div>
                        </TooltipTrigger>
                        <TooltipContent side="bottom" className="max-w-md">
                          <p className="font-mono text-xs break-all">{commandDisplay}</p>
                        </TooltipContent>
                      </Tooltip>
                    </CardContent>

                    <CardFooter className="flex justify-end gap-2 pt-3 border-t bg-muted/20 px-4 py-3">
                      <Button variant="ghost" size="sm" onClick={() => setEditingServer(server)}>
                        <Edit className="h-4 w-4 mr-1.5" /> Edit
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-destructive hover:text-destructive hover:bg-destructive/10"
                        onClick={() => handleDeleteWithLoading(server.id)}
                        disabled={deletingId === server.id}
                      >
                        {deletingId === server.id ? (
                          <Loader2 className="h-4 w-4 mr-1.5 animate-spin" />
                        ) : (
                          <Trash2 className="h-4 w-4 mr-1.5" />
                        )}
                        Delete
                      </Button>
                    </CardFooter>
                  </Card>
                );
              })}
              {mcpServers?.length === 0 && (
                <Card className="col-span-full border-dashed">
                  <CardContent className="flex flex-col items-center justify-center py-16">
                    <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted mb-3">
                      <Server className="h-6 w-6 text-muted-foreground" />
                    </div>
                    <h3 className="text-lg font-semibold">No MCP servers configured</h3>
                    <p className="mt-2 text-sm text-muted-foreground text-center max-w-md">
                      MCP (Model Context Protocol) servers extend your agents with additional capabilities like file access, database queries, and more.
                    </p>
                    <Button size="sm" onClick={() => setIsAddDialogOpen(true)} className="mt-4 gap-1.5">
                      <Plus className="h-3.5 w-3.5" />
                      Add Server
                    </Button>
                  </CardContent>
                </Card>
              )}
            </div>
          </TooltipProvider>
        )}
      </div>

      <AddServerDialog
        open={isAddDialogOpen}
        onOpenChange={setIsAddDialogOpen}
        templates={templates}
        onCreate={(data, templateId) => createServerMutation.mutate({ server: data, templateId })}
        isLoading={createServerMutation.isPending}
      />

      {editingServer && (
        <EditServerDialog
          server={editingServer}
          open={!!editingServer}
          onOpenChange={(open) => !open && setEditingServer(null)}
          onUpdate={(id, data) => updateServerMutation.mutate({ id, server: data })}
          isLoading={updateServerMutation.isPending}
        />
      )}
    </div>
  );
}

function AddServerDialog({ open, onOpenChange, templates, onCreate, isLoading }: any) {
  const [activeTab, setActiveTab] = useState("templates");
  const [formVersion, setFormVersion] = useState(0);
  const [formData, setFormData] = useState<McpServerUpdate>({
    name: "",
    server_type: "stdio",
    command: "",
    args: [],
    env: {},
    url: "",
    is_enabled: true
  });

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      setFormData({
        name: "",
        server_type: "stdio",
        command: "",
        args: [],
        env: {},
        url: "",
        is_enabled: true
      });
      setFormVersion(v => v + 1);
      setActiveTab("templates");
    }
  }, [open]);

  const handleSubmit = () => {
    onCreate(formData, activeTab === "templates" ? undefined : undefined);
  };

  const loadTemplate = (template: McpTemplate) => {
    setFormData({
      name: template.name,
      server_type: template.server_type,
      command: template.command,
      args: template.args,
      env: template.env || {},
      url: template.url,
      is_enabled: true
    });
    setFormVersion(v => v + 1);
    setActiveTab("custom");
    toast.info(`Loaded template: ${template.name}`);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Add MCP Server</DialogTitle>
          <DialogDescription>
            Configure a new Model Context Protocol server.
          </DialogDescription>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="templates">From Template</TabsTrigger>
            <TabsTrigger value="custom">Custom Configuration</TabsTrigger>
          </TabsList>

          <TabsContent value="templates" className="space-y-4 py-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {templates.map((tpl: McpTemplate) => (
                <Card key={tpl.id} className="cursor-pointer hover:border-primary transition-colors" onClick={() => loadTemplate(tpl)}>
                  <CardHeader className="pb-3">
                    <CardTitle className="text-base">{tpl.name}</CardTitle>
                    <CardDescription className="text-xs">{tpl.description}</CardDescription>
                  </CardHeader>
                  <CardContent className="p-4 pt-0">
                    <Badge variant="secondary" className="text-xs">{tpl.server_type}</Badge>
                  </CardContent>
                </Card>
              ))}
            </div>
          </TabsContent>

          <TabsContent value="custom" className="space-y-4 py-4">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Name</Label>
                <Input
                  value={formData.name}
                  onChange={e => setFormData({ ...formData, name: e.target.value })}
                  placeholder="My MCP Server"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Type</Label>
                  <Select
                    value={formData.server_type}
                    onValueChange={(val: any) => setFormData({ ...formData, server_type: val })}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="stdio">Stdio (Command)</SelectItem>
                      <SelectItem value="sse">SSE (HTTP)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {formData.server_type === 'stdio' ? (
                <>
                  <div className="space-y-2">
                    <Label>Command</Label>
                    <Input
                      value={formData.command}
                      onChange={e => setFormData({ ...formData, command: e.target.value })}
                      placeholder="npx, python, etc."
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Arguments (one per line)</Label>
                    <StringListEditor
                      key={formVersion}
                      initialData={formData.args || []}
                      onChange={(args) => setFormData({ ...formData, args })}
                      placeholder="-y"
                    />
                  </div>
                </>
              ) : (
                <div className="space-y-2">
                  <Label>Server URL</Label>
                  <Input
                    value={formData.url}
                    onChange={e => setFormData({ ...formData, url: e.target.value })}
                    placeholder="http://localhost:8000/sse"
                  />
                </div>
              )}

              <div className="space-y-2">
                <Label>Environment Variables</Label>
                <KeyValueEditor
                  key={formVersion}
                  initialData={formData.env || {}}
                  onChange={(env) => setFormData({ ...formData, env })}
                  keyPlaceholder="API_KEY"
                  valuePlaceholder="secret..."
                />
              </div>
            </div>

            <div className="flex justify-end pt-4">
              <Button onClick={handleSubmit} disabled={isLoading}>
                {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Create Server
              </Button>
            </div>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}

function EditServerDialog({ server, open, onOpenChange, onUpdate, isLoading }: any) {
  const [formData, setFormData] = useState<McpServerUpdate>({
    name: server.name,
    server_type: server.server_type,
    command: server.command,
    args: server.args || [],
    env: server.env || {},
    url: server.url,
    is_enabled: server.is_enabled
  });

  const handleSubmit = () => {
    onUpdate(server.id, formData);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Edit MCP Server</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>Name</Label>
            <Input
              value={formData.name}
              onChange={e => setFormData({ ...formData, name: e.target.value })}
            />
          </div>

          {/* Simplified edit form, usually type change is tricky, let's allow basic edits */}
          {formData.server_type === 'stdio' ? (
            <>
              <div className="space-y-2">
                <Label>Command</Label>
                <Input
                  value={formData.command}
                  onChange={e => setFormData({ ...formData, command: e.target.value })}
                />
              </div>
              <div className="space-y-2">
                <Label>Arguments</Label>
                <StringListEditor
                  initialData={formData.args || []}
                  onChange={(args) => setFormData({ ...formData, args })}
                />
              </div>
            </>
          ) : (
            <div className="space-y-2">
              <Label>Server URL</Label>
              <Input
                value={formData.url}
                onChange={e => setFormData({ ...formData, url: e.target.value })}
              />
            </div>
          )}

          <div className="space-y-2">
            <Label>Environment Variables</Label>
            <KeyValueEditor
              initialData={formData.env || {}}
              onChange={(env) => setFormData({ ...formData, env })}
            />
          </div>

          <div className="flex justify-end pt-4">
            <Button onClick={handleSubmit} disabled={isLoading}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Save Changes
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
