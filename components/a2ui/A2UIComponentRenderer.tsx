/**
 * A2UI Component Renderer
 * Renders individual A2UI components based on the A2UI specification
 */

import React from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Checkbox } from '@/components/ui/checkbox';

interface ComponentDef {
  id: string;
  weight?: number;
  component: Record<string, any>;
}

interface RenderContext {
  components: Map<string, ComponentDef>;
  dataModel: any;
  onAction?: (actionName: string, context: any) => void;
}

/**
 * Resolve data binding path in the data model
 */
function resolveDataPath(dataModel: any, path: string): any {
  if (!path) return undefined;

  // Handle root path
  if (path === '/') return dataModel;

  // Remove leading slash and split
  const parts = path.replace(/^\//, '').split('/');
  let current = dataModel;

  for (const part of parts) {
    if (current === undefined || current === null) return undefined;
    current = current[part];
  }

  return current;
}

/**
 * Render text content with data binding support
 */
function renderText(textDef: any, dataModel: any): string {
  if (!textDef) return '';

  if (textDef.literalString) {
    return textDef.literalString;
  } else if (textDef.path) {
    const value = resolveDataPath(dataModel, textDef.path);
    return value !== undefined ? String(value) : '';
  }

  return '';
}

/**
 * Render an Icon component
 */
function renderIconComponent(props: any): React.ReactNode {
  const iconName = props.icon || 'circle';
  const size = props.size || 'medium';

  // Map size to pixel values
  const sizeMap = {
    small: 16,
    medium: 24,
    large: 32,
  };

  const pixelSize = sizeMap[size as keyof typeof sizeMap] || 24;

  // For now, use simple emoji/unicode icons as placeholders
  // In production, you'd use a proper icon library like lucide-react
  const iconMap: Record<string, string> = {
    check: '✓',
    close: '✕',
    info: 'ℹ',
    warning: '⚠',
    error: '✕',
    arrow_right: '→',
    arrow_left: '←',
    arrow_up: '↑',
    arrow_down: '↓',
    circle: '●',
    star: '★',
    heart: '♥',
    menu: '☰',
    settings: '⚙',
    search: '🔍',
    home: '🏠',
    user: '👤',
  };

  const icon = iconMap[iconName] || '●';

  return (
    <span
      className="inline-flex items-center justify-center text-muted-foreground"
      style={{ fontSize: `${pixelSize}px` }}
      title={iconName}
    >
      {icon}
    </span>
  );
}

/**
 * Render a Divider component
 */
function renderDividerComponent(props: any): React.ReactNode {
  const orientation = props.orientation || 'horizontal';

  if (orientation === 'vertical') {
    return (
      <div className="w-px h-full bg-border mx-2" />
    );
  }

  return (
    <hr className="my-4 border-border" />
  );
}

/**
 * Render a Spacer component
 */
function renderSpacerComponent(props: any): React.ReactNode {
  const size = props.size || 'medium';

  const sizeMap = {
    small: 'h-2',
    medium: 'h-4',
    large: 'h-8',
  };

  const className = sizeMap[size as keyof typeof sizeMap] || 'h-4';

  return <div className={className} />;
}

/**
 * Render a Text component
 */
function renderTextComponent(props: any, dataModel: any): React.ReactNode {
  const text = renderText(props.text, dataModel);
  const usageHint = props.usageHint || 'body';

  switch (usageHint) {
    case 'h1':
      return <h1 className="text-3xl font-bold mb-4 text-foreground">{text}</h1>;
    case 'h2':
      return <h2 className="text-2xl font-semibold mb-3 text-foreground">{text}</h2>;
    case 'h3':
      return <h3 className="text-xl font-semibold mb-2 text-foreground">{text}</h3>;
    case 'h4':
      return <h4 className="text-lg font-semibold mb-2 text-foreground">{text}</h4>;
    case 'caption':
      return <p className="text-sm text-muted-foreground mb-1">{text}</p>;
    case 'body':
    default:
      return <p className="mb-2 text-foreground leading-relaxed whitespace-pre-wrap">{text}</p>;
  }
}

/**
 * Render a Button component
 */
function renderButtonComponent(
  props: any,
  childId: string | undefined,
  context: RenderContext
): React.ReactNode {
  const isPrimary = props.primary === true;
  const action = props.action;

  const handleClick = () => {
    if (action && context.onAction) {
      context.onAction(action.name, action.context || []);
    }
  };

  // Render child component as button content
  let content = 'Button';
  if (childId) {
    const childComp = context.components.get(childId);
    if (childComp) {
      const componentType = Object.keys(childComp.component)[0];
      if (componentType === 'Text') {
        content = renderText(childComp.component.Text.text, context.dataModel);
      }
    }
  }

  return (
    <Button
      variant={isPrimary ? 'default' : 'secondary'}
      onClick={handleClick}
      className="mr-2 mb-2 transition-all hover:scale-105"
      size="default"
    >
      {content}
    </Button>
  );
}

/**
 * Render a Column component
 */
function renderColumnComponent(
  props: any,
  context: RenderContext
): React.ReactNode {
  const children = props.children;
  const alignment = props.alignment || 'start';

  let childIds: string[] = [];

  if (children?.explicitList) {
    childIds = children.explicitList;
  } else if (children?.template) {
    // Handle template-based rendering with data binding
    const templateId = children.template.componentId;
    const dataBinding = children.template.dataBinding;

    if (dataBinding) {
      const data = resolveDataPath(context.dataModel, dataBinding);
      if (data && typeof data === 'object') {
        childIds = Object.keys(data).map(key => `${templateId}-${key}`);
      }
    }
  }

  const alignmentClass = {
    start: 'items-start',
    center: 'items-center',
    end: 'items-end',
  }[alignment] || 'items-start';

  return (
    <div className={`flex flex-col ${alignmentClass} gap-2 w-full`}>
      {childIds.map((childId) => (
        <ComponentRenderer
          key={childId}
          componentId={childId}
          context={context}
        />
      ))}
    </div>
  );
}

/**
 * Render a Row component
 */
function renderRowComponent(
  props: any,
  context: RenderContext
): React.ReactNode {
  const children = props.children;
  const distribution = props.distribution || 'start';

  let childIds: string[] = [];

  if (children?.explicitList) {
    childIds = children.explicitList;
  }

  const distributionClass = {
    start: 'justify-start',
    center: 'justify-center',
    end: 'justify-end',
    spaceBetween: 'justify-between',
    spaceAround: 'justify-around',
  }[distribution] || 'justify-start';

  return (
    <div className={`flex flex-row ${distributionClass} gap-2 w-full`}>
      {childIds.map((childId) => (
        <ComponentRenderer
          key={childId}
          componentId={childId}
          context={context}
        />
      ))}
    </div>
  );
}

/**
 * Render a Card component
 */
function renderCardComponent(
  props: any,
  childId: string | undefined,
  context: RenderContext
): React.ReactNode {
  return (
    <Card className="mb-4 w-full">
      <CardContent className="pt-4">
        {childId && (
          <ComponentRenderer
            componentId={childId}
            context={context}
          />
        )}
      </CardContent>
    </Card>
  );
}

/**
 * Render a List component
 */
function renderListComponent(
  props: any,
  context: RenderContext
): React.ReactNode {
  const children = props.children;

  if (children?.template) {
    const templateId = children.template.componentId;
    const dataBinding = children.template.dataBinding;

    if (dataBinding) {
      const data = resolveDataPath(context.dataModel, dataBinding);

      if (data && typeof data === 'object') {
        const items = Object.entries(data);

        return (
          <div className="space-y-2 w-full">
            {items.map(([key, value]) => {
              // Create a scoped data model for this item
              const scopedContext = {
                ...context,
                dataModel: value,
              };

              return (
                <ComponentRenderer
                  key={key}
                  componentId={templateId}
                  context={scopedContext}
                />
              );
            })}
          </div>
        );
      }
    }
  }

  return null;
}

/**
 * Main component renderer
 */
export function ComponentRenderer({
  componentId,
  context,
}: {
  componentId: string;
  context: RenderContext;
}): React.ReactNode {
  const componentDef = context.components.get(componentId);

  if (!componentDef) {
    return <div className="text-red-500 text-sm">Component not found: {componentId}</div>;
  }

  const component = componentDef.component;
  const componentType = Object.keys(component)[0];
  const props = component[componentType];

  try {
    switch (componentType) {
      case 'Text':
        return renderTextComponent(props, context.dataModel);

      case 'Button':
        return renderButtonComponent(props, props.child, context);

      case 'Column':
        return renderColumnComponent(props, context);

      case 'Row':
        return renderRowComponent(props, context);

      case 'Card':
        return renderCardComponent(props, props.child, context);

      case 'List':
        return renderListComponent(props, context);

      case 'Icon':
        return renderIconComponent(props);

      case 'Divider':
        return renderDividerComponent(props);

      case 'Spacer':
        return renderSpacerComponent(props);

      default:
        // Log unsupported components to console instead of showing in UI
        console.warn(`Unsupported A2UI component: ${componentType}`, { componentId, props });
        return null; // Silently skip unsupported components
    }
  } catch (error) {
    console.error(`Error rendering component ${componentId}:`, error);
    return (
      <div className="text-red-500 text-sm border border-red-300 p-2 rounded">
        Error rendering {componentType}
      </div>
    );
  }
}
