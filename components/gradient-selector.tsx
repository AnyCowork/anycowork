import { cn } from "@/lib/utils";
import { Button } from "./ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { Palette } from "lucide-react";

interface GradientSelectorProps {
  onSelect: (gradient: string) => void;
  currentGradient?: string;
}

// Predefined gradient templates
export const GRADIENT_TEMPLATES = [
  {
    id: "gradient-1",
    name: "Ocean Blue",
    value: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
  },
  {
    id: "gradient-2",
    name: "Sunset",
    value: "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)",
  },
  {
    id: "gradient-3",
    name: "Forest",
    value: "linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)",
  },
  {
    id: "gradient-4",
    name: "Purple Dream",
    value: "linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)",
  },
  {
    id: "gradient-5",
    name: "Fire",
    value: "linear-gradient(135deg, #ff9a56 0%, #ff6a88 100%)",
  },
  {
    id: "gradient-6",
    name: "Mint",
    value: "linear-gradient(135deg, #81fbb8 0%, #28c76f 100%)",
  },
  {
    id: "gradient-7",
    name: "Lavender",
    value: "linear-gradient(135deg, #e0c3fc 0%, #8ec5fc 100%)",
  },
  {
    id: "gradient-8",
    name: "Peach",
    value: "linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%)",
  },
  {
    id: "gradient-9",
    name: "Sky",
    value: "linear-gradient(135deg, #a1c4fd 0%, #c2e9fb 100%)",
  },
  {
    id: "gradient-10",
    name: "Rose",
    value: "linear-gradient(135deg, #fbc2eb 0%, #a6c1ee 100%)",
  },
  {
    id: "gradient-11",
    name: "Emerald",
    value: "linear-gradient(135deg, #d299c2 0%, #fef9d7 100%)",
  },
  {
    id: "gradient-12",
    name: "Cosmic",
    value: "linear-gradient(135deg, #fa709a 0%, #fee140 100%)",
  },
  {
    id: "gradient-13",
    name: "Aurora",
    value: "linear-gradient(135deg, #30cfd0 0%, #330867 100%)",
  },
  {
    id: "gradient-14",
    name: "Candy",
    value: "linear-gradient(135deg, #ff6e7f 0%, #bfe9ff 100%)",
  },
  {
    id: "gradient-15",
    name: "Twilight",
    value: "linear-gradient(135deg, #4e54c8 0%, #8f94fb 100%)",
  },
  {
    id: "gradient-16",
    name: "Coral",
    value: "linear-gradient(135deg, #f857a6 0%, #ff5858 100%)",
  },
];

export const GradientSelector = ({
  onSelect,
  currentGradient,
}: GradientSelectorProps) => {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="text-xs">
          <Palette className="mr-2 h-4 w-4" />
          Choose gradient
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-80 p-3" align="start">
        <div className="space-y-2">
          <p className="text-sm font-medium">Select a gradient</p>
          <div className="grid grid-cols-4 gap-2">
            {GRADIENT_TEMPLATES.map((gradient) => (
              <button
                key={gradient.id}
                onClick={() => onSelect(gradient.value)}
                className={cn(
                  "h-12 w-full rounded-md border-2 transition-all hover:scale-105",
                  currentGradient === gradient.value
                    ? "border-primary ring-2 ring-primary ring-offset-2"
                    : "border-transparent hover:border-muted-foreground/20"
                )}
                style={{ background: gradient.value }}
                title={gradient.name}
              />
            ))}
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
};
