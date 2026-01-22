import React, { useState, useRef, useEffect } from "react";
import { useRAGChat } from "@/hooks/useRag";
import {
  Loader2,
  Send,
  Bot,
  User,
  Sparkles,
  Trash2,
  RotateCw,
  Cpu,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { useConfirm } from "@/components/ui/confirm-dialog";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";

interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  backend?: string;
  sources?: Array<{
    page_id: string;
    snippet?: string;
    score: number;
  }>;
}

export function ChatWidget() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [currentModel, setCurrentModel] = useState<string>("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const navigate = useNavigate();
  const chatMutation = useRAGChat();
  const { confirm, ConfirmDialog } = useConfirm();

  // Fetch current model info on mount
  useEffect(() => {
    fetch("/api/rag/stats")
      .then((res) => res.json())
      .then((data) => {
        if (data.backend) {
          setCurrentModel(data.backend);
        }
      })
      .catch((err) => console.error("Failed to fetch model info:", err));
  }, []);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || chatMutation.isPending) return;

    const userMessage: Message = { role: "user", content: input };
    setMessages((prev) => [...prev, userMessage]);
    setInput("");

    try {
      const response = await chatMutation.mutateAsync({
        question: input,
        mode: "hybrid",
      });

      const assistantMessage: Message = {
        role: "assistant",
        content: response.answer,
        backend: response.backend,
        sources: response.sources.map((s) => ({
          page_id: s.page_id,
          snippet: s.snippet,
          score: s.score,
        })),
      };

      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error: any) {
      console.error("Chat error:", error);

      // Extract error message
      const errorText =
        error?.message || "Sorry, I encountered an error. Please try again.";

      const errorMessage: Message = {
        role: "assistant",
        content: `⚠️ **Error**: ${errorText}`,
      };
      setMessages((prev) => [...prev, errorMessage]);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleNewChat = () => {
    setMessages([]);
    setInput("");
  };

  const handleClear = async () => {
    const confirmed = await confirm(
      "Are you sure you want to clear this conversation?",
      {
        title: "Clear Conversation",
        variant: "destructive",
      }
    );
    if (confirmed) {
      setMessages([]);
    }
  };

  return (
    <>
      <ConfirmDialog />
      <div className="flex flex-col h-full">
        {/* Header */}
      <div className="flex-shrink-0 border-b px-8 py-4">
        <div className="max-w-4xl mx-auto flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-primary/10">
              <Sparkles className="w-5 h-5 text-primary" />
            </div>
            <div>
              <h1 className="text-xl font-semibold">Chat with Your Notes</h1>
              <p className="text-sm text-muted-foreground flex items-center gap-2">
                Ask questions and get AI-powered answers from your knowledge
                base
                {currentModel && (
                  <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-muted text-xs">
                    <Cpu className="w-3 h-3" />
                    {currentModel}
                  </span>
                )}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleNewChat}
              title="New conversation"
            >
              <RotateCw className="w-4 h-4 mr-2" />
              New Chat
            </Button>
            {messages.length > 0 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={handleClear}
                title="Clear conversation"
              >
                <Trash2 className="w-4 h-4 mr-2" />
                Clear
              </Button>
            )}
          </div>
        </div>
      </div>

      {/* Messages Area - Scrollable */}
      <div className="flex-1 overflow-y-auto px-8 py-6">
        <div className="max-w-4xl mx-auto space-y-6">
          {messages.length === 0 && (
            <div className="flex flex-col items-center justify-center text-center py-12">
              <div className="flex items-center justify-center w-16 h-16 rounded-full bg-muted mb-4">
                <Bot className="w-8 h-8 text-muted-foreground" />
              </div>
              <h2 className="text-lg font-semibold mb-2">
                Start a Conversation
              </h2>
              <p className="text-muted-foreground max-w-sm mb-6">
                Ask me anything about your notes and I'll search through them to
                provide helpful answers
              </p>
              <div className="grid gap-2 w-full max-w-md text-left">
                <div className="text-xs font-medium text-muted-foreground mb-1">
                  Try asking:
                </div>
                <button
                  onClick={() => setInput("What did I write about today?")}
                  className="p-3 text-sm text-left rounded-lg border bg-card hover:bg-accent transition-colors"
                >
                  "What did I write about today?"
                </button>
                <button
                  onClick={() => setInput("Summarize my recent notes")}
                  className="p-3 text-sm text-left rounded-lg border bg-card hover:bg-accent transition-colors"
                >
                  "Summarize my recent notes"
                </button>
                <button
                  onClick={() => setInput("What are my TODO items?")}
                  className="p-3 text-sm text-left rounded-lg border bg-card hover:bg-accent transition-colors"
                >
                  "What are my TODO items?"
                </button>
              </div>
            </div>
          )}

          {messages.map((msg, i) => (
            <div
              key={i}
              className={`flex gap-4 ${
                msg.role === "user" ? "justify-end" : "justify-start"
              }`}
            >
              {msg.role === "assistant" && (
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center mt-1">
                  <Bot className="w-4 h-4 text-primary" />
                </div>
              )}

              <div
                className={`flex flex-col max-w-[75%] ${msg.role === "user" ? "items-end" : "items-start"}`}
              >
                <div
                  className={`rounded-lg px-4 py-3 ${
                    msg.role === "user"
                      ? "bg-primary text-primary-foreground"
                      : "bg-muted"
                  }`}
                >
                  {msg.role === "user" ? (
                    <p className="text-sm whitespace-pre-wrap">{msg.content}</p>
                  ) : (
                    <div className="prose prose-sm dark:prose-invert max-w-none">
                      <ReactMarkdown
                        remarkPlugins={[remarkGfm]}
                        rehypePlugins={[rehypeRaw]}
                        components={{
                          code({
                            node,
                            inline,
                            className,
                            children,
                            ...props
                          }: any) {
                            const match = /language-(\w+)/.exec(
                              className || ""
                            );
                            return !inline && match ? (
                              <SyntaxHighlighter
                                style={oneDark}
                                language={match[1]}
                                PreTag="div"
                                className="rounded-md text-xs"
                                {...props}
                              >
                                {String(children).replace(/\n$/, "")}
                              </SyntaxHighlighter>
                            ) : (
                              <code
                                className="bg-muted px-1.5 py-0.5 rounded text-xs font-mono"
                                {...props}
                              >
                                {children}
                              </code>
                            );
                          },
                          p: ({ children }) => (
                            <p className="mb-2 last:mb-0">{children}</p>
                          ),
                          ul: ({ children }) => (
                            <ul className="list-disc list-inside mb-2 space-y-1">
                              {children}
                            </ul>
                          ),
                          ol: ({ children }) => (
                            <ol className="list-decimal list-inside mb-2 space-y-1">
                              {children}
                            </ol>
                          ),
                          li: ({ children }) => (
                            <li className="text-sm">{children}</li>
                          ),
                          h1: ({ children }) => (
                            <h1 className="text-lg font-bold mb-2 mt-3">
                              {children}
                            </h1>
                          ),
                          h2: ({ children }) => (
                            <h2 className="text-base font-bold mb-2 mt-3">
                              {children}
                            </h2>
                          ),
                          h3: ({ children }) => (
                            <h3 className="text-sm font-bold mb-1 mt-2">
                              {children}
                            </h3>
                          ),
                          blockquote: ({ children }) => (
                            <blockquote className="border-l-4 border-primary/30 pl-3 italic my-2">
                              {children}
                            </blockquote>
                          ),
                          table: ({ children }) => (
                            <div className="overflow-x-auto my-2">
                              <table className="min-w-full divide-y divide-border text-xs">
                                {children}
                              </table>
                            </div>
                          ),
                          th: ({ children }) => (
                            <th className="px-3 py-2 text-left font-semibold bg-muted">
                              {children}
                            </th>
                          ),
                          td: ({ children }) => (
                            <td className="px-3 py-2 border-t border-border">
                              {children}
                            </td>
                          ),
                        }}
                      >
                        {msg.content}
                      </ReactMarkdown>
                    </div>
                  )}
                </div>

                {/* Model badge for assistant messages */}
                {msg.role === "assistant" && msg.backend && (
                  <div className="mt-1 px-2 py-0.5 rounded-full bg-muted/50 text-xs text-muted-foreground flex items-center gap-1">
                    <Cpu className="w-3 h-3" />
                    {msg.backend}
                  </div>
                )}

                {/* Sources */}
                {msg.sources && msg.sources.length > 0 && (
                  <div className="mt-3 space-y-2 w-full">
                    <p className="text-xs font-medium text-muted-foreground">
                      📚 Sources:
                    </p>
                    {msg.sources.slice(0, 3).map((source, idx) => (
                      <button
                        key={idx}
                        onClick={() => navigate(`/documents/${source.page_id}`)}
                        className="w-full text-left text-xs bg-card rounded-lg border p-3 hover:bg-accent transition-colors"
                      >
                        <p className="line-clamp-2 mb-1.5">{source.snippet}</p>
                        <p className="text-muted-foreground">
                          Relevance: {(source.score * 100).toFixed(0)}%
                        </p>
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {msg.role === "user" && (
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-muted flex items-center justify-center mt-1">
                  <User className="w-4 h-4" />
                </div>
              )}
            </div>
          ))}

          {chatMutation.isPending && (
            <div className="flex gap-4 justify-start">
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center mt-1">
                <Bot className="w-4 h-4 text-primary" />
              </div>
              <div className="bg-muted rounded-lg px-4 py-3">
                <div className="flex items-center gap-2">
                  <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
                  <span className="text-sm text-muted-foreground">
                    Thinking...
                  </span>
                </div>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>
      </div>

      {/* Input Area - Fixed at Bottom */}
      <div className="flex-shrink-0 border-t px-8 py-4 bg-background">
        <div className="max-w-4xl mx-auto">
          <div className="flex gap-3 items-start">
            <div className="flex-1">
              <textarea
                ref={textareaRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Ask about your notes..."
                className="w-full resize-none rounded-lg border bg-background px-4 py-3 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                rows={3}
                disabled={chatMutation.isPending}
              />
              <p className="text-xs text-muted-foreground mt-2 px-1">
                Press Enter to send, Shift+Enter for new line • Supports
                Markdown
              </p>
            </div>
            <button
              onClick={handleSend}
              disabled={chatMutation.isPending || !input.trim()}
              className="flex-shrink-0 h-12 w-12 rounded-full bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center transition-colors mt-0.5"
              title="Send message"
            >
              {chatMutation.isPending ? (
                <Loader2 className="w-5 h-5 animate-spin" />
              ) : (
                <Send className="w-5 h-5" />
              )}
            </button>
          </div>
        </div>
      </div>
      </div>
    </>
  );
}
