---
name: theme-factory
description: Generate structured JSON theme configurations for UI components, presentations, or documents. Supports color palettes, typography, and spacing.
license: MIT
---

# Theme Factory Skill

This skill generates structured theme configurations in JSON format, suitable for modern UI frameworks (Tailwind, Material UI, Shadcn) or custom styling systems.

## Core Capabilities

1.  **Generate Themes**: Create comprehensive theme objects with colors, fonts, and spacing.
2.  **Color Palettes**: Generate accessible color schemes (primary, secondary, accent, neutral).
3.  **Typography Scales**: Define font families, sizes, and line heights.
4.  **Format Adaptation**: Output for Tailwind (`tailwind.config.js`), CSS variables (`:root`), or generic JSON.

## Workflow Examples

### 1. Generate JSON Theme

**Prompt**: "Create a dark mode theme for a developer dashboard."

**Output**:

```json
{
  "name": "dark-dev-dashboard",
  "colors": {
    "background": "#0f172a",
    "foreground": "#f8fafc",
    "primary": {
      "DEFAULT": "#3b82f6",
      "hover": "#2563eb",
      "active": "#1d4ed8"
    },
    "secondary": "#64748b",
    "accent": "#10b981",
    "error": "#ef4444",
    "warning": "#f59e0b",
    "success": "#22c55e"
  },
  "fonts": {
    "heading": "Inter, system-ui, sans-serif",
    "body": "Roboto Mono, monospace"
  },
  "spacing": {
    "sm": "0.5rem",
    "md": "1rem",
    "lg": "1.5rem",
    "xl": "2rem"
  },
  "borderRadius": {
    "default": "0.375rem",
    "card": "0.5rem"
  }
}
```

### 2. Generate Tailwind Config Fragment

**Prompt**: "Give me a tailwind color config for a 'sunset' theme."

**Output**:

```javascript
module.exports = {
  theme: {
    extend: {
      colors: {
        sunset: {
          50: '#fff7ed',
          100: '#ffedd5',
          200: '#fed7aa',
          300: '#fdba74',
          400: '#fb923c',
          500: '#f97316',
          600: '#ea580c',
          700: '#c2410c',
          800: '#9a3412',
          900: '#7c2d12',
        },
      }
    }
  }
}
```

## Best Practices

*   **Accessibility**: Ensure contrast ratios meet WCAG AA standards.
*   **Consistency**: Use standard naming conventions (primary, secondary, success, error).
*   **Scalability**: Include shades (50-900) for versatile usage.
