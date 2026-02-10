/**
 * AnyCowork App - Root component with optimized routing and providers
 */

import { lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { QueryProvider } from "@/components/providers/query-provider";
import { ThemeProvider } from "@/components/providers/theme-provider";
import { Toaster } from "sonner";

// Eager load layout (critical path)
import MainLayout from "@/src/layouts/MainLayout";

// Lazy load other pages for code splitting
const SettingsPage = lazy(() => import("@/src/routes/SettingsPage"));
const AgentsPage = lazy(() => import("@/src/routes/AgentsPage"));
const ChatPage = lazy(() => import("@/src/routes/ChatPage"));
const SkillsPage = lazy(() => import("@/src/routes/SkillsPage"));
const MCPServersPage = lazy(() => import("@/src/routes/MCPServersPage"));
const ConversationsPage = lazy(() => import("@/src/routes/ConversationsPage"));
const AppsPage = lazy(() => import("@/src/routes/AppsPage"));
const TranscribeTool = lazy(() => import("@/src/components/tools/TranscribeTool").then(module => ({ default: module.TranscribeTool })));
// const ConnectionsPage = lazy(() => import("@/src/routes/ConnectionsPage"));

// Loading fallback component
function PageLoader() {
  return (
    <div className="flex items-center justify-center h-screen">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
    </div>
  );
}

function App() {
  return (
    <QueryProvider>
      <ThemeProvider
        attribute="class"
        defaultTheme="system"
        enableSystem
        disableTransitionOnChange
        storageKey="anycowork-theme"
      >
        <Toaster position="bottom-center" />
        <BrowserRouter
          future={{
            v7_startTransition: true,
            v7_relativeSplatPath: true,
          }}
        >
          <Suspense fallback={<PageLoader />}>
            <Routes>
              <Route path="/" element={<Navigate to="/chat" replace />} />
              <Route element={<MainLayout />}>
                <Route path="/chat/:sessionId?" element={<ChatPage />} />
                <Route path="/conversations" element={<ConversationsPage />} />
                <Route path="/agents" element={<AgentsPage />} />
                <Route path="/skills" element={<SkillsPage />} />
                <Route path="/mcp" element={<MCPServersPage />} />
                <Route path="/apps" element={<AppsPage />} />
                <Route path="/apps/transcribe" element={<TranscribeTool />} />
                {/* <Route path="/connections" element={<ConnectionsPage />} /> */}
                <Route path="/settings" element={<SettingsPage />} />
              </Route>
              <Route path="*" element={<Navigate to="/chat" replace />} />
            </Routes>
          </Suspense>
        </BrowserRouter>
      </ThemeProvider>
    </QueryProvider>
  );
}

export default App;
