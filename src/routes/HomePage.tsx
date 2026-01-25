/**
 * Home Page - Dashboard with system status
 */

import { Link } from "react-router-dom";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import {
  Bot,
  MessageSquare,
  Settings,
  Activity,
  CheckCircle2,
  XCircle,
  Send,
  Zap,
  Sparkles,
} from "lucide-react";
import {
  useGatewayStatus,
  useSessions,
  useMessagingStatus,
  useServerInfo,
} from "@/lib/hooks/use-anycowork";

function HomePage() {
  const { data: gatewayData, isLoading: isGatewayLoading } = useGatewayStatus();
  const { data: sessionsData, isLoading: isSessionsLoading } = useSessions();
  const { data: messagingData, isLoading: isMessagingLoading } = useMessagingStatus();
  const { data: serverInfo, isLoading: isServerInfoLoading } = useServerInfo();

  const isLoading =
    isGatewayLoading || isSessionsLoading || isMessagingLoading || isServerInfoLoading;

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  if (isLoading) {
    return (
      <div className="flex-1 overflow-auto p-8">
        <div className="max-w-6xl mx-auto space-y-8">
          <Skeleton className="h-12 w-64" />
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {[1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-32" />
            ))}
          </div>
        </div>
      </div>
    );
  }

  const sessions = sessionsData?.sessions || [];
  const recentSessions = sessions.slice(0, 5);
  const activeSessions = sessions.filter((s) => s.status === "active").length;

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-6xl mx-auto p-12 space-y-12">
        {/* Header */}
        <div className="space-y-4">
          <h1 className="text-4xl font-semibold tracking-tight text-foreground">Welcome to AnyCowork</h1>
          <p className="text-xl text-muted-foreground font-light">
            Your personal, collaborative AI control center.
          </p>
        </div>

        {/* Quick Actions */}
        <div className="flex gap-3">
          <Link to="/settings">
            <Button variant="outline" className="gap-2" size="lg">
              <Settings className="h-4 w-4" />
              Configure
            </Button>
          </Link>
        </div>

        {/* System Status Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Gateway Status */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-blue-100 dark:bg-blue-900/50">
                  <Activity className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                </div>
                <div>
                  <CardTitle className="text-base">Gateway</CardTitle>
                  <CardDescription className="text-xs">WebSocket Control Plane</CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Status</span>
                {gatewayData?.status === "running" ? (
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400">
                    <CheckCircle2 className="h-3 w-3" />
                    Running
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-red-50 text-red-700 dark:bg-red-900/30 dark:text-red-400">
                    <XCircle className="h-3 w-3" />
                    Offline
                  </span>
                )}
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Clients</span>
                <span className="text-sm font-medium">{gatewayData?.connected_clients || 0}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Uptime</span>
                <span className="text-sm font-medium">{formatUptime(gatewayData?.uptime || 0)}</span>
              </div>
            </CardContent>
          </Card>

          {/* Agent Sessions */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-purple-100 dark:bg-purple-900/50">
                  <Bot className="h-5 w-5 text-purple-600 dark:text-purple-400" />
                </div>
                <div>
                  <CardTitle className="text-base">Agent Sessions</CardTitle>
                  <CardDescription className="text-xs">Active AI Conversations</CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Total</span>
                <span className="text-2xl font-semibold">{sessions.length}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Active</span>
                <span className="text-sm font-medium">{activeSessions}</span>
              </div>
            </CardContent>
          </Card>

          {/* AI Provider */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-blue-100 dark:bg-blue-900/50">
                  <Sparkles className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                </div>
                <div>
                  <CardTitle className="text-base">AI Provider</CardTitle>
                  <CardDescription className="text-xs">Current Model</CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Provider</span>
                <span className="text-sm font-medium capitalize">{serverInfo?.ai_provider || "unknown"}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Model</span>
                <span className="text-xs font-medium">{serverInfo?.model || "unknown"}</span>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Messaging Platforms */}
        <Card>
          <CardHeader>
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-xl bg-emerald-100/50 dark:bg-emerald-900/20 text-emerald-600 dark:text-emerald-400">
                <Send className="h-5 w-5" />
              </div>
              <div>
                <CardTitle>Messaging Platforms</CardTitle>
                <CardDescription>
                  Connected messaging services
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 gap-4">
              {/* Telegram */}
              <div className="p-4 rounded-lg border">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <MessageSquare className="h-5 w-5" />
                    <span className="font-medium">Telegram</span>
                  </div>
                  {messagingData?.telegram?.connected ? (
                    <Badge variant="outline" className="gap-1.5 bg-emerald-50 dark:bg-emerald-950/30">
                      <CheckCircle2 className="h-3 w-3" />
                      Connected
                    </Badge>
                  ) : (
                    <Badge variant="outline" className="gap-1.5 bg-slate-50 dark:bg-slate-950/30">
                      <XCircle className="h-3 w-3" />
                      Disconnected
                    </Badge>
                  )}
                </div>
                {messagingData?.telegram?.connected && (
                  <div className="space-y-1">
                    <p className="text-xs text-muted-foreground">
                      Bot: {messagingData.telegram.bot_username || "N/A"}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      Active chats: {messagingData.telegram.active_sessions || 0}
                    </p>
                  </div>
                )}
              </div>


            </div>
          </CardContent>
        </Card>

        {/* Recent Sessions */}
        {recentSessions.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle>Recent Sessions</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {recentSessions.map((session) => (
                  <div
                    key={session.id}
                    className="flex items-center gap-3 p-3 rounded-lg border hover:bg-accent transition-colors"
                  >
                    <div className="p-2 rounded-lg bg-purple-100 dark:bg-purple-900/50">
                      <Bot className="h-4 w-4 text-purple-600 dark:text-purple-400" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="font-medium text-sm truncate">{session.id}</p>
                      <p className="text-xs text-muted-foreground">
                        {formatDate(session.created_at)}
                      </p>
                    </div>
                    <Badge variant="outline" className="text-xs">
                      {session.status}
                    </Badge>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Getting Started */}
        {sessions.length === 0 && (
          <Card>
            <CardContent className="text-center py-12">
              <div className="p-4 rounded-full bg-muted w-fit mx-auto mb-4">
                <Zap className="h-12 w-12 text-muted-foreground" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Get Started with AnyCowork</h3>
              <p className="text-muted-foreground mb-6 max-w-md mx-auto">
                Configure your messaging platforms to begin using your AI assistant.
              </p>
              <Link to="/settings">
                <Button variant="outline" className="gap-2">
                  <Settings className="h-4 w-4" />
                  Settings
                </Button>
              </Link>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}

export default HomePage;
