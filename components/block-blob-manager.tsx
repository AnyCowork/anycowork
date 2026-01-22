/**
 * Component for managing blobs (images/files) within a block
 */
import { useEffect, useState } from "react";
import { useBlockBlobs } from "@/hooks/useBlockBlobs";
import { Button } from "@/components/ui/button";
import { useConfirm } from "@/components/ui/confirm-dialog";
import {
  Trash2,
  Download,
  FileIcon,
  ImageIcon,
  FileText,
  FileSpreadsheet,
  FileCode,
  FileArchive,
  FileVideo,
  FileAudio,
  File,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface BlockBlobManagerProps {
  blockId: string;
  className?: string;
}

interface Blob {
  id: string;
  filename: string;
  size: number;
  mime_type: string;
  url: string;
  created_at: number;
}

export const BlockBlobManager = ({
  blockId,
  className,
}: BlockBlobManagerProps) => {
  const { blobs, loading, fetchBlockBlobs, deleteBlob } = useBlockBlobs();
  const [localBlobs, setLocalBlobs] = useState<Blob[]>([]);
  const { confirm, ConfirmDialog } = useConfirm();

  useEffect(() => {
    if (blockId) {
      fetchBlockBlobs(blockId).then(setLocalBlobs).catch(console.error);
    }
  }, [blockId, fetchBlockBlobs]);

  const handleDelete = async (blobId: string) => {
    const confirmed = await confirm("Delete this file?", {
      title: "Delete File",
      variant: "destructive",
    });
    if (confirmed) {
      try {
        await deleteBlob(blobId, false);
        setLocalBlobs((prev) => prev.filter((b) => b.id !== blobId));
      } catch (error) {
        console.error("Failed to delete blob:", error);
      }
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const getFileExtension = (filename: string): string => {
    const parts = filename.split(".");
    return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : "";
  };

  const getFileIcon = (filename: string, mimeType: string) => {
    const ext = getFileExtension(filename);

    // Images
    if (mimeType.startsWith("image/")) {
      return <ImageIcon className="h-5 w-5 text-blue-500" />;
    }

    // Videos
    if (
      mimeType.startsWith("video/") ||
      ["mp4", "webm", "avi", "mov", "mkv"].includes(ext)
    ) {
      return <FileVideo className="h-5 w-5 text-purple-500" />;
    }

    // Audio
    if (
      mimeType.startsWith("audio/") ||
      ["mp3", "wav", "ogg", "flac", "m4a"].includes(ext)
    ) {
      return <FileAudio className="h-5 w-5 text-pink-500" />;
    }

    // Documents
    if (["pdf", "doc", "docx", "txt", "rtf", "odt"].includes(ext)) {
      return <FileText className="h-5 w-5 text-red-500" />;
    }

    // Spreadsheets
    if (["xls", "xlsx", "csv", "ods"].includes(ext)) {
      return <FileSpreadsheet className="h-5 w-5 text-green-500" />;
    }

    // Code files
    if (
      [
        "js",
        "ts",
        "jsx",
        "tsx",
        "py",
        "java",
        "cpp",
        "c",
        "h",
        "css",
        "html",
        "json",
        "xml",
        "yaml",
        "yml",
        "sh",
        "bash",
      ].includes(ext)
    ) {
      return <FileCode className="h-5 w-5 text-yellow-500" />;
    }

    // Archives
    if (["zip", "rar", "tar", "gz", "7z", "bz2"].includes(ext)) {
      return <FileArchive className="h-5 w-5 text-orange-500" />;
    }

    // Default
    return <File className="h-5 w-5 text-gray-500" />;
  };

  if (loading && localBlobs.length === 0) {
    return (
      <div className="text-sm text-muted-foreground">Loading files...</div>
    );
  }

  if (localBlobs.length === 0) {
    return null;
  }

  return (
    <>
      <ConfirmDialog />
      <div className={cn("space-y-2", className)}>
        <div className="text-sm font-medium text-muted-foreground">
          Attached Files
        </div>
      <div className="grid grid-cols-1 gap-2">
        {localBlobs.map((blob) => (
          <div
            key={blob.id}
            className="flex items-center gap-3 p-3 border rounded-lg hover:bg-accent/50 transition-colors"
          >
            <div className="flex-shrink-0">
              {getFileIcon(blob.filename, blob.mime_type)}
            </div>

            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium truncate">
                {blob.filename}
              </div>
              <div className="text-xs text-muted-foreground">
                {formatFileSize(blob.size)} • {blob.mime_type}
              </div>
            </div>

            <div className="flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => window.open(blob.url, "_blank")}
                title="Download"
              >
                <Download className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleDelete(blob.id)}
                title="Delete"
              >
                <Trash2 className="h-4 w-4 text-destructive" />
              </Button>
            </div>
          </div>
        ))}
      </div>
      </div>
    </>
  );
};
