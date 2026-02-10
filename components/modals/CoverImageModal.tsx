import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { useCoverImage } from "@/hooks/useCoverImage";
import { SingleImageDropzone } from "@/components/single-image-dropzone";
import { useState } from "react";
import { useUpdateDocument } from "@/hooks/useDocuments";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GRADIENT_TEMPLATES } from "@/components/gradient-selector";
import { cn } from "@/lib/utils";

export const CoverImageModal = () => {
  const [file, setFile] = useState<File>();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const update = useUpdateDocument();
  const coverImage = useCoverImage();

  const onClose = () => {
    setFile(undefined);
    setIsSubmitting(false);
    coverImage.onClose();
  };

  const onChange = async (file?: File) => {
    if (file) {
      setIsSubmitting(true);
      setFile(file);

      // File upload disabled - EdgeStore removed
      console.warn("File upload is disabled");

      // Simply close the modal without uploading
      onClose();
    }
  };

  const onGradientSelect = (gradient: string) => {
    const documentId = coverImage.documentId;
    if (documentId) {
      update.mutate({
        id: documentId,
        coverImage: gradient,
      });
      onClose();
    }
  };

  return (
    <Dialog open={coverImage.isOpen} onOpenChange={coverImage.onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="text-center text-lg font-semibold">
            Cover Image
          </DialogTitle>
          <DialogDescription className="sr-only">
            Upload a cover image or choose a gradient for your document
          </DialogDescription>
        </DialogHeader>
        <Tabs defaultValue="gradient" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="gradient">Gradient</TabsTrigger>
            <TabsTrigger value="upload">Upload</TabsTrigger>
          </TabsList>
          <TabsContent value="gradient" className="mt-4">
            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">
                Select a gradient for your cover
              </p>
              <div className="grid grid-cols-4 gap-3">
                {GRADIENT_TEMPLATES.map((gradient) => (
                  <button
                    key={gradient.id}
                    onClick={() => onGradientSelect(gradient.value)}
                    className={cn(
                      "h-20 w-full rounded-md border-2 transition-all hover:scale-105 hover:border-primary",
                      "border-muted-foreground/20"
                    )}
                    style={{ background: gradient.value }}
                    title={gradient.name}
                  />
                ))}
              </div>
            </div>
          </TabsContent>
          <TabsContent value="upload" className="mt-4">
            <SingleImageDropzone
              className="w-full outline-none"
              disabled={isSubmitting}
              value={file}
              onChange={onChange}
            />
            {/* <div className="p-4 text-center border dashed">Upload disabled</div> */}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};
