import { useEffect, useState, type PointerEvent } from "react";
import { Minus, Square, X, Bug, Sun, Moon, Monitor } from "lucide-react";
import { Window } from "@tauri-apps/api/window";
import { anycoworkApi } from "@/lib/anycowork-api";
import { useTheme } from "@/components/providers/theme-provider";

export function TitleBar() {
    const [appWindow, setAppWindow] = useState<Window | null>(null);
    const [isDevMode, setIsDevMode] = useState(false);
    const { theme, setTheme, resolvedTheme } = useTheme();

    useEffect(() => {
        import("@tauri-apps/api/window").then((module) => {
            // In Tauri v2, getCurrentWindow returns the Window instance directly
            // @ts-ignore - Handle potential version mismatch or type issues
            const win = module.getCurrentWindow();
            setAppWindow(win);
        }).catch(err => {
            console.error("Failed to load Tauri window module:", err);
        });
    }, []);

    useEffect(() => {
        anycoworkApi.isDevMode().then(setIsDevMode).catch(() => setIsDevMode(false));
    }, []);

    const handleMinimize = () => appWindow?.minimize();
    const handleMaximize = () => appWindow?.toggleMaximize();
    const handleClose = () => appWindow?.close();
    const handleToggleDevtools = () => anycoworkApi.toggleDevtools();

    const handleToggleTheme = () => {
        const themes: Array<"light" | "dark" | "system"> = ["light", "dark", "system"];
        const currentIndex = themes.indexOf(theme);
        const nextIndex = (currentIndex + 1) % themes.length;
        setTheme(themes[nextIndex]);
    };

    const handleDrag = (e: PointerEvent) => {
        // Only drag if the target is the container itself or explicitly strictly marked
        // This prevents dragging when clicking buttons if they propagate
        const target = e.target as HTMLElement;
        // Don't start dragging if clicking on buttons
        if (target.tagName === 'BUTTON' || target.closest('button')) {
            return;
        }
        if (appWindow) {
            appWindow.startDragging();
        }
    };

    return (
        <div
            data-tauri-drag-region
            onPointerDown={handleDrag}
            className="h-8 flex justify-between items-center bg-muted/80 border-b border-border/60 select-none w-full cursor-default rounded-t-xl flex-shrink-0"
        >
            <div className="flex items-center px-3 pointer-events-none">
                <span className="text-xs font-semibold text-foreground/70">
                    AnyCowork
                </span>
            </div>

            <div className="flex h-full">
                {isDevMode && (
                    <button
                        onClick={handleToggleDevtools}
                        className="inline-flex justify-center items-center h-full w-9 hover:bg-accent focus:outline-none transition-colors"
                        title="Toggle Debug Console"
                    >
                        <Bug className="h-4 w-4" strokeWidth={1.5} />
                    </button>
                )}
                <button
                    onClick={handleToggleTheme}
                    className="inline-flex justify-center items-center h-full w-9 hover:bg-accent focus:outline-none transition-colors"
                    title={`Theme: ${theme} (click to cycle)`}
                >
                    {theme === "system" ? (
                        <Monitor className="h-4 w-4" strokeWidth={1.5} />
                    ) : resolvedTheme === "dark" ? (
                        <Moon className="h-4 w-4" strokeWidth={1.5} />
                    ) : (
                        <Sun className="h-4 w-4" strokeWidth={1.5} />
                    )}
                </button>
                <button
                    onClick={handleMinimize}
                    className="inline-flex justify-center items-center h-full w-9 hover:bg-accent focus:outline-none transition-colors"
                >
                    <Minus className="h-4 w-4" />
                </button>
                <button
                    onClick={handleMaximize}
                    className="inline-flex justify-center items-center h-full w-9 hover:bg-accent focus:outline-none transition-colors"
                >
                    <Square className="h-3 w-3" />
                </button>
                <button
                    onClick={handleClose}
                    className="inline-flex justify-center items-center h-full w-9 hover:bg-destructive hover:text-destructive-foreground focus:outline-none transition-colors rounded-tr-xl"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>
        </div>
    );
}
