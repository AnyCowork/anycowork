---
title: UI Design Guidelines
---

# AnyCowork UI Design Guidelines

**Design Philosophy**: Clean, Modern, Anthropic-Inspired  
**Last Updated**: 2026-01-17  
**Version**: 0.2.0

## Overview

AnyCowork's UI follows our specific design language: clean, modern, minimalist, with focus on clarity and functionality. The design emphasizes readability, intuitive interactions, and a calming aesthetic.

---

## Design Principles

### 1. Clarity First
- Clear hierarchy
- Obvious interactions
- Minimal cognitive load
- Purposeful elements only

### 2. Modern Minimalism
- Generous whitespace
- Clean typography
- Subtle animations
- No visual clutter

### 3. Thoughtful Interactions
- Smooth transitions
- Responsive feedback
- Progressive disclosure
- Contextual help

### 4. Accessibility
- High contrast ratios
- Keyboard navigation
- Screen reader support
- Focus indicators

---

## Color Palette

### Primary Colors (Current Implementation)

```css
/* Warm Neutrals - Beige/Cream Palette */
--bg-primary: #F5EFE7;        /* Main background - warm beige */
--bg-secondary: #EDE7DF;      /* Secondary background - darker beige */
--bg-tertiary: #E5DDD3;       /* Tertiary background */
--bg-sidebar: #F5EFE7;        /* Sidebar background - warm beige */
--bg-chat: #FDFCFB;           /* Chat area background - off-white */

/* Text Colors */
--text-primary: #2C2825;      /* Primary text - dark brown */
--text-secondary: #6B6560;    /* Secondary text - medium brown */
--text-tertiary: #9C9691;     /* Tertiary text - light brown */
--text-placeholder: #B8B3AE;  /* Placeholder text */

/* Accent - Coral/Salmon */
--accent-primary: #D97757;    /* Primary accent - coral/salmon */
--accent-hover: #C96847;      /* Hover state - darker coral */
--accent-light: #E89B7F;      /* Light accent */
--accent-subtle: #F5E5DF;     /* Subtle accent background */

/* Message Bubbles */
--message-user-bg: #F5E5DF;   /* User message background - light coral */
--message-user-text: #8B5A3C; /* User message text - brown */
--message-assistant-bg: #E8E3DB; /* Assistant message background - light gray-beige */
--message-assistant-text: #2C2825; /* Assistant message text */

/* Status Colors */
--success: #10B981;           /* Success - green */
--success-bg: #D1FAE5;        /* Success background */
--warning: #F59E0B;           /* Warning - amber */
--error: #EF4444;             /* Error - red */
--info: #3B82F6;              /* Info - blue */

/* Borders */
--border-subtle: #E5DDD3;     /* Subtle borders */
--border-default: #D6CFC5;    /* Default borders */
--border-strong: #B8B3AE;     /* Strong borders */

/* Avatar & Icons */
--avatar-bg: #D97757;         /* Avatar background - coral */
--avatar-text: #FFFFFF;       /* Avatar text/icon - white */
--icon-primary: #6B6560;      /* Primary icon color */
--icon-secondary: #9C9691;    /* Secondary icon color */
```

### Dark Mode Colors

```css
/* Dark Mode Backgrounds */
--bg-primary-dark: #0C0A09;   /* Main background - near black */
--bg-secondary-dark: #1C1917; /* Secondary background */
--bg-tertiary-dark: #292524;  /* Subtle backgrounds */

/* Dark Mode Text */
--text-primary-dark: #FAFAF9; /* Primary text - near white */
--text-secondary-dark: #A8A29E; /* Secondary text */
--text-tertiary-dark: #78716C; /* Tertiary text */

/* Dark Mode Accent (same) */
--accent-primary-dark: #F97316;
--accent-hover-dark: #FB923C;
--accent-subtle-dark: #431407;
```

---

## Typography

### Font Stack

```css
/* Primary Font - Sans Serif */
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI',
             Roboto, 'Helvetica Neue', Arial, sans-serif;

/* Monospace Font - Code */
--font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas,
             'Liberation Mono', Menlo, Courier, monospace;
```

### Type Scale

```css
/* Headings */
--text-4xl: 2.25rem;  /* 36px - Page titles */
--text-3xl: 1.875rem; /* 30px - Section titles */
--text-2xl: 1.5rem;   /* 24px - Card titles */
--text-xl: 1.25rem;   /* 20px - Subsection titles */
--text-lg: 1.125rem;  /* 18px - Emphasized text */

/* Body */
--text-base: 1rem;    /* 16px - Default body text */
--text-sm: 0.875rem;  /* 14px - Small text */
--text-xs: 0.75rem;   /* 12px - Captions, labels */
```

### Font Weights

```css
--font-light: 300;
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
--font-bold: 700;
```

### Line Heights

```css
--leading-tight: 1.25;
--leading-normal: 1.5;
--leading-relaxed: 1.75;
```

---

## Spacing System

Based on 4px base unit:

```css
--space-0: 0;
--space-1: 0.25rem;  /* 4px */
--space-2: 0.5rem;   /* 8px */
--space-3: 0.75rem;  /* 12px */
--space-4: 1rem;     /* 16px */
--space-5: 1.25rem;  /* 20px */
--space-6: 1.5rem;   /* 24px */
--space-8: 2rem;     /* 32px */
--space-10: 2.5rem;  /* 40px */
--space-12: 3rem;    /* 48px */
--space-16: 4rem;    /* 64px */
--space-20: 5rem;    /* 80px */
```

---

## Component Styles

### Buttons

```tsx
// Primary Button (Coral Accent)
<button className="
  px-4 py-2.5
  bg-[#D97757] hover:bg-[#C96847]
  text-white font-medium
  rounded-lg
  transition-colors duration-150
  focus:outline-none focus:ring-2 focus:ring-[#D97757] focus:ring-offset-2
">
  New Chat
</button>

// Secondary Button (Outline)
<button className="
  px-4 py-2
  border border-[#D6CFC5] hover:border-[#B8B3AE]
  text-[#2C2825]
  bg-transparent hover:bg-[#EDE7DF]
  rounded-lg
  transition-all duration-150
">
  Secondary Action
</button>

// Ghost Button (Sidebar Item)
<button className="
  w-full px-3 py-2
  text-[#6B6560] hover:text-[#2C2825]
  bg-transparent hover:bg-[#EDE7DF]
  rounded-lg text-left
  transition-all duration-150
  flex items-center gap-2
">
  <Icon className="w-4 h-4" />
  <span>Menu Item</span>
</button>
```

### Input Fields

```tsx
// Chat Input (Beige background)
<input
  type="text"
  className="
    w-full px-4 py-3
    bg-[#F5EFE7] border border-[#E5DDD3]
    rounded-xl
    text-[#2C2825] placeholder:text-[#B8B3AE]
    focus:border-[#D97757] focus:ring-2 focus:ring-[#F5E5DF]
    transition-all duration-150
  "
  placeholder="Type your message..."
/>

// Standard Input (White background)
<input
  type="text"
  className="
    w-full px-4 py-2.5
    bg-white border border-[#D6CFC5]
    rounded-lg
    text-[#2C2825] placeholder:text-[#B8B3AE]
    focus:border-[#D97757] focus:ring-2 focus:ring-[#F5E5DF]
    transition-all duration-150
  "
  placeholder="Enter text..."
/>

// Search Input
<div className="relative">
  <SearchIcon className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[#9C9691]" />
  <input
    type="search"
    className="
      w-full pl-10 pr-4 py-2
      bg-[#F5EFE7] border border-[#E5DDD3]
      rounded-lg
      text-[#2C2825] placeholder:text-[#B8B3AE]
      focus:border-[#D97757] focus:ring-2 focus:ring-[#F5E5DF]
    "
    placeholder="Search..."
  />
</div>
```

### Cards

```tsx
// Standard Card
<div className="
  bg-white
  border border-[#E5DDD3]
  rounded-xl
  p-6
  shadow-sm hover:shadow-md
  transition-shadow duration-200
">
  {/* Card content */}
</div>

// Chat History Item
<button className="
  w-full px-3 py-2.5
  bg-white hover:bg-[#EDE7DF]
  border border-[#E5DDD3]
  rounded-lg
  text-left
  transition-colors duration-150
">
  <div className="flex items-center gap-2">
    <MessageSquareIcon className="w-4 h-4 text-[#9C9691]" />
    <span className="text-sm text-[#2C2825] truncate">
      API Test Agent
    </span>
  </div>
</button>

// Agent Card (with status)
<div className="
  bg-white
  border border-[#E5DDD3]
  rounded-xl
  p-5
  hover:shadow-md
  transition-shadow duration-200
">
  <div className="flex items-start justify-between mb-3">
    <div className="flex items-center gap-3">
      <div className="w-10 h-10 bg-[#D97757] rounded-lg flex items-center justify-center">
        <BotIcon className="w-5 h-5 text-white" />
      </div>
      <div>
        <h3 className="text-sm font-semibold text-[#2C2825]">AI Assistant</h3>
        <p className="text-xs text-[#9C9691]">General purpose agent</p>
      </div>
    </div>
    <span className="
      inline-flex items-center gap-1
      px-2.5 py-1
      bg-[#D1FAE5] text-[#059669]
      text-xs font-medium
      rounded-full
    ">
      <span className="w-1.5 h-1.5 bg-[#10B981] rounded-full"></span>
      active
    </span>
  </div>
</div>
```

### Chat Message Bubbles

```tsx
// User message (right-aligned, coral tint)
<div className="flex justify-end mb-3">
  <div className="
    bg-[#F5E5DF]
    text-[#8B5A3C]
    rounded-2xl rounded-br-md
    px-4 py-3
    max-w-[80%]
    shadow-sm
  ">
    What is your name
  </div>
  <div className="
    w-8 h-8 ml-2 flex-shrink-0
    bg-[#D97757] rounded-full
    flex items-center justify-center
    text-white text-sm font-medium
  ">
    U
  </div>
</div>

// Assistant message (left-aligned, gray-beige)
<div className="flex justify-start mb-3">
  <div className="
    w-8 h-8 mr-2 flex-shrink-0
    bg-[#D97757] rounded-full
    flex items-center justify-center
    text-white text-sm
  ">
    <BotIcon className="w-4 h-4" />
  </div>
  <div className="
    bg-[#E8E3DB]
    text-[#2C2825]
    rounded-2xl rounded-bl-md
    px-4 py-3
    max-w-[80%]
    shadow-sm
  ">
    I do not have a name. I am a large language model, trained by Google.
  </div>
</div>

// Message with timestamp
<div className="flex items-end gap-2">
  <div className="message-bubble">
    Message content
  </div>
  <span className="text-xs text-[#9C9691]">
    9:02:07 AM
  </span>
</div>
```

### Status Badges

```tsx
// Active (Green)
<span className="
  inline-flex items-center gap-1
  px-2.5 py-1
  bg-[#D1FAE5] text-[#059669]
  text-xs font-medium
  rounded-full
">
  <span className="w-1.5 h-1.5 bg-[#10B981] rounded-full"></span>
  active
</span>

// Pending
<span className="
  px-2.5 py-1
  bg-amber-50 text-amber-700
  text-xs font-medium
  rounded-full
">
  Pending
</span>

// Error
<span className="
  px-2.5 py-1
  bg-red-50 text-red-700
  text-xs font-medium
  rounded-full
">
  Error
</span>
```

### Avatar Component

```tsx
// User Avatar (Coral background)
<div className="
  w-8 h-8
  bg-[#D97757] rounded-full
  flex items-center justify-center
  text-white text-sm font-medium
  shadow-sm
">
  U
</div>

// Agent Avatar (with icon)
<div className="
  w-8 h-8
  bg-[#D97757] rounded-full
  flex items-center justify-center
  text-white
  shadow-sm
">
  <BotIcon className="w-4 h-4" />
</div>

// Large Avatar (Profile)
<div className="
  w-12 h-12
  bg-[#D97757] rounded-full
  flex items-center justify-center
  text-white text-lg font-semibold
  shadow-md
">
  VA
</div>
```

---

## Layout Patterns

### Dashboard Layout

```tsx
<div className="min-h-screen bg-[#FDFCFB] flex">
  {/* Sidebar */}
  <aside className="
    fixed left-0 top-0 h-full w-64
    bg-[#F5EFE7]
    overflow-y-auto
    flex flex-col
  ">
    {/* Logo/Header */}
    <div className="p-4 border-b border-[#E5DDD3]">
      <div className="flex items-center gap-2">
        <div className="w-8 h-8 bg-[#D97757] rounded-lg flex items-center justify-center">
          <span className="text-white font-bold text-sm">AC</span>
        </div>
        <div>
          <h1 className="text-sm font-semibold text-[#2C2825]">AnyCowork</h1>
          <p className="text-xs text-[#9C9691]">Agents Platform</p>
        </div>
      </div>
    </div>

    {/* New Chat Button */}
    <div className="p-4">
      <button className="
        w-full px-4 py-2.5
        bg-[#D97757] hover:bg-[#C96847]
        text-white font-medium text-sm
        rounded-lg
        transition-colors duration-150
        flex items-center justify-center gap-2
      ">
        <PlusIcon className="w-4 h-4" />
        New Chat
      </button>
    </div>

    {/* Workspace Section */}
    <div className="px-4 py-2">
      <h2 className="text-xs font-semibold text-[#9C9691] uppercase tracking-wide mb-2">
        Workspace
      </h2>
      <nav className="space-y-1">
        <button className="
          w-full px-3 py-2
          text-[#2C2825] bg-[#EDE7DF]
          rounded-lg text-left text-sm
          flex items-center gap-2
        ">
          <MessageSquareIcon className="w-4 h-4" />
          Chat
        </button>
      </nav>
    </div>

    {/* Control Center Section */}
    <div className="px-4 py-2">
      <h2 className="text-xs font-semibold text-[#9C9691] uppercase tracking-wide mb-2">
        Control Center
      </h2>
      <nav className="space-y-1">
        <button className="
          w-full px-3 py-2
          text-[#6B6560] hover:bg-[#EDE7DF]
          rounded-lg text-left text-sm
          flex items-center gap-2
          transition-colors duration-150
        ">
          <UsersIcon className="w-4 h-4" />
          Agents
        </button>
        {/* More menu items */}
      </nav>
    </div>

    {/* Settings at bottom */}
    <div className="mt-auto p-4 border-t border-[#E5DDD3]">
      <button className="
        w-full px-3 py-2
        text-[#6B6560] hover:bg-[#EDE7DF]
        rounded-lg text-left text-sm
        flex items-center gap-2
      ">
        <SettingsIcon className="w-4 h-4" />
        Settings
      </button>
    </div>
  </aside>

  {/* Main content */}
  <main className="ml-64 flex-1">
    {/* Page content */}
  </main>
</div>
```

### Chat Interface

```tsx
<div className="flex h-screen bg-[#FDFCFB]">
  {/* Sidebar (see Dashboard Layout) */}
  
  {/* Chat Area */}
  <div className="flex-1 ml-64 flex flex-col">
    {/* Header */}
    <header className="
      flex-shrink-0
      bg-white
      px-6 py-4
      flex items-center justify-between
    ">
      <div className="flex items-center gap-3">
        <div className="w-8 h-8 bg-[#D97757] rounded-full flex items-center justify-center">
          <BotIcon className="w-4 h-4 text-white" />
        </div>
        <div>
          <h2 className="text-sm font-semibold text-[#2C2825]">AI Assistant</h2>
          <p className="text-xs text-[#9C9691]">Agent · 18h Test Agent</p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <span className="
          inline-flex items-center gap-1
          px-2.5 py-1
          bg-[#D1FAE5] text-[#059669]
          text-xs font-medium
          rounded-full
        ">
          <span className="w-1.5 h-1.5 bg-[#10B981] rounded-full"></span>
          active
        </span>
      </div>
    </header>

    {/* Messages */}
    <div className="
      flex-1 overflow-y-auto
      px-6 py-6
    ">
      <div className="max-w-3xl mx-auto space-y-4">
        {/* Assistant message */}
        <div className="flex justify-start">
          <div className="w-8 h-8 mr-2 flex-shrink-0 bg-[#D97757] rounded-full flex items-center justify-center">
            <BotIcon className="w-4 h-4 text-white" />
          </div>
          <div className="bg-[#E8E3DB] text-[#2C2825] rounded-2xl rounded-bl-md px-4 py-3 max-w-[80%] shadow-sm">
            <p>That looks like a random string of letters: is there something specific you'd like to do or ask about?</p>
          </div>
        </div>

        {/* User message */}
        <div className="flex justify-end">
          <div className="bg-[#F5E5DF] text-[#8B5A3C] rounded-2xl rounded-br-md px-4 py-3 max-w-[80%] shadow-sm">
            <p>What is your name</p>
          </div>
          <div className="w-8 h-8 ml-2 flex-shrink-0 bg-[#D97757] rounded-full flex items-center justify-center text-white text-sm font-medium">
            U
          </div>
        </div>
      </div>
    </div>

    {/* Input */}
    <div className="
      flex-shrink-0
      bg-white
      px-6 py-4
    ">
      <div className="max-w-3xl mx-auto">
        <div className="relative">
          <input
            type="text"
            placeholder="Type your message..."
            className="
              w-full px-4 py-3 pr-12
              bg-[#F5EFE7]
              border border-[#E5DDD3]
              rounded-xl
              text-[#2C2825] placeholder:text-[#B8B3AE]
              focus:outline-none focus:border-[#D97757] focus:ring-2 focus:ring-[#F5E5DF]
              transition-all duration-150
            "
          />
          <button className="
            absolute right-2 top-1/2 -translate-y-1/2
            w-8 h-8
            bg-[#D97757] hover:bg-[#C96847]
            rounded-lg
            flex items-center justify-center
            text-white
            transition-colors duration-150
          ">
            <SendIcon className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
```

---

## Current Implementation Notes

### Observed Design Patterns (v0.2.0)

Based on the current implementation, the following design patterns are in use:

#### Color Scheme
- **Primary Background**: Warm beige (#F5EFE7) for sidebar and input areas
- **Chat Background**: Off-white (#FDFCFB) for main chat area
- **Accent Color**: Coral/salmon (#D97757) for buttons, avatars, and active states
- **Message Bubbles**: 
  - User messages: Light coral tint (#F5E5DF)
  - Assistant messages: Light gray-beige (#E8E3DB)

#### Typography
- **Font Family**: System font stack (likely -apple-system, BlinkMacSystemFont, or similar)
- **Sidebar Labels**: Uppercase, small text with letter-spacing for section headers
- **Message Text**: Regular weight for body, medium weight for names/titles

#### Spacing & Layout
- **Sidebar Width**: ~256px (16rem)
- **Message Padding**: 16px horizontal, 12px vertical
- **Avatar Size**: 32px (2rem) for chat messages
- **Border Radius**: 
  - Buttons: 8px (rounded-lg)
  - Message bubbles: 16px with one sharp corner (rounded-2xl + rounded-bl/br-md)
  - Avatars: Full circle (rounded-full)

#### Interactive Elements
- **Hover States**: Subtle background color change on sidebar items
- **Active States**: Darker background for selected items
- **Focus States**: Ring with coral accent color
- **Transitions**: 150ms for most interactions

#### Component Hierarchy
1. **Sidebar** (Left, fixed)
   - Logo/branding at top
   - Primary action button (New Chat)
   - Workspace section
   - Control Center section
   - Settings at bottom

2. **Main Chat Area** (Right, flexible)
   - Header with agent info and status
   - Scrollable message area
   - Fixed input at bottom

#### Accessibility Features
- Clear visual hierarchy
- Sufficient color contrast
- Icon + text labels for navigation
- Status indicators with both color and text
- Rounded corners for better visual flow

---

## Animation & Transitions

### Timing Functions

```css
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in: cubic-bezier(0.4, 0, 1, 1);
```

### Durations

```css
--duration-fast: 150ms;
--duration-normal: 200ms;
--duration-slow: 300ms;
```

### Common Transitions

```css
/* Hover transitions */
.transition-hover {
  transition: all 150ms cubic-bezier(0.4, 0, 0.2, 1);
}

/* Fade in */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Slide up */
@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

---

## Icons

### Icon Library
Use **Lucide React** or **Heroicons**

```bash
npm install lucide-react
```

### Icon Usage

```tsx
import { MessageSquare, Settings, Users } from 'lucide-react'

// Default size: 20px
<MessageSquare className="w-5 h-5 text-text-secondary" />

// Large size: 24px
<Settings className="w-6 h-6 text-text-primary" />
```

---

## Responsive Design

### Breakpoints

```css
/* Mobile first approach */
--screen-sm: 640px;   /* Small devices */
--screen-md: 768px;   /* Tablets */
--screen-lg: 1024px;  /* Laptops */
--screen-xl: 1280px;  /* Desktops */
--screen-2xl: 1536px; /* Large screens */
```

### Responsive Patterns

```tsx
// Responsive text
<h1 className="text-2xl md:text-3xl lg:text-4xl">
  Responsive Heading
</h1>

// Responsive layout
<div className="
  grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3
  gap-4 md:gap-6 lg:gap-8
">
  {/* Grid items */}
</div>

// Responsive sidebar
<aside className="
  hidden lg:block
  lg:w-64
  lg:fixed lg:left-0 lg:top-0 lg:h-full
">
  {/* Sidebar */}
</aside>
```

---

## Accessibility

### Focus Indicators

```css
/* Default focus ring */
.focus-ring {
  @apply focus:outline-none focus:ring-2 focus:ring-accent-primary focus:ring-offset-2;
}

/* Focus-visible (keyboard only) */
.focus-visible-ring {
  @apply focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary;
}
```

### Semantic HTML

```tsx
// Use proper heading hierarchy
<h1>Page Title</h1>
<h2>Section Title</h2>
<h3>Subsection Title</h3>

// Use nav for navigation
<nav aria-label="Main navigation">
  {/* Navigation items */}
</nav>

// Use main for main content
<main>
  {/* Page content */}
</main>

// Use proper button/link semantics
<button type="button">Action</button>
<a href="/page">Link</a>
```

### ARIA Labels

```tsx
// Icon-only buttons
<button aria-label="Close dialog">
  <X className="w-5 h-5" />
</button>

// Loading states
<div role="status" aria-live="polite">
  Loading...
</div>

// Form labels
<label htmlFor="email">Email</label>
<input id="email" type="email" />
```

---

## Component Library

### Recommended Stack

```json
{
  "ui": "shadcn/ui",
  "styling": "Tailwind CSS",
  "icons": "lucide-react",
  "animations": "framer-motion"
}
```

### Installation

```bash
# Tailwind CSS
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p

# shadcn/ui
npx shadcn-ui@latest init

# Icons
npm install lucide-react

# Animations
npm install framer-motion
```

---

## Best Practices

### Do's ✅

- Use consistent spacing (4px grid)
- Maintain clear visual hierarchy
- Provide immediate feedback for interactions
- Use semantic HTML elements
- Test with keyboard navigation
- Ensure sufficient color contrast (WCAG AA)
- Use loading states for async operations
- Implement error boundaries
- Progressive enhancement

### Don'ts ❌

- Don't use bright, harsh colors
- Don't overcrowd the interface
- Don't rely on color alone for information
- Don't use tiny touch targets (<44px)
- Don't disable zoom on mobile
- Don't use autoplay media without controls
- Don't skip focus indicators
- Don't use ambiguous link text ("click here")

---

## Dark Mode Implementation

### CSS Variables Approach

```css
/* tailwind.config.js */
module.exports = {
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: 'var(--bg-primary)',
        foreground: 'var(--text-primary)',
        accent: 'var(--accent-primary)',
      }
    }
  }
}
```

### Toggle Component

```tsx
const ThemeToggle = () => {
  const [theme, setTheme] = useState('light')

  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light'
    setTheme(newTheme)
    document.documentElement.classList.toggle('dark')
  }

  return (
    <button onClick={toggleTheme} aria-label="Toggle theme">
      {theme === 'light' ? <Moon /> : <Sun />}
    </button>
  )
}
```

---

## Example Components

### Navigation Bar

```tsx
<nav className="
  flex items-center justify-between
  bg-white border-b border-border-subtle
  px-6 py-4
">
  {/* Logo */}
  <div className="flex items-center space-x-2">
    <Logo className="w-8 h-8 text-accent-primary" />
    <span className="text-xl font-semibold text-text-primary">
      AnyCowork
    </span>
  </div>

  {/* Navigation Links */}
  <div className="flex items-center space-x-6">
    <a href="/dashboard" className="text-text-secondary hover:text-text-primary">
      Dashboard
    </a>
    <a href="/agents" className="text-text-secondary hover:text-text-primary">
      Agents
    </a>
    <a href="/settings" className="text-text-secondary hover:text-text-primary">
      Settings
    </a>
  </div>

  {/* User Menu */}
  <button className="
    flex items-center space-x-2
    px-3 py-2 rounded-lg
    hover:bg-bg-secondary
  ">
    <Avatar />
    <ChevronDown className="w-4 h-4" />
  </button>
</nav>
```

### Agent Card

```tsx
<div className="
  bg-white border border-border-subtle
  rounded-xl p-6
  hover:shadow-md transition-shadow duration-200
">
  {/* Header */}
  <div className="flex items-start justify-between mb-4">
    <div>
      <h3 className="text-lg font-semibold text-text-primary mb-1">
        Code Assistant
      </h3>
      <p className="text-sm text-text-secondary">
        Helps with coding tasks
      </p>
    </div>
    <span className="
      px-2.5 py-1
      bg-green-50 text-green-700
      text-xs font-medium rounded-full
    ">
      Active
    </span>
  </div>

  {/* Stats */}
  <div className="grid grid-cols-2 gap-4 mb-4">
    <div>
      <p className="text-xs text-text-tertiary mb-1">Messages</p>
      <p className="text-2xl font-semibold text-text-primary">127</p>
    </div>
    <div>
      <p className="text-xs text-text-tertiary mb-1">Uptime</p>
      <p className="text-2xl font-semibold text-text-primary">4h</p>
    </div>
  </div>

  {/* Actions */}
  <div className="flex space-x-2">
    <button className="
      flex-1 px-4 py-2
      bg-accent-primary hover:bg-accent-hover
      text-white font-medium rounded-lg
      transition-colors duration-150
    ">
      Open Chat
    </button>
    <button className="
      px-4 py-2
      border border-border-default hover:bg-bg-secondary
      rounded-lg transition-colors duration-150
    " aria-label="Settings">
      <Settings className="w-5 h-5 text-text-secondary" />
    </button>
  </div>
</div>
```

---

## Resources

### Tools
- **Figma**: Design mockups
- **Tailwind CSS**: Utility-first CSS
- **shadcn/ui**: Component library
- **Lucide**: Icon library

### Documentation
- **Tailwind CSS**: https://tailwindcss.com/docs
- **shadcn/ui**: https://ui.shadcn.com
- **Lucide Icons**: https://lucide.dev
- **WCAG Guidelines**: https://www.w3.org/WAI/WCAG21/quickref/

---

**Last Updated**: 2026-01-17
**Maintained By**: AnyCowork Design Team
**Current Version**: 0.2.0 - Updated with actual implementation styling
