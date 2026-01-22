/**
 * MCP Servers Management Page - Manage global Model Context Protocol servers
 * MCP Servers are global components that can be used by any agent
 */

import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Server, Plus, CheckCircle2, XCircle, AlertCircle, Loader2, Info } from "lucide-react";
import { anycoworkApi } from "@/lib/anycowork-api";

export default function MCPServersPage() {
  const { data: mcpServers, isLoading } = useQuery({
    queryKey: ['mcp-servers'],
    queryFn: () => anycoworkApi.listMCPServers()
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "connected":
      case "online":
        return <CheckCircle2 className="h-4 w-4 text-green-500" />;
      case "offline":
        return <XCircle className="h-4 w-4 text-slate-500" />;
      case "error":
        return <AlertCircle className="h-4 w-4 text-red-500" />;
      default:
        return <Info className="h-4 w-4 text-slate-500" />;
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto max-w-7xl px-8 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                <Server className="h-5 w-5 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-2xl font-bold">MCP Servers</h1>
                <p className="text-sm text-muted-foreground">
                  Manage global Model Context Protocol servers available to all agents
                </p>
              </div>
            </div>
            <Button size="lg" className="gap-2">
              <Plus className="h-4 w-4" />
              Add MCP Server
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="mx-auto max-w-7xl p-8">
        {isLoading ? (
          <div className="flex justify-center p-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="grid gap-6 md:grid-cols-2">
            {mcpServers?.map((server: any) => (
              <Card key={server.id} className="group relative overflow-hidden transition-all hover:shadow-lg">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                        <Server className="h-5 w-5" />
                      </div>
                      <div>
                        <CardTitle className="text-lg">{server.name}</CardTitle>
                        <div className="mt-1 flex items-center gap-2">
                          {getStatusIcon(server.status)}
                          <Badge variant="outline" className="text-xs">
                            {server.type || "http"}
                          </Badge>
                        </div>
                      </div>
                    </div>
                  </div>
                  {server.description && <CardDescription className="mt-3">{server.description}</CardDescription>}
                </CardHeader>

                <CardContent className="space-y-4">
                  <div className="space-y-2 text-sm">
                    {server.endpoint && (
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">Endpoint:</span>
                        <code className="rounded bg-muted px-2 py-1 text-xs">{server.endpoint}</code>
                      </div>
                    )}
                    <div className="flex items-center justify-between">
                      <span className="text-muted-foreground">Status:</span>
                      <Badge
                        variant="outline"
                        className={
                          server.status === "connected" || server.status === "online"
                            ? "border-green-500 text-green-500"
                            : server.status === "offline"
                              ? "border-slate-500 text-slate-500"
                              : "border-red-500 text-red-500"
                        }
                      >
                        {server.status}
                      </Badge>
                    </div>
                  </div>

                  <div className="flex gap-2 pt-2">
                    <Button variant="outline" size="sm" className="flex-1">
                      Configure
                    </Button>
                    <Button variant="outline" size="sm" className="flex-1">
                      Test Connection
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
            {mcpServers?.length === 0 && (
              <div className="col-span-2 text-center py-12 text-muted-foreground">
                No MCP servers configured yet.
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
