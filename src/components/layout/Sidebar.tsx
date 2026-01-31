/**
 * Sidebar Navigation - Claude/Anthropic inspired design
 * Comprehensive agentic platform navigation
 */

import { Link, useLocation } from "react-router-dom";
import { cn } from "@/lib/utils";
import {
  Home,
  MessageSquare,
  FolderOpen,
  Zap,
  Settings,
  Plus,
  Bot,
  Users,
  Wrench,
  Server,
  Sparkles,
  LayoutDashboard,
  BrainCircuit,
  Hammer,
  PanelLeft // Added PanelLeft
} from "lucide-react";
import * as React from "react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

interface NavItem {
  name: string;
  href: string;
  icon: React.ComponentType<{ className?: string }>;
}

const workspaceNavigation: NavItem[] = [
  { name: "Chat", href: "/chat", icon: MessageSquare },
];

const studioNavigation: NavItem[] = [
  { name: "Agents", href: "/agents", icon: Bot },
  { name: "Skills", href: "/skills", icon: Hammer },
  { name: "MCP Servers", href: "/mcp", icon: Server },
  // { name: "Connections", href: "/connections", icon: Users }, // Federation - Not fully implemented yet
];

const bottomNavigation: NavItem[] = [
  { name: "Settings", href: "/settings", icon: Settings },
];

export function Sidebar() {
  const location = useLocation();
  const [isCollapsed, setIsCollapsed] = React.useState(true);

  return (
    <div
      className={cn(
        "flex h-full flex-col border-r border-border/60 bg-muted/70 transition-all duration-300 ease-in-out",
        isCollapsed ? "w-16" : "w-64"
      )}
    >
      {/* Logo & Toggle Header */}
      <div className={cn("flex h-14 items-center border-b border-border/50 bg-background/30", isCollapsed ? "justify-center px-0" : "px-4 justify-between")}>
        <div className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80 shrink-0">
            <Sparkles className="h-4 w-4 text-primary-foreground" />
          </div>
          {!isCollapsed && (
            <div className="overflow-hidden whitespace-nowrap">
              <h1 className="text-sm font-semibold">AnyCowork</h1>
              <p className="text-xs text-muted-foreground">Your AI Coworker</p>
            </div>
          )}
        </div>

        {/* Toggle Button (Open State) */}
        {!isCollapsed && (
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground hover:text-foreground"
            onClick={() => setIsCollapsed(true)}
          >
            <PanelLeft className="h-4 w-4" />
          </Button>
        )}
      </div>

      {/* Toggle Button (Closed State) */}
      {isCollapsed && (
        <div className="flex justify-center p-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground hover:text-foreground"
            onClick={() => setIsCollapsed(false)}
          >
            <PanelLeft className="h-4 w-4" />
          </Button>
        </div>
      )}

      {/* Navigation Groups */}
      <nav className="flex-1 overflow-y-auto px-2 space-y-6 pt-2">

        {/* Workspace */}
        <div className="space-y-1">
          {workspaceNavigation.map((item) => {
            const isActive = location.pathname === item.href ||
              (item.href === "/chat" && location.pathname.startsWith("/chat"));

            return (
              <Link key={item.href} to={item.href}>
                <div
                  className={cn(
                    "flex items-center gap-3 rounded-lg py-2.5 text-sm font-medium transition-all duration-200",
                    isCollapsed ? "justify-center px-0" : "px-3",
                    isActive
                      ? "bg-primary/85 text-primary-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
                  )}
                  title={isCollapsed ? item.name : undefined}
                >
                  <item.icon className="h-4 w-4 shrink-0" />
                  {!isCollapsed && <span className="overflow-hidden whitespace-nowrap">{item.name}</span>}
                </div>
              </Link>
            );
          })}
        </div>

        {/* Studio */}
        <div className="space-y-1">
          {!isCollapsed && (
            <div className="px-2 py-1.5 text-xs font-semibold text-foreground/50 uppercase tracking-wider overflow-hidden whitespace-nowrap">
              Control Center
            </div>
          )}
          {studioNavigation.map((item) => {
            const isActive = location.pathname === item.href ||
              location.pathname.startsWith(item.href + "/");

            return (
              <Link key={item.href} to={item.href}>
                <div
                  className={cn(
                    "flex items-center gap-3 rounded-lg py-2 text-sm font-medium transition-all duration-200",
                    isCollapsed ? "justify-center px-0" : "px-3",
                    isActive
                      ? "bg-primary/15 text-primary shadow-sm"
                      : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
                  )}
                  title={isCollapsed ? item.name : undefined}
                >
                  <item.icon className="h-4 w-4 shrink-0" />
                  {!isCollapsed && <span className="overflow-hidden whitespace-nowrap">{item.name}</span>}
                </div>
              </Link>
            );
          })}
        </div>
      </nav>

      {/* Footer / Settings */}
      <div className="border-t border-border/50 p-2 space-y-1 bg-background/20">
        {bottomNavigation.map((item) => {
          const isActive = location.pathname === item.href;

          return (
            <Link key={item.href} to={item.href}>
              <div
                className={cn(
                  "flex items-center gap-3 rounded-lg py-2 text-sm font-medium transition-colors",
                  isCollapsed ? "justify-center px-0" : "px-3",
                  isActive
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:bg-background/60 hover:text-foreground"
                )}
                title={isCollapsed ? item.name : undefined}
              >
                <item.icon className="h-4 w-4 shrink-0" />
                {!isCollapsed && <span className="overflow-hidden whitespace-nowrap">{item.name}</span>}
              </div>
            </Link>
          );
        })}
        {!isCollapsed && (
          <div className="pt-2 px-3 pb-1">
            <p className="text-[10px] text-foreground/30 font-mono">
              v0.1.0-alpha
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
