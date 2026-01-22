import { cn } from "@/lib/utils";
import { Button } from "./ui/button";
import { ImageIcon, X } from "lucide-react";
import { useCoverImage } from "@/hooks/useCoverImage";
import { useParams } from "react-router-dom";
import { Skeleton } from "./ui/skeleton";
import { useRemoveCoverImage } from "@/hooks/useDocuments";

interface CoverImageProps {
  url?: string;
  preview?: boolean;
}

export const Cover = ({ url, preview }: CoverImageProps) => {
  const params = useParams();
  const coverImage = useCoverImage();
  const removeCoverImage = useRemoveCoverImage();

  const onRemove = async () => {
    const documentId = params?.documentId as string | undefined;
    if (documentId) {
      removeCoverImage.mutate(documentId);
    }
  };

  // Check if the URL is a gradient (starts with linear-gradient or radial-gradient)
  const isGradient =
    url?.startsWith("linear-gradient") || url?.startsWith("radial-gradient");

  return (
    <div
      className={cn(
        "group relative h-[35vh] w-full",
        !url && "h-[12vh]",
        url && !isGradient && "bg-muted"
      )}
      style={isGradient ? { background: url } : undefined}
    >
      {url && !isGradient && (
        <img src={url} alt="cover" className="w-full h-full object-cover" />
      )}
      {url && !preview && (
        <div className="absolute bottom-5 left-1/2 -translate-x-1/2 flex items-center gap-x-2 opacity-0 group-hover:opacity-100">
          <Button
            onClick={() => {
              const documentId = params?.documentId as string | undefined;
              if (documentId) {
                coverImage.onOpen(documentId);
              }
            }}
            className="text-xs text-muted-foreground"
            variant="outline"
            size="sm"
          >
            <ImageIcon className="mr-2 h-4 w-4" />
            Change cover
          </Button>
          <Button
            onClick={onRemove}
            className="text-xs text-muted-foreground"
            variant="outline"
            size="sm"
          >
            <X className="mr-2 h-4 w-4" />
            Remove
          </Button>
        </div>
      )}
      {!url && !preview && (
        <div className="absolute bottom-5 left-1/2 -translate-x-1/2 flex items-center gap-x-2 opacity-0 group-hover:opacity-100">
          <Button
            onClick={() => {
              const documentId = params?.documentId as string | undefined;
              if (documentId) {
                coverImage.onOpen(documentId);
              }
            }}
            className="text-xs text-muted-foreground"
            variant="outline"
            size="sm"
          >
            <ImageIcon className="mr-2 h-4 w-4" />
            Add cover
          </Button>
        </div>
      )}
    </div>
  );
};

Cover.Skeleton = function CoverSkeleton() {
  return <Skeleton className="h-[12vh] w-full" />;
};
