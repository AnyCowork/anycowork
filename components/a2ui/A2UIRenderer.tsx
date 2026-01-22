/**
 * A2UI Renderer
 * Main renderer for A2UI surfaces
 */

import React from 'react';
import { A2UIMessage, A2UIProcessor, A2UISurface } from '@/src/lib/a2ui-processor';
import { ComponentRenderer } from './A2UIComponentRenderer';

interface A2UIRendererProps {
  messages: A2UIMessage[];
  onAction?: (actionName: string, context: any) => void;
  variant?: 'default' | 'minimal';
}

/**
 * Render a single A2UI surface
 */
function SurfaceRenderer({
  surface,
  onAction,
  variant = 'default',
}: {
  surface: A2UISurface;
  onAction?: (actionName: string, context: any) => void;
  variant?: 'default' | 'minimal';
}) {
  const rootComponent = surface.root ? surface.components.get(surface.root) : null;

  if (!rootComponent) {
    return (
      <div className="text-muted-foreground text-sm p-4 border rounded">
        No root component found for surface: {surface.id}
      </div>
    );
  }

  const context = {
    components: surface.components,
    dataModel: surface.dataModel,
    onAction,
  };

  const styles = surface.styles || {};
  const styleProps: React.CSSProperties = {};

  if (styles.primaryColor) {
    styleProps.borderColor = styles.primaryColor;
  }
  if (styles.font) {
    styleProps.fontFamily = styles.font;
  }

  const containerClass = variant === 'minimal'
    ? "a2ui-surface w-full align-top"
    : "a2ui-surface p-6 border rounded-xl bg-card shadow-sm hover:shadow-md transition-shadow";

  return (
    <div
      className={containerClass}
      style={styleProps}
    >
      <ComponentRenderer
        componentId={surface.root!}
        context={context}
      />
    </div>
  );
}

/**
 * Main A2UI Renderer component
 */
export function A2UIRenderer({ messages, onAction, variant = 'default' }: A2UIRendererProps & { variant?: 'default' | 'minimal' }) {
  const processor = React.useMemo(() => {
    const proc = new A2UIProcessor();
    proc.processMessages(messages);
    return proc;
  }, [messages]);

  const surfaces = processor.getAllSurfaces();

  if (surfaces.length === 0) {
    return null;
  }

  return (
    <div className="a2ui-container space-y-4">
      {surfaces.map((surface) => (
        <SurfaceRenderer
          key={surface.id}
          surface={surface}
          onAction={onAction}
          variant={variant}
        />
      ))}
    </div>
  );
}

// Legacy screen-based renderer for backward compatibility
export function A2UIScreenRenderer({ screen }: { screen: any }) {
  if (!screen || !screen.components) return null;

  return (
    <div className="a2ui-screen p-4 border rounded-lg bg-background">
      {screen.title && <h3 className="text-lg font-bold mb-4">{screen.title}</h3>}
      <div className="text-muted-foreground text-sm">
        Legacy screen format (not fully supported)
      </div>
    </div>
  );
}
