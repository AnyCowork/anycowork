import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Mic, ArrowRight, Box } from "lucide-react";

interface AppDefinition {
  id: string;
  name: string;
  description: string;
  icon: React.ElementType;
  path: string;
  status: "ready" | "coming-soon";
  badge?: string;
}

const APPS: AppDefinition[] = [
  {
    id: "transcribe",
    name: "Transcribe",
    description: "Convert audio and video files to text using local AI models. Powered by Whisper and other engines.",
    icon: Mic,
    path: "/apps/transcribe",
    status: "ready",
    badge: "New"
  },
  // Future apps can be added here
];

export default function AppsPage() {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      {/* Header */}
      <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto max-w-7xl px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80">
                <Box className="h-4 w-4 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-xl font-bold">Apps & Tools</h1>
                <p className="text-xs text-muted-foreground">
                  Productivity tools and AI-powered utilities ready to use
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="mx-auto max-w-7xl p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {APPS.map((app) => (
            <Card
              key={app.id}
              className="group cursor-pointer hover:shadow-md transition-all border-border/60 hover:border-primary/50 relative overflow-hidden"
              onClick={() => app.status === "ready" && navigate(app.path)}
            >
              {app.status === "ready" && (
                <div className="absolute inset-0 bg-gradient-to-br from-transparent via-transparent to-primary/5 opacity-0 group-hover:opacity-100 transition-opacity" />
              )}

              <CardHeader className="pb-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="p-2.5 rounded-xl bg-primary/10 text-primary group-hover:scale-105 transition-transform duration-300">
                    <app.icon className="w-6 h-6" />
                  </div>
                  {app.badge && (
                    <span className="px-2.5 py-0.5 rounded-full bg-primary text-[10px] font-bold text-primary-foreground uppercase tracking-wider">
                      {app.badge}
                    </span>
                  )}
                </div>
                <CardTitle className="text-xl group-hover:text-primary transition-colors">
                  {app.name}
                </CardTitle>
                <CardDescription className="line-clamp-2 mt-1.5">
                  {app.description}
                </CardDescription>
              </CardHeader>
              <CardContent>
                {app.status === "ready" ? (
                  <div className="flex items-center text-sm font-medium text-primary opacity-0 group-hover:opacity-100 transform translate-y-2 group-hover:translate-y-0 transition-all duration-300">
                    Open App <ArrowRight className="ml-1 w-4 h-4" />
                  </div>
                ) : (
                  <div className="flex items-center text-sm font-medium text-muted-foreground">
                    Coming Soon
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}
