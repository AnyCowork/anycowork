/**
 * Main Layout - Comprehensive agentic platform layout with sidebar
 */

import { Outlet } from "react-router-dom";
import { Sidebar } from "@/src/components/layout/Sidebar";
import { useIsDesktopApp } from "@/src/hooks/useIsDesktopApp";

import { TitleBar } from "@/src/components/layout/TitleBar";

function MainLayout() {
  const isDesktop = useIsDesktopApp();

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-background rounded-xl border border-border/50">
      {isDesktop && <TitleBar />}

      {/* Main app content with adjustment for titlebar */}
      <div className={`flex flex-1 overflow-hidden ${isDesktop ? 'pt-8' : ''}`}>
        {/* Sidebar Navigation */}
        <Sidebar />

        {/* Main content area */}
        <main className="flex-1 overflow-y-auto overflow-x-hidden relative">
          <Outlet />
        </main>
      </div>
    </div>
  );
}

export default MainLayout;
