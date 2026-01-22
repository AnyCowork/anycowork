import * as React from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title?: string;
  description: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  variant?: "default" | "destructive";
}

export function ConfirmDialog({
  open,
  onOpenChange,
  title = "Are you sure?",
  description,
  confirmText = "Confirm",
  cancelText = "Cancel",
  onConfirm,
  variant = "default",
}: ConfirmDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{cancelText}</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            className={
              variant === "destructive"
                ? "bg-destructive text-destructive-foreground hover:bg-destructive/90"
                : ""
            }
          >
            {confirmText}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

// Hook for imperative usage
export function useConfirm() {
  const [state, setState] = React.useState<{
    open: boolean;
    title?: string;
    description: string;
    confirmText?: string;
    cancelText?: string;
    variant?: "default" | "destructive";
    resolve?: (value: boolean) => void;
  }>({
    open: false,
    description: "",
  });

  const confirm = React.useCallback(
    (
      description: string,
      options?: {
        title?: string;
        confirmText?: string;
        cancelText?: string;
        variant?: "default" | "destructive";
      }
    ): Promise<boolean> => {
      return new Promise((resolve) => {
        setState({
          open: true,
          description,
          title: options?.title,
          confirmText: options?.confirmText,
          cancelText: options?.cancelText,
          variant: options?.variant,
          resolve,
        });
      });
    },
    []
  );

  const handleConfirm = React.useCallback(() => {
    state.resolve?.(true);
    setState((prev) => ({ ...prev, open: false }));
  }, [state.resolve]);

  const handleCancel = React.useCallback(() => {
    state.resolve?.(false);
    setState((prev) => ({ ...prev, open: false }));
  }, [state.resolve]);

  const ConfirmDialogComponent = React.useCallback(
    () => (
      <ConfirmDialog
        open={state.open}
        onOpenChange={(open) => {
          if (!open) handleCancel();
        }}
        title={state.title}
        description={state.description}
        confirmText={state.confirmText}
        cancelText={state.cancelText}
        variant={state.variant}
        onConfirm={handleConfirm}
      />
    ),
    [state, handleConfirm, handleCancel]
  );

  return { confirm, ConfirmDialog: ConfirmDialogComponent };
}
