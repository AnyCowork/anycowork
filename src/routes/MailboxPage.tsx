/**
 * MailboxPage - Email client UI with account switching
 * Outlook/Gmail-style layout: folder sidebar, thread list, reading pane
 */

import { useState, useMemo } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Inbox,
  Send,
  Archive,
  PenSquare,
  Mail,
  MailOpen,
  User,
} from "lucide-react";
import {
  useMailThreads,
  useMailMessages,
  useUnreadMailCount,
  useSendMail,
  useReplyToMail,
  useMarkThreadRead,
  useArchiveThread,
  useAgents,
} from "@/lib/hooks/use-anycowork";
import type { Agent, MailThread } from "@/lib/anycowork-api";

type Folder = "inbox" | "sent" | "archive";

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return "now";
  if (diffMin < 60) return `${diffMin}m`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h`;
  const diffDays = Math.floor(diffHr / 24);
  if (diffDays < 7) return `${diffDays}d`;
  return date.toLocaleDateString();
}

export default function MailboxPage() {
  const [selectedAccountId, setSelectedAccountId] = useState<string | undefined>(undefined);
  const [selectedFolder, setSelectedFolder] = useState<Folder>("inbox");
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null);
  const [isComposeOpen, setIsComposeOpen] = useState(false);
  const [replyText, setReplyText] = useState("");

  // Compose state
  const [composeTo, setComposeTo] = useState<string>("");
  const [composeSubject, setComposeSubject] = useState("");
  const [composeBody, setComposeBody] = useState("");

  const { data: agents = [] } = useAgents();
  const { data: threads = [] } = useMailThreads(
    selectedAccountId,
    selectedFolder === "archive" ? "inbox" : selectedFolder,
    selectedFolder === "archive" ? true : undefined
  );
  const { data: messages = [] } = useMailMessages(selectedThreadId || "");
  const { data: unreadCount = 0 } = useUnreadMailCount(selectedAccountId);

  const sendMail = useSendMail();
  const replyToMail = useReplyToMail();
  const markRead = useMarkThreadRead();
  const archiveThread = useArchiveThread();

  const selectedThread = useMemo(
    () => threads.find((t: MailThread) => t.id === selectedThreadId),
    [threads, selectedThreadId]
  );

  const currentAccountLabel = useMemo(() => {
    if (!selectedAccountId) return "You";
    const agent = agents.find((a: Agent) => a.id === selectedAccountId);
    return agent ? `${agent.avatar || ""} ${agent.name}` : "Unknown";
  }, [selectedAccountId, agents]);

  const handleSelectThread = (threadId: string) => {
    setSelectedThreadId(threadId);
    const thread = threads.find((t: MailThread) => t.id === threadId);
    if (thread && !thread.is_read) {
      markRead.mutate(threadId);
    }
  };

  const handleSendMail = () => {
    if (!composeSubject.trim() || !composeBody.trim() || !composeTo) return;

    const fromAgentId = selectedAccountId || null;
    const toAgentId = composeTo === "user" ? null : composeTo;

    sendMail.mutate(
      { fromAgentId, toAgentId, subject: composeSubject, body: composeBody },
      {
        onSuccess: () => {
          setIsComposeOpen(false);
          setComposeTo("");
          setComposeSubject("");
          setComposeBody("");
        },
      }
    );
  };

  const handleReply = () => {
    if (!replyText.trim() || !selectedThreadId) return;
    const fromAgentId = selectedAccountId || null;
    replyToMail.mutate(
      { threadId: selectedThreadId, fromAgentId, content: replyText },
      { onSuccess: () => setReplyText("") }
    );
  };

  const folders: { key: Folder; label: string; icon: typeof Inbox }[] = [
    { key: "inbox", label: "Inbox", icon: Inbox },
    { key: "sent", label: "Sent", icon: Send },
    { key: "archive", label: "Archive", icon: Archive },
  ];

  return (
    <div className="flex h-full flex-col">
      {/* Top bar */}
      <div className="flex items-center justify-between border-b px-4 py-2.5 bg-background/50">
        <div className="flex items-center gap-3">
          <Mail className="h-5 w-5 text-primary" />
          <h1 className="text-lg font-semibold">Mails</h1>

          {/* Account Switcher */}
          <Select
            value={selectedAccountId || "user"}
            onValueChange={(val) => {
              setSelectedAccountId(val === "user" ? undefined : val);
              setSelectedThreadId(null);
              setSelectedFolder("inbox");
            }}
          >
            <SelectTrigger className="w-48 h-8 text-sm">
              <SelectValue placeholder="Select account" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="user">
                <div className="flex items-center gap-2">
                  <User className="h-3.5 w-3.5" />
                  <span>You</span>
                </div>
              </SelectItem>
              {agents.map((agent: Agent) => (
                <SelectItem key={agent.id} value={agent.id}>
                  <div className="flex items-center gap-2">
                    <span>{agent.avatar || ""}</span>
                    <span>{agent.name}</span>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <Button size="sm" onClick={() => setIsComposeOpen(true)}>
          <PenSquare className="h-4 w-4 mr-1.5" />
          Compose
        </Button>
      </div>

      {/* Main layout */}
      <div className="flex flex-1 overflow-hidden">
        {/* Folder sidebar */}
        <div className="w-40 shrink-0 border-r bg-muted/30 p-2 space-y-1">
          {folders.map((f) => (
            <button
              key={f.key}
              onClick={() => {
                setSelectedFolder(f.key);
                setSelectedThreadId(null);
              }}
              className={cn(
                "flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                selectedFolder === f.key
                  ? "bg-primary/10 text-primary"
                  : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
              )}
            >
              <f.icon className="h-4 w-4" />
              <span>{f.label}</span>
              {f.key === "inbox" && unreadCount > 0 && (
                <Badge variant="destructive" className="ml-auto h-5 min-w-[20px] text-[10px] px-1">
                  {unreadCount}
                </Badge>
              )}
            </button>
          ))}
        </div>

        {/* Thread list */}
        <div className="w-80 shrink-0 border-r overflow-hidden flex flex-col">
          <ScrollArea className="flex-1">
            {threads.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
                <MailOpen className="h-10 w-10 mb-3 opacity-40" />
                <p className="text-sm">No emails</p>
              </div>
            ) : (
              <div className="divide-y">
                {threads.map((thread: MailThread) => (
                  <button
                    key={thread.id}
                    onClick={() => handleSelectThread(thread.id)}
                    className={cn(
                      "w-full text-left px-3 py-3 transition-colors hover:bg-muted/50",
                      selectedThreadId === thread.id && "bg-primary/5",
                      !thread.is_read && "bg-primary/[0.02]"
                    )}
                  >
                    <div className="flex items-start gap-2">
                      {!thread.is_read && (
                        <div className="mt-1.5 h-2 w-2 rounded-full bg-primary shrink-0" />
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-2">
                          <span className="flex items-center gap-1.5 text-xs text-muted-foreground truncate">
                            {thread.last_sender_avatar && (
                              <span>{thread.last_sender_avatar}</span>
                            )}
                            {thread.last_sender_name || "Unknown"}
                          </span>
                          <span className="text-[10px] text-muted-foreground shrink-0">
                            {formatRelativeTime(thread.updated_at)}
                          </span>
                        </div>
                        <p className={cn("text-sm truncate mt-0.5", !thread.is_read && "font-semibold")}>
                          {thread.subject}
                        </p>
                        {thread.last_message_preview && (
                          <p className="text-xs text-muted-foreground truncate mt-0.5">
                            {thread.last_message_preview}
                          </p>
                        )}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </ScrollArea>
        </div>

        {/* Reading pane */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {!selectedThread ? (
            <div className="flex flex-1 items-center justify-center text-muted-foreground">
              <div className="text-center">
                <Mail className="h-12 w-12 mx-auto mb-3 opacity-30" />
                <p className="text-sm">Select a thread to read</p>
              </div>
            </div>
          ) : (
            <>
              {/* Thread header */}
              <div className="flex items-center justify-between border-b px-4 py-3">
                <div>
                  <h2 className="text-base font-semibold">{selectedThread.subject}</h2>
                  <p className="text-xs text-muted-foreground">
                    {messages.length} message{messages.length !== 1 ? "s" : ""}
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    archiveThread.mutate(selectedThread.id);
                    setSelectedThreadId(null);
                  }}
                >
                  <Archive className="h-3.5 w-3.5 mr-1.5" />
                  Archive
                </Button>
              </div>

              {/* Messages */}
              <ScrollArea className="flex-1 px-4 py-3">
                <div className="space-y-4 max-w-2xl">
                  {messages.map((msg) => (
                    <div key={msg.id} className="rounded-lg border p-3">
                      <div className="flex items-center gap-2 mb-2">
                        <div className="flex items-center gap-1.5 text-sm font-medium">
                          {msg.sender_avatar && <span>{msg.sender_avatar}</span>}
                          {msg.sender_name || (msg.sender_type === "user" ? "You" : "Agent")}
                        </div>
                        <span className="text-xs text-muted-foreground">
                          {formatRelativeTime(msg.created_at)}
                        </span>
                      </div>
                      <div className="text-sm whitespace-pre-wrap">{msg.content}</div>
                    </div>
                  ))}
                </div>
              </ScrollArea>

              {/* Reply box */}
              <div className="border-t px-4 py-3">
                <div className="flex gap-2 max-w-2xl">
                  <Textarea
                    placeholder={`Reply as ${currentAccountLabel}...`}
                    value={replyText}
                    onChange={(e) => setReplyText(e.target.value)}
                    className="min-h-[60px] resize-none text-sm"
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
                        handleReply();
                      }
                    }}
                  />
                  <Button
                    size="sm"
                    onClick={handleReply}
                    disabled={!replyText.trim() || replyToMail.isPending}
                    className="self-end"
                  >
                    <Send className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Compose Dialog */}
      <Dialog open={isComposeOpen} onOpenChange={setIsComposeOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>Compose Email</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1.5">
              <label className="text-sm font-medium">From</label>
              <div className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm bg-muted/30">
                {currentAccountLabel}
              </div>
            </div>
            <div className="space-y-1.5">
              <label className="text-sm font-medium">To</label>
              <Select value={composeTo} onValueChange={setComposeTo}>
                <SelectTrigger className="text-sm">
                  <SelectValue placeholder="Select recipient..." />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="user">
                    <div className="flex items-center gap-2">
                      <User className="h-3.5 w-3.5" />
                      <span>You (User)</span>
                    </div>
                  </SelectItem>
                  {agents
                    .filter((a: Agent) => a.id !== selectedAccountId)
                    .map((agent: Agent) => (
                      <SelectItem key={agent.id} value={agent.id}>
                        <div className="flex items-center gap-2">
                          <span>{agent.avatar || ""}</span>
                          <span>{agent.name}</span>
                        </div>
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1.5">
              <label className="text-sm font-medium">Subject</label>
              <Input
                value={composeSubject}
                onChange={(e) => setComposeSubject(e.target.value)}
                placeholder="Email subject..."
                className="text-sm"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-sm font-medium">Body</label>
              <Textarea
                value={composeBody}
                onChange={(e) => setComposeBody(e.target.value)}
                placeholder="Write your message..."
                className="min-h-[120px] text-sm"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsComposeOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleSendMail}
              disabled={!composeTo || !composeSubject.trim() || !composeBody.trim() || sendMail.isPending}
            >
              <Send className="h-4 w-4 mr-1.5" />
              Send
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
