import { useRef } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useSettings } from "@/hooks/useSettings";
import { useRAGFullConfig, useUpdateRAGFullConfig } from "@/hooks/useSettingsConfig";
import { useImportNotionFile, useImportNotionZip } from "@/hooks/useNotionImport";
import { ModeToggle } from "../mode-toggle";
import { Save, Loader2, Upload, FileText, Package } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import type { RAGFullConfig } from "@/lib/settings-api";

export const SettingsModal = () => {
  const settings = useSettings();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const zipInputRef = useRef<HTMLInputElement>(null);

  // Fetch RAG config
  const { data: config, isLoading: loading } = useRAGFullConfig();
  const updateConfig = useUpdateRAGFullConfig();

  // Import mutations
  // const importFile = useImportNotionFile();
  // const importZip = useImportNotionZip();
  const importFile = useImportNotionFile();
  const importZip = useImportNotionZip();

  const handleSaveConfig = () => {
    if (config) {
      updateConfig.mutate(config);
    }
  };

  const handleNotionFileImport = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    importFile.mutate(
      files[0],
      {
        onSettled: () => {
          if (fileInputRef.current) {
            fileInputRef.current.value = "";
          }
        },
      }
    );
  };

  const handleNotionZipImport = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    importZip.mutate(
      files[0],
      {
        onSettled: () => {
          if (zipInputRef.current) {
            zipInputRef.current.value = "";
          }
        },
      }
    );
  };

  const isImporting = importFile.isPending || importZip.isPending;

  return (
    <Dialog open={settings.isOpen} onOpenChange={settings.onClose}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader className="border-b pb-3 pr-8">
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1">
              <DialogTitle className="text-lg font-medium">
                Settings
              </DialogTitle>
              <DialogDescription>
                Configure appearance and RAG backends
              </DialogDescription>
            </div>
            {config && (
              <Button
                onClick={handleSaveConfig}
                disabled={updateConfig.isPending}
                size="sm"
                className="shrink-0"
              >
                {updateConfig.isPending ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Saving...
                  </>
                ) : (
                  <>
                    <Save className="h-4 w-4 mr-2" />
                    Save
                  </>
                )}
              </Button>
            )}
          </div>
        </DialogHeader>

        <div className="space-y-6 py-4">
          <Tabs defaultValue="appearance" className="w-full">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="appearance">Appearance</TabsTrigger>
              <TabsTrigger value="rag">RAG & Chat</TabsTrigger>
              <TabsTrigger value="import">
                <Upload className="h-4 w-4 mr-2" />
                Import
              </TabsTrigger>
            </TabsList>

            {/* Appearance Tab */}
            <TabsContent value="appearance" className="space-y-4 mt-4">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-y-1">
                  <Label>Theme</Label>
                  <span className="text-[0.8rem] text-muted-foreground">
                    Customize how it looks on your device
                  </span>
                </div>
                <ModeToggle />
              </div>
            </TabsContent>

            {/* RAG Configuration Tab */}
            <TabsContent value="rag" className="space-y-4 mt-4">
              {loading ? (
                <div className="flex items-center gap-2 py-8 justify-center">
                  <Loader2 className="h-5 w-5 animate-spin" />
                  <span className="text-sm text-muted-foreground">
                    Loading RAG settings...
                  </span>
                </div>
              ) : config ? (
                <div className="space-y-4">
                  {/* General Settings */}
                  <div className="space-y-3 p-4 rounded-lg bg-muted/30">
                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>Enable RAG</Label>
                        <p className="text-xs text-muted-foreground">
                          Enable semantic search and chat features
                        </p>
                      </div>
                      <Switch
                        checked={config.enabled}
                        onCheckedChange={(checked) => {
                          // Update config immutably
                          updateConfig.mutate({ ...config, enabled: checked });
                        }}
                      />
                    </div>

                    <div className="space-y-2">
                      <Label>Backend</Label>
                      <p className="text-sm text-muted-foreground">
                        LEANN - Fast vector-based search with 97% storage
                        reduction
                      </p>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label>Auto-index on save</Label>
                        <p className="text-xs text-muted-foreground">
                          Automatically index documents when saved
                        </p>
                      </div>
                      <Switch
                        checked={config.auto_index}
                        onCheckedChange={(checked) => {
                          updateConfig.mutate({ ...config, auto_index: checked });
                        }}
                      />
                    </div>
                  </div>

                  {/* LEANN Settings */}
                  <div className="space-y-4">
                    <h3 className="text-sm font-medium">LEANN Configuration</h3>

                    <div className="space-y-3">
                      <div className="space-y-2">
                        <Label>Chat Model</Label>
                        <Input
                          value={config.leann.llm_model}
                          onChange={(e) => {
                            const newConfig = {
                              ...config,
                              leann: {
                                ...config.leann,
                                llm_model: e.target.value,
                              },
                            };
                            updateConfig.mutate(newConfig);
                          }}
                          placeholder="qwen3:1.7b"
                        />
                        <p className="text-xs text-muted-foreground">
                          Ollama model (e.g., qwen3:1.7b, llama3.2:3b,
                          functiongemma-270m-it)
                        </p>
                      </div>

                      <div className="space-y-2">
                        <Label>Embedding Model</Label>
                        <Input
                          value={config.leann.embedding_model}
                          onChange={(e) => {
                            const newConfig = {
                              ...config,
                              leann: {
                                ...config.leann,
                                embedding_model: e.target.value,
                              },
                            };
                            updateConfig.mutate(newConfig);
                          }}
                          placeholder="sentence-transformers/all-MiniLM-L6-v2"
                        />
                      </div>
                    </div>
                  </div>

                  {/* Model Suggestions */}
                  <div className="p-3 rounded-lg bg-blue-500/5 border border-blue-500/20">
                    <p className="text-sm font-medium mb-2">
                      Model Suggestions
                    </p>
                    <div className="space-y-2 text-xs">
                      <div>
                        <code className="text-blue-600 dark:text-blue-400">
                          functiongemma-270m-it
                        </code>
                        <span className="text-muted-foreground">
                          {" "}
                          - Small & fast (270M params)
                        </span>
                      </div>
                      <div>
                        <code className="text-blue-600 dark:text-blue-400">
                          llama3.2:3b
                        </code>
                        <span className="text-muted-foreground">
                          {" "}
                          - Balanced (3B params)
                        </span>
                      </div>
                      <div>
                        <code className="text-blue-600 dark:text-blue-400">
                          qwen3:1.7b
                        </code>
                        <span className="text-muted-foreground">
                          {" "}
                          - Recommended (1.7B params, default)
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              ) : null}
            </TabsContent>

            {/* Import Tab */}
            <TabsContent value="import" className="space-y-4 mt-4">
              <div className="space-y-4">
                {/* Notion Import */}
                <div className="p-4 rounded-lg border">
                  <div className="flex items-center gap-2 mb-3">
                    <Package className="h-5 w-5" />
                    <h3 className="font-medium">Import from Notion</h3>
                  </div>
                  <p className="text-sm text-muted-foreground mb-4">
                    Import your Notion pages and databases
                  </p>

                  <div className="space-y-4">
                    <div className="p-3 rounded-lg bg-muted/50 space-y-2">
                      <p className="text-xs font-medium">
                        How to export from Notion:
                      </p>
                      <ol className="text-xs text-muted-foreground space-y-1 list-decimal list-inside">
                        <li>Open your Notion workspace</li>
                        <li>Click the "..." menu on any page</li>
                        <li>Select "Export"</li>
                        <li>Choose "Markdown & CSV" format</li>
                        <li>Download the ZIP file</li>
                      </ol>
                    </div>

                    <div className="space-y-3">
                      <div className="space-y-2">
                        <Label className="text-sm">
                          Import Single Page (.md file)
                        </Label>
                        <div className="flex gap-2">
                          <Input
                            ref={fileInputRef}
                            type="file"
                            accept=".md"
                            onChange={handleNotionFileImport}
                            disabled={isImporting}
                            className="flex-1 text-sm"
                          />
                          <Button
                            onClick={() => fileInputRef.current?.click()}
                            disabled={isImporting}
                            variant="outline"
                            size="sm"
                          >
                            <FileText className="h-4 w-4 mr-2" />
                            Choose
                          </Button>
                        </div>
                      </div>

                      <div className="space-y-2">
                        <Label className="text-sm">
                          Import Full Export (.zip file)
                        </Label>
                        <div className="flex gap-2">
                          <Input
                            ref={zipInputRef}
                            type="file"
                            accept=".zip"
                            onChange={handleNotionZipImport}
                            disabled={isImporting}
                            className="flex-1 text-sm"
                          />
                          <Button
                            onClick={() => zipInputRef.current?.click()}
                            disabled={isImporting}
                            variant="outline"
                            size="sm"
                          >
                            <Package className="h-4 w-4 mr-2" />
                            Choose
                          </Button>
                        </div>
                      </div>

                      {isImporting && (
                        <div className="space-y-2">
                          <div className="flex items-center gap-2">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            <span className="text-sm">Importing...</span>
                          </div>
                          <Progress value={50} className="w-full" />
                        </div>
                      )}
                    </div>
                  </div>
                </div>

                {/* Other Import Options */}
                <div className="p-4 rounded-lg border border-dashed">
                  <p className="text-sm font-medium mb-2">Coming Soon</p>
                  <p className="text-xs text-muted-foreground mb-3">
                    Markdown files, Obsidian vault, and more
                  </p>
                  <Button variant="outline" size="sm" disabled>
                    <FileText className="h-4 w-4 mr-2" />
                    More Options
                  </Button>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </DialogContent>
    </Dialog>
  );
};
