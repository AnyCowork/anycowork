# Native Web Components for Dialogs

This directory contains native web component implementations for common dialog patterns using Radix UI primitives.

## Confirm Dialog

A replacement for `window.confirm()` using native web components.

### Usage

```tsx
import { useConfirm } from "@/components/ui/confirm-dialog";

function MyComponent() {
  const { confirm, ConfirmDialog } = useConfirm();

  const handleDelete = async () => {
    const confirmed = await confirm("Are you sure you want to delete this item?", {
      title: "Delete Item",
      confirmText: "Delete",
      cancelText: "Cancel",
      variant: "destructive", // or "default"
    });

    if (confirmed) {
      // Perform delete action
    }
  };

  return (
    <>
      <ConfirmDialog />
      <button onClick={handleDelete}>Delete</button>
    </>
  );
}
```

### Options

- `title`: Dialog title (default: "Are you sure?")
- `confirmText`: Confirm button text (default: "Confirm")
- `cancelText`: Cancel button text (default: "Cancel")
- `variant`: Button style - "default" or "destructive" (default: "default")

## Prompt Dialog

A replacement for `window.prompt()` using native web components.

### Usage

```tsx
import { usePrompt } from "@/components/ui/prompt-dialog";

function MyComponent() {
  const { prompt, PromptDialog } = usePrompt();

  const handleRename = async () => {
    const newName = await prompt({
      title: "Rename Item",
      description: "Enter a new name for this item",
      label: "Name",
      placeholder: "Enter name...",
      defaultValue: "Current Name",
      confirmText: "Rename",
      cancelText: "Cancel",
    });

    if (newName !== null) {
      // User confirmed with value
      console.log("New name:", newName);
    } else {
      // User cancelled
    }
  };

  return (
    <>
      <PromptDialog />
      <button onClick={handleRename}>Rename</button>
    </>
  );
}
```

### Options

- `title`: Dialog title (default: "Input Required")
- `description`: Optional description text
- `label`: Label for input field
- `placeholder`: Input placeholder text
- `defaultValue`: Default input value
- `confirmText`: Confirm button text (default: "Submit")
- `cancelText`: Cancel button text (default: "Cancel")

## Benefits

- **Accessible**: Built on Radix UI primitives with proper ARIA attributes
- **Themeable**: Uses your existing design system tokens
- **Type-safe**: Full TypeScript support
- **Async/await**: Clean promise-based API
- **Customizable**: Extensive styling and configuration options
- **Keyboard navigation**: Full keyboard support out of the box
