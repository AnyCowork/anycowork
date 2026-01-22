/**
 * Navigation section types
 */
export type NavigationSection = "knowledge" | "chat" | "calendar";

export interface NavigationSectionConfig {
  id: NavigationSection;
  label: string;
  icon: string;
  path: string;
}

export const NAVIGATION_SECTIONS: NavigationSectionConfig[] = [
  {
    id: "knowledge",
    label: "Knowledge Base",
    icon: "BookOpen",
    path: "/documents",
  },
  {
    id: "chat",
    label: "Chat",
    icon: "MessageSquare",
    path: "/chat",
  },
  {
    id: "calendar",
    label: "Calendar",
    icon: "Calendar",
    path: "/calendar",
  },
];
