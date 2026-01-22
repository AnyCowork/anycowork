/**
 * Skills Management Page - Browse and configure global skills
 * Skills are global components that can be used by any agent
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Wrench, Plus, Loader2, Download, CheckCircle2 } from "lucide-react";
import { anycoworkApi } from "@/lib/anycowork-api";
import { useState } from "react";
import { useToast } from "@/hooks/use-toast";

export default function SkillsPage() {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [showMarketplace, setShowMarketplace] = useState(false);

  // Fetch installed skills
  const { data: installedSkills, isLoading: loadingInstalled } = useQuery({
    queryKey: ['skills'],
    queryFn: () => anycoworkApi.listSkills()
  });

  // Fetch marketplace skills
  const { data: marketplaceSkills, isLoading: loadingMarketplace } = useQuery({
    queryKey: ['marketplace-skills'],
    queryFn: () => anycoworkApi.listMarketplaceSkills(),
    enabled: showMarketplace
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

  // Group installed skills by category
  const skillsByCategory: Record<string, any[]> = {};
  if (installedSkills) {
    installedSkills.forEach((skill: any) => {
      const category = skill.config?.category || "General";
      if (!skillsByCategory[category]) {
        skillsByCategory[category] = [];
      }
      skillsByCategory[category].push(skill);
    });
  }
  const categories = Object.keys(skillsByCategory);

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto max-w-7xl px-8 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                <Wrench className="h-5 w-5 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-2xl font-bold">Skills Management</h1>
                <p className="text-sm text-muted-foreground">
                  Browse and configure global skills available to all agents
                </p>
              </div>
            </div>
            <Button 
              size="lg" 
              className="gap-2"
              onClick={() => setShowMarketplace(!showMarketplace)}
            >
              <Plus className="h-4 w-4" />
              {showMarketplace ? "Show Installed" : "Browse Marketplace"}
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="mx-auto max-w-7xl p-8 space-y-8">
        {showMarketplace ? (
          // Marketplace View
          <>
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold">Available Skills</h2>
              <p className="text-sm text-muted-foreground">
                {marketplaceSkills?.length || 0} skills available
              </p>
            </div>
            {loadingMarketplace ? (
              <div className="flex justify-center p-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : (
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {marketplaceSkills?.map((skill: any) => (
                  <Card key={skill.id} className="relative">
                    <CardHeader>
                      <div className="flex items-start justify-between">
                        <CardTitle className="text-base">{skill.name}</CardTitle>
                        {skill.is_installed ? (
                          <Badge variant="default" className="gap-1">
                            <CheckCircle2 className="h-3 w-3" />
                            Installed
                          </Badge>
                        ) : (
                          <Button
                            size="sm"
                            variant="outline"
                            className="gap-1"
                            onClick={() => installMutation.mutate(skill.dir_name)}
                            disabled={installMutation.isPending}
                          >
                            <Download className="h-3 w-3" />
                            Install
                          </Button>
                        )}
                      </div>
                      <CardDescription>{skill.description}</CardDescription>
                    </CardHeader>
                  </Card>
                ))}
                {marketplaceSkills?.length === 0 && (
                  <div className="col-span-full text-center py-12 text-muted-foreground">
                    No skills available in marketplace.
                  </div>
                )}
              </div>
            )}
          </>
        ) : (
          // Installed Skills View
          <>
            {loadingInstalled ? (
              <div className="flex justify-center p-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : (
              <>
                {categories.length === 0 && (
                  <div className="text-center py-12 text-muted-foreground">
                    No skills installed yet. Browse the marketplace to add skills.
                  </div>
                )}
                {categories.map((category) => (
                  <Card key={category}>
                    <CardHeader>
                      <div className="flex items-center gap-3">
                        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
                          <Wrench className="h-4 w-4" />
                        </div>
                        <div>
                          <CardTitle>{category}</CardTitle>
                          <CardDescription>
                            {skillsByCategory[category].length} skills available
                          </CardDescription>
                        </div>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                        {skillsByCategory[category].map((skill) => (
                          <Card key={skill.id} className="relative">
                            <CardHeader>
                              <div className="flex items-start justify-between">
                                <CardTitle className="text-base">{skill.name}</CardTitle>
                                <Badge variant={skill.enabled ? "default" : "secondary"}>
                                  {skill.enabled ? "Enabled" : "Disabled"}
                                </Badge>
                              </div>
                              <CardDescription>{skill.description}</CardDescription>
                            </CardHeader>
                          </Card>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </>
            )}
          </>
        )}
      </div>
    </div>
  );
}
