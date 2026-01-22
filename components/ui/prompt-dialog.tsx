import * as React from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface PromptDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title?: string;
  description?: string;
  label?: string;
  placeholder?: string;
  defaultValue?: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: (value: string) => void;
}

export function PromptDialog({
  open,
  onOpenChange,
  title = "Input Required",
  description,
  label,
  placeholder = "",
  defaultValue = "",
  confirmText = "Submit",
  cancelText = "Cancel",
  onConfirm,
}: PromptDialogProps) {
  const [value, setValue] = React.useState(defaultValue);

  React.useEffect(() => {
    if (open) {
      setValue(defaultValue);
    }
  }, [open, defaultValue]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onConfirm(value);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
            {description && <DialogDescription>{description}</DialogDescription>}
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              {label && <Label htmlFor="prompt-input">{label}</Label>}
              <Input
                id="prompt-input"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                placeholder={placeholder}
                autoFocus
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              {cancelText}
            </Button>
            <Button type="submit">{confirmText}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

// Hook for imperative usage
export function usePrompt() {
  const [state, setState] = React.useState<{
    open: boolean;
    title?: string;
    description?: string;
    label?: string;
    placeholder?: string;
    defaultValue?: string;
    confirmText?: string;
    cancelText?: string;
    resolve?: (value: string | null) => void;
  }>({
    open: false,
  });

  const prompt = React.useCallback(
    (
      options: {
        title?: string;
        description?: string;
        label?: string;
        placeholder?: string;
        defaultValue?: string;
        confirmText?: string;
        cancelText?: string;
      } = {}
    ): Promise<string | null> => {
      return new Promise((resolve) => {
        setState({
          open: true,
          ...options,
          resolve,
        });
      });
    },
    []
  );

  const handleConfirm = React.useCallback(
    (value: string) => {
      state.resolve?.(value);
      setState((prev) => ({ ...prev, open: false }));
    },
    [state.resolve]
  );

  const handleCancel = React.useCallback(() => {
    state.resolve?.(null);
    setState((prev) => ({ ...prev, open: false }));
  }, [state.resolve]);

  const PromptDialogComponent = React.useCallback(
    () => (
      <PromptDialog
        open={state.open}
        onOpenChange={(open) => {
          if (!open) handleCancel();
        }}
        title={state.title}
        description={state.description}
        label={state.label}
        placeholder={state.placeholder}
        defaultValue={state.defaultValue}
        confirmText={state.confirmText}
        cancelText={state.cancelText}
        onConfirm={handleConfirm}
      />
    ),
    [state, handleConfirm, handleCancel]
  );

  return { prompt, PromptDialog: PromptDialogComponent };
}
