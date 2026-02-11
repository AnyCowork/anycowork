import { useState, useRef, useCallback } from "react";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Search, PanelLeftClose, PanelLeft } from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { Agent } from "@/lib/anycowork-api";

interface CharacterPanelProps {
  agents: Agent[];
  selectedAgentId: string;
  onSelectCharacter: (agentId: string) => void;
  isCollapsed: boolean;
  onToggleCollapse: () => void;
}

function getCharacterEmoji(agent: Agent): string {
  return agent.avatar || agent.name.slice(0, 2).toUpperCase();
}

const MIN_WIDTH = 140;
const MAX_WIDTH = 320;
const DEFAULT_WIDTH = 200;

export function CharacterPanel({
  agents,
  selectedAgentId,
  onSelectCharacter,
  isCollapsed,
  onToggleCollapse,
}: CharacterPanelProps) {
  const [search, setSearch] = useState("");
  const [width, setWidth] = useState(DEFAULT_WIDTH);
  const isResizing = useRef(false);
  const panelRef = useRef<HTMLDivElement>(null);

  const filtered = search
    ? agents.filter(
        (a) =>
          a.name.toLowerCase().includes(search.toLowerCase()) ||
          (a.description || "").toLowerCase().includes(search.toLowerCase())
      )
    : agents;

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isResizing.current = true;
    const startX = e.clientX;
    const startWidth = width;

    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing.current) return;
      const delta = e.clientX - startX;
      const newWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
      setWidth(newWidth);
    };

    const handleMouseUp = () => {
      isResizing.current = false;
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, [width]);

  // Collapsed: icon-only strip
  if (isCollapsed) {
    return (
      <div className="w-11 border-r border-border/50 bg-card/50 flex flex-col h-full items-center py-1.5 gap-0.5">
        <button
          className="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/60 transition-colors mb-1"
          onClick={onToggleCollapse}
          title="Expand characters"
        >
          <PanelLeft className="h-3.5 w-3.5" />
        </button>
        <ScrollArea className="flex-1 w-full">
          <div className="flex flex-col items-center gap-0.5 px-1">
            <TooltipProvider delayDuration={150}>
              {agents.map((agent) => (
                <Tooltip key={agent.id}>
                  <TooltipTrigger asChild>
                    <button
                      onClick={() => onSelectCharacter(agent.id)}
                      className={cn(
                        "w-7 h-7 rounded-full flex items-center justify-center text-xs transition-all shrink-0",
                        "hover:bg-muted/80",
                        agent.id === selectedAgentId
                          ? "ring-[1.5px] ring-primary bg-primary/10"
                          : ""
                      )}
                    >
                      {getCharacterEmoji(agent)}
                    </button>
                  </TooltipTrigger>
                  <TooltipContent side="right" className="text-xs font-medium">
                    {agent.name}
                  </TooltipContent>
                </Tooltip>
              ))}
            </TooltipProvider>
          </div>
        </ScrollArea>
      </div>
    );
  }

  // Expanded: resizable panel
  return (
    <div
      ref={panelRef}
      className="relative border-r border-border/50 bg-card/50 flex flex-col h-full select-none"
      style={{ width }}
    >
      {/* Header */}
      <div className="flex items-center gap-1.5 px-2 py-1.5 border-b border-border/40">
        <button
          className="h-6 w-6 flex items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-muted/60 transition-colors shrink-0"
          onClick={onToggleCollapse}
          title="Collapse"
        >
          <PanelLeftClose className="h-3.5 w-3.5" />
        </button>
        <div className="flex-1 relative">
          <Search className="absolute left-1.5 top-1/2 -translate-y-1/2 h-3 w-3 text-muted-foreground/60 pointer-events-none" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search..."
            className="w-full h-6 pl-5 pr-1.5 text-xs bg-muted/40 border-0 rounded focus:outline-none focus:bg-muted/70 placeholder:text-muted-foreground/40 transition-colors"
          />
        </div>
      </div>

      {/* Character list */}
      <ScrollArea className="flex-1">
        <div className="py-0.5">
          {filtered.map((agent) => (
            <button
              key={agent.id}
              onClick={() => onSelectCharacter(agent.id)}
              className={cn(
                "w-full flex items-center gap-2 px-2 py-1.5 text-left transition-colors",
                "hover:bg-muted/50",
                agent.id === selectedAgentId
                  ? "bg-primary/8 text-foreground"
                  : "text-muted-foreground"
              )}
            >
              <span className="text-sm shrink-0 w-6 text-center">
                {getCharacterEmoji(agent)}
              </span>
              <span className="text-xs font-medium truncate flex-1">
                {agent.name}
              </span>
            </button>
          ))}
          {filtered.length === 0 && (
            <div className="px-3 py-4 text-xs text-muted-foreground/60 text-center">
              No matches
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Resize handle */}
      <div
        className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-primary/20 active:bg-primary/30 transition-colors z-10"
        onMouseDown={handleMouseDown}
      />
    </div>
  );
}
