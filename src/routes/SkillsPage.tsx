/**
 * Skills Management Page - Browse and configure global skills
 * Skills are global components that can be used by any agent
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Wrench, Plus, Loader2, Download, CheckCircle2, FolderOpen, FileArchive, Search, Shield, Trash2, Eye, ArrowUpDown, Filter, Box, Zap, Container } from "lucide-react";
import { anycoworkApi, AgentSkill, MarketplaceSkill, SkillFile } from "@/lib/anycowork-api";
import { useState, useMemo } from "react";
import { useToast } from "@/hooks/useToast";
import { open } from "@tauri-apps/plugin-dialog";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

type SortOption = 'name' | 'date' | 'category';
type FilterStatus = 'all' | 'enabled' | 'disabled';

export default function SkillsPage() {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const { confirm, ConfirmDialog } = useConfirm();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedSkill, setSelectedSkill] = useState<AgentSkill | null>(null);
  const [selectedFile, setSelectedFile] = useState<SkillFile | null>(null);
  const [showDetailDialog, setShowDetailDialog] = useState(false);
  const [showImportDialog, setShowImportDialog] = useState(false);
  const [showFileDialog, setShowFileDialog] = useState(false);
  const [togglingSkillId, setTogglingSkillId] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<SortOption>('name');
  const [filterStatus, setFilterStatus] = useState<FilterStatus>('all');
  const [filterCategory, setFilterCategory] = useState<string>('all');

  // Fetch installed skills
  const { data: installedSkills, isLoading: loadingInstalled } = useQuery({
    queryKey: ['skills'],
    queryFn: () => anycoworkApi.listSkills()
  });

  // Fetch marketplace skills
  const { data: marketplaceSkills, isLoading: loadingMarketplace } = useQuery({
    queryKey: ['marketplace-skills'],
    queryFn: () => anycoworkApi.listMarketplaceSkills()
  });

  // Fetch skill files when a skill is selected
  const { data: skillFiles } = useQuery({
    queryKey: ['skill-files', selectedSkill?.id],
    queryFn: () => selectedSkill ? anycoworkApi.getSkillFiles(selectedSkill.id) : Promise.resolve([]),
    enabled: !!selectedSkill
  });

  // Check Docker availability
  const { data: dockerAvailable } = useQuery({
    queryKey: ['docker-available'],
    queryFn: () => anycoworkApi.checkDockerAvailable()
  });

  // Install skill mutation
  const installMutation = useMutation({
    mutationFn: (dirName: string) => anycoworkApi.installSkill(dirName),
    onSuccess: () => {
      toast({
        title: "Success",
        description: "Skill installed successfully",
      });
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['marketplace-skills'] });
    },
    onError: (error: any) => {
      toast({
        title: "Error",
        description: error.message || "Failed to install skill",
        variant: "destructive",
      });
    },
  });

  // Toggle skill mutation with loading state
  const toggleMutation = useMutation({
    mutationFn: (skillId: string) => anycoworkApi.toggleSkill(skillId),
    onSuccess: (updatedSkill) => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      setTogglingSkillId(null);
      toast({
        title: "Success",
        description: `Skill ${updatedSkill.enabled === 1 ? 'enabled' : 'disabled'} successfully`,
      });
    },
    onError: (error: any) => {
      toast({
        title: "Error",
        description: error.message || "Failed to toggle skill",
        variant: "destructive",
      });
      setTogglingSkillId(null);
    },
  });

  const handleToggleSkill = (skillId: string) => {
    setTogglingSkillId(skillId);
    toggleMutation.mutate(skillId);
  };

  // Delete skill mutation
  const deleteMutation = useMutation({
    mutationFn: (skillId: string) => anycoworkApi.deleteSkill(skillId),
    onSuccess: () => {
      toast({
        title: "Success",
        description: "Skill deleted successfully",
      });
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['marketplace-skills'] });
      setShowDetailDialog(false);
      setSelectedSkill(null);
    },
    onError: (error: any) => {
      toast({
        title: "Error",
        description: error.message || "Failed to delete skill",
        variant: "destructive",
      });
    },
  });

  const handleDeleteSkill = async (skillId: string, skillName: string) => {
    const confirmed = await confirm(
      `Are you sure you want to delete "${skillName}"? This action cannot be undone.`,
      {
        title: "Delete Skill",
        variant: "destructive",
      }
    );
    if (confirmed) {
      deleteMutation.mutate(skillId);
    }
  };

  // Import from directory mutation
  const importDirMutation = useMutation({
    mutationFn: (path: string) => anycoworkApi.importSkillFromDirectory(path),
    onSuccess: () => {
      toast({
        title: "Success",
        description: "Skill imported successfully",
      });
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      setShowImportDialog(false);
    },
    onError: (error: any) => {
      toast({
        title: "Error",
        description: error.message || "Failed to import skill",
        variant: "destructive",
      });
    },
  });

  // Import from ZIP mutation
  const importZipMutation = useMutation({
    mutationFn: (path: string) => anycoworkApi.importSkillFromZip(path),
    onSuccess: () => {
      toast({
        title: "Success",
        description: "Skill imported successfully",
      });
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      setShowImportDialog(false);
    },
    onError: (error: any) => {
      toast({
        title: "Error",
        description: error.message || "Failed to import skill",
        variant: "destructive",
      });
    },
  });

  // Handle import from directory
  const handleImportDirectory = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select skill directory"
    });
    if (selected && typeof selected === 'string') {
      importDirMutation.mutate(selected);
    }
  };

  // Handle import from ZIP
  const handleImportZip = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'ZIP Files', extensions: ['zip'] }],
      title: "Select skill ZIP file"
    });
    if (selected && typeof selected === 'string') {
      importZipMutation.mutate(selected);
    }
  };

  // Filter skills based on search
  const filteredInstalled = installedSkills?.filter((skill: AgentSkill) =>
    skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    skill.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
    skill.category?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  const filteredMarketplace = marketplaceSkills?.filter((skill: MarketplaceSkill) => {
    const matchesSearch = skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.category?.toLowerCase().includes(searchQuery.toLowerCase());
    // Hide already installed skills from marketplace
    return matchesSearch && !skill.is_installed;
  }) || [];

  // Apply status and category filters
  const statusFilteredSkills = useMemo(() => {
    let filtered = filteredInstalled;

    if (filterStatus === 'enabled') {
      filtered = filtered.filter(s => s.enabled === 1);
    } else if (filterStatus === 'disabled') {
      filtered = filtered.filter(s => s.enabled === 0);
    }

    if (filterCategory !== 'all') {
      filtered = filtered.filter(s => s.category === filterCategory);
    }

    return filtered;
  }, [filteredInstalled, filterStatus, filterCategory]);

  // Sort skills
  const sortedSkills = useMemo(() => {
    const sorted = [...statusFilteredSkills];

    switch (sortBy) {
      case 'name':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'date':
        sorted.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
        break;
      case 'category':
        sorted.sort((a, b) => (a.category || 'General').localeCompare(b.category || 'General'));
        break;
    }

    return sorted;
  }, [statusFilteredSkills, sortBy]);

  // Get all unique categories
  const allCategories = useMemo(() => {
    const categories = new Set<string>();
    installedSkills?.forEach(skill => {
      categories.add(skill.category || 'General');
    });
    return Array.from(categories).sort();
  }, [installedSkills]);

  // Group installed skills by category
  const skillsByCategory: Record<string, AgentSkill[]> = {};
  sortedSkills.forEach((skill: AgentSkill) => {
    const category = skill.category || "General";
    if (!skillsByCategory[category]) {
      skillsByCategory[category] = [];
    }
    skillsByCategory[category].push(skill);
  });
  const categories = Object.keys(skillsByCategory).sort();

  // Count tools in skill content
  const countTools = (skillContent: string): number => {
    const toolMatches = skillContent.match(/##\s*Tool\s*\d+:/gi);
    return toolMatches ? toolMatches.length : 0;
  };

  // Get execution mode badge
  const getExecutionModeBadge = (mode: string) => {
    switch (mode) {
      case 'sandbox':
        return { icon: Container, label: 'Sandbox', variant: 'secondary' as const };
      case 'direct':
        return { icon: Zap, label: 'Direct', variant: 'default' as const };
      case 'flexible':
      default:
        return { icon: Box, label: 'Flexible', variant: 'outline' as const };
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      <ConfirmDialog />
      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto max-w-7xl px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                <Wrench className="h-4 w-4 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-xl font-bold">Skills Management</h1>
                <p className="text-xs text-muted-foreground">
                  Browse and configure global skills available to all agents
                </p>
              </div>
            </div>
            <div className="flex items-center gap-3">
              {dockerAvailable && (
                <Badge variant="outline" className="gap-1.5">
                  <Shield className="h-3.5 w-3.5" />
                  Docker Available
                </Badge>
              )}
              <Dialog open={showImportDialog} onOpenChange={setShowImportDialog}>
                <DialogTrigger asChild>
                  <Button size="sm" className="gap-1.5">
                    <Plus className="h-3.5 w-3.5" />
                    Import Skill
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Import Skill</DialogTitle>
                    <DialogDescription>
                      Import a skill from a local directory or ZIP file
                    </DialogDescription>
                  </DialogHeader>
                  <div className="grid gap-4 py-4">
                    <Button
                      variant="outline"
                      className="h-24 flex-col gap-2"
                      onClick={handleImportDirectory}
                      disabled={importDirMutation.isPending}
                    >
                      <FolderOpen className="h-8 w-8" />
                      <span>Import from Directory</span>
                    </Button>
                    <Button
                      variant="outline"
                      className="h-24 flex-col gap-2"
                      onClick={handleImportZip}
                      disabled={importZipMutation.isPending}
                    >
                      <FileArchive className="h-8 w-8" />
                      <span>Import from ZIP</span>
                    </Button>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="mx-auto max-w-7xl p-6">
        <Tabs defaultValue="installed" className="space-y-6">
          <div className="flex items-center justify-between gap-4">
            <TabsList>
              <TabsTrigger value="installed">
                Installed ({sortedSkills.length})
              </TabsTrigger>
              <TabsTrigger value="marketplace">
                Marketplace ({filteredMarketplace.length})
              </TabsTrigger>
            </TabsList>

            <div className="flex items-center gap-2">
              {/* Search */}
              <div className="relative w-64">
                <Search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search skills..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-8"
                />
              </div>

              {/* Status Filter */}
              <Select value={filterStatus} onValueChange={(v) => setFilterStatus(v as FilterStatus)}>
                <SelectTrigger className="w-32">
                  <Filter className="h-4 w-4 mr-2" />
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="enabled">Enabled</SelectItem>
                  <SelectItem value="disabled">Disabled</SelectItem>
                </SelectContent>
              </Select>

              {/* Category Filter */}
              <Select value={filterCategory} onValueChange={setFilterCategory}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="Category" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Categories</SelectItem>
                  {allCategories.map(cat => (
                    <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                  ))}
                </SelectContent>
              </Select>

              {/* Sort */}
              <Select value={sortBy} onValueChange={(v) => setSortBy(v as SortOption)}>
                <SelectTrigger className="w-32">
                  <ArrowUpDown className="h-4 w-4 mr-2" />
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="name">Name</SelectItem>
                  <SelectItem value="date">Date Added</SelectItem>
                  <SelectItem value="category">Category</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          {/* Installed Skills Tab */}
          <TabsContent value="installed" className="space-y-6">
            {loadingInstalled ? (
              <div className="flex justify-center p-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : categories.length === 0 ? (
              <Card className="border-dashed">
                <CardContent className="flex flex-col items-center justify-center py-16">
                  <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted mb-3">
                    <Wrench className="h-8 w-8 text-muted-foreground" />
                  </div>
                  <h3 className="text-lg font-semibold">No skills installed</h3>
                  <p className="mt-2 text-sm text-muted-foreground text-center max-w-md">
                    Skills extend your agents with specialized capabilities. Browse the marketplace or import a custom skill to get started.
                  </p>
                  <div className="flex gap-3 mt-4">
                    <Button variant="outline" size="sm" onClick={() => setShowImportDialog(true)} className="gap-1.5">
                      <Plus className="h-3.5 w-3.5" />
                      Import Skill
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ) : (
              categories.map((category) => (
                <Card key={category}>
                  <CardHeader className="pb-3">
                    <div className="flex items-center gap-3">
                      <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
                        <Wrench className="h-4 w-4" />
                      </div>
                      <div>
                        <CardTitle>{category}</CardTitle>
                        <CardDescription>
                          {skillsByCategory[category].length} skills
                        </CardDescription>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                      {skillsByCategory[category].map((skill) => {
                        const toolCount = countTools(skill.skill_content);
                        const execMode = getExecutionModeBadge(skill.execution_mode);
                        const ExecutionIcon = execMode.icon;

                        return (
                          <Card
                            key={skill.id}
                            className="relative cursor-pointer hover:border-primary/50 transition-all hover:shadow-md"
                            onClick={() => {
                              setSelectedSkill(skill);
                              setShowDetailDialog(true);
                            }}
                          >
                            <CardHeader className="pb-3">
                              <div className="flex items-start justify-between gap-2">
                                <CardTitle className="text-base line-clamp-1">{skill.name}</CardTitle>
                                <div className="flex items-center gap-2 shrink-0" onClick={(e) => e.stopPropagation()}>
                                  <div className="relative">
                                    {togglingSkillId === skill.id && (
                                      <div className="absolute inset-0 flex items-center justify-center z-10">
                                        <Loader2 className="h-4 w-4 animate-spin text-primary" />
                                      </div>
                                    )}
                                    <Switch
                                      checked={skill.enabled === 1}
                                      onCheckedChange={() => handleToggleSkill(skill.id)}
                                      disabled={togglingSkillId === skill.id}
                                      className={togglingSkillId === skill.id ? "opacity-30" : ""}
                                    />
                                  </div>
                                </div>
                              </div>
                              <CardDescription className="line-clamp-2 min-h-[2.5rem]">
                                {skill.description}
                              </CardDescription>
                            </CardHeader>
                            <CardContent className="pt-0">
                              <div className="flex flex-wrap gap-2">
                                {/* Tool Count */}
                                {toolCount > 0 && (
                                  <TooltipProvider>
                                    <Tooltip>
                                      <TooltipTrigger asChild>
                                        <Badge variant="outline" className="gap-1 cursor-help">
                                          <Wrench className="h-3 w-3" />
                                          {toolCount}
                                        </Badge>
                                      </TooltipTrigger>
                                      <TooltipContent>
                                        <p>{toolCount} tool{toolCount !== 1 ? 's' : ''} available</p>
                                      </TooltipContent>
                                    </Tooltip>
                                  </TooltipProvider>
                                )}

                                {/* Execution Mode */}
                                <TooltipProvider>
                                  <Tooltip>
                                    <TooltipTrigger asChild>
                                      <Badge variant={execMode.variant} className="gap-1 cursor-help">
                                        <ExecutionIcon className="h-3 w-3" />
                                        {execMode.label}
                                      </Badge>
                                    </TooltipTrigger>
                                    <TooltipContent>
                                      <p>
                                        {skill.execution_mode === 'sandbox' && 'Runs in isolated Docker container'}
                                        {skill.execution_mode === 'direct' && 'Runs directly on host system'}
                                        {skill.execution_mode === 'flexible' && 'Adapts to agent settings'}
                                      </p>
                                    </TooltipContent>
                                  </Tooltip>
                                </TooltipProvider>

                                {/* Sandbox Required */}
                                {skill.requires_sandbox === 1 && (
                                  <TooltipProvider>
                                    <Tooltip>
                                      <TooltipTrigger asChild>
                                        <Badge variant="secondary" className="gap-1 cursor-help">
                                          <Shield className="h-3 w-3" />
                                          Sandbox
                                        </Badge>
                                      </TooltipTrigger>
                                      <TooltipContent>
                                        <p>Requires Docker sandbox</p>
                                      </TooltipContent>
                                    </Tooltip>
                                  </TooltipProvider>
                                )}
                              </div>
                            </CardContent>
                          </Card>
                        );
                      })}
                    </div>
                  </CardContent>
                </Card>
              ))
            )}
          </TabsContent>

          {/* Marketplace Tab */}
          <TabsContent value="marketplace" className="space-y-6">
            {loadingMarketplace ? (
              <div className="flex justify-center p-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : filteredMarketplace.length === 0 ? (
              <Card className="border-dashed">
                <CardContent className="flex flex-col items-center justify-center py-16">
                  <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted mb-3">
                    <Download className="h-8 w-8 text-muted-foreground" />
                  </div>
                  <h3 className="text-lg font-semibold">No skills found</h3>
                  <p className="mt-2 text-sm text-muted-foreground text-center max-w-md">
                    {searchQuery ? 'Try adjusting your search query.' : 'All marketplace skills are already installed.'}
                  </p>
                </CardContent>
              </Card>
            ) : (
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {filteredMarketplace.map((skill: MarketplaceSkill) => (
                  <Card key={skill.id} className="relative hover:border-primary/50 transition-all hover:shadow-md">
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          <CardTitle className="text-base line-clamp-1">{skill.name}</CardTitle>
                          {skill.category && (
                            <Badge variant="outline" className="mt-1">
                              {skill.category}
                            </Badge>
                          )}
                        </div>
                        <Button
                          size="sm"
                          variant="outline"
                          className="gap-1 shrink-0"
                          onClick={() => installMutation.mutate(skill.dir_name)}
                          disabled={installMutation.isPending}
                        >
                          {installMutation.isPending ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <Download className="h-3 w-3" />
                          )}
                          Install
                        </Button>
                      </div>
                      <CardDescription className="line-clamp-3 min-h-[3.75rem]">
                        {skill.description}
                      </CardDescription>
                    </CardHeader>
                  </Card>
                ))}
              </div>
            )}
          </TabsContent>
        </Tabs>
      </div>

      {/* Skill Detail Dialog */}
      <Dialog open={showDetailDialog} onOpenChange={setShowDetailDialog}>
        <DialogContent className="max-w-2xl max-h-[80vh]">
          {selectedSkill && (
            <>
              <DialogHeader>
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <DialogTitle className="text-xl">{selectedSkill.name}</DialogTitle>
                    <DialogDescription>
                      Version {selectedSkill.version} • {selectedSkill.category || "General"}
                    </DialogDescription>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    {selectedSkill.requires_sandbox === 1 && (
                      <Badge variant="secondary" className="gap-1">
                        <Shield className="h-3 w-3" />
                        Sandbox
                      </Badge>
                    )}
                    <Badge variant={selectedSkill.enabled === 1 ? "default" : "secondary"}>
                      {selectedSkill.enabled === 1 ? "Enabled" : "Disabled"}
                    </Badge>
                  </div>
                </div>
              </DialogHeader>

              <Separator />

              <ScrollArea className="h-[400px] pr-4">
                <div className="space-y-4">
                  <div>
                    <h4 className="font-medium mb-2">Description</h4>
                    <p className="text-sm text-muted-foreground">{selectedSkill.description}</p>
                  </div>

                  {/* Skill Metadata */}
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <h4 className="font-medium mb-2 text-sm">Execution Mode</h4>
                      <div className="flex items-center gap-2">
                        {(() => {
                          const execMode = getExecutionModeBadge(selectedSkill.execution_mode);
                          const ExecutionIcon = execMode.icon;
                          return (
                            <Badge variant={execMode.variant} className="gap-1">
                              <ExecutionIcon className="h-3 w-3" />
                              {execMode.label}
                            </Badge>
                          );
                        })()}
                      </div>
                    </div>
                    <div>
                      <h4 className="font-medium mb-2 text-sm">Tools</h4>
                      <Badge variant="outline" className="gap-1">
                        <Wrench className="h-3 w-3" />
                        {countTools(selectedSkill.skill_content)} tool{countTools(selectedSkill.skill_content) !== 1 ? 's' : ''}
                      </Badge>
                    </div>
                  </div>

                  <div>
                    <h4 className="font-medium mb-2">Skill Content</h4>
                    <div className="bg-muted rounded-lg p-4 max-h-48 overflow-auto">
                      <pre className="text-xs whitespace-pre-wrap font-mono">{selectedSkill.skill_content}</pre>
                    </div>
                  </div>

                  {skillFiles && skillFiles.length > 0 && (
                    <div>
                      <h4 className="font-medium mb-2">Bundled Files ({skillFiles.length})</h4>
                      <div className="space-y-2">
                        {skillFiles.map((file: SkillFile) => (
                          <div key={file.id} className="flex items-center justify-between bg-muted rounded-lg p-2 hover:bg-muted/80 transition-colors">
                            <div className="flex items-center gap-2 flex-1 min-w-0">
                              <Badge variant="outline" className="shrink-0">{file.file_type}</Badge>
                              <span className="text-sm truncate">{file.relative_path}</span>
                            </div>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                setSelectedFile(file);
                                setShowFileDialog(true);
                              }}
                            >
                              <Eye className="h-4 w-4" />
                            </Button>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {selectedSkill.source_path && (
                    <div>
                      <h4 className="font-medium mb-2">Source</h4>
                      <p className="text-sm text-muted-foreground font-mono break-all">{selectedSkill.source_path}</p>
                    </div>
                  )}

                  <div>
                    <h4 className="font-medium mb-2 text-sm">Created</h4>
                    <p className="text-sm text-muted-foreground">
                      {new Date(selectedSkill.created_at).toLocaleString()}
                    </p>
                  </div>
                </div>
              </ScrollArea>

              <DialogFooter className="gap-2 sm:gap-2">
                <Button variant="outline" onClick={() => setShowDetailDialog(false)}>
                  Close
                </Button>
                <Button
                  variant="destructive"
                  onClick={() => handleDeleteSkill(selectedSkill.id, selectedSkill.name)}
                  disabled={deleteMutation.isPending}
                  className="gap-2"
                >
                  {deleteMutation.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Trash2 className="h-4 w-4" />
                  )}
                  Delete Skill
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>

      {/* File Viewer Dialog */}
      <Dialog open={showFileDialog} onOpenChange={setShowFileDialog}>
        <DialogContent className="max-w-3xl max-h-[80vh]">
          {selectedFile && (
            <>
              <DialogHeader>
                <DialogTitle className="flex items-center gap-2">
                  <Badge variant="outline">{selectedFile.file_type}</Badge>
                  {selectedFile.relative_path}
                </DialogTitle>
                <DialogDescription>
                  Bundled file content
                </DialogDescription>
              </DialogHeader>

              <Separator />

              <ScrollArea className="h-[500px]">
                <div className="bg-muted rounded-lg p-4">
                  <pre className="text-xs whitespace-pre-wrap font-mono">{selectedFile.content}</pre>
                </div>
              </ScrollArea>

              <DialogFooter>
                <Button variant="outline" onClick={() => setShowFileDialog(false)}>
                  Close
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
