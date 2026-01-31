import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Plus, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";

interface KeyValueEditorProps {
    initialData?: Record<string, string>;
    onChange: (data: Record<string, string>) => void;
    keyPlaceholder?: string;
    valuePlaceholder?: string;
    addButtonText?: string;
}

interface KeyValuePair {
    id: string; // internal id for react keys
    key: string;
    value: string;
}

export function KeyValueEditor({
    initialData = {},
    onChange,
    keyPlaceholder = "Key",
    valuePlaceholder = "Value",
    addButtonText = "Add Entry",
}: KeyValueEditorProps) {
    // Convert object to array for easier editing
    const [pairs, setPairs] = useState<KeyValuePair[]>(() =>
        Object.entries(initialData).map(([key, value]) => ({
            id: crypto.randomUUID(),
            key,
            value,
        }))
    );

    // Sync changes to parent
    useEffect(() => {
        const newData: Record<string, string> = {};
        pairs.forEach((pair) => {
            if (pair.key.trim()) {
                newData[pair.key.trim()] = pair.value;
            }
        });
        onChange(newData);
    }, [pairs]); // Ideally we might want to debounce this or only call on blur/change if it causes performance issues, but for small lists it's fine.

    const addPair = () => {
        setPairs([...pairs, { id: crypto.randomUUID(), key: "", value: "" }]);
    };

    const removePair = (id: string) => {
        setPairs(pairs.filter((p) => p.id !== id));
    };

    const updatePair = (id: string, field: "key" | "value", newValue: string) => {
        setPairs(
            pairs.map((p) => (p.id === id ? { ...p, [field]: newValue } : p))
        );
    };

    return (
        <div className="space-y-2">
            <div className="space-y-2">
                {pairs.length === 0 && (
                    <div className="text-sm text-muted-foreground italic text-center py-2 border border-dashed rounded-md">
                        No entries defined.
                    </div>
                )}
                {pairs.map((pair) => (
                    <div key={pair.id} className="flex gap-2 items-center">
                        <Input
                            value={pair.key}
                            onChange={(e) => updatePair(pair.id, "key", e.target.value)}
                            placeholder={keyPlaceholder}
                            className="flex-1 font-mono text-xs"
                        />
                        <Input
                            value={pair.value}
                            onChange={(e) => updatePair(pair.id, "value", e.target.value)}
                            placeholder={valuePlaceholder}
                            className="flex-1 font-mono text-xs"
                        />
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => removePair(pair.id)}
                            className="text-muted-foreground hover:text-destructive h-8 w-8"
                        >
                            <Trash2 className="h-4 w-4" />
                        </Button>
                    </div>
                ))}
            </div>
            <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={addPair}
                className="gap-2 w-full border-dashed"
            >
                <Plus className="h-3 w-3" />
                {addButtonText}
            </Button>
        </div>
    );
}
