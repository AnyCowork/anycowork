/**
 * Settings Page - Claude/Anthropic inspired design
 */

import React, { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ProviderSelect } from "@/components/ProviderSelect";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { Textarea } from "@/components/ui/textarea";
import {
  Sparkles,
  MessageSquare,
  Save,
  Loader2,
  CheckCircle2,
  XCircle,
  Brain,
  Zap,
  Shield,
  Play,
  ShieldCheck,
  ShieldAlert,
  Plus,
  Trash2,
  Terminal,
  Wrench,
  BrainCircuit,
} from "lucide-react";
import {
  useAIConfig,
  useUpdateAIConfig,
  useMessagingConfig,
  useUpdateMessagingConfig,
  useTestTelegramConnection,
  useExecutionSettings,
  useUpdateExecutionSettings,
  useSetExecutionMode,
  useAvailableModels,
} from "@/lib/hooks/use-anycowork";
import { ExecutionMode } from "@/lib/anycowork-api";
import { toast } from "sonner";

function SettingsPage() {
  const { data: aiConfig, isLoading: aiLoading } = useAIConfig();
  const { data: messagingConfig, isLoading: messagingLoading } = useMessagingConfig();
  const { data: executionSettings, isLoading: executionLoading } = useExecutionSettings();
  const { data: availableModels } = useAvailableModels();
  const updateAI = useUpdateAIConfig();
  const updateMessaging = useUpdateMessagingConfig();
  const updateExecution = useUpdateExecutionSettings();
  const setExecutionMode = useSetExecutionMode();
  const testTelegram = useTestTelegramConnection();

  const [activeTab, setActiveTab] = useState("ai");
  const [newCommandPattern, setNewCommandPattern] = useState("");
  const [newTool, setNewTool] = useState("");

  // Local state for form inputs
  const [aiForm, setAIForm] = useState({
    provider: "",
    anthropic_api_key: "",
    anthropic_model: "claude-opus-4-5-20251101",
    openai_api_key: "",
    openai_model: "gpt-5",
    gemini_api_key: "",
    gemini_model: "gemini-3-pro",
    max_tokens: 4096,
  });

  const [messagingForm, setMessagingForm] = useState({
    telegram_enabled: false,
    telegram_bot_token: "",
  });

  // Update form when data loads
  React.useEffect(() => {
    if (aiConfig) {
      setAIForm({
        provider: aiConfig.provider,
        anthropic_api_key: aiConfig.anthropic_api_key,
        anthropic_model: aiConfig.anthropic_model,
        openai_api_key: aiConfig.openai_api_key,
        openai_model: aiConfig.openai_model,
        gemini_api_key: aiConfig.gemini_api_key,
        gemini_model: aiConfig.gemini_model,
        max_tokens: aiConfig.max_tokens,
      });
    }
  }, [aiConfig]);

  React.useEffect(() => {
    if (messagingConfig) {
      setMessagingForm({
        telegram_enabled: messagingConfig.telegram.enabled,
        telegram_bot_token: messagingConfig.telegram.bot_token,
      });
    }
  }, [messagingConfig]);

  const handleSaveAI = () => {
    updateAI.mutate(aiForm);
  };

  const handleSaveMessaging = () => {
    updateMessaging.mutate({
      telegram: {
        enabled: messagingForm.telegram_enabled,
        bot_token: messagingForm.telegram_bot_token,
      },
    });
  };

  const handleTestTelegram = () => {
    if (!messagingForm.telegram_bot_token) {
      toast.error("Please enter a bot token first");
      return;
    }
    testTelegram.mutate(messagingForm.telegram_bot_token);
  };

  const handleModeChange = (mode: ExecutionMode) => {
    setExecutionMode.mutate(mode);
  };

  const handleAddCommandPattern = () => {
    if (!newCommandPattern.trim()) return;
    const currentPatterns = executionSettings?.whitelisted_commands || [];
    updateExecution.mutate({
      whitelisted_commands: [...currentPatterns, newCommandPattern.trim()],
    });
    setNewCommandPattern("");
  };

  const handleRemoveCommandPattern = (pattern: string) => {
    const currentPatterns = executionSettings?.whitelisted_commands || [];
    updateExecution.mutate({
      whitelisted_commands: currentPatterns.filter(p => p !== pattern),
    });
  };

  const handleAddTool = () => {
    if (!newTool.trim()) return;
    const currentTools = executionSettings?.whitelisted_tools || [];
    updateExecution.mutate({
      whitelisted_tools: [...currentTools, newTool.trim()],
    });
    setNewTool("");
  };

  const handleRemoveTool = (tool: string) => {
    const currentTools = executionSettings?.whitelisted_tools || [];
    updateExecution.mutate({
      whitelisted_tools: currentTools.filter(t => t !== tool),
    });
  };

  if (aiLoading || messagingLoading || executionLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="max-w-6xl mx-auto px-8 py-6">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/10">
              <Shield className="h-6 w-6 text-primary" />
            </div>
            <div>
              <h1 className="text-2xl font-semibold">Settings</h1>
              <p className="text-sm text-muted-foreground">
                Configure your AnyCowork instance
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-6xl mx-auto px-8 py-8">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <TabsList className="grid w-full grid-cols-3 max-w-xl">
            <TabsTrigger value="ai" className="gap-2">
              <Sparkles className="h-4 w-4" />
              AI Providers
            </TabsTrigger>
            <TabsTrigger value="execution" className="gap-2">
              <ShieldCheck className="h-4 w-4" />
              Execution
            </TabsTrigger>
            <TabsTrigger value="messaging" className="gap-2">
              <MessageSquare className="h-4 w-4" />
              Messaging
            </TabsTrigger>
          </TabsList>

          {/* AI Providers Tab */}
          <TabsContent value="ai" className="space-y-6">
            {/* Provider Selection */}
            <Card className="border-2">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Brain className="h-5 w-5" />
                  Active Provider
                </CardTitle>
                <CardDescription>
                  Select which AI provider to use for agent responses
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ProviderSelect
                  value={aiForm.provider}
                  onChange={(value) => setAIForm({ ...aiForm, provider: value })}
                />
              </CardContent>
            </Card>

            {/* Anthropic Configuration */}
            <Card
              className={
                aiForm.provider === "anthropic"
                  ? "border-2 border-primary"
                  : ""
              }
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Sparkles className="h-5 w-5" />
                    <CardTitle>Anthropic (Claude)</CardTitle>
                  </div>
                  {aiForm.provider === "anthropic" && (
                    <Badge variant="default">Active</Badge>
                  )}
                </div>
                <CardDescription>
                  Configure Claude API access and model selection
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="anthropic_api_key">API Key</Label>
                  <Input
                    id="anthropic_api_key"
                    type="password"
                    placeholder="sk-ant-..."
                    value={aiForm.anthropic_api_key}
                    onChange={(e) =>
                      setAIForm({
                        ...aiForm,
                        anthropic_api_key: e.target.value,
                      })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="anthropic_model">Model</Label>
                  <Select
                    value={aiForm.anthropic_model}
                    onValueChange={(value) =>
                      setAIForm({ ...aiForm, anthropic_model: value })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels?.providers?.anthropic?.models?.map((model) => (
                        <SelectItem key={model.id} value={model.id}>
                          {model.name}
                        </SelectItem>
                      )) || (
                          <>
                            <SelectItem value="claude-opus-4-5-20251101">Claude Opus 4.5</SelectItem>
                            <SelectItem value="claude-sonnet-4-20250514">Claude Sonnet 4</SelectItem>
                            <SelectItem value="claude-haiku-3-5-20241022">Claude Haiku 3.5</SelectItem>
                          </>
                        )}
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            {/* OpenAI Configuration */}
            <Card
              className={
                aiForm.provider === "openai" ? "border-2 border-primary" : ""
              }
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Zap className="h-5 w-5" />
                    <CardTitle>OpenAI (GPT)</CardTitle>
                  </div>
                  {aiForm.provider === "openai" && (
                    <Badge variant="default">Active</Badge>
                  )}
                </div>
                <CardDescription>
                  Configure OpenAI API access and model selection
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="openai_api_key">API Key</Label>
                  <Input
                    id="openai_api_key"
                    type="password"
                    placeholder="sk-..."
                    value={aiForm.openai_api_key}
                    onChange={(e) =>
                      setAIForm({ ...aiForm, openai_api_key: e.target.value })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="openai_model">Model</Label>
                  <Select
                    value={aiForm.openai_model}
                    onValueChange={(value) =>
                      setAIForm({ ...aiForm, openai_model: value })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels?.providers?.openai?.models?.map((model) => (
                        <SelectItem key={model.id} value={model.id}>
                          {model.name}
                        </SelectItem>
                      )) || (
                          <>
                            <SelectItem value="gpt-5">GPT-5</SelectItem>
                            <SelectItem value="gpt-4o">GPT-4o</SelectItem>
                            <SelectItem value="gpt-4-turbo">GPT-4 Turbo</SelectItem>
                            <SelectItem value="o1">O1</SelectItem>
                          </>
                        )}
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            {/* Gemini Configuration */}
            <Card
              className={
                aiForm.provider === "gemini" ? "border-2 border-primary" : ""
              }
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Brain className="h-5 w-5" />
                    <CardTitle>Google (Gemini)</CardTitle>
                  </div>
                  {aiForm.provider === "gemini" && (
                    <Badge variant="default">Active</Badge>
                  )}
                </div>
                <CardDescription>
                  Configure Gemini API access and model selection
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="gemini_api_key">API Key</Label>
                  <Input
                    id="gemini_api_key"
                    type="password"
                    placeholder="AIza..."
                    value={aiForm.gemini_api_key}
                    onChange={(e) =>
                      setAIForm({ ...aiForm, gemini_api_key: e.target.value })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="gemini_model">Model</Label>
                  <Select
                    value={aiForm.gemini_model}
                    onValueChange={(value) =>
                      setAIForm({ ...aiForm, gemini_model: value })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels?.providers?.gemini?.models?.map((model) => (
                        <SelectItem key={model.id} value={model.id}>
                          {model.name}
                        </SelectItem>
                      )) || (
                          <>
                            <SelectItem value="gemini-3-pro">Gemini 3 Pro</SelectItem>
                            <SelectItem value="gemini-2.5-pro">Gemini 2.5 Pro</SelectItem>
                            <SelectItem value="gemini-2.0-flash">Gemini 2.0 Flash</SelectItem>
                          </>
                        )}
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            {/* Save Button */}
            <div className="flex justify-end">
              <Button
                onClick={handleSaveAI}
                disabled={updateAI.isPending}
                size="lg"
                className="gap-2"
              >
                {updateAI.isPending ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Saving...
                  </>
                ) : (
                  <>
                    <Save className="h-4 w-4" />
                    Save Configuration
                  </>
                )}
              </Button>
            </div>
          </TabsContent>

          {/* Execution Settings Tab */}
          <TabsContent value="execution" className="space-y-6">
            {/* Execution Mode Selection */}
            <Card className="border-2">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <ShieldCheck className="h-5 w-5" />
                  Execution Mode
                </CardTitle>
                <CardDescription>
                  Control how the agent executes commands and file operations
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <Card
                    className={`cursor-pointer transition-all hover:border-primary ${executionSettings?.mode === ExecutionMode.Autopilot ? "border-primary ring-1 ring-primary" : ""
                      }`}
                    onClick={() => handleModeChange(ExecutionMode.Autopilot)}
                  >
                    <CardHeader className="p-4">
                      <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <Zap className="h-4 w-4 text-orange-500" />
                        Autopilot
                      </CardTitle>
                      <CardDescription className="text-xs">
                        AI runs autonomously. No user confirmation required per step.
                      </CardDescription>
                    </CardHeader>
                  </Card>

                  <Card
                    className={`cursor-pointer transition-all hover:border-primary ${executionSettings?.mode === ExecutionMode.SmartApproval ? "border-primary ring-1 ring-primary" : ""
                      }`}
                    onClick={() => handleModeChange(ExecutionMode.SmartApproval)}
                  >
                    <CardHeader className="p-4">
                      <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <BrainCircuit className="h-4 w-4 text-blue-500" />
                        Smart Approval
                      </CardTitle>
                      <CardDescription className="text-xs">
                        AI requests approval only for sensitive or unknown tools.
                      </CardDescription>
                    </CardHeader>
                  </Card>

                  <Card
                    className={`cursor-pointer transition-all hover:border-primary ${executionSettings?.mode === ExecutionMode.RequireApproval ? "border-primary ring-1 ring-primary" : ""
                      }`}
                    onClick={() => handleModeChange(ExecutionMode.RequireApproval)}
                  >
                    <CardHeader className="p-4">
                      <CardTitle className="text-sm font-medium flex items-center gap-2">
                        <ShieldAlert className="h-4 w-4 text-red-500" />
                        Require Approval
                      </CardTitle>
                      <CardDescription className="text-xs">
                        AI asks for permission before executing any tool command.
                      </CardDescription>
                    </CardHeader>
                  </Card>
                </div>
              </CardContent>
            </Card>

            {/* Whitelisted Commands */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Terminal className="h-5 w-5" />
                  Whitelisted Commands
                </CardTitle>
                <CardDescription>
                  Commands matching these patterns will be auto-approved in Smart Approval mode.
                  Patterns use regex (e.g., ^ls for commands starting with "ls").
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Input
                    placeholder="^git\s+status"
                    value={newCommandPattern}
                    onChange={(e) => setNewCommandPattern(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleAddCommandPattern()}
                  />
                  <Button onClick={handleAddCommandPattern} disabled={!newCommandPattern.trim()}>
                    <Plus className="h-4 w-4" />
                  </Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {executionSettings?.whitelisted_commands?.map((pattern) => (
                    <Badge
                      key={pattern}
                      variant="secondary"
                      className="flex items-center gap-1 py-1 px-2"
                    >
                      <code className="text-xs">{pattern}</code>
                      <button
                        onClick={() => handleRemoveCommandPattern(pattern)}
                        className="ml-1 hover:text-destructive"
                      >
                        <XCircle className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Whitelisted Tools */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Wrench className="h-5 w-5" />
                  Whitelisted Tools
                </CardTitle>
                <CardDescription>
                  These tools will be auto-approved in Smart Approval mode.
                  Read-only tools like file_read and file_list are safe to whitelist.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Input
                    placeholder="file_read"
                    value={newTool}
                    onChange={(e) => setNewTool(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleAddTool()}
                  />
                  <Button onClick={handleAddTool} disabled={!newTool.trim()}>
                    <Plus className="h-4 w-4" />
                  </Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {executionSettings?.whitelisted_tools?.map((tool) => (
                    <Badge
                      key={tool}
                      variant="secondary"
                      className="flex items-center gap-1 py-1 px-2"
                    >
                      {tool}
                      <button
                        onClick={() => handleRemoveTool(tool)}
                        className="ml-1 hover:text-destructive"
                      >
                        <XCircle className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Blacklisted Commands (Info Only) */}
            <Card className="border-destructive/20">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-destructive">
                  <ShieldAlert className="h-5 w-5" />
                  Blocked Commands
                </CardTitle>
                <CardDescription>
                  These dangerous command patterns are always blocked, even in Autopilot mode.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {executionSettings?.blacklisted_commands?.map((pattern) => (
                    <Badge
                      key={pattern}
                      variant="destructive"
                      className="py-1 px-2"
                    >
                      <code className="text-xs">{pattern}</code>
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Messaging Tab */}
          <TabsContent value="messaging" className="space-y-6">
            {/* Telegram Configuration */}
            <Card className="border-2">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <MessageSquare className="h-5 w-5" />
                    <CardTitle>Telegram Bot</CardTitle>
                  </div>
                  <Switch
                    checked={messagingForm.telegram_enabled}
                    onCheckedChange={(checked) =>
                      setMessagingForm({
                        ...messagingForm,
                        telegram_enabled: checked,
                      })
                    }
                  />
                </div>
                <CardDescription>
                  Connect your Telegram bot to receive messages from users
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="telegram_bot_token">Bot Token</Label>
                  <Input
                    id="telegram_bot_token"
                    type="password"
                    placeholder="1234567890:ABCdefGHIjklMNOpqrsTUVwxyz"
                    value={messagingForm.telegram_bot_token}
                    onChange={(e) =>
                      setMessagingForm({
                        ...messagingForm,
                        telegram_bot_token: e.target.value,
                      })
                    }
                  />
                  <p className="text-xs text-muted-foreground">
                    Get your bot token from{" "}
                    <a
                      href="https://t.me/BotFather"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary hover:underline"
                    >
                      @BotFather
                    </a>
                  </p>
                </div>

                <Button
                  variant="outline"
                  onClick={handleTestTelegram}
                  disabled={testTelegram.isPending}
                  className="gap-2"
                >
                  {testTelegram.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      Testing...
                    </>
                  ) : (
                    <>
                      <CheckCircle2 className="h-4 w-4" />
                      Test Connection
                    </>
                  )}
                </Button>
              </CardContent>
            </Card>

            {/* Save Button */}
            <div className="flex justify-end">
              <Button
                onClick={handleSaveMessaging}
                disabled={updateMessaging.isPending}
                size="lg"
                className="gap-2"
              >
                {updateMessaging.isPending ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Saving...
                  </>
                ) : (
                  <>
                    <Save className="h-4 w-4" />
                    Save Configuration
                  </>
                )}
              </Button>
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

export default SettingsPage;
