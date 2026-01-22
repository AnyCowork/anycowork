import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Search, Sparkles, Filter } from 'lucide-react';

interface TemplateSummary {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  tags: string[];
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  file: string;
}

interface TemplateCategory {
  id: string;
  name: string;
  description: string;
}

interface TemplateCatalog {
  version: string;
  templates: TemplateSummary[];
  categories: TemplateCategory[];
}

interface AgentTemplatesProps {
  onSelectTemplate: (templateId: string) => void;
}

const difficultyColors = {
  beginner: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  intermediate: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  advanced: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
};

export const AgentTemplates: React.FC<AgentTemplatesProps> = ({ onSelectTemplate }) => {
  const [catalog, setCatalog] = useState<TemplateCatalog | null>(null);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedDifficulty, setSelectedDifficulty] = useState<string>('all');

  useEffect(() => {
    fetchCatalog();
  }, []);

  const fetchCatalog = async () => {
    try {
      const response = await fetch('/api/templates/catalog');
      const data = await response.json();
      setCatalog(data);
    } catch (error) {
      console.error('Failed to fetch template catalog:', error);
    } finally {
      setLoading(false);
    }
  };

  const filteredTemplates = catalog?.templates.filter((template) => {
    const matchesSearch =
      searchQuery === '' ||
      template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesCategory = selectedCategory === 'all' || template.category === selectedCategory;
    const matchesDifficulty = selectedDifficulty === 'all' || template.difficulty === selectedDifficulty;

    return matchesSearch && matchesCategory && matchesDifficulty;
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (!catalog) {
    return (
      <div className="text-center text-muted-foreground p-8">
        Failed to load templates. Please try again.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <Sparkles className="h-6 w-6 text-primary" />
          <h2 className="text-2xl font-bold">Agent Templates</h2>
        </div>
        <p className="text-muted-foreground">
          Choose from pre-configured agent templates to get started quickly
        </p>
      </div>

      {/* Search and Filters */}
      <div className="space-y-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search templates by name, description, or tags..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>

        <div className="flex flex-wrap gap-2">
          <div className="flex items-center gap-2">
            <Filter className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">Category:</span>
          </div>
          <Button
            variant={selectedCategory === 'all' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSelectedCategory('all')}
          >
            All
          </Button>
          {catalog.categories.map((category) => (
            <Button
              key={category.id}
              variant={selectedCategory === category.id ? 'default' : 'outline'}
              size="sm"
              onClick={() => setSelectedCategory(category.id)}
            >
              {category.name}
            </Button>
          ))}
        </div>

        <div className="flex flex-wrap gap-2">
          <span className="text-sm text-muted-foreground">Difficulty:</span>
          <Button
            variant={selectedDifficulty === 'all' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSelectedDifficulty('all')}
          >
            All
          </Button>
          {['beginner', 'intermediate', 'advanced'].map((difficulty) => (
            <Button
              key={difficulty}
              variant={selectedDifficulty === difficulty ? 'default' : 'outline'}
              size="sm"
              onClick={() => setSelectedDifficulty(difficulty)}
            >
              {difficulty.charAt(0).toUpperCase() + difficulty.slice(1)}
            </Button>
          ))}
        </div>
      </div>

      {/* Templates Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredTemplates?.map((template) => (
          <Card
            key={template.id}
            className="hover:shadow-lg transition-shadow cursor-pointer"
            onClick={() => onSelectTemplate(template.id)}
          >
            <CardHeader>
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-3xl">{template.icon}</span>
                  <CardTitle className="text-lg">{template.name}</CardTitle>
                </div>
                <Badge className={difficultyColors[template.difficulty]}>
                  {template.difficulty}
                </Badge>
              </div>
              <CardDescription className="line-clamp-2">{template.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-1">
                {template.tags.slice(0, 4).map((tag) => (
                  <Badge key={tag} variant="secondary" className="text-xs">
                    {tag}
                  </Badge>
                ))}
                {template.tags.length > 4 && (
                  <Badge variant="secondary" className="text-xs">
                    +{template.tags.length - 4}
                  </Badge>
                )}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {filteredTemplates?.length === 0 && (
        <div className="text-center text-muted-foreground p-8">
          No templates found matching your criteria.
        </div>
      )}
    </div>
  );
};

export default AgentTemplates;
