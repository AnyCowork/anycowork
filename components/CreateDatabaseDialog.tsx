"use client";

import React, { useState } from "react";
import { X, Table as TableIcon, LayoutGrid } from "lucide-react";

interface CreateDatabaseDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (title: string, viewType: "table" | "board") => void;
}

export const CreateDatabaseDialog = ({
  isOpen,
  onClose,
  onCreate,
}: CreateDatabaseDialogProps) => {
  const [title, setTitle] = useState("");
  const [viewType, setViewType] = useState<"table" | "board">("table");

  if (!isOpen) return null;

  const handleCreate = () => {
    if (!title.trim()) {
      alert("Please enter a database name");
      return;
    }
    onCreate(title, viewType);
    setTitle("");
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-background border border-border rounded-lg shadow-lg w-full max-w-md p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Create Database</h2>
          <button
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">
              Database Name
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="My Database"
              className="w-full px-3 py-2 border border-border rounded-md bg-background"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">View Type</label>
            <div className="grid grid-cols-2 gap-2">
              <button
                onClick={() => setViewType("table")}
                className={`p-4 border rounded-md flex flex-col items-center gap-2 transition-colors ${
                  viewType === "table"
                    ? "border-accent bg-accent/10"
                    : "border-border hover:bg-muted"
                }`}
              >
                <TableIcon className="h-6 w-6" />
                <span className="text-sm font-medium">Table View</span>
              </button>
              <button
                onClick={() => setViewType("board")}
                className={`p-4 border rounded-md flex flex-col items-center gap-2 transition-colors ${
                  viewType === "board"
                    ? "border-accent bg-accent/10"
                    : "border-border hover:bg-muted"
                }`}
              >
                <LayoutGrid className="h-6 w-6" />
                <span className="text-sm font-medium">Board View</span>
              </button>
            </div>
          </div>

          <div className="flex gap-2 justify-end pt-4">
            <button
              onClick={onClose}
              className="px-4 py-2 border border-border rounded-md hover:bg-muted"
            >
              Cancel
            </button>
            <button
              onClick={handleCreate}
              className="px-4 py-2 bg-accent text-accent-foreground rounded-md hover:bg-accent/90"
            >
              Create
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
