/**
 * Demo component showing how to upload and manage blobs
 */
import { useState, useRef } from "react";
import { useBlockBlobs } from "@/hooks/useBlockBlobs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Upload, Loader2 } from "lucide-react";
import { BlockBlobManager } from "./block-blob-manager";

interface BlobUploadDemoProps {
  blockId?: string;
}

export const BlobUploadDemo = ({ blockId }: BlobUploadDemoProps) => {
  const { uploadBlob, loading, error } = useBlockBlobs();
  const [uploadedBlobId, setUploadedBlobId] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = async (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      const blob = await uploadBlob(file, blockId);
      setUploadedBlobId(blob.id);

      // Reset input
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    } catch (err) {
      console.error("Upload failed:", err);
    }
  };

  return (
    <div className="space-y-4 p-4 border rounded-lg">
      <div>
        <h3 className="text-lg font-semibold mb-2">Upload Files to Block</h3>
        <p className="text-sm text-muted-foreground mb-4">
          Upload images and files that will be associated with this block.
        </p>
      </div>

      <div className="flex items-center gap-2">
        <Input
          ref={fileInputRef}
          type="file"
          onChange={handleFileSelect}
          disabled={loading}
          className="flex-1"
          accept="image/*,application/pdf,.doc,.docx,.txt,.md"
        />
        <Button
          onClick={() => fileInputRef.current?.click()}
          disabled={loading}
          size="sm"
        >
          {loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Upload className="h-4 w-4" />
          )}
        </Button>
      </div>

      {error && (
        <div className="text-sm text-destructive bg-destructive/10 p-2 rounded">
          {error}
        </div>
      )}

      {uploadedBlobId && (
        <div className="text-sm text-green-600 bg-green-50 dark:bg-green-900/20 p-2 rounded">
          File uploaded successfully!
        </div>
      )}

      {blockId && (
        <div className="mt-6">
          <BlockBlobManager blockId={blockId} />
        </div>
      )}
    </div>
  );
};
