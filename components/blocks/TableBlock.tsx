"use client";

import React, { useState } from "react";
import { Edit2, Save, X, Trash2, Plus, LayoutGrid, Table as TableIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface TableData {
  headers: string[];
  rows: string[][];
  hasHeader: boolean;
}

interface TableBlockProps {
  data: TableData;
  onChange?: (data: TableData) => void;
  readOnly?: boolean;
}

export const TableBlock = ({ data, onChange, readOnly = false }: TableBlockProps) => {
  const [isEditing, setIsEditing] = useState(false);
  const [localData, setLocalData] = useState<TableData>(data);
  const [viewMode, setViewMode] = useState<"table" | "kanban">("table");

  const handleSave = () => {
    onChange?.(localData);
    setIsEditing(false);
  };

  const handleCancel = () => {
    setLocalData(data);
    setIsEditing(false);
  };

  const updateCell = (rowIndex: number, colIndex: number, value: string) => {
    const newRows = [...localData.rows];
    newRows[rowIndex] = [...newRows[rowIndex]];
    newRows[rowIndex][colIndex] = value;
    setLocalData({ ...localData, rows: newRows });
  };

  const updateHeader = (colIndex: number, value: string) => {
    const newHeaders = [...localData.headers];
    newHeaders[colIndex] = value;
    setLocalData({ ...localData, headers: newHeaders });
  };

  const addRow = () => {
    const newRow = new Array(localData.headers.length).fill("");
    setLocalData({ ...localData, rows: [...localData.rows, newRow] });
  };

  const deleteRow = (rowIndex: number) => {
    if (localData.rows.length <= 1) return;
    const newRows = localData.rows.filter((_, i) => i !== rowIndex);
    setLocalData({ ...localData, rows: newRows });
  };

  const addColumn = () => {
    const newHeaders = [...localData.headers, `Column ${localData.headers.length + 1}`];
    const newRows = localData.rows.map((row) => [...row, ""]);
    setLocalData({ ...localData, headers: newHeaders, rows: newRows });
  };

  const deleteColumn = (colIndex: number) => {
    if (localData.headers.length <= 1) return;
    const newHeaders = localData.headers.filter((_, i) => i !== colIndex);
    const newRows = localData.rows.map((row) => row.filter((_, i) => i !== colIndex));
    setLocalData({ ...localData, headers: newHeaders, rows: newRows });
  };

  // Check if this table can be displayed as Kanban (has Status column)
  const statusColIndex = localData.headers.findIndex((h) =>
    h.toLowerCase().includes("status")
  );
  const canBeKanban = statusColIndex !== -1;

  const renderTableView = () => (
    <div className="w-full overflow-x-auto">
      <table className="min-w-full divide-y divide-border">
        <thead className="bg-muted/50">
          <tr>
            {localData.headers.map((header, colIndex) => (
              <th
                key={colIndex}
                className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase tracking-wider"
              >
                {isEditing ? (
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={header}
                      onChange={(e) => updateHeader(colIndex, e.target.value)}
                      className="flex-1 px-2 py-1 text-xs border border-border rounded bg-background"
                    />
                    <button
                      onClick={() => deleteColumn(colIndex)}
                      className="text-destructive hover:text-destructive/80"
                      title="Delete column"
                    >
                      <Trash2 className="h-3 w-3" />
                    </button>
                  </div>
                ) : (
                  header
                )}
              </th>
            ))}
            {isEditing && (
              <th className="px-4 py-3">
                <button
                  onClick={addColumn}
                  className="text-xs text-accent hover:text-accent/80 flex items-center gap-1"
                  title="Add column"
                >
                  <Plus className="h-3 w-3" />
                </button>
              </th>
            )}
          </tr>
        </thead>
        <tbody className="bg-background divide-y divide-border">
          {localData.rows.map((row, rowIndex) => (
            <tr key={rowIndex} className="hover:bg-muted/30">
              {row.map((cell, colIndex) => (
                <td key={colIndex} className="px-4 py-3 whitespace-nowrap text-sm">
                  {isEditing ? (
                    <input
                      type="text"
                      value={cell}
                      onChange={(e) => updateCell(rowIndex, colIndex, e.target.value)}
                      className="w-full px-2 py-1 text-sm border border-border rounded bg-background"
                    />
                  ) : (
                    <span className={cn(!cell && "text-muted-foreground italic")}>
                      {cell || "Empty"}
                    </span>
                  )}
                </td>
              ))}
              {isEditing && (
                <td className="px-4 py-3">
                  <button
                    onClick={() => deleteRow(rowIndex)}
                    className="text-destructive hover:text-destructive/80"
                    title="Delete row"
                  >
                    <Trash2 className="h-3 w-3" />
                  </button>
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>

      {isEditing && (
        <button
          onClick={addRow}
          className="mt-2 px-3 py-1 text-xs text-accent hover:text-accent/80 flex items-center gap-1"
        >
          <Plus className="h-3 w-3" />
          Add Row
        </button>
      )}
    </div>
  );

  const renderKanbanView = () => {
    if (statusColIndex === -1) return null;

    // Group rows by status
    const statusGroups: Record<string, string[][]> = {};
    localData.rows.forEach((row) => {
      const status = row[statusColIndex] || "No Status";
      if (!statusGroups[status]) {
        statusGroups[status] = [];
      }
      statusGroups[status].push(row);
    });

    // Get all unique statuses to ensure consistent column order
    const allStatuses = Array.from(new Set(localData.rows.map(row => row[statusColIndex] || "No Status")));

    // Add common statuses if they don't exist
    const commonStatuses = ["Not started", "In progress", "Done"];
    commonStatuses.forEach(status => {
      if (!allStatuses.includes(status)) {
        allStatuses.push(status);
        statusGroups[status] = [];
      }
    });

    const addNewCard = (status: string) => {
      if (readOnly) return;

      const newRow = new Array(localData.headers.length).fill("");
      newRow[statusColIndex] = status;

      // Set first non-status column to "Untitled"
      const firstContentCol = localData.headers.findIndex((_, idx) => idx !== statusColIndex);
      if (firstContentCol !== -1) {
        newRow[firstContentCol] = "Untitled";
      }

      setLocalData({ ...localData, rows: [...localData.rows, newRow] });
      if (onChange) {
        onChange({ ...localData, rows: [...localData.rows, newRow] });
      }
    };

    return (
      <div className="flex gap-4 overflow-x-auto pb-4 min-h-[400px]">
        {allStatuses.map((status) => {
          const rows = statusGroups[status] || [];
          return (
            <div key={status} className="flex-shrink-0 w-72">
              <div className="bg-muted/30 rounded-lg">
                {/* Column Header */}
                <div className="px-3 py-2 border-b border-border">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className={cn(
                        "w-2 h-2 rounded-full",
                        status === "Done" && "bg-green-500",
                        status === "In progress" && "bg-blue-500",
                        status === "Not started" && "bg-gray-400"
                      )} />
                      <span className="font-medium text-sm">{status}</span>
                    </div>
                    <span className="text-xs text-muted-foreground">{rows.length}</span>
                  </div>
                </div>

                {/* Cards */}
                <div className="p-2 space-y-2">
                  {rows.map((row, idx) => {
                    // Find the first non-status, non-empty column for the card title
                    let cardTitle = "Untitled";
                    const cardFields: Array<{ label: string; value: string }> = [];

                    localData.headers.forEach((header, colIndex) => {
                      if (colIndex === statusColIndex) return;
                      const value = row[colIndex];
                      if (value) {
                        if (cardTitle === "Untitled") {
                          cardTitle = value;
                        } else {
                          cardFields.push({ label: header, value });
                        }
                      }
                    });

                    return (
                      <div
                        key={idx}
                        className="bg-background border border-border rounded-md p-3 hover:border-accent/50 transition-colors cursor-pointer group"
                      >
                        <div className="font-medium text-sm mb-2">{cardTitle}</div>
                        {cardFields.length > 0 && (
                          <div className="space-y-1">
                            {cardFields.map((field, fieldIdx) => (
                              <div key={fieldIdx} className="text-xs">
                                <span className="text-muted-foreground">{field.label}: </span>
                                <span>{field.value}</span>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    );
                  })}

                  {/* Add New Card Button */}
                  {!readOnly && (
                    <button
                      onClick={() => addNewCard(status)}
                      className="w-full px-3 py-2 text-left text-sm text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded-md transition-colors flex items-center gap-2"
                    >
                      <Plus className="h-4 w-4" />
                      New page
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="my-4 border border-border rounded-lg overflow-hidden">
      {/* Toolbar */}
      <div className="bg-muted/30 px-4 py-2 border-b border-border flex items-center justify-between">
        <div className="flex items-center gap-3">
          {canBeKanban && !isEditing && (
            <div className="flex items-center gap-1 bg-background rounded-md p-1 border border-border">
              <button
                onClick={() => setViewMode("table")}
                className={cn(
                  "px-2 py-1 rounded text-xs flex items-center gap-1 transition-colors",
                  viewMode === "table"
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-muted"
                )}
              >
                <TableIcon className="h-3 w-3" />
                Table
              </button>
              <button
                onClick={() => setViewMode("kanban")}
                className={cn(
                  "px-2 py-1 rounded text-xs flex items-center gap-1 transition-colors",
                  viewMode === "kanban"
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-muted"
                )}
              >
                <LayoutGrid className="h-3 w-3" />
                Board
              </button>
            </div>
          )}
          {viewMode === "table" && (
            <span className="text-xs text-muted-foreground">
              {localData.rows.length} rows × {localData.headers.length} columns
            </span>
          )}
        </div>

        {!readOnly && viewMode === "table" && (
          <div className="flex items-center gap-2">
            {isEditing ? (
              <>
                <button
                  onClick={handleSave}
                  className="px-3 py-1 text-xs bg-accent hover:bg-accent/80 text-accent-foreground rounded flex items-center gap-1"
                >
                  <Save className="h-3 w-3" />
                  Save
                </button>
                <button
                  onClick={handleCancel}
                  className="px-3 py-1 text-xs border border-border hover:bg-muted rounded flex items-center gap-1"
                >
                  <X className="h-3 w-3" />
                  Cancel
                </button>
              </>
            ) : (
              <button
                onClick={() => setIsEditing(true)}
                className="px-3 py-1 text-xs border border-border hover:bg-muted rounded flex items-center gap-1"
              >
                <Edit2 className="h-3 w-3" />
                Edit
              </button>
            )}
          </div>
        )}
      </div>

      {/* Content */}
      <div className="p-4">
        {viewMode === "table" ? renderTableView() : renderKanbanView()}
      </div>
    </div>
  );
};
