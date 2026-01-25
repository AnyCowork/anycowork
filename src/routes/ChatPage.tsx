/**
 * Chat Page - Real-time conversation with AI agent
 */

import React, { useState, useRef, useEffect } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Send,
  Bot,
  User,
  Loader2,
  Plus,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  X,
  Zap,
  ListTodo,
  Brain,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { listen } from '@tauri-apps/api/event';
import { cn } from "@/lib/utils";
import { useCreateSession, useSessions, useAgents, useDeleteSession, useServerInfo } from "@/lib/hooks/use-anycowork";
import { anycoworkApi, PlanUpdate } from "@/lib/anycowork-api"; // Added
import { A2UIRenderer } from "@/components/a2ui/A2UIRenderer";
import { A2UIMessage } from "@/src/lib/a2ui-processor";
import { useConfirm } from "@/components/ui/confirm-dialog";

// Available AI models (✅ = Tested and verified working)
const AI_MODELS = [
  // Gemini 3 Series (Latest - Most Intelligent)
  { value: "gemini-3-pro-preview", label: "Gemini 3 Pro ⭐", provider: "gemini" },
  { value: "gemini-3-flash-preview", label: "Gemini 3 Flash", provider: "gemini" },

  // Gemini 2.0 Series (Latest)
  { value: "gemini-2.0-flash-exp", label: "Gemini 2.0 Flash Exp ⭐", provider: "gemini" },
  { value: "gemini-2.0-flash", label: "Gemini 2.0 Flash", provider: "gemini" },

  // Gemini 1.5 Series (Stable)
  { value: "gemini-1.5-pro", label: "Gemini 1.5 Pro", provider: "gemini" },
  { value: "gemini-1.5-flash", label: "Gemini 1.5 Flash", provider: "gemini" },

  // Claude Models (Opus 4.5 default, Haiku tested ✅)
  { value: "claude-opus-4.5", label: "Claude Opus 4.5 ⭐", provider: "anthropic" },
  { value: "claude-3-5-sonnet-20241022", label: "Claude 3.5 Sonnet", provider: "anthropic" },
  { value: "claude-3-opus-20240229", label: "Claude 3 Opus", provider: "anthropic" },
  { value: "claude-3-haiku-20240307", label: "Claude 3 Haiku", provider: "anthropic" },

  // OpenAI Models (GPT-5 default, GPT-4 tested ✅)
  { value: "gpt-5", label: "GPT-5 ⭐", provider: "openai" },
  { value: "gpt-5.2", label: "GPT-5.2", provider: "openai" },
  { value: "gpt-5-mini", label: "GPT-5 Mini", provider: "openai" },
  { value: "gpt-4o", label: "GPT-4o", provider: "openai" },
  { value: "gpt-4o-mini", label: "GPT-4o Mini", provider: "openai" },
  { value: "gpt-4-turbo", label: "GPT-4 Turbo", provider: "openai" },
];

interface Message {
  id: string;
  role: "user" | "assistant" | "system" | "model";
  content: string;
  a2uiMessages?: A2UIMessage[];
  message_type?: "normal" | "thinking" | "system";
  timestamp: number;
  step?: ExecutionStep;
}

// Execution job types
interface ExecutionStep {
  id: string;
  tool_name: string;
  tool_args: Record<string, any>;
  status: "pending" | "running" | "waiting_approval" | "approved" | "rejected" | "completed" | "failed";
  result?: string;
  error?: string;
  requires_approval: boolean;
  approval_reason?: string;
  created_at: string;
  completed_at?: string;
}

// Plan/Scratchpad
interface PlanState {
  tasks: {
    id: string;
    description: string;
    status: 'pending' | 'running' | 'completed' | 'failed';
    result?: string;
  }[];
}

interface ExecutionJob {
  id: string;
  session_id: string;
  query: string;
  status: "running" | "waiting_approval" | "completed" | "failed" | "cancelled";
  steps: ExecutionStep[];
  current_step_index: number;
  final_response?: string;
  error?: string;
  created_at: string;
  completed_at?: string;
}

interface ThinkingItemProps {
  content: string;
  isFinished: boolean;
}

function ThinkingItem({ content, isFinished }: ThinkingItemProps) {
  const [isOpen, setIsOpen] = useState(!isFinished);

  // Auto-collapse when finished (with delay)
  useEffect(() => {
    if (isFinished) {
      const timer = setTimeout(() => setIsOpen(false), 800);
      return () => clearTimeout(timer);
    } else {
      setIsOpen(true);
    }
  }, [isFinished]);

  return (
    <div className="rounded-lg border bg-muted/20 overflow-hidden mb-2">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium text-muted-foreground hover:bg-muted/30 transition-colors text-left"
      >
        <Brain className={cn("h-3.5 w-3.5", !isFinished && "animate-pulse text-primary")} />
        <span className="flex-1">Thinking Process</span>
        <div className={cn("transition-transform duration-200 text-muted-foreground/50", isOpen ? "rotate-90" : "")}>
          <ChevronRight className="h-3.5 w-3.5" />
        </div>
      </button>
      {isOpen && (
        <div className="px-3 pb-2 pt-0 animate-in slide-in-from-top-1 duration-200">
          <div className="pl-4 border-l-2 border-primary/20 ml-1.5 my-1">
            <div className="text-xs text-muted-foreground/80 leading-relaxed font-mono whitespace-pre-wrap break-words bg-black/5 dark:bg-white/5 p-2 rounded">
              {content.replace(/^> 🧠 \*\*Thinking\*\*: /, '')}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default function ChatPage() {
  const { sessionId } = useParams();
  const navigate = useNavigate();

  // Hooks
  const { data: sessionsData, refetch: refetchSessions } = useSessions();
  const { data: agents = [] } = useAgents();
  const createSession = useCreateSession();
  const deleteSession = useDeleteSession();
  const { data: serverInfo } = useServerInfo();
  const { confirm, ConfirmDialog } = useConfirm();

  // State
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [selectedAgentId, setSelectedAgentId] = useState<string>("");
  const [selectedModel, setSelectedModel] = useState<string>("gemini-2.0-flash");
  const [executionMode, setExecutionMode] = useState<"planning" | "fast">("planning"); // Default to planning

  // Tab state for multi-session management
  const [openTabs, setOpenTabs] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState<string>("");

  // Refs
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Derived State
  const currentSession = React.useMemo(() => {
    if (!sessionId || !sessionsData?.sessions) return null;
    return sessionsData.sessions.find((s) => s.id === sessionId);
  }, [sessionId, sessionsData]);

  const activeAgent = React.useMemo(() => {
    if (selectedAgentId) return agents.find(a => a.id === selectedAgentId);
    return null;
  }, [selectedAgentId, agents]);

  const selectedModelInfo = React.useMemo(() => {
    return AI_MODELS.find(m => m.value === selectedModel);
  }, [selectedModel]);

  // Effects
  useEffect(() => {
    // Set default agent if none selected and agents are available
    if (agents.length > 0 && !selectedAgentId) {
      // Try to find "AnyCoworker Default" or use first
      const defaultAgent = agents.find(a => a.name.includes("Default")) || agents[0];
      if (defaultAgent) {
        console.log("Auto-selecting default agent:", defaultAgent.name);
        setSelectedAgentId(defaultAgent.id);
      }
    }
  }, [agents, selectedAgentId]);

  // Sync model with active agent
  useEffect(() => {
    if (activeAgent?.ai_config?.model) {
      // Check if model exists in our list to be safe, otherwise default might stay or handled elsewhere
      // But assuming agent config is valid or we just want to reflect it
      setSelectedModel(activeAgent.ai_config.model);
    }
  }, [activeAgent]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Load chat history when session changes
  useEffect(() => {
    if (sessionId && sessionId !== "new") {
      // Fetch message history from API
      const loadMessages = async () => {
        try {
          const loadedMessagesData = await anycoworkApi.getSessionMessages(sessionId);
          if (loadedMessagesData) {
            const loadedMessages: Message[] = loadedMessagesData.map((msg: any) => {
              // Parse A2UI from content if present
              let content = msg.content;
              if (msg.role === 'system' || msg.role === 'tool') {
                console.log('Raw system/tool message:', { id: msg.id, role: msg.role, content: msg.content, metadata: msg.metadata_json });
              }
              let a2uiMessages: A2UIMessage[] | undefined = msg.a2ui_messages;
              let step: ExecutionStep | undefined = undefined;

              // 1. Try to restore ExecutionStep from System messages (Tool Results)
              // Note: Backend currently saves tool results as 'user' (for some reason), so we check that too.
              if (msg.role === 'system' || msg.role === 'tool' || (msg.role === 'user' && content.startsWith("Tool '"))) {

                let toolName = "";
                let toolArgs = {};
                let resultStr = "";
                let isToolMessage = false;

                // A. Try parsing from Metadata (preferred)
                if (msg.metadata_json) {
                  try {
                    const metadata = JSON.parse(msg.metadata_json);

                    // If metadata exists, it might be the args.
                    // Parse tool name from content (reliable prefix)
                    const nameMatch = content.match(/^Tool '([^']+)' result:/);
                    if (nameMatch) {
                      toolName = nameMatch[1];
                      toolArgs = metadata; // The metadata IS the args object

                      // Extract result string: "Tool 'name' result: <RESULT>"
                      const prefix = `Tool '${toolName}' result: `;
                      if (content.startsWith(prefix)) {
                        resultStr = content.substring(prefix.length);
                      } else {
                        // Fallback if prefix matches loosely
                        const resMatch = content.match(/^Tool '[^']+' result: ([\s\S]*)$/);
                        if (resMatch) {
                          resultStr = resMatch[1];
                        } else {
                          resultStr = content;
                        }
                      }

                      isToolMessage = true;
                    }
                  } catch (e) {
                    // Metadata parsing failed
                  }
                }

                // B. Fallback to Regex (Legacy or if metadata strategy failed)
                if (!isToolMessage) {
                  const toolResultMatch = content.match(/^Tool '([^']+)' result: ([\s\S]*)$/);
                  if (toolResultMatch) {
                    toolName = toolResultMatch[1];
                    resultStr = toolResultMatch[2];
                    isToolMessage = true;

                    try {
                      if (msg.metadata_json) {
                        toolArgs = JSON.parse(msg.metadata_json);
                      }
                    } catch (e) { /* ignore */ }
                  }
                }

                if (isToolMessage) {
                  step = {
                    id: `step-${msg.id}`,
                    tool_name: toolName,
                    tool_args: toolArgs,
                    status: 'completed',
                    result: resultStr,
                    requires_approval: false,
                    created_at: msg.created_at
                  };

                  // Force role to system for UI consistency if it was user or tool
                  msg.role = 'system';

                  // Update content to just the result part so downstream parsers (A2UI) work on the output
                  content = resultStr;
                }
              }

              // 2. Hide raw JSON tool calls from Assistant (Request)
              // The backend saves the raw JSON request `{ "tool": "...", "args": ... }` as an assistant message.
              // Since we render the *Result* (above) as a full step (which shows input/output), we don't need this raw texts.
              if (msg.role === 'assistant') {
                try {
                  const trimmed = content.trim();
                  if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
                    const parsed = JSON.parse(trimmed);
                    if (parsed.tool && parsed.args) {
                      // This is a tool call request. Hide it.
                      return null;
                    }
                  }
                } catch (e) { /* Not JSON */ }
              }

              if (content && content.includes('---a2ui_JSON---')) {
                const parts = content.split('---a2ui_JSON---');
                const textPart = parts[0].trim();
                const jsonPart = parts[1]?.trim();

                if (jsonPart) {
                  try {
                    // Clean JSON (remove markdown code blocks if present)
                    const cleanedJson = jsonPart
                      .replace(/^```json\s*/, '')
                      .replace(/\s*```$/, '')
                      .trim();

                    if (cleanedJson && cleanedJson !== '[]') {
                      a2uiMessages = JSON.parse(cleanedJson);
                      content = textPart; // Use only text part if A2UI was parsed
                    } else {
                      content = textPart; // Remove empty A2UI delimiter
                    }
                  } catch (e) {
                    console.error('Failed to parse A2UI from message:', e);
                    // Keep original content if parsing fails
                  }
                }
              }

              return {
                id: msg.id,
                role: msg.role,
                content: content,
                a2uiMessages: a2uiMessages,
                step: step,
                timestamp: new Date(msg.created_at).getTime() // Convert string/timestamp to number
              };
            }).filter(Boolean) as Message[]; // Filter out nulls (hidden messages)

            setMessages(loadedMessages);
          }
        } catch (error) {
          console.error("Error loading message history:", error);
        }
      };
      loadMessages();
    } else {
      // Clear messages for new session or when no session selected (after deletion)
      setMessages([]);
    }
  }, [sessionId]);

  // Handle New Session Creation
  const hasCreatedSession = useRef(false);

  useEffect(() => {
    if (sessionId === "new" && !isLoading && selectedAgentId && !hasCreatedSession.current) {
      hasCreatedSession.current = true;
      setIsLoading(true);

      // Create session with selected agent ID
      const agentId = selectedAgentId;

      createSession.mutate(
        agentId,
        {
          onSuccess: (newSession) => {
            navigate(`/chat/${newSession.id}`, { replace: true });
            setIsLoading(false);
            refetchSessions();
          },
          onError: () => {
            setIsLoading(false);
            hasCreatedSession.current = false; // Reset on error so user can retry
            navigate('/chat'); // Fallback
          },
        }
      );
    }

    // Reset flag when leaving "new" session
    if (sessionId !== "new") {
      hasCreatedSession.current = false;
    }
  }, [sessionId, selectedAgentId, selectedModel, selectedModelInfo]);

  // Tab Management: Load from localStorage on mount
  useEffect(() => {
    const savedTabs = localStorage.getItem('chatTabs');
    if (savedTabs) {
      try {
        const tabs = JSON.parse(savedTabs) as string[];
        // Ensure unique tabs
        const uniqueTabs = Array.from(new Set(tabs));
        setOpenTabs(uniqueTabs);
      } catch (e) {
        console.error('Failed to parse saved tabs:', e);
      }
    }
  }, []);

  // Tab Management: Save to localStorage when tabs change
  useEffect(() => {
    if (openTabs.length > 0) {
      // Ensure unique tabs before saving
      const uniqueTabs = Array.from(new Set(openTabs));
      localStorage.setItem('chatTabs', JSON.stringify(uniqueTabs));
    }
  }, [openTabs]);

  // Tab Management: Sync URL with active tab
  useEffect(() => {
    if (sessionId && sessionId !== 'new') {
      setActiveTab(sessionId);
      // Use functional update to avoid race conditions
      setOpenTabs(prev => {
        if (prev.includes(sessionId)) {
          return prev;
        }
        return [sessionId, ...prev];
      });
    }
  }, [sessionId]);

  // Tab Management: Save active tab to localStorage
  useEffect(() => {
    if (activeTab) {
      localStorage.setItem('chatActiveTab', activeTab);
    }
  }, [activeTab]);

  // Auto-load latest chat or restored active tab when returning to /chat without session ID
  useEffect(() => {
    if (!sessionId && sessionsData?.sessions && sessionsData.sessions.length > 0) {
      // 1. Try to restore last active tab
      const lastActiveTab = localStorage.getItem('chatActiveTab');
      const lastActiveSession = sessionsData.sessions.find(s => s.id === lastActiveTab);

      if (lastActiveTab && lastActiveSession) {
        navigate(`/chat/${lastActiveTab}`, { replace: true });
        return;
      }

      // 2. Fallback: Sort sessions by updated_at (most recent first)
      const sortedSessions = [...sessionsData.sessions].sort((a, b) => {
        const dateA = new Date(a.updated_at || a.created_at).getTime();
        const dateB = new Date(b.updated_at || b.created_at).getTime();
        return dateB - dateA;
      });

      // Navigate to the most recent session
      const latestSession = sortedSessions[0];
      if (latestSession) {
        navigate(`/chat/${latestSession.id}`, { replace: true });
      }
    }
  }, [sessionId, sessionsData, navigate]);

  // Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const ctrlOrCmd = isMac ? e.metaKey : e.ctrlKey;

      // Ctrl/Cmd + T: New chat
      if (ctrlOrCmd && e.key === 't') {
        e.preventDefault();
        handleNewChat();
        return;
      }

      // Ctrl/Cmd + W: Close current tab
      if (ctrlOrCmd && e.key === 'w' && activeTab) {
        e.preventDefault();
        const fakeEvent = { stopPropagation: () => { } } as React.MouseEvent;
        closeTab(fakeEvent, activeTab);
        return;
      }

      // Ctrl/Cmd + Tab: Next tab
      if (ctrlOrCmd && e.key === 'Tab' && !e.shiftKey && openTabs.length > 0) {
        e.preventDefault();
        const currentIndex = openTabs.indexOf(activeTab);
        const nextIndex = (currentIndex + 1) % openTabs.length;
        if (openTabs[nextIndex]) {
          openTab(openTabs[nextIndex]);
        }
        return;
      }

      // Ctrl/Cmd + Shift + Tab: Previous tab
      if (ctrlOrCmd && e.key === 'Tab' && e.shiftKey && openTabs.length > 0) {
        e.preventDefault();
        const currentIndex = openTabs.indexOf(activeTab);
        const prevIndex = currentIndex === 0 ? openTabs.length - 1 : currentIndex - 1;
        if (openTabs[prevIndex]) {
          openTab(openTabs[prevIndex]);
        }
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeTab, openTabs]);

  // Tab Management: Open tab function
  const openTab = (sessionIdToOpen: string) => {
    // Use functional update to ensure no duplicates
    setOpenTabs(prev => {
      if (prev.includes(sessionIdToOpen)) {
        return prev;
      }
      return [sessionIdToOpen, ...prev];
    });
    setActiveTab(sessionIdToOpen);
    navigate(`/chat/${sessionIdToOpen}`);
  };

  // Tab Management: Close tab function
  const closeTab = (e: React.MouseEvent, sessionIdToClose: string) => {
    e.stopPropagation();
    const newTabs = openTabs.filter(id => id !== sessionIdToClose);
    setOpenTabs(newTabs);

    // If closing active tab, switch to last remaining tab
    if (activeTab === sessionIdToClose && newTabs.length > 0) {
      const newActive = newTabs[newTabs.length - 1];
      setActiveTab(newActive);
      navigate(`/chat/${newActive}`);
    } else if (newTabs.length === 0) {
      navigate('/chat');
    }
  };

  // Tab Management: Get session title
  const getSessionTitle = (sessionIdForTitle: string) => {
    const session = sessionsData?.sessions?.find(s => s.id === sessionIdForTitle);
    if (session?.title) {
      return session.title;
    }
    return "New Chat";
  };


  // Create new chat handler
  const handleNewChat = () => {
    navigate("/chat/new");
  };

  const handleDeleteSession = async (e: React.MouseEvent, id: string) => {
    e.preventDefault();
    e.stopPropagation();
    const confirmed = await confirm("Delete this conversation?", {
      title: "Delete Conversation",
      variant: "destructive",
    });
    if (confirmed) {
      deleteSession.mutate(id, {
        onSuccess: () => {
          if (sessionId === id) navigate("/chat");
          refetchSessions();
        }
      });
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const messageContent = input.trim();
    if (!messageContent) return;

    // If no session, create one first
    if (!sessionId || sessionId === 'new') {
      const agentId = selectedAgentId || agents[0]?.id;
      if (!agentId) {
        console.error("No agent available to create session");
        return;
      }

      setIsLoading(true);
      createSession.mutate(agentId, {
        onSuccess: async (newSession) => {
          // Navigate to the new session
          navigate(`/chat/${newSession.id}`, { replace: true });
          // Send message immediately with the new session ID
          await sendMessage(messageContent, undefined, false, newSession.id, executionMode);
        },
        onError: (error) => {
          console.error("Failed to create session:", error);
          setIsLoading(false);
        }
      });
    } else {
      await sendMessage(messageContent, undefined, false, sessionId, executionMode);
    }
  };

  // State for streaming thinking messages
  const [thinkingMessage, setThinkingMessage] = useState<string | null>(null);
  const [reasoning, setReasoning] = useState<string>("");
  const [isReasoningOpen, setIsReasoningOpen] = useState(false);

  // Auto-expand reasoning when it starts
  useEffect(() => {
    if (reasoning && !isReasoningOpen && thinkingMessage === "Thinking...") {
      setIsReasoningOpen(true);
    }
  }, [reasoning, thinkingMessage]);

  // State for execution job tracking
  const [currentJob, setCurrentJob] = useState<ExecutionJob | null>(null);
  const [pendingApproval, setPendingApproval] = useState<ExecutionStep | null>(null);
  const [activePlan, setActivePlan] = useState<PlanState | null>(null);
  const [isPlanOpen, setIsPlanOpen] = useState(true);

  // Listen for Tauri events
  useEffect(() => {
    let unlisten: () => void;

    async function startListening() {
      if (!sessionId || sessionId === 'new') return;

      console.log(`Starting listener for session:${sessionId}`);
      unlisten = await listen(`session:${sessionId}`, (event: any) => {
        const payload = event.payload;
        console.log("Event received:", payload);

        // Handle Event Types
        if (payload.type === 'token') {
          // Handle streaming text
          setMessages((prev) => {
            const lastMsg = prev[prev.length - 1];

            // Highlight errors
            let contentToAdd = payload.content;
            if (typeof contentToAdd === 'string' && (contentToAdd.startsWith("Error:") || contentToAdd.includes("ProviderError"))) {
              // Format as a markdown alert for visibility
              contentToAdd = `\n\n> [!CAUTION]\n> **AI Provider Error**\n> ${contentToAdd}\n\n`;
            }

            if (lastMsg && lastMsg.role === 'assistant' && String(lastMsg.id).startsWith('streaming-')) {
              // Append to existing assistant message
              return [
                ...prev.slice(0, -1),
                { ...lastMsg, content: lastMsg.content + contentToAdd }
              ];
            } else {
              // Create new assistant message
              return [
                ...prev,
                {
                  id: `streaming-${Date.now()}`,
                  role: 'assistant',
                  content: contentToAdd,
                  timestamp: Date.now(),
                }
              ];
            }
          });
        } else if (payload.type === 'job_started') {
          setCurrentJob(payload.job);
          setThinkingMessage("Starting job...");
        } else if (payload.type === 'permission_request') {
          const req = payload.request;
          // Map PermissionRequest to ExecutionStep for UI compatibility
          const step: ExecutionStep = {
            id: req.id,
            tool_name: req.metadata && req.metadata.command ? "bash" : (req.metadata && req.metadata.operation ? "filesystem" : "System"),
            tool_args: req.metadata || {},
            status: "waiting_approval",
            requires_approval: true,
            created_at: new Date().toISOString()
          };
          setPendingApproval(step);
          if (req.message) {
            setThinkingMessage(req.message);
          }
          // If no message, keep the previous "thinking" message (e.g. "Analyzing Cargo.toml...")
        } else if (payload.type === 'thinking') {
          // Show thinking in status bar
          setThinkingMessage(payload.message);

          // ALSO add to chat history for debugging per user request, preventing spam
          setMessages(prev => {
            const lastMsg = prev[prev.length - 1];
            if (lastMsg && String(lastMsg.id).startsWith('thinking-')) {
              // Append to existing thinking message
              return [
                ...prev.slice(0, -1),
                { ...lastMsg, content: lastMsg.content + payload.message }
              ];
            } else {
              return [
                ...prev,
                {
                  id: `thinking-${Date.now()}`,
                  role: 'system', // Use system role for visibility
                  content: `> 🧠 **Thinking**: ${payload.message}`,
                  timestamp: Date.now(),
                }
              ];
            }
          });
        } else if (payload.type === 'step_started') {
          // Show that a step is starting
          const step = payload.step;
          let operationDesc = `Executing ${step.tool_name}`;

          // Add operation details if available
          if (step.tool_name === 'bash' && step.tool_args.command) {
            operationDesc = `Running: \`${step.tool_args.command}\``;
          } else if (step.tool_name === 'filesystem') {
            const op = step.tool_args.operation || 'operation';
            const path = step.tool_args.path || step.tool_args.directory || '';
            operationDesc = `${op} ${path}`;
          } else if (step.tool_name === 'search' || step.tool_name === 'grep_search') {
            const pattern = step.tool_args.pattern || step.tool_args.query || '';
            operationDesc = `Searching for: ${pattern}`;
          }

          setThinkingMessage(operationDesc);
        } else if (payload.type === 'approval_required') {
          // payload.step matches ExecutionStep interface
          setPendingApproval(payload.step);
          // Don't overwrite thinking message with generic "Waiting..."
          // setThinkingMessage("Waiting for approval...");
        } else if (payload.type === 'step_approved') {
          setPendingApproval(null);

          // Helper to get consistent detailed name
          const getStepDetails = (step: ExecutionStep) => {
            if (step.tool_name === "filesystem" || (step.tool_args && step.tool_args.operation)) {
              const op = (step.tool_args.operation || "access").toUpperCase();
              const path = step.tool_args.path || step.tool_args.directory || "";
              return { label: `Filesystem ${op}`, details: path };
            }
            if (step.tool_name === "bash" || step.tool_name === "run_command" || step.tool_args.command) {
              return { label: "Execute Command", details: step.tool_args.command || "unknown" };
            }
            // Generic handling for other tools
            // Check for common "args" patterns or just dump the first string arg
            const keys = Object.keys(step.tool_args).filter(k => k !== 'cwd' && k !== 'env'); // Ignore boilerplate args
            let details = "";
            if (keys.length > 0) {
              // Try to find a meaningful value to show
              const firstVal = step.tool_args[keys[0]];
              if (typeof firstVal === 'string') details = firstVal;
              else details = JSON.stringify(firstVal).slice(0, 50); // Truncate
            }
            return { label: step.tool_name, details: details };
          };

          const approvedMsg: Message = {
            id: `approved-${payload.step.id}`,
            role: "system",
            content: "", // Content rendered via step prop
            step: payload.step,
            timestamp: Date.now(),
          };
          setMessages(prev => [...prev, approvedMsg]);
        } else if (payload.type === 'step_rejected') {
          setPendingApproval(null);

          // Helper duplicated intentionally for stability in this localized scope if needed, 
          // but we can just inline the check since we use it once here.
          // Actually, let's reuse the logic pattern.
          const getStepDetails = (step: ExecutionStep) => {
            if (step.tool_name === "filesystem" || (step.tool_args && step.tool_args.operation)) {
              const op = (step.tool_args.operation || "access").toUpperCase();
              const path = step.tool_args.path || step.tool_args.directory || "";
              return { label: `Filesystem ${op}`, details: path };
            }
            if (step.tool_name === "bash" || step.tool_name === "run_command" || step.tool_args.command) {
              return { label: "Execute Command", details: step.tool_args.command || "unknown" };
            }
            const keys = Object.keys(step.tool_args).filter(k => k !== 'cwd' && k !== 'env');
            let details = "";
            if (keys.length > 0) {
              const firstVal = step.tool_args[keys[0]];
              if (typeof firstVal === 'string') details = firstVal;
              else details = JSON.stringify(firstVal).slice(0, 50);
            }
            return { label: step.tool_name, details: details };
          };

          const rejectedMsg: Message = {
            id: `rejected-${payload.step.id}`,
            role: "system",
            content: `✗ Rejected **${getStepDetails(payload.step).label}**` + (getStepDetails(payload.step).details ? `: \`${getStepDetails(payload.step).details}\`` : ''),
            timestamp: Date.now(),
          };
          setMessages(prev => [...prev, rejectedMsg]);
        } else if (payload.type === 'plan_update') {
          // Sync plan state
          setActivePlan(payload.plan as PlanUpdate);
        } else if (payload.type === 'step_completed') {
          // Show tool output
          setPendingApproval(null); // Clear approval state
          const step = payload.step;

          // Create a structured step object for custom rendering
          const toolMsg: Message = {
            id: `step-${step.id}`,
            role: "system",
            content: "", // Will be rendered via step property
            step: step,
            timestamp: Date.now(),
          };

          let a2uiMessages: A2UIMessage[] | undefined;

          // Attempt to parse result for file listings and clean output
          try {
            let isFileList = false;
            let fileList: string[] = [];
            let displayContent = step.result || "";

            // Try parsing JSON first (handles both filesystem arrays and bash objects)
            let parsed: any = null;
            try {
              // Only try parsing if it looks like JSON/object to avoid parsing simple strings oddly
              if (displayContent.trim().startsWith('{') || displayContent.trim().startsWith('[')) {
                parsed = JSON.parse(displayContent);
              }
            } catch (e) { /* ignore */ }

            // 1. Handle Bash/Command JSON wrapper { stdout: "...", exit_code: 0 }
            if ((step.tool_name === "bash" || step.tool_name === "run_command" || step.tool_name === "grep_search") && parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
              if (parsed.stdout !== undefined) {
                displayContent = parsed.stdout;
              } else if (parsed.stderr) {
                displayContent = parsed.stderr;
              }
            }

            // 2. Handle Filesystem Array (from list_dir etc)
            if ((step.tool_name === "filesystem" || step.tool_name === "list_dir" || step.tool_name === "list_files") && Array.isArray(parsed) && parsed.every(s => typeof s === 'string')) {
              isFileList = true;
              fileList = parsed;
            }

            // 3. Detect file list in stdout (e.g. from 'find' command)
            if (!isFileList && displayContent) {
              const lines = displayContent.split('\n').map(l => l.trim()).filter(l => l.length > 0);
              // Heuristic: If we have multiple lines and they look like paths
              if (lines.length > 0 && lines.length < 100) {
                const looksLikePath = lines.every(l => l.includes('/') || l.includes('\\') || l.startsWith('./') || l.includes('.') || l.match(/^[a-zA-Z0-9._-]+$/));
                if (looksLikePath && (lines.length > 1 || lines[0].includes('/'))) {
                  isFileList = true;
                  fileList = lines;
                }
              }
            }

            // 4. Render - check for file lists for A2UI
            if (isFileList && fileList.length > 0) {
              a2uiMessages = createFileListA2UI(fileList, step.tool_name);
            }
          } catch (e) {
            // Ignore parsing errors
          }

          // Attach a2ui messages if we have them
          if (a2uiMessages) {
            toolMsg.a2uiMessages = a2uiMessages;
          }

          setMessages(prev => [...prev, toolMsg]);
        } else if (payload.type === 'job_completed') {
          setCurrentJob(null);
          setPendingApproval(null); // Clear approval state
          setThinkingMessage(null);
          setIsLoading(false); // Fix: Clear loading state so "Thinking..." disappears

          // If we were streaming, the "final" message is just the accumulated stream.
          // We can just update the ID of the last streaming message to mark it as final/stable.
          setMessages((prev) => {
            const lastMsg = prev[prev.length - 1];
            if (lastMsg && lastMsg.role === 'assistant' && String(lastMsg.id).startsWith('streaming-')) {
              return [
                ...prev.slice(0, -1),
                { ...lastMsg, id: `final-${payload.job.id}`, content: payload.message } // Ensure content matches exactly what backend sent
              ];
            } else {
              // Fallback if no stream happened
              const finalMsg: Message = {
                id: `final-${payload.job.id}`,
                role: "assistant",
                content: payload.message,
                timestamp: Date.now(),
              };
              return [...prev, finalMsg];
            }
          });
        }
      });
    }

    startListening();

    return () => {
      if (unlisten) unlisten();
    };
  }, [sessionId]);

  const sendMessage = async (messageContent: string, actionContext?: any, skipUserMessage?: boolean, explicitSessionId?: string, executionMode?: string) => {
    const targetSessionId = explicitSessionId || sessionId;
    if (!messageContent || !targetSessionId) return;

    // Validate that we have an agent selected
    const agentId = selectedAgentId || agents[0]?.id;
    if (!agentId) {
      console.error("No agent selected");
      const errorMessage: Message = {
        id: Date.now().toString(),
        role: "system",
        content: "Error: No agent selected. Please select an agent from the dropdown above.",
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMessage]);
      return;
    }

    if (!skipUserMessage) {
      const userMessage: Message = {
        id: Date.now().toString(),
        role: "user",
        content: messageContent,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, userMessage]);
    }

    setInput("");
    setIsLoading(true); // Will be reset when job_completed event fires? OR we manage it differently.
    // Actually, for "chat" command, it returns "started" immediately.
    // So we shouldn't block UI "loading" state strictly on the promise.

    try {
      // This returns "started"
      await anycoworkApi.sendMessage(targetSessionId, messageContent);

    } catch (error) {
      console.error("Error sending message:", error);
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: "system",
        content: `Error: ${error}`,
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMessage]);
      setIsLoading(false);
    }
  };

  const handleApprove = async () => {
    if (!pendingApproval) return;
    try {
      await anycoworkApi.approveAction(pendingApproval.id);
      // UI update will happen on 'step_approved' event
    } catch (e) {
      console.error("Failed to approve:", e);
    }
  };

  const handleReject = async () => {
    if (!pendingApproval) return;
    try {
      await anycoworkApi.rejectAction(pendingApproval.id);
    } catch (e) {
      console.error("Failed to reject:", e);
    }
  };


  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      handleSubmit(e);
    }
  };

  /**
   * Handle A2UI action triggers from interactive components
   */
  const handleA2UIAction = async (actionName: string, actionContext: any[]) => {
    console.log('A2UI Action triggered:', actionName, actionContext);

    // Convert action context array to object
    const contextObj: Record<string, any> = {};
    if (actionContext && Array.isArray(actionContext)) {
      for (const item of actionContext) {
        if (item.key && item.value) {
          // Extract value based on type
          if (item.value.literalString !== undefined) {
            contextObj[item.key] = item.value.literalString;
          } else if (item.value.literalNumber !== undefined) {
            contextObj[item.key] = item.value.literalNumber;
          } else if (item.value.literalBoolean !== undefined) {
            contextObj[item.key] = item.value.literalBoolean;
          } else if (item.value.path !== undefined) {
            // For path-based values, we'd need to resolve from data model
            // For now, just store the path
            contextObj[item.key] = item.value.path;
          }
        }
      }
    }

    // Handle special actions
    if (actionName === 'download_file') {
      // Handle file download
      const filePath = contextObj.filePath || contextObj.file_path;
      if (filePath) {
        console.warn("File download not yet implemented for Tauri:", filePath);
        const errorMessage: Message = {
          id: Date.now().toString(),
          role: "system",
          content: `⚠ File download not supported in this version yet. Path: ${filePath}`,
          timestamp: Date.now(),
        };
        setMessages((prev) => [...prev, errorMessage]);
      }
      return; // Don't send to agent
    }

    // Format action name for display (convert snake_case to Title Case)
    const formatActionName = (name: string) => {
      return name
        .split('_')
        .map(word => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
    };

    // Create a user message that represents the action
    const actionDisplayMessage: Message = {
      id: Date.now().toString(),
      role: "user",
      content: `🎯 ${formatActionName(actionName)}`,
      timestamp: Date.now(),
    };

    setMessages((prev) => [...prev, actionDisplayMessage]);

    // Send the action to the agent (skip adding another user message)
    await sendMessage(`[Action: ${actionName}]`, {
      action: actionName,
      ...contextObj
    }, true); // skipUserMessage = true
  };

  /**
   * Helper to create A2UI message for file lists (Minimal Layout)
   */
  const createFileListA2UI = (files: string[], toolName: string): A2UIMessage[] => {
    const surfaceId = `files-${Date.now()}`;
    const fileItems = files.map(f => ({
      name: f,
      type: f.includes('.') ? 'file' : 'folder',
      emoji: f.includes('.') ? '📄' : '📁',
      path: f
    }));

    return [
      {
        beginRendering: {
          surfaceId,
          root: 'root',
        }
      },
      {
        dataModelUpdate: {
          surfaceId,
          contents: [
            {
              key: 'files',
              valueList: fileItems.map(f => [
                { key: 'name', valueString: f.name },
                { key: 'emoji', valueString: f.emoji },
                { key: 'path', valueString: f.path }
              ])
            },
            { key: 'title', valueString: `Files (${files.length})` }
          ]
        }
      },
      {
        surfaceUpdate: {
          surfaceId,
          components: [
            {
              id: 'root',
              component: {
                Column: {
                  children: { explicitList: ['header', 'list'] }
                }
              }
            },
            {
              id: 'header',
              component: {
                Column: {
                  children: { explicitList: ['title-row', 'divider'] }
                }
              }
            },
            {
              id: 'title-row',
              component: {
                Row: {
                  children: { explicitList: ['icon-main', 'title'] },
                  distribution: 'start'
                }
              }
            },
            {
              id: 'icon-main',
              component: {
                Icon: { icon: 'home', size: 'small' }
              }
            },
            {
              id: 'title',
              component: {
                Text: {
                  text: { path: 'title' },
                  usageHint: 'h4'
                }
              }
            },
            {
              id: 'divider',
              component: {
                Divider: {}
              }
            },
            {
              id: 'list',
              component: {
                List: {
                  children: {
                    template: {
                      componentId: 'file-row',
                      dataBinding: '/files'
                    }
                  }
                }
              }
            },
            {
              id: 'file-row',
              component: {
                Row: {
                  children: { explicitList: ['icon', 'name'] },
                  distribution: 'start'
                }
              }
            },
            {
              id: 'icon',
              component: {
                Text: {
                  text: { path: 'emoji' }
                }
              }
            },
            {
              id: 'name',
              component: {
                Text: {
                  text: { path: 'name' }
                }
              }
            }
          ]
        }
      }
    ];
  };

  const renderApprovalContent = (step: ExecutionStep) => {
    let label = step.tool_name;
    let details = "";

    // 1. Filesystem
    if (step.tool_name === "filesystem" || (step.tool_args && step.tool_args.operation)) {
      label = `Filesystem ${(step.tool_args.operation || "access").toUpperCase()}`;
      details = step.tool_args.path || step.tool_args.directory || "unknown path";
    }
    // 2. Command
    else if (step.tool_name === "bash" || step.tool_name === "run_command" || step.tool_args.command) {
      label = "Execute Command";
      details = step.tool_args.command || "unknown command";
    }
    // 3. Generic Fallback
    else {
      // Try to find reasonable details
      const keys = Object.keys(step.tool_args).filter(k => k !== 'cwd' && k !== 'env');
      if (keys.length > 0) {
        const firstVal = step.tool_args[keys[0]];
        if (typeof firstVal === 'string') details = firstVal;
        else if (typeof firstVal === 'number') details = String(firstVal);
        else details = JSON.stringify(step.tool_args);
      }
    }

    return (
      <div className="flex flex-col">
        <span className="font-semibold text-amber-800 dark:text-amber-200 flex items-center gap-2 text-sm">
          {label}
        </span>
        {details && (
          <span className="text-xs text-amber-700 dark:text-amber-300 font-mono mt-1 break-all bg-black/5 dark:bg-white/10 px-1.5 py-0.5 rounded w-fit max-h-[100px] overflow-y-auto">
            {details}
          </span>
        )}
      </div>
    );
  };

  return (
    <>
      <ConfirmDialog />
      <div className="flex flex-col h-full overflow-hidden">
        {/* Tab Bar */}
        <div className="border-b bg-background/50 backdrop-blur-sm px-2 py-1.5 flex items-center gap-2">
          <Tabs value={activeTab} className="flex-1 overflow-hidden">
            <div className="flex items-center gap-2">
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8 shrink-0 hover:bg-muted/50 text-muted-foreground transition-colors rounded-lg"
                    title="History"
                  >
                    <Clock className="h-4 w-4" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-80 p-0" align="start">
                  <div className="flex items-center px-4 py-2 border-b bg-muted/20">
                    <span className="font-semibold text-xs">Recent Chats</span>
                  </div>
                  <ScrollArea className="h-[300px]">
                    <div className="p-1 space-y-1">
                      {sessionsData?.sessions?.length === 0 && (
                        <div className="text-xs text-muted-foreground text-center py-8">
                          No history found
                        </div>
                      )}
                      {sessionsData?.sessions?.slice().sort((a, b) => b.updated_at - a.updated_at).map(session => (
                        <Button
                          key={session.id}
                          variant="ghost"
                          className={cn(
                            "w-full justify-start text-left h-auto py-2 px-3 text-sm font-normal",
                            activeTab === session.id ? "bg-accent/50 text-accent-foreground" : ""
                          )}
                          onClick={() => {
                            openTab(session.id);
                          }}
                        >
                          <div className="flex flex-col gap-0.5 overflow-hidden">
                            <span className="truncate font-medium">{session.title || session.agent_config?.name || 'Untitled Chat'}</span>
                            <span className="text-[10px] text-muted-foreground truncate">
                              {new Date(session.updated_at * 1000).toLocaleDateString()} • {session.id.slice(0, 8)}
                            </span>
                          </div>
                        </Button>
                      ))}
                    </div>
                  </ScrollArea>
                </PopoverContent>
              </Popover>

              <Button
                onClick={handleNewChat}
                size="sm"
                variant="ghost"
                className="h-8 px-4 gap-2 shrink-0 bg-muted/30 hover:bg-muted text-muted-foreground hover:text-foreground transition-all rounded-t-lg rounded-b-none border-b border-transparent min-w-[100px] justify-start"
              >
                <Plus className="h-4 w-4" />
                <span className="text-sm font-medium">New Chat</span>
              </Button>

              <div className="w-[1px] h-4 bg-border/50 mx-1" /> {/* Divider */}

              <div className="flex-1 overflow-x-auto overflow-y-hidden [&::-webkit-scrollbar]:hidden -mb-px">
                <TabsList className="h-9 bg-transparent p-0 gap-1 inline-flex items-end">
                  {openTabs.map(tabId => (
                    <TabsTrigger
                      key={tabId}
                      value={tabId}
                      onClick={() => openTab(tabId)}
                      className={cn(
                        "group relative h-8 px-3 py-1.5 text-xs font-medium rounded-t-lg rounded-b-none border border-transparent transition-all duration-200",
                        // Inactive state
                        "hover:bg-muted/50 text-muted-foreground hover:text-foreground",
                        // Active State styling (Seamless Chrome-like)
                        "data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:font-semibold",
                        "data-[state=active]:border-border data-[state=active]:border-b-transparent",
                        "data-[state=active]:shadow-none data-[state=active]:z-10",
                        "min-w-[120px] max-w-[200px]",
                        "flex items-center gap-2 justify-between"
                      )}
                    >
                      <span className="truncate flex-1 text-left">{getSessionTitle(tabId)}</span>
                      <div
                        role="button"
                        tabIndex={0}
                        className="h-4 w-4 p-0 opacity-0 group-hover:opacity-100 hover:bg-destructive/20 transition-opacity shrink-0 inline-flex items-center justify-center rounded-md"
                        onClick={(e) => closeTab(e, tabId)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            closeTab(e as any, tabId);
                          }
                        }}
                      >
                        <X className="h-3 w-3" />
                      </div>
                    </TabsTrigger>
                  ))}
                </TabsList>
              </div>

            </div>
          </Tabs>
        </div>

        <div className="flex-1 flex flex-row overflow-hidden relative">
          {/* Chat Area */}
          <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-6 space-y-6">
              {(!sessionId || sessionId === 'new') ? (
                <div className="h-full flex flex-col items-center justify-center text-center space-y-6 max-w-2xl mx-auto">
                  <div className="flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-primary to-primary/80">
                    <Bot className="h-10 w-10 text-primary-foreground" />
                  </div>
                  <div className="space-y-2">
                    <h2 className="text-3xl font-bold tracking-tight">Welcome to AnyCowork</h2>
                    <p className="text-lg text-muted-foreground">Your AI coworker is ready to assist you</p>
                  </div>
                  <div className="text-sm text-muted-foreground space-y-2">
                    <p>Type your message below to start a new conversation.</p>
                    <p className="text-xs">A new session will be created automatically when you send your first message.</p>
                  </div>
                </div>
              ) : messages.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-center space-y-4 text-muted-foreground">
                  <Avatar className="h-16 w-16 bg-muted/50">
                    <AvatarFallback><Bot className="h-8 w-8" /></AvatarFallback>
                  </Avatar>
                  <div>
                    <h3 className="font-semibold text-lg">How can I help you?</h3>
                    <p className="text-sm">Select an agent and start chatting.</p>
                  </div>
                </div>
              ) : null}

              {messages.map((message) => (
                <div
                  key={message.id}
                  className={cn(
                    "flex gap-4 max-w-full",
                    message.role === "user" && "flex-row-reverse",
                    (message.role === "system" || message.message_type === "thinking") && "pl-8 gap-2"
                  )}
                >
                  {/* Hide avatar for system/thinking messages */}
                  {message.role !== "system" && message.message_type !== "thinking" && (
                    <Avatar className="h-8 w-8 shrink-0">
                      <AvatarFallback
                        className={cn(
                          message.role === "user"
                            ? "bg-primary text-primary-foreground"
                            : "bg-muted"
                        )}
                      >
                        {message.role === "user" ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
                      </AvatarFallback>
                    </Avatar>
                  )}

                  <div
                    className={cn(
                      "flex-1 space-y-2 max-w-full overflow-hidden",
                      message.role === "user" && "flex flex-col items-end"
                    )}
                  >
                    <div
                      className={cn(
                        "inline-block",
                        message.role === "system" || message.message_type === "thinking"
                          ? message.step
                            ? "" // No background wrapper for tool execution messages
                            : "px-3 py-1.5 text-xs text-muted-foreground bg-muted/40 border border-border/30 rounded-lg shadow-sm"
                          : message.role === "user"
                            ? "px-4 py-3 bg-accent text-accent-foreground border border-accent-foreground/10 rounded-2xl rounded-br-md break-words shadow-sm"
                            : "px-4 py-3 bg-secondary text-secondary-foreground border border-border/50 rounded-2xl rounded-bl-md max-w-full overflow-hidden break-words shadow-sm"
                      )}
                    >
                      {/* Show text content only if:
                        1. There's no A2UI AND no Step (Action History)
                        2. Or there's meaningful text content
                    */}
                      {message.step ? (
                        // Compact Tool Execution Display - No background wrapper
                        <div className="flex flex-col gap-0 max-w-2xl">
                          {/* Compact Header - Always Visible */}
                          <div className="flex items-center justify-between px-3 py-2 rounded-lg bg-muted/20 hover:bg-muted/30 transition-colors">
                            <div className="flex items-center gap-2 flex-1 min-w-0">
                              <div className={cn(
                                "h-6 w-6 rounded-md flex items-center justify-center shrink-0",
                                message.step.status === 'failed' || message.id.startsWith('rejected')
                                  ? "bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400"
                                  : "bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-400"
                              )}>
                                {message.step.status === 'failed' || message.id.startsWith('rejected')
                                  ? <XCircle className="h-4 w-4" />
                                  : <CheckCircle className="h-4 w-4" />}
                              </div>
                              <div className="flex-1 min-w-0">
                                <div className="text-xs font-semibold text-foreground/90 truncate">
                                  {message.step.tool_name === "filesystem" ? "📁 Filesystem" :
                                    message.step.tool_name === "bash" ? "⚡ Command" :
                                      message.step.tool_name === "search" ? "🔍 Search" :
                                        "🔧 " + message.step.tool_name.replace(/_/g, ' ')}
                                </div>
                                <div className="text-[10px] text-muted-foreground truncate">
                                  {(() => {
                                    // Show operation summary
                                    const args = message.step.tool_args;
                                    if (message.step.tool_name === "bash" && args.command) {
                                      return args.command.substring(0, 60) + (args.command.length > 60 ? '...' : '');
                                    } else if (message.step.tool_name === "filesystem" && args.operation) {
                                      return `${args.operation} ${args.path || args.directory || ''}`.substring(0, 60);
                                    } else if (message.step.tool_name === "search") {
                                      return `${args.pattern || args.query || ''}`.substring(0, 60);
                                    }
                                    return Object.keys(args).slice(0, 2).join(', ');
                                  })()}
                                </div>
                              </div>
                            </div>
                            <Badge variant="outline" className="text-[10px] h-5 px-1.5 font-mono opacity-60 shrink-0">
                              {new Date(message.timestamp).toLocaleTimeString()}
                            </Badge>
                          </div>

                          {/* Collapsible Details */}
                          <details className="group">
                            <summary className="flex items-center gap-2 cursor-pointer text-xs font-medium text-muted-foreground hover:text-foreground transition-colors px-3 py-2 hover:bg-muted/10 rounded-lg">
                              <div className="transition-transform group-open:rotate-90">
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                              </div>
                              <span>Show Details</span>
                            </summary>

                            <div className="px-3 py-2 space-y-2 text-xs">
                              {/* Request Section - Collapsible */}
                              {message.step.tool_args && Object.keys(message.step.tool_args).length > 0 && (
                                <details className="group/req">
                                  <summary className="flex items-center gap-2 cursor-pointer font-medium text-muted-foreground hover:text-foreground py-1">
                                    <div className="transition-transform group-open/req:rotate-90">
                                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="9 18 15 12 9 6"></polyline></svg>
                                    </div>
                                    <span>Request Parameters</span>
                                  </summary>
                                  <div className="mt-1 ml-4 bg-muted/20 rounded-md p-2">
                                    <pre className="text-[10px] font-mono text-muted-foreground whitespace-pre-wrap break-words max-h-40 overflow-y-auto">
                                      {JSON.stringify(message.step.tool_args, null, 2)}
                                    </pre>
                                  </div>
                                </details>
                              )}

                              {/* Output Section - Collapsible */}
                              {message.step.result && (
                                <details className="group/out" open={message.a2uiMessages && message.a2uiMessages.length > 0}>
                                  <summary className="flex items-center gap-2 cursor-pointer font-medium text-muted-foreground hover:text-foreground py-1">
                                    <div className="transition-transform group-open/out:rotate-90">
                                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="9 18 15 12 9 6"></polyline></svg>
                                    </div>
                                    <span>Output</span>
                                    <Badge variant="secondary" className="text-[9px] h-4 px-1">
                                      {message.step.result.length > 1000 ? `${(message.step.result.length / 1000).toFixed(1)}k chars` : `${message.step.result.length} chars`}
                                    </Badge>
                                  </summary>
                                  <div className="mt-1 ml-4">
                                    {message.a2uiMessages && message.a2uiMessages.length > 0 ? (
                                      <A2UIRenderer
                                        messages={message.a2uiMessages}
                                        onAction={handleA2UIAction}
                                        variant="minimal"
                                      />
                                    ) : (
                                      <div className="bg-muted/20 rounded-md p-2">
                                        <pre className="text-[10px] font-mono text-muted-foreground whitespace-pre-wrap break-words max-h-60 overflow-y-auto">
                                          {(() => {
                                            try {
                                              const parsed = JSON.parse(message.step.result);
                                              return JSON.stringify(parsed, null, 2);
                                            } catch {
                                              return message.step.result;
                                            }
                                          })()}
                                        </pre>
                                      </div>
                                    )}
                                  </div>
                                </details>
                              )}

                              {/* Error Section */}
                              {message.step.error && (
                                <div className="bg-red-50/50 dark:bg-red-950/20 rounded-md p-2">
                                  <div className="text-[10px] font-semibold text-red-600 dark:text-red-400 mb-1 flex items-center gap-1">
                                    <AlertTriangle className="h-3 w-3" />
                                    Error
                                  </div>
                                  <pre className="text-[10px] font-mono text-red-500 dark:text-red-400 whitespace-pre-wrap break-words max-h-40 overflow-y-auto">
                                    {message.step.error}
                                  </pre>
                                </div>
                              )}
                            </div>
                          </details>
                        </div>
                      ) : message.message_type === "thinking" ? (
                        <ThinkingItem
                          content={message.content}
                          // Use index from map callback if available, otherwise just check last
                          isFinished={message.id !== messages[messages.length - 1].id || !isLoading}
                        />
                      ) : (
                        message.content && (
                          !message.a2uiMessages ||
                          message.a2uiMessages.length === 0 ||
                          (message.content.trim() && message.content.length < 200)
                        ) && (
                          <ReactMarkdown
                            className={cn(
                              "prose dark:prose-invert max-w-none text-sm break-words",
                              message.role === "user" ? "prose-p:text-accent-foreground" : "prose-p:text-secondary-foreground"
                            )}
                            remarkPlugins={[remarkGfm]}
                            components={{
                              pre: ({ node, ...props }) => (
                                <div className="overflow-x-auto w-full my-2 rounded-lg bg-black/10 dark:bg-black/30 p-2">
                                  <pre className="whitespace-pre-wrap break-words text-xs" {...props} />
                                </div>
                              ),
                              code: ({ node, inline, ...props }: any) =>
                                inline ? (
                                  <code className="bg-black/10 dark:bg-black/30 rounded px-1 py-0.5 text-xs break-all" {...props} />
                                ) : (
                                  <code className="block whitespace-pre-wrap break-words text-xs" {...props} />
                                )
                            }}
                          >
                            {message.content}
                          </ReactMarkdown>
                        )
                      )}
                      {/* Only render A2UI separately if not part of a step (already rendered in step details) */}
                      {message.a2uiMessages && message.a2uiMessages.length > 0 && !message.step && (
                        <div className={cn(
                          "w-full",
                          message.content && message.content.trim() && message.content.length < 200 ? "mt-4" : ""
                        )}>
                          <A2UIRenderer
                            messages={message.a2uiMessages}
                            onAction={handleA2UIAction}
                            variant="minimal"
                          />
                        </div>
                      )}
                    </div>
                    {/* Only show timestamp for non-system messages */}
                    {message.role !== "system" && message.message_type !== "thinking" && (
                      <p className="text-[10px] text-muted-foreground">
                        {message.timestamp ? new Date(message.timestamp).toLocaleTimeString() : ''}
                      </p>
                    )}
                  </div>
                </div>
              ))}

              {/* Approval Required UI - Just the action buttons */}
              {/* Approval Required UI */}
              {pendingApproval && (
                <div className="flex gap-4 animate-in fade-in slide-in-from-bottom-2 duration-300 pl-2">
                  <Avatar className="h-8 w-8 shrink-0">
                    <AvatarFallback className="bg-amber-100 dark:bg-amber-900 border border-amber-200 dark:border-amber-700">
                      <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400" />
                    </AvatarFallback>
                  </Avatar>
                  <div className="flex flex-col gap-3 rounded-xl border border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/50 px-4 py-3 shadow-sm max-w-[80%] min-w-[300px]">
                    <div className="flex-1 min-w-0">
                      {renderApprovalContent(pendingApproval)}
                      {thinkingMessage && (
                        <div className="mt-3 flex items-start gap-2 text-xs text-muted-foreground bg-amber-100/50 dark:bg-amber-900/20 p-2 rounded-lg border border-amber-200/50 dark:border-amber-800/50">
                          <Bot className="h-3.5 w-3.5 mt-0.5 shrink-0 opacity-70" />
                          <span className="italic">"{thinkingMessage}"</span>
                        </div>
                      )}
                    </div>
                    <div className="flex items-center justify-end gap-2 mt-1 pt-2 border-t border-amber-200/50 dark:border-amber-800/50">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={handleReject}
                        className="gap-1 h-8 px-3 text-red-600 border-red-200 hover:bg-red-50 hover:text-red-700 dark:text-red-400 dark:border-red-900 dark:hover:bg-red-950"
                      >
                        <XCircle className="h-3.5 w-3.5" /> Deny
                      </Button>
                      <Button
                        size="sm"
                        onClick={handleApprove}
                        className="gap-1 bg-green-600 hover:bg-green-700 text-white h-8 px-3 shadow-sm"
                      >
                        <CheckCircle className="h-3.5 w-3.5" /> Approve
                      </Button>
                    </div>
                  </div>
                </div>
              )}

              {/* Execution Progress */}
              {currentJob && !pendingApproval && currentJob.steps.length > 0 && (
                <div className="flex gap-4">
                  <Avatar className="h-8 w-8 shrink-0">
                    <AvatarFallback className="bg-muted"><Bot className="h-4 w-4" /></AvatarFallback>
                  </Avatar>
                  <div className="flex-1 max-w-md">
                    <div className="rounded-lg border bg-muted/30 p-3 space-y-2">
                      <div className="flex items-center gap-2 text-sm font-medium">
                        <Loader2 className="h-4 w-4 animate-spin" />
                        Executing...
                      </div>
                      <div className="space-y-1">
                        {currentJob.steps.map((step) => (
                          <div key={step.id} className="flex items-center gap-2 text-xs">
                            {step.status === "completed" ? (
                              <CheckCircle className="h-3 w-3 text-green-500" />
                            ) : step.status === "running" || step.status === "approved" ? (
                              <Loader2 className="h-3 w-3 animate-spin text-blue-500" />
                            ) : step.status === "waiting_approval" ? (
                              <Clock className="h-3 w-3 text-amber-500" />
                            ) : step.status === "rejected" || step.status === "failed" ? (
                              <XCircle className="h-3 w-3 text-red-500" />
                            ) : (
                              <Clock className="h-3 w-3 text-muted-foreground" />
                            )}
                            <span className={cn(
                              step.status === "completed" && "text-muted-foreground",
                              step.status === "rejected" || step.status === "failed" && "text-red-500"
                            )}>
                              {step.tool_name}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Simple Loading (when no job tracking) */}
              {isLoading && !currentJob && !pendingApproval && (
                <div className="flex gap-4">
                  <Avatar className="h-8 w-8 shrink-0">
                    <AvatarFallback className="bg-muted"><Bot className="h-4 w-4" /></AvatarFallback>
                  </Avatar>
                  <div className="flex flex-col gap-2 max-w-full">
                    <div className="flex items-center gap-2 rounded-lg bg-muted px-4 py-2 w-fit">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span className="text-sm text-muted-foreground">
                        {thinkingMessage || "Thinking..."}
                      </span>
                    </div>

                    {reasoning && (
                      <div className="rounded-lg border bg-muted/30 max-w-2xl overflow-hidden">
                        <div
                          className="flex items-center gap-2 px-3 py-2 bg-muted/50 cursor-pointer hover:bg-muted/70 text-xs font-medium text-muted-foreground"
                          onClick={() => setIsReasoningOpen(!isReasoningOpen)}
                        >
                          {isReasoningOpen ? (
                            <div className="h-0 w-0 border-l-[4px] border-l-transparent border-t-[6px] border-t-muted-foreground border-r-[4px] border-r-transparent transform" />
                          ) : (
                            <div className="h-0 w-0 border-t-[4px] border-t-transparent border-l-[6px] border-l-muted-foreground border-b-[4px] border-b-transparent transform" />
                          )}
                          Thinking Process
                        </div>
                        {isReasoningOpen && (
                          <div className="p-3 bg-black/5 dark:bg-black/20">
                            <pre className="text-xs text-muted-foreground whitespace-pre-wrap break-words font-mono">
                              {reasoning}
                            </pre>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>

            {/* Input Area - Redesigned with inline mode and model selection */}
            <div className="p-4 bg-background/95 backdrop-blur border-t supports-[backdrop-filter]:bg-background/60">
              <form onSubmit={handleSubmit} className="mx-auto max-w-3xl">
                {/* Main input with send button */}
                <div className="relative">
                  <Textarea
                    ref={textareaRef}
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        handleSubmit(e);
                      }
                    }}
                    placeholder="Ask anything (Ctrl+L), @ to mention, / for workflows"
                    className="min-h-[80px] max-h-[200px] resize-none pr-12 pb-12 rounded-xl border-muted-foreground/20"
                    disabled={isLoading}
                    data-testid="chat-input"
                  />
                  <Button
                    type="submit"
                    size="icon"
                    disabled={!input.trim() || isLoading}
                    className="absolute bottom-3 right-3 h-9 w-9 rounded-lg"
                  >
                    {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Send className="h-4 w-4" />}
                  </Button>
                </div>

                {/* Mode and Model Selection - Inline at bottom of input */}
                <div className="flex items-center gap-2 mt-2 px-1">
                  {/* Mode Selector */}
                  <div className="flex items-center gap-1">
                    <button
                      type="button"
                      onClick={() => setExecutionMode(executionMode === "planning" ? "fast" : "planning")}
                      className={cn(
                        "group flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium transition-all",
                        "hover:bg-muted border border-transparent",
                        executionMode === "planning"
                          ? "bg-muted/60 text-foreground"
                          : "text-muted-foreground hover:text-foreground"
                      )}
                    >
                      <div className={cn(
                        "transition-transform",
                        executionMode === "planning" ? "" : "rotate-90"
                      )}>
                        {executionMode === "planning" ? (
                          <ListTodo className="h-3.5 w-3.5" />
                        ) : (
                          <Zap className="h-3.5 w-3.5" />
                        )}
                      </div>
                      <span className="capitalize">{executionMode}</span>
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="opacity-50">
                        <polyline points="6 9 12 15 18 9"></polyline>
                      </svg>
                    </button>
                  </div>

                  {/* Model Selector */}
                  <Select value={selectedModel} onValueChange={setSelectedModel}>
                    <SelectTrigger className={cn(
                      "h-8 w-auto gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium",
                      "border-transparent hover:bg-muted bg-muted/60",
                      "focus:ring-0 shadow-none"
                    )}>
                      <div className="flex items-center gap-1.5">
                        <span className="text-muted-foreground text-xs">
                          {selectedModelInfo?.provider === 'gemini' ? '✨' :
                            selectedModelInfo?.provider === 'anthropic' ? '🤖' : '🧠'}
                        </span>
                        <SelectValue placeholder="Select Model" />
                      </div>
                    </SelectTrigger>
                    <SelectContent align="start" className="max-h-80">
                      <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                        AI Models
                      </div>
                      {AI_MODELS.map((model) => (
                        <SelectItem key={model.value} value={model.value} className="text-xs">
                          <div className="flex items-center gap-2">
                            <span className="opacity-60">
                              {model.provider === 'gemini' ? '✨' :
                                model.provider === 'anthropic' ? '🤖' : '🧠'}
                            </span>
                            {model.label}
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  {/* Agent Info - Right aligned */}
                  <div className="ml-auto flex items-center gap-1.5 text-xs text-muted-foreground">
                    <Bot className="h-3 w-3" />
                    <span className="max-w-[150px] truncate">
                      {activeAgent?.name || 'Default Agent'}
                    </span>
                  </div>
                </div>
              </form>
            </div>
          </div>

          {/* Right Sidebar: Plan / Context */}
          {activePlan && (
            <div className={cn(
              "border-l border-border bg-card transition-all duration-300 flex flex-col h-full",
              isPlanOpen ? "w-80 p-4" : "w-12 py-4 items-center"
            )}>
              <div className="flex items-center justify-between mb-4">
                {isPlanOpen && (
                  <h3 className="font-semibold text-sm uppercase tracking-wider text-muted-foreground flex items-center gap-2">
                    <Clock className="w-4 h-4" /> Agent Plan
                  </h3>
                )}
                <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setIsPlanOpen(!isPlanOpen)}>
                  {isPlanOpen ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
                </Button>
              </div>

              {isPlanOpen && (
                <div className="flex flex-col gap-2 overflow-y-auto">
                  {activePlan.tasks.map((task, idx) => (
                    <div key={task.id || idx} className={cn(
                      "p-3 rounded-lg border text-sm",
                      task.status === 'running' ? "bg-accent border-accent animate-pulse" :
                        task.status === 'completed' ? "bg-green-500/10 border-green-500/20" :
                          task.status === 'failed' ? "bg-red-500/10 border-red-500/20" :
                            "bg-background/50 border-border opacity-70"
                    )}>
                      <div className="flex items-start gap-2">
                        <div className="mt-0.5">
                          {task.status === 'completed' ? <CheckCircle className="w-4 h-4 text-green-500" /> :
                            task.status === 'failed' ? <XCircle className="w-4 h-4 text-red-500" /> :
                              task.status === 'running' ? <Loader2 className="w-4 h-4 text-primary animate-spin" /> :
                                <div className="w-4 h-4 rounded-full border-2 border-muted" />}
                        </div>
                        <div className="flex-1 space-y-1">
                          <p className={cn("font-medium leading-none", task.status === 'completed' && "line-through text-muted-foreground")}>
                            {task.description}
                          </p>
                          <span className="text-xs text-muted-foreground capitalize">{task.status}</span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

