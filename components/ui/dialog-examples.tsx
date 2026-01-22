/**
 * Example component demonstrating the usage of confirm and prompt dialogs
 * This can be used as a reference or in a component library/storybook
 */

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { usePrompt } from "@/components/ui/prompt-dialog";
import { Trash2, Edit, AlertTriangle, Info } from "lucide-react";

export function DialogExamples() {
  const { confirm, ConfirmDialog } = useConfirm();
  const { prompt, PromptDialog } = usePrompt();
  const [lastResult, setLastResult] = useState<string>("");

  // Example 1: Simple confirmation
  const handleSimpleConfirm = async () => {
    const confirmed = await confirm("Are you sure you want to proceed?");
    setLastResult(`Simple confirm: ${confirmed ? "Yes" : "No"}`);
  };

  // Example 2: Destructive action
  const handleDelete = async () => {
    const confirmed = await confirm("This action cannot be undone. Delete this item?", {
      title: "Delete Item",
      confirmText: "Delete",
      cancelText: "Cancel",
      variant: "destructive",
    });
    setLastResult(`Delete: ${confirmed ? "Deleted" : "Cancelled"}`);
  };

  // Example 3: Custom confirmation
  const handleCustomConfirm = async () => {
    const confirmed = await confirm(
      "This will affect all users in your organization. Continue?",
      {
        title: "Confirm Action",
        confirmText: "Yes, Continue",
        cancelText: "No, Go Back",
        variant: "default",
      }
    );
    setLastResult(`Custom confirm: ${confirmed ? "Confirmed" : "Cancelled"}`);
  };

  // Example 4: Simple prompt
  const handleSimplePrompt = async () => {
    const value = await prompt({
      title: "Enter Your Name",
      placeholder: "John Doe",
    });
    setLastResult(`Simple prompt: ${value !== null ? value : "Cancelled"}`);
  };

  // Example 5: Prompt with default value
  const handleRename = async () => {
    const newName = await prompt({
      title: "Rename Item",
      description: "Enter a new name for this item",
      label: "Item Name",
      placeholder: "Enter name...",
      defaultValue: "My Item",
      confirmText: "Rename",
      cancelText: "Cancel",
    });
    setLastResult(`Rename: ${newName !== null ? newName : "Cancelled"}`);
  };

  // Example 6: Prompt for sensitive data
  const handleApiKey = async () => {
    const apiKey = await prompt({
      title: "Enter API Key",
      description: "Your API key will be stored securely",
      label: "API Key",
      placeholder: "sk-...",
      confirmText: "Save",
      cancelText: "Cancel",
    });
    setLastResult(`API Key: ${apiKey !== null ? "Saved" : "Cancelled"}`);
  };

  return (
    <>
      <ConfirmDialog />
      <PromptDialog />
      
      <div className="container mx-auto p-8 space-y-6">
        <div>
          <h1 className="text-3xl font-bold mb-2">Dialog Components Examples</h1>
          <p className="text-muted-foreground">
            Native web component replacements for window.confirm() and window.prompt()
          </p>
        </div>

        {lastResult && (
          <Card className="border-primary">
            <CardHeader>
              <CardTitle className="text-sm">Last Result</CardTitle>
            </CardHeader>
            <CardContent>
              <code className="text-sm">{lastResult}</code>
            </CardContent>
          </Card>
        )}

        <div className="grid gap-6 md:grid-cols-2">
          {/* Confirm Dialog Examples */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <AlertTriangle className="h-5 w-5" />
                Confirm Dialogs
              </CardTitle>
              <CardDescription>
                Replacements for window.confirm() with better UX
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <Button onClick={handleSimpleConfirm} variant="outline" className="w-full">
                <Info className="h-4 w-4 mr-2" />
                Simple Confirmation
              </Button>
              
              <Button onClick={handleDelete} variant="destructive" className="w-full">
                <Trash2 className="h-4 w-4 mr-2" />
                Destructive Action
              </Button>
              
              <Button onClick={handleCustomConfirm} variant="default" className="w-full">
                Custom Confirmation
              </Button>
            </CardContent>
          </Card>

          {/* Prompt Dialog Examples */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Edit className="h-5 w-5" />
                Prompt Dialogs
              </CardTitle>
              <CardDescription>
                Replacements for window.prompt() with better UX
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <Button onClick={handleSimplePrompt} variant="outline" className="w-full">
                Simple Prompt
              </Button>
              
              <Button onClick={handleRename} variant="outline" className="w-full">
                <Edit className="h-4 w-4 mr-2" />
                Rename with Default
              </Button>
              
              <Button onClick={handleApiKey} variant="outline" className="w-full">
                Prompt for Sensitive Data
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Code Examples */}
        <Card>
          <CardHeader>
            <CardTitle>Usage Example</CardTitle>
          </CardHeader>
          <CardContent>
            <pre className="bg-muted p-4 rounded-lg overflow-x-auto text-sm">
              <code>{`import { useConfirm } from "@/components/ui/confirm-dialog";
import { usePrompt } from "@/components/ui/prompt-dialog";

function MyComponent() {
  const { confirm, ConfirmDialog } = useConfirm();
  const { prompt, PromptDialog } = usePrompt();

  const handleDelete = async () => {
    const confirmed = await confirm("Delete this item?", {
      title: "Delete Item",
      variant: "destructive",
    });
    if (confirmed) {
      // Delete the item
    }
  };

  const handleRename = async () => {
    const newName = await prompt({
      title: "Rename Item",
      defaultValue: "Current Name",
    });
    if (newName !== null) {
      // Rename the item
    }
  };

  return (
    <>
      <ConfirmDialog />
      <PromptDialog />
      {/* Your component content */}
    </>
  );
}`}</code>
            </pre>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
