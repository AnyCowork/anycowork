/**
 * Conversations Management Page - Browse and manage chat history
 */

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { FolderOpen, Search, Bot, User, Clock, Archive, Trash2 } from "lucide-react";
import { Link } from "react-router-dom";

export default function ConversationsPage() {
  // Mock conversations for demonstration
  // Mock conversations for demonstration
  // Use a fixed timestamp for mock data to avoid purity issues validation
  const NOW = 1715620000000;

  const conversations = [
    {
      id: "conv-1",
      sessionId: "session-123",
      agentName: "Research Assistant",
      lastMessage: "Here's the analysis of the research papers you requested...",
      messageCount: 24,
      timestamp: NOW - 3600000,
      status: "active",
    },
    {
      id: "conv-2",
      sessionId: "session-456",
      agentName: "Code Helper",
      lastMessage: "I've reviewed the pull request and found a few issues...",
      messageCount: 15,
      timestamp: NOW - 7200000,
      status: "active",
    },
    {
      id: "conv-3",
      sessionId: "session-789",
      agentName: "Research Assistant",
      lastMessage: "The data visualization is complete.",
      messageCount: 8,
      timestamp: NOW - 86400000,
      status: "completed",
    },
    {
      id: "conv-4",
      sessionId: "session-101",
      agentName: "Code Helper",
      lastMessage: "Starting code refactoring process...",
      messageCount: 42,
      timestamp: NOW - 172800000,
      status: "active",
    },
  ];

  const formatTimestamp = (timestamp: number) => {
    const now = NOW;
    const diff = now - timestamp;
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (hours < 1) return "Just now";
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto max-w-7xl px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                <FolderOpen className="h-4 w-4 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-xl font-bold">Conversations</h1>
                <p className="text-xs text-muted-foreground">
                  Browse and manage your chat history
                </p>
              </div>
            </div>
          </div>

          {/* Search and Filters */}
          <div className="mt-4 flex gap-3">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search conversations..."
                className="pl-10"
              />
            </div>
            <Button variant="outline">All Agents</Button>
            <Button variant="outline">All Status</Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="mx-auto max-w-7xl p-6">
        <div className="space-y-4">
          {conversations.map((conversation) => (
            <Card key={conversation.id} className="group relative overflow-hidden transition-all hover:shadow-md">
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
                        <Bot className="h-4 w-4" />
                      </div>
                      <div>
                        <CardTitle className="text-lg">{conversation.agentName}</CardTitle>
                        <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                          <Clock className="h-3.5 w-3.5" />
                          {formatTimestamp(conversation.timestamp)}
                          <span>•</span>
                          <span>{conversation.messageCount} messages</span>
                          <span>•</span>
                          <code className="rounded bg-muted px-1">{conversation.sessionId}</code>
                        </div>
                      </div>
                    </div>
                    <CardDescription className="mt-3 line-clamp-2">
                      {conversation.lastMessage}
                    </CardDescription>
                  </div>
                  <Badge
                    variant={conversation.status === "active" ? "default" : "secondary"}
                    className="ml-4"
                  >
                    {conversation.status}
                  </Badge>
                </div>
              </CardHeader>

              <CardContent>
                <div className="flex gap-2">
                  <Link to={`/chat/${conversation.sessionId}`} className="flex-1">
                    <Button variant="outline" size="sm" className="w-full">
                      Open Chat
                    </Button>
                  </Link>
                  <Button variant="outline" size="sm">
                    <Archive className="h-4 w-4" />
                  </Button>
                  <Button variant="outline" size="sm">
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}
