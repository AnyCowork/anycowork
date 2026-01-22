import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";

interface ProviderSelectProps {
    value: string;
    onChange: (value: string) => void;
}

export function ProviderSelect({ value, onChange }: ProviderSelectProps) {
    return (
        <Select value={value} onValueChange={onChange}>
            <SelectTrigger className="w-[200px]">
                <SelectValue placeholder="Select provider" />
            </SelectTrigger>
            <SelectContent>
                <SelectItem value="anthropic">Anthropic (Claude)</SelectItem>
                <SelectItem value="openai">OpenAI (GPT)</SelectItem>
                <SelectItem value="gemini">Google (Gemini)</SelectItem>
            </SelectContent>
        </Select>
    );
}
