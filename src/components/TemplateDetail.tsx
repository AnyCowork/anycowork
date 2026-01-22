import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ArrowLeft, Sparkles, Wrench, Lightbulb, MessageSquare, Copy, Check } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

interface SuggestedTool {
  name: string;
  description: string;
  skill_id: string;
}

interface TemplateDetail {
  id: string;
  name: string;
  description: string;
  version: string;
  icon: string;
  agent_config: any;
  suggested_tools: SuggestedTool[];
  example_tasks: string[];
  starter_prompts: string[];
}

interface TemplateDetailProps {
  templateId: string;
  onBack: () => void;
  onUseTemplate: (config: any) => void;
}

export const TemplateDetail: React.FC<TemplateDetailProps> = ({
  templateId,
  onBack,
  onUseTemplate,
}) => {
  const [template, setTemplate] = useState<TemplateDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [copiedPrompt, setCopiedPrompt] = useState<number | null>(null);

  useEffect(() => {
    fetchTemplate();
  }, [templateId]);

  const fetchTemplate = async () => {
    try {
      const response = await fetch(`/api/templates/${templateId}`);
      const data = await response.json();
      setTemplate(data);
    } catch (error) {
      console.error('Failed to fetch template:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCopyPrompt = (prompt: string, index: number) => {
    navigator.clipboard.writeText(prompt);
    setCopiedPrompt(index);
    setTimeout(() => setCopiedPrompt(null), 2000);
  };

  const handleUseTemplate = () => {
    if (template) {
      onUseTemplate(template.agent_config);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (!template) {
    return (
      <div className="text-center text-muted-foreground p-8">
        Failed to load template. Please try again.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="space-y-2">
          <Button variant="ghost" size="sm" onClick={onBack} className="mb-2">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Templates
          </Button>
          <div className="flex items-center gap-3">
            <span className="text-5xl">{template.icon}</span>
            <div>
              <h2 className="text-3xl font-bold">{template.name}</h2>
              <p className="text-muted-foreground">{template.description}</p>
            </div>
          </div>
        </div>
        <Button size="lg" onClick={handleUseTemplate}>
          <Sparkles className="h-4 w-4 mr-2" />
          Use This Template
        </Button>
      </div>

      {/* Content Tabs */}
      <Tabs defaultValue="overview" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="tools">Tools</TabsTrigger>
          <TabsTrigger value="examples">Examples</TabsTrigger>
          <TabsTrigger value="prompts">Starter Prompts</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Agent Configuration</CardTitle>
              <CardDescription>How this agent is configured</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h4 className="font-semibold mb-2">Personality</h4>
                <div className="flex flex-wrap gap-2">
                  <Badge variant="secondary">
                    {template.agent_config.characteristics.personality}
                  </Badge>
                  <Badge variant="secondary">
                    {template.agent_config.characteristics.verbosity} verbosity
                  </Badge>
                  <Badge variant="secondary">
                    {template.agent_config.characteristics.formality}
                  </Badge>
                </div>
              </div>

              <div>
                <h4 className="font-semibold mb-2">AI Model</h4>
                <div className="space-y-1 text-sm">
                  <p>
                    <span className="text-muted-foreground">Provider:</span>{' '}
                    {template.agent_config.ai_config.provider}
                  </p>
                  <p>
                    <span className="text-muted-foreground">Model:</span>{' '}
                    {template.agent_config.ai_config.model}
                  </p>
                  <p>
                    <span className="text-muted-foreground">Temperature:</span>{' '}
                    {template.agent_config.ai_config.temperature}
                  </p>
                </div>
              </div>

              <div>
                <h4 className="font-semibold mb-2">System Prompt</h4>
                <ScrollArea className="h-64 w-full rounded-md border p-4">
                  <pre className="text-sm whitespace-pre-wrap">
                    {template.agent_config.system_prompt}
                  </pre>
                </ScrollArea>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="tools" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Wrench className="h-5 w-5" />
                Suggested Tools
              </CardTitle>
              <CardDescription>Tools that work well with this agent</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {template.suggested_tools.map((tool, index) => (
                  <div key={index} className="border rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div>
                        <h4 className="font-semibold">{tool.name}</h4>
                        <p className="text-sm text-muted-foreground">{tool.description}</p>
                      </div>
                      <Badge variant="outline">{tool.skill_id}</Badge>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="examples" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Lightbulb className="h-5 w-5" />
                Example Tasks
              </CardTitle>
              <CardDescription>What you can do with this agent</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2">
                {template.example_tasks.map((task, index) => (
                  <li key={index} className="flex items-start gap-2">
                    <span className="text-primary mt-1">•</span>
                    <span>{task}</span>
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="prompts" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <MessageSquare className="h-5 w-5" />
                Starter Prompts
              </CardTitle>
              <CardDescription>Copy these prompts to get started quickly</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {template.starter_prompts.map((prompt, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between border rounded-lg p-3 hover:bg-accent transition-colors"
                  >
                    <span className="text-sm">{prompt}</span>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleCopyPrompt(prompt, index)}
                    >
                      {copiedPrompt === index ? (
                        <Check className="h-4 w-4 text-green-500" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default TemplateDetail;
