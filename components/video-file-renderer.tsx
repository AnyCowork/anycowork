/**
 * Custom renderer for video files in BlockNote
 * Since BlockNote doesn't have native video blocks, we render videos as enhanced file blocks
 */
interface VideoFileRendererProps {
  url: string;
  filename?: string;
  mimeType?: string;
}

export const VideoFileRenderer = ({
  url,
  filename,
  mimeType,
}: VideoFileRendererProps) => {
  const isVideo = (() => {
    if (mimeType?.startsWith("video/")) return true;
    if (filename) {
      const ext = filename.split(".").pop()?.toLowerCase();
      const videoExtensions = ["mp4", "webm", "ogg", "mov", "avi", "mkv"];
      if (ext && videoExtensions.includes(ext)) return true;
    }
    return false;
  })();

  if (!isVideo) {
    // Not a video, render as regular file link
    return (
      <a
        href={url}
        download={filename}
        className="inline-flex items-center gap-2 px-4 py-2 border rounded-lg hover:bg-accent"
      >
        <span>📎</span>
        <span>{filename || "Download file"}</span>
      </a>
    );
  }

  // Render video player
  return (
    <div className="my-4">
      <video
        controls
        className="w-full max-w-4xl rounded-lg shadow-lg"
        style={{ maxHeight: "600px" }}
      >
        <source src={url} type={mimeType || "video/mp4"} />
        Your browser doesn't support video playback.
      </video>
      {filename && (
        <div className="mt-2 text-sm text-muted-foreground">{filename}</div>
      )}
    </div>
  );
};
