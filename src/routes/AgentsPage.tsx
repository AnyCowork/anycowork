/**
 * Agents Management Page - Create and manage AI agents
 * Each agent has its own characteristics, skills, MCP servers, messaging, and configs
 */

import { useState, useEffect } from "react";
import {
  useAgents, useCreateAgent, useUpdateAgent, useDeleteAgent,
} from "@/lib/hooks/use-anycowork";
import { anycoworkApi } from "@/lib/anycowork-api";
import type { Agent as AgentType, AgentCreate as AgentCreateType } from "@/lib/anycowork-api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Bot, Plus, Trash2, Edit, Brain, Sparkles, MessageSquare, Server, Wrench, Key, Database, Shield, ShieldCheck, ShieldAlert, CheckCircle2, AlertCircle, Info, Loader2, Box
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { toast } from "sonner";
import { useQuery } from "@tanstack/react-query";
import { useConfirm } from "@/components/ui/confirm-dialog";

interface AgentFormProps {
  agent?: AgentType;
  onClose: () => void;
}

export default function AgentsPage() {
  const { data: agents = [], isLoading } = useAgents();
  const deleteAgent = useDeleteAgent();
  const { confirm, ConfirmDialog } = useConfirm();

  const [selectedAgent, setSelectedAgent] = useState<AgentType | null>(null);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);

  const handleCreateAgent = () => {
    setIsCreateDialogOpen(true);
  };

  const handleEditAgent = (agent: AgentType) => {
    setSelectedAgent(agent);
    setIsEditDialogOpen(true);
  };

  const handleDeleteAgent = async (agentId: string) => {
    const confirmed = await confirm("Are you sure you want to delete this agent?", {
      title: "Delete Agent",
      variant: "destructive",
    });
    if (confirmed) {
      deleteAgent.mutate(agentId);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <>
      <ConfirmDialog />
      <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
        {/* Header */}
        <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <div className="mx-auto max-w-7xl px-8 py-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                  <Bot className="h-5 w-5 text-primary-foreground" />
                </div>
                <div>
                  <h1 className="text-2xl font-bold">Agents Management</h1>
                  <p className="text-sm text-muted-foreground">
                    Create and configure AI agents with unique characteristics
                  </p>
                </div>
              </div>
              <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
                <DialogTrigger asChild>
                  <Button size="lg" className="gap-2" onClick={handleCreateAgent}>
                    <Plus className="h-4 w-4" />
                    Create Agent
                  </Button>
                </DialogTrigger>
                <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
                  <DialogHeader>
                    <DialogTitle>Create New Agent</DialogTitle>
                    <DialogDescription>
                      Configure your agent with specific characteristics, skills, and connections
                    </DialogDescription>
                  </DialogHeader>
                  <AgentForm onClose={() => setIsCreateDialogOpen(false)} />
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <div className="mx-auto max-w-7xl p-8">
          {agents.length === 0 ? (
            <Card className="border-dashed">
              <CardContent className="flex flex-col items-center justify-center py-12">
                <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
                  <Bot className="h-8 w-8 text-muted-foreground" />
                </div>
                <h3 className="mt-4 text-lg font-semibold">No agents yet</h3>
                <p className="mt-2 text-sm text-muted-foreground">
                  Create your first AI agent to get started
                </p>
                <Button onClick={handleCreateAgent} className="mt-4 gap-2">
                  <Plus className="h-4 w-4" />
                  Create Agent
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {agents.map((agent) => (
                <Card
                  key={agent.id}
                  className="group relative overflow-hidden transition-all hover:shadow-lg hover:border-primary/50"
                >
                  <CardHeader className="p-4 pb-2">
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                          <Bot className="h-5 w-5 text-primary-foreground" />
                        </div>
                        <div>
                          <CardTitle className="text-base font-semibold">{agent.name}</CardTitle>
                          <Badge
                            variant="outline"
                            className={cn(
                              "mt-1 text-[10px] px-1.5 py-0 h-5",
                              agent.status === "active" && "border-green-500 text-green-500",
                              agent.status === "inactive" && "border-slate-500 text-slate-500",
                              agent.status === "error" && "border-red-500 text-red-500"
                            )}
                          >
                            {agent.status}
                          </Badge>
                        </div>
                      </div>
                    </div>
                    <CardDescription className="mt-1 line-clamp-2 text-xs">{agent.description}</CardDescription>
                  </CardHeader>

                  <CardContent className="p-4 pt-2 space-y-3">
                    {/* AI Provider */}
                    <div className="flex items-center gap-2 text-xs">
                      <Brain className="h-3.5 w-3.5 text-muted-foreground" />
                      <span className="text-muted-foreground">Provider:</span>
                      <span className="font-medium capitalize">{agent.ai_config?.provider || "N/A"}</span>
                    </div>

                    {/* Expertise */}
                    <div>
                      <div className="mb-1.5 flex items-center gap-2 text-xs text-muted-foreground">
                        <Sparkles className="h-3.5 w-3.5" />
                        Expertise
                      </div>
                      <div className="flex flex-wrap gap-1">
                        {(agent.characteristics?.expertise || []).length > 0 ? (
                          (agent.characteristics?.expertise || []).slice(0, 3).map((exp) => (
                            <Badge key={exp} variant="secondary" className="text-[10px] px-1.5 py-0 h-5">
                              {exp}
                            </Badge>
                          ))
                        ) : (
                          <span className="text-[10px] text-muted-foreground italic">None defined</span>
                        )}
                        {(agent.characteristics?.expertise || []).length > 3 && (
                          <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-5">+{(agent.characteristics?.expertise || []).length - 3}</Badge>
                        )}
                      </div>
                    </div>

                    <Separator />

                    <div className="grid grid-cols-2 gap-2 text-xs">
                      <div className="flex items-center gap-2">
                        <Wrench className="h-3.5 w-3.5 text-muted-foreground" />
                        <span className="text-muted-foreground">{agent.skills.length} skills</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Server className="h-3.5 w-3.5 text-muted-foreground" />
                        <span className="text-muted-foreground">{agent.mcp_servers.length} MCPs</span>
                      </div>
                    </div>

                    <div className="flex gap-2 pt-1">
                      <Button
                        variant="outline"
                        size="sm"
                        className="flex-1 gap-2 h-8 text-xs"
                        onClick={() => handleEditAgent(agent)}
                      >
                        <Edit className="h-3 w-3" />
                        Edit
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        className="gap-2 h-8 text-xs text-destructive hover:bg-destructive hover:text-destructive-foreground"
                        onClick={() => handleDeleteAgent(agent.id)}
                      >
                        <Trash2 className="h-3 w-3" />
                        Delete
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>

        {/* Edit Agent Dialog */}
        <Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
          <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle>Edit Agent</DialogTitle>
              <DialogDescription>
                Update your agent's configuration and settings
              </DialogDescription>
            </DialogHeader>
            {selectedAgent && (
              <AgentForm
                agent={selectedAgent}
                onClose={() => setIsEditDialogOpen(false)}
              />
            )}
          </DialogContent>
        </Dialog>
      </div>
    </>
  );
}

function AgentForm({ agent, onClose }: AgentFormProps) {
  const createAgent = useCreateAgent();
  const updateAgent = useUpdateAgent();

  // Check if this is the default agent (read-only for skills/mcp)
  const isDefaultAgent = agent?.name === "AnyCoworker Default";

  // Fetch available capabilities
  const { data: allSkills = [] } = useQuery({ queryKey: ['skills'], queryFn: () => anycoworkApi.listSkills() });
  const { data: allMCPServers = [] } = useQuery({ queryKey: ['mcp_servers'], queryFn: () => anycoworkApi.listMCPServers() });

  const defaultFormData = {
    name: "",
    description: "",
    characteristics: {
      personality: "",
      tone: "",
      expertise: [],
    },
    ai_config: {
      provider: "gemini" as any,
      model: "gemini-2.0-flash",
      temperature: 0.7,
      max_tokens: 4096,
    },
    // Default system prompt for Coworker behavior
    system_prompt: `You are an intelligent AI Coworker designed to help with daily office tasks.
Your goal is to be proactive, organized, and helpful. 
You should:
1. Ask clarifying questions when requirements are vague.
2. Build and maintain a todo list for complex tasks.
3. Use available tools to search for information, manage files, and execute code.
4. Report progress regularly.`,
    workflow_type: "sequential" as any,
    skills: [],
    mcp_servers: [],
    messaging_connections: [],
    knowledge_bases: [],
    api_keys: {},
    execution_settings: {
      mode: "require_approval" as const,
      sandbox_mode: "flexible" as const, // "sandbox", "direct", or "flexible"
      whitelisted_commands: [
        "^ls(\\s|$)",
        "^pwd$",
        "^cat\\s",
        "^git\\s+status",
        "^git\\s+log",
        "^git\\s+diff",
      ],
      whitelisted_tools: [
        "file_read",
        "file_list",
        "get_workflow_status",
        "list_workflows",
      ],
      blacklisted_commands: [
        "rm\\s+-rf",
        "sudo\\s",
        "curl.*\\|\\s*(bash|sh)",
      ],
    },
  };

  const [formData, setFormData] = useState<Partial<AgentType>>(() => {
    if (agent) {
      // Ensure ai_config has all required fields when editing
      return {
        ...agent,
        ai_config: {
          provider: agent.ai_config?.provider || "gemini",
          model: agent.ai_config?.model || "gemini-3-flash-preview",
          temperature: agent.ai_config?.temperature ?? 0.7,
          max_tokens: agent.ai_config?.max_tokens ?? 4096,
        },
      };
    }
    return defaultFormData;
  });

  // Update form data when agent prop changes (for edit mode)
  useEffect(() => {
    if (agent) {
      setFormData({
        ...agent,
        ai_config: {
          provider: agent.ai_config?.provider || "gemini",
          model: agent.ai_config?.model || "gemini-3-flash-preview",
          temperature: agent.ai_config?.temperature ?? 0.7,
          max_tokens: agent.ai_config?.max_tokens ?? 4096,
        },
      });
    }
  }, [agent]);

  const handleSave = () => {
    if (agent) {
      updateAgent.mutate(
        { agentId: agent.id, data: formData },
        { onSuccess: onClose }
      );
    } else {
      createAgent.mutate(formData as AgentCreateType, { onSuccess: onClose });
    }
  };

  const toggleSkill = (skillId: string) => {
    setFormData(prev => {
      const current = prev.skills || [];
      if (current.includes(skillId)) {
        return { ...prev, skills: current.filter(id => id !== skillId) };
      } else {
        return { ...prev, skills: [...current, skillId] };
      }
    });
  };

  const toggleMCP = (mcpId: string) => {
    setFormData(prev => {
      const current = prev.mcp_servers || [];
      if (current.includes(mcpId)) {
        return { ...prev, mcp_servers: current.filter(id => id !== mcpId) };
      } else {
        return { ...prev, mcp_servers: [...current, mcpId] };
      }
    });
  };

  const [jsonErrors, setJsonErrors] = useState<Record<string, string>>({});

  const validateJson = (field: string, value: string): boolean => {
    try {
      if (value.trim()) {
        JSON.parse(value);
      }
      setJsonErrors(prev => ({ ...prev, [field]: '' }));
      return true;
    } catch (e) {
      setJsonErrors(prev => ({ ...prev, [field]: 'Invalid JSON format' }));
      return false;
    }
  };

  return (
    <Tabs defaultValue="basic" className="w-full">
      <ScrollArea className="w-full whitespace-nowrap">
        <TabsList className="inline-flex w-max gap-1 p-1">
          <TabsTrigger value="basic" className="gap-1.5">
            <Info className="h-3.5 w-3.5" />
            Basic
          </TabsTrigger>
          <TabsTrigger value="ai" className="gap-1.5">
            <Brain className="h-3.5 w-3.5" />
            AI Config
          </TabsTrigger>
          <TabsTrigger value="execution" className="gap-1.5">
            <Shield className="h-3.5 w-3.5" />
            Execution
          </TabsTrigger>
          <TabsTrigger value="skills" className="gap-1.5">
            <Wrench className="h-3.5 w-3.5" />
            Skills
          </TabsTrigger>
          <TabsTrigger value="mcp" className="gap-1.5">
            <Server className="h-3.5 w-3.5" />
            MCP
          </TabsTrigger>
          <TabsTrigger value="messaging" className="gap-1.5">
            <MessageSquare className="h-3.5 w-3.5" />
            Messaging
          </TabsTrigger>
          <TabsTrigger value="advanced" className="gap-1.5">
            <Key className="h-3.5 w-3.5" />
            Advanced
          </TabsTrigger>
        </TabsList>
        <ScrollBar orientation="horizontal" />
      </ScrollArea>

      {/* Basic Info Tab */}
      <TabsContent value="basic" className="space-y-4 py-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label htmlFor="name" className="flex items-center gap-1">
              Agent Name
              <span className="text-destructive">*</span>
            </Label>
            <Input
              id="name"
              placeholder="e.g., Research Assistant"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className={!formData.name?.trim() ? "border-amber-500 focus:border-amber-500" : ""}
            />
            {!formData.name?.trim() && (
              <p className="text-xs text-amber-600">Name is required</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="personality">Personality</Label>
            <Input
              id="personality"
              placeholder="e.g., Analytical and thorough"
              value={formData.characteristics?.personality}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  characteristics: { ...formData.characteristics!, personality: e.target.value },
                })
              }
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="description">Description</Label>
          <Textarea
            id="description"
            placeholder="What does this agent specialize in?"
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            className="min-h-[80px]"
          />
          <p className="text-xs text-muted-foreground">Brief description of the agent's purpose and capabilities</p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="tone">Communication Tone</Label>
          <Input
            id="tone"
            placeholder="e.g., Professional and formal"
            value={formData.characteristics?.tone}
            onChange={(e) =>
              setFormData({
                ...formData,
                characteristics: { ...formData.characteristics!, tone: e.target.value },
              })
            }
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="expertise" className="flex items-center gap-2">
            Expertise
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="h-3.5 w-3.5 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent>
                  <p>Separate multiple areas with commas</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </Label>
          <Input
            id="expertise"
            placeholder="e.g., Research, Data Analysis, Academic Writing"
            value={formData.characteristics?.expertise?.join(", ")}
            onChange={(e) =>
              setFormData({
                ...formData,
                characteristics: {
                  ...formData.characteristics!,
                  expertise: e.target.value.split(",").map((s) => s.trim()).filter(Boolean),
                },
              })
            }
          />
          {formData.characteristics?.expertise && formData.characteristics.expertise.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-2">
              {formData.characteristics.expertise.map((exp, i) => (
                <Badge key={i} variant="secondary" className="text-xs">
                  {exp}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </TabsContent>

      {/* AI Config, Skills, MCP, Messaging Tabs implementation similarly fixed... */}
      {/* For brevity in this edit, I will implement the key functional parts */}

      <TabsContent value="ai" className="space-y-4 py-4">
        <div className="space-y-2">
          <Label>AI Provider</Label>
          <Select
            value={formData.ai_config?.provider}
            onValueChange={(value: any) => {
              // Reset model when provider changes
              const defaultModels: Record<string, string> = {
                anthropic: "claude-3-5-sonnet-20241022",
                openai: "gpt-5.2",
                gemini: "gemini-2.0-flash"
              };
              setFormData({
                ...formData,
                ai_config: {
                  ...formData.ai_config!,
                  provider: value,
                  model: defaultModels[value] || ""
                }
              });
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select Provider" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="anthropic">Anthropic (Claude)</SelectItem>
              <SelectItem value="openai">OpenAI (GPT)</SelectItem>
              <SelectItem value="gemini">Google Gemini</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label>Model</Label>
          <Select
            value={formData.ai_config?.model}
            onValueChange={(value: any) =>
              setFormData({ ...formData, ai_config: { ...formData.ai_config!, model: value } })
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="Select Model" />
            </SelectTrigger>
            <SelectContent>
              {formData.ai_config?.provider === "gemini" && (
                <>
                  <SelectItem value="gemini-3-flash-preview">Gemini 3 Pro (Preview) - Most Intelligent</SelectItem>
                  <SelectItem value="gemini-3-flash-preview">Gemini 3 Flash (Preview) - Balanced</SelectItem>
                  <SelectItem value="gemini-2.0-flash-exp">Gemini 2.0 Flash Exp (Preview) - Best Performance</SelectItem>
                  <SelectItem value="gemini-2.0-flash">Gemini 2.0 Flash</SelectItem>
                  <SelectItem value="gemini-1.5-pro">Gemini 1.5 Pro - Stable</SelectItem>
                  <SelectItem value="gemini-1.5-flash">Gemini 1.5 Flash</SelectItem>
                </>
              )}
              {formData.ai_config?.provider === "anthropic" && (
                <>
                  <SelectItem value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet - Most Capable</SelectItem>
                  <SelectItem value="claude-3-5-haiku-20241022">Claude 3.5 Haiku - Fast</SelectItem>
                  <SelectItem value="claude-3-opus-20240229">Claude 3 Opus</SelectItem>
                  <SelectItem value="claude-3-sonnet-20240229">Claude 3 Sonnet</SelectItem>
                  <SelectItem value="claude-3-haiku-20240307">Claude 3 Haiku</SelectItem>
                </>
              )}
              {formData.ai_config?.provider === "openai" && (
                <>
                  <SelectItem value="gpt-5.2">GPT-5.2 - Best for Coding & Agentic</SelectItem>
                  <SelectItem value="gpt-5">GPT-5 - Reasoning Model</SelectItem>
                  <SelectItem value="gpt-5-mini">GPT-5 Mini - Fast & Efficient</SelectItem>
                  <SelectItem value="gpt-5-nano">GPT-5 Nano - Fastest</SelectItem>
                  <SelectItem value="gpt-4.1">GPT-4.1 - Smartest Non-Reasoning</SelectItem>
                  <SelectItem value="gpt-4o">GPT-4o - Fast & Intelligent</SelectItem>
                  <SelectItem value="gpt-4o-mini">GPT-4o Mini - Affordable</SelectItem>
                  <SelectItem value="gpt-4-turbo">GPT-4 Turbo</SelectItem>
                </>
              )}
            </SelectContent>
          </Select>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label>Temperature</Label>
            <Input
              type="number"
              min="0"
              max="2"
              step="0.1"
              placeholder="0.7"
              value={formData.ai_config?.temperature ?? 0.7}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  ai_config: {
                    ...formData.ai_config!,
                    temperature: parseFloat(e.target.value) || 0.7
                  }
                })
              }
            />
          </div>
          <div className="space-y-2">
            <Label>Max Tokens</Label>
            <Input
              type="number"
              min="256"
              max="32768"
              step="256"
              placeholder="4096"
              value={formData.ai_config?.max_tokens ?? 4096}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  ai_config: {
                    ...formData.ai_config!,
                    max_tokens: parseInt(e.target.value) || 4096
                  }
                })
              }
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label>System Prompt</Label>
          <Textarea
            className="min-h-[300px] font-mono text-sm"
            value={formData.system_prompt}
            onChange={(e) => setFormData({ ...formData, system_prompt: e.target.value })}
          />
        </div>
      </TabsContent>

      {/* Execution Settings Tab */}
      <TabsContent value="execution" className="space-y-4 py-4">
        <div className="rounded-md border p-4 space-y-6">
          <div>
            <h3 className="text-sm font-medium mb-2 flex items-center gap-2">
              <Shield className="h-4 w-4" />
              Execution Mode
            </h3>
            <p className="text-xs text-muted-foreground mb-4">
              Control how the agent executes commands and file operations.
            </p>
            <Select
              value={formData.execution_settings?.mode || "require_approval"}
              onValueChange={(value: any) =>
                setFormData({
                  ...formData,
                  execution_settings: {
                    ...formData.execution_settings!,
                    mode: value,
                  },
                })
              }
            >
              <SelectTrigger>
                <SelectValue placeholder="Select execution mode" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="autopilot">
                  <div className="flex items-center gap-2">
                    <ShieldAlert className="h-4 w-4 text-yellow-500" />
                    <span>Autopilot - Execute without approval</span>
                  </div>
                </SelectItem>
                <SelectItem value="require_approval">
                  <div className="flex items-center gap-2">
                    <ShieldCheck className="h-4 w-4 text-green-500" />
                    <span>Require Approval - Safe mode (Recommended)</span>
                  </div>
                </SelectItem>
                <SelectItem value="smart_approval">
                  <div className="flex items-center gap-2">
                    <Shield className="h-4 w-4 text-blue-500" />
                    <span>Smart Approval - Auto-approve whitelisted</span>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Separator />

          <div>
            <h3 className="text-sm font-medium mb-2 flex items-center gap-2">
              <Box className="h-4 w-4" />
              Skill/Tool Sandbox Mode
            </h3>
            <p className="text-xs text-muted-foreground mb-4">
              Controls how skills and tools execute commands. Sandbox mode uses Docker isolation for security.
            </p>
            <Select
              value={formData.execution_settings?.sandbox_mode || "flexible"}
              onValueChange={(value: any) =>
                setFormData({
                  ...formData,
                  execution_settings: {
                    ...formData.execution_settings!,
                    sandbox_mode: value,
                  },
                })
              }
            >
              <SelectTrigger>
                <SelectValue placeholder="Select sandbox mode" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="sandbox">
                  <div className="flex items-center gap-2">
                    <Box className="h-4 w-4 text-green-500" />
                    <span>Sandbox - Docker isolation (Recommended)</span>
                  </div>
                </SelectItem>
                <SelectItem value="direct">
                  <div className="flex items-center gap-2">
                    <ShieldAlert className="h-4 w-4 text-red-500" />
                    <span>Direct - No sandboxing (Fast but less secure)</span>
                  </div>
                </SelectItem>
                <SelectItem value="flexible">
                  <div className="flex items-center gap-2">
                    <Shield className="h-4 w-4 text-blue-500" />
                    <span>Flexible - Respect skill preference</span>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground mt-2 p-2 bg-muted/50 rounded">
              <strong>Note:</strong> Agent sandbox mode overrides skill settings for safety.
              <br />• <strong>Sandbox:</strong> Forces all skills to use Docker (requires Docker)
              <br />• <strong>Direct:</strong> Skips sandbox even if skill prefers it
              <br />• <strong>Flexible:</strong> Uses sandbox if Docker available & skill requests it
            </p>
          </div>

          <Separator />

          <div className="space-y-4">
            <div>
              <Label className="flex items-center gap-2 mb-2">
                <ShieldCheck className="h-4 w-4 text-green-500" />
                Whitelisted Tools
              </Label>
              <p className="text-xs text-muted-foreground mb-2">
                Tools that can execute without approval (one per line)
              </p>
              <Textarea
                className="font-mono text-sm min-h-[100px]"
                placeholder="file_read&#10;file_list&#10;get_workflow_status"
                value={formData.execution_settings?.whitelisted_tools?.join('\n') || ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    execution_settings: {
                      ...formData.execution_settings!,
                      whitelisted_tools: e.target.value.split('\n').filter(Boolean),
                    },
                  })
                }
              />
            </div>

            <div>
              <Label className="flex items-center gap-2 mb-2">
                <ShieldCheck className="h-4 w-4 text-green-500" />
                Whitelisted Commands (Regex)
              </Label>
              <p className="text-xs text-muted-foreground mb-2">
                Command patterns that can execute without approval (one per line)
              </p>
              <Textarea
                className="font-mono text-sm min-h-[100px]"
                placeholder="^ls(\s|$)&#10;^pwd$&#10;^git\s+status"
                value={formData.execution_settings?.whitelisted_commands?.join('\n') || ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    execution_settings: {
                      ...formData.execution_settings!,
                      whitelisted_commands: e.target.value.split('\n').filter(Boolean),
                    },
                  })
                }
              />
            </div>

            <div>
              <Label className="flex items-center gap-2 mb-2">
                <ShieldAlert className="h-4 w-4 text-red-500" />
                Blacklisted Commands (Regex)
              </Label>
              <p className="text-xs text-muted-foreground mb-2">
                Commands that ALWAYS require approval, even in autopilot mode
              </p>
              <Textarea
                className="font-mono text-sm min-h-[100px]"
                placeholder="rm\s+-rf&#10;sudo\s&#10;curl.*\|\s*(bash|sh)"
                value={formData.execution_settings?.blacklisted_commands?.join('\n') || ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    execution_settings: {
                      ...formData.execution_settings!,
                      blacklisted_commands: e.target.value.split('\n').filter(Boolean),
                    },
                  })
                }
              />
            </div>
          </div>

          <div className="p-3 bg-muted/50 rounded-lg">
            <p className="text-xs text-muted-foreground">
              <strong>Mode Descriptions:</strong>
              <br />• <strong>Autopilot:</strong> Execute all commands without asking (except blacklisted)
              <br />• <strong>Require Approval:</strong> Show approval UI for writes and commands (read-only tools bypass)
              <br />• <strong>Smart Approval:</strong> Auto-approve whitelisted commands/tools, require approval for others
            </p>
          </div>
        </div>
      </TabsContent>

      <TabsContent value="skills" className="space-y-4 py-4">
        {isDefaultAgent && (
          <Alert className="mb-4">
            <Info className="h-4 w-4" />
            <AlertDescription>
              The default agent uses all enabled skills in the system. Skills are managed globally in the Skills page.
            </AlertDescription>
          </Alert>
        )}
        <div className="rounded-md border p-4">
          <h3 className="mb-4 text-sm font-medium">
            {isDefaultAgent ? "Available Skills (Read-only)" : "Select Enabled Skills"}
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {allSkills.map((skill: any) => (
              <div key={skill.id} className={cn(
                "flex items-start space-x-3 rounded-md border p-3",
                !isDefaultAgent && "hover:bg-muted/50"
              )}>
                <Checkbox
                  id={`skill-${skill.id}`}
                  checked={isDefaultAgent ? true : formData.skills?.includes(skill.id)}
                  onCheckedChange={() => !isDefaultAgent && toggleSkill(skill.id)}
                  disabled={isDefaultAgent}
                />
                <div className="grid gap-1.5 leading-none">
                  <label
                    htmlFor={`skill-${skill.id}`}
                    className={cn(
                      "text-sm font-medium leading-none",
                      isDefaultAgent && "cursor-default opacity-70"
                    )}
                  >
                    {skill.name}
                  </label>
                  <p className="text-xs text-muted-foreground">
                    {skill.description}
                  </p>
                </div>
              </div>
            ))}
            {allSkills.length === 0 && <p className="text-sm text-muted-foreground">No skills available.</p>}
          </div>
        </div>
      </TabsContent>

      <TabsContent value="mcp" className="space-y-4 py-4">
        {isDefaultAgent && (
          <Alert className="mb-4">
            <Info className="h-4 w-4" />
            <AlertDescription>
              The default agent uses all enabled MCP servers in the system. MCP servers are managed globally in the Settings page.
            </AlertDescription>
          </Alert>
        )}
        <div className="rounded-md border p-4">
          <h3 className="mb-4 text-sm font-medium">
            {isDefaultAgent ? "Available MCP Servers (Read-only)" : "Connect MCP Servers"}
          </h3>
          <div className="grid grid-cols-1 gap-4">
            {allMCPServers.map((server: any) => (
              <div key={server.id} className="flex items-center space-x-3 rounded-md border p-3">
                <Checkbox
                  id={`mcp-${server.id}`}
                  checked={isDefaultAgent ? true : formData.mcp_servers?.includes(server.id)}
                  onCheckedChange={() => !isDefaultAgent && toggleMCP(server.id)}
                  disabled={isDefaultAgent}
                />
                <div className="flex-1">
                  <label 
                    htmlFor={`mcp-${server.id}`} 
                    className={cn(
                      "text-sm font-medium",
                      isDefaultAgent && "cursor-default opacity-70"
                    )}
                  >
                    {server.name}
                  </label>
                  <p className="text-xs text-muted-foreground">{server.server_url}</p>
                </div>
                <Badge variant="outline">{server.transport}</Badge>
              </div>
            ))}
            {allMCPServers.length === 0 && <p className="text-sm text-muted-foreground">No MCP servers available.</p>}
          </div>
        </div>
      </TabsContent>

      <TabsContent value="messaging" className="space-y-4 py-4">
        <div className="rounded-md border p-4">
          <div className="mb-4">
            <h3 className="text-sm font-medium mb-2">Platform Configuration</h3>
            <p className="text-xs text-muted-foreground">
              Configure platform-specific credentials for this agent. If not configured here, global settings from Settings page will be used.
            </p>
          </div>

          <div className="space-y-6">
            {/* Telegram Configuration */}
            <div className="space-y-3 p-4 border rounded-lg">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <MessageSquare className="h-4 w-4" />
                  <Label className="text-base font-medium">Telegram</Label>
                </div>
                <Checkbox
                  checked={formData.messaging_connections?.includes('telegram') || false}
                  onCheckedChange={(checked) => {
                    const current = formData.messaging_connections || [];
                    const updated = checked
                      ? [...current, 'telegram']
                      : current.filter(id => id !== 'telegram');
                    setFormData({ ...formData, messaging_connections: updated });
                  }}
                />
              </div>

              {formData.messaging_connections?.includes('telegram') && (
                <div className="space-y-3 mt-3 pl-6">
                  <div className="space-y-2">
                    <Label htmlFor="telegram_bot_token" className="text-sm">Bot Token (Optional - overrides global)</Label>
                    <Input
                      id="telegram_bot_token"
                      type="password"
                      placeholder="Leave empty to use global settings"
                      value={formData.platform_configs?.telegram?.bot_token || ''}
                      onChange={(e) => {
                        const configs = formData.platform_configs || {};
                        setFormData({
                          ...formData,
                          platform_configs: {
                            ...configs,
                            telegram: {
                              ...configs.telegram,
                              bot_token: e.target.value,
                              enabled: true
                            }
                          }
                        });
                      }}
                    />
                    {formData.platform_configs?.telegram?.bot_token && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          const token = formData.platform_configs?.telegram?.bot_token;
                          if (token) {
                            anycoworkApi.testTelegramConnection(token).then((result) => {
                              if (result.success) {
                                toast.success(`Connected! Bot: @${result.bot_username}`);
                              } else {
                                toast.error(result.error || 'Connection failed');
                              }
                            }).catch((err) => {
                              toast.error(`Test failed: ${err.message || err}`);
                            });
                          }
                        }}
                        className="gap-2"
                      >
                        <CheckCircle2 className="h-4 w-4" />
                        Test Connection
                      </Button>
                    )}
                    <p className="text-xs text-muted-foreground">
                      If provided, this agent will use its own Telegram bot. Otherwise, it will use the global bot token from Settings.
                    </p>
                  </div>
                </div>
              )}
            </div>

          </div>

          <div className="mt-4 p-3 bg-muted/50 rounded-lg">
            <p className="text-xs text-muted-foreground">
              <strong>Note:</strong> Per-agent platform configurations allow different agents to use different messaging accounts.
              This is useful when you want multiple agents monitoring different Telegram bots.
            </p>
          </div>
        </div>
      </TabsContent>

      <TabsContent value="advanced" className="space-y-4 py-4">
        <Alert className="mb-4">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            Advanced settings require careful configuration. Invalid JSON will prevent saving.
          </AlertDescription>
        </Alert>

        <div className="rounded-md border p-4 space-y-6">
          <div className="space-y-2">
            <Label htmlFor="working_directories" className="flex items-center gap-2">
              <Database className="h-4 w-4" />
              Allowed Working Directories
            </Label>
            <Textarea
              id="working_directories"
              placeholder="/home/user/workspace&#10;/tmp/agent"
              value={formData.working_directories?.join('\n') || ""}
              onChange={(e) => setFormData({ ...formData, working_directories: e.target.value.split('\n').filter(Boolean) })}
              className="font-mono text-sm min-h-[80px]"
            />
            <p className="text-xs text-muted-foreground">One directory per line. Agent will only access these directories.</p>
          </div>

          <Separator />

          <div className="space-y-2">
            <Label className="flex items-center gap-2">
              <Shield className="h-4 w-4" />
              Permissions (JSON)
            </Label>
            <Textarea
              className={cn(
                "font-mono text-sm min-h-[100px]",
                jsonErrors.permissions && "border-destructive focus:border-destructive"
              )}
              placeholder='{"allow_fs": true, "allow_net": true}'
              value={typeof formData.permissions === 'string' ? formData.permissions : JSON.stringify(formData.permissions || {}, null, 2)}
              onChange={(e) => {
                const val = e.target.value;
                if (validateJson('permissions', val)) {
                  try {
                    setFormData({ ...formData, permissions: JSON.parse(val) });
                  } catch {
                    // Keep as-is while typing
                  }
                }
              }}
              onBlur={(e) => validateJson('permissions', e.target.value)}
            />
            {jsonErrors.permissions ? (
              <p className="text-xs text-destructive flex items-center gap-1">
                <AlertCircle className="h-3 w-3" />
                {jsonErrors.permissions}
              </p>
            ) : (
              <p className="text-xs text-muted-foreground">Configure file system, network, and other permissions.</p>
            )}
          </div>

          <Separator />

          <div className="space-y-2">
            <Label className="flex items-center gap-2">
              <Key className="h-4 w-4" />
              API Keys (JSON)
            </Label>
            <Textarea
              className={cn(
                "font-mono text-sm min-h-[100px]",
                jsonErrors.apiKeys && "border-destructive focus:border-destructive"
              )}
              placeholder='{"GITHUB_TOKEN": "ghp_...", "OPENAI_KEY": "sk-..."}'
              value={JSON.stringify(formData.api_keys || {}, null, 2)}
              onChange={(e) => {
                const val = e.target.value;
                if (validateJson('apiKeys', val)) {
                  try {
                    setFormData({ ...formData, api_keys: JSON.parse(val) });
                  } catch {
                    // Keep as-is while typing
                  }
                }
              }}
              onBlur={(e) => validateJson('apiKeys', e.target.value)}
            />
            {jsonErrors.apiKeys ? (
              <p className="text-xs text-destructive flex items-center gap-1">
                <AlertCircle className="h-3 w-3" />
                {jsonErrors.apiKeys}
              </p>
            ) : (
              <p className="text-xs text-muted-foreground">Store API keys securely for this agent. Keys are encrypted at rest.</p>
            )}
          </div>
        </div>
      </TabsContent>

      <div className="mt-6 flex justify-end gap-3 pt-4 border-t">
        <Button variant="outline" onClick={onClose}>
          Cancel
        </Button>
        <Button
          onClick={handleSave}
          disabled={!formData.name?.trim() || createAgent.isPending || updateAgent.isPending}
          className="min-w-[120px]"
        >
          {(createAgent.isPending || updateAgent.isPending) && (
            <Loader2 className="h-4 w-4 animate-spin mr-2" />
          )}
          {agent ? "Save Changes" : "Create Agent"}
        </Button>
      </div>
    </Tabs>
  );
}
