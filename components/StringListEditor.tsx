import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Plus, Trash2, GripVertical } from "lucide-react";
import { useEffect, useState } from "react";

interface StringListEditorProps {
    initialData?: string[];
    onChange: (data: string[]) => void;
    placeholder?: string;
    addButtonText?: string;
}

interface StringItem {
    id: string;
    value: string;
}

export function StringListEditor({
    initialData = [],
    onChange,
    placeholder = "Value",
    addButtonText = "Add Item",
}: StringListEditorProps) {
    const [items, setItems] = useState<StringItem[]>(() =>
        initialData.map((value) => ({
            id: crypto.randomUUID(),
            value,
        }))
    );

    useEffect(() => {
        onChange(items.map((item) => item.value));
    }, [items]);

    const addItem = () => {
        setItems([...items, { id: crypto.randomUUID(), value: "" }]);
    };

    const removeItem = (id: string) => {
        setItems(items.filter((item) => item.id !== id));
    };

    const updateItem = (id: string, newValue: string) => {
        setItems(
            items.map((item) => (item.id === id ? { ...item, value: newValue } : item))
        );
    };

    return (
        <div className="space-y-2">
            <div className="space-y-2">
                {items.length === 0 && (
                    <div className="text-sm text-muted-foreground italic text-center py-2 border border-dashed rounded-md">
                        No items defined.
                    </div>
                )}
                {items.map((item, index) => (
                    <div key={item.id} className="flex gap-2 items-center group">
                        <div className="text-muted-foreground cursor-grab opacity-50 group-hover:opacity-100">
                            <GripVertical className="h-4 w-4" />
                        </div>
                        <Input
                            value={item.value}
                            onChange={(e) => updateItem(item.id, e.target.value)}
                            placeholder={placeholder}
                            className="flex-1 font-mono text-xs"
                        />
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => removeItem(item.id)}
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
                onClick={addItem}
                className="gap-2 w-full border-dashed"
            >
                <Plus className="h-3 w-3" />
                {addButtonText}
            </Button>
        </div>
    );
}
