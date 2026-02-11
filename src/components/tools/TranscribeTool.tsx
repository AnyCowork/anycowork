import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { FileAudio, Mic, Upload, Loader2, AlertCircle, FileText, Download, Play } from 'lucide-react';
import { toast } from 'sonner';

interface DownloadProgress {
  downloaded: number;
  total: number;
  phase: 'downloading' | 'extracting' | 'done';
}

export function TranscribeTool() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcription, setTranscription] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Model state
  const [isModelReady, setIsModelReady] = useState(false);
  const [isCheckingModel, setIsCheckingModel] = useState(true);
  const [isDownloadingModel, setIsDownloadingModel] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);

  useEffect(() => {
    checkModel();
  }, []);

  // Listen for download progress events
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<DownloadProgress>('model-download-progress', (event) => {
      setDownloadProgress(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const checkModel = async () => {
    setIsCheckingModel(true);
    try {
      const ready = await invoke<boolean>('check_model_status');
      setIsModelReady(ready);
    } catch (err) {
      console.error('Failed to check model status:', err);
      setIsModelReady(false);
    } finally {
      setIsCheckingModel(false);
    }
  };

  const handleDownloadModel = async () => {
    setIsDownloadingModel(true);
    setDownloadProgress(null);
    try {
      await invoke('download_model');
      setIsModelReady(true);
      toast.success('Model downloaded successfully!');
    } catch (err) {
      console.error('Failed to download model:', err);
      toast.error('Failed to download model. Please check your internet connection.');
    } finally {
      setIsDownloadingModel(false);
      setDownloadProgress(null);
    }
  };

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Audio/Video',
          extensions: ['wav', 'mp3', 'm4a', 'ogg', 'flac', 'mp4', 'mov', 'mkv']
        }]
      });

      if (selected) {
        setFilePath(selected as string);
        setTranscription(null);
        setError(null);
      }
    } catch (err) {
      console.error('Failed to select file:', err);
      toast.error('Failed to select file');
    }
  };

  const handleTryExample = async () => {
    try {
      const path = await invoke<string>('get_sample_audio_path');
      setFilePath(path);
      setTranscription(null);
      setError(null);
    } catch (err) {
      console.error('Failed to load sample audio:', err);
      toast.error('Sample audio not available. Make sure you are running in development mode.');
    }
  };

  const handleTranscribe = async () => {
    if (!filePath) return;

    setIsTranscribing(true);
    setError(null);
    setTranscription(null);

    try {
      const text = await invoke<string>('transcribe_file', { path: filePath });
      setTranscription(text);
      toast.success('Transcription complete!');
    } catch (err) {
      console.error('Transcription failed:', err);
      setError(typeof err === 'string' ? err : 'Transcription failed.');
      toast.error('Transcription failed');
    } finally {
      setIsTranscribing(false);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  if (isCheckingModel) {
    return (
      <div className="flex items-center justify-center h-full min-h-[400px]">
        <Loader2 className="w-8 h-8 animate-spin text-primary" />
        <span className="ml-2 text-muted-foreground">Checking AI models...</span>
      </div>
    );
  }

  if (!isModelReady) {
    const progressPercent = downloadProgress && downloadProgress.total > 0
      ? Math.round((downloadProgress.downloaded / downloadProgress.total) * 100)
      : 0;

    return (
      <div className="flex flex-col h-full max-w-4xl mx-auto p-6 space-y-6">
        <div className="flex items-center gap-3 mb-2">
          <div className="h-8 w-8 rounded-lg bg-primary/10 text-primary flex items-center justify-center">
            <Mic className="h-4 w-4" />
          </div>
          <div>
            <h1 className="text-xl font-bold tracking-tight">Transcribe</h1>
            <p className="text-muted-foreground">Convert audio and video files to text using local AI models.</p>
          </div>
        </div>

        <Card className="border-primary/20 bg-primary/5">
          <CardHeader>
            <CardTitle>AI Model Required</CardTitle>
            <CardDescription>
              To transcribe audio locally on your device, we need to download the Parakeet AI model (~100MB).
              This only needs to be done once.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col items-center justify-center py-4">
            {isDownloadingModel && downloadProgress ? (
              <div className="w-full max-w-sm space-y-3">
                <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary rounded-full transition-all duration-300"
                    style={{ width: downloadProgress.phase === 'extracting' ? '100%' : `${progressPercent}%` }}
                  />
                </div>
                <p className="text-sm text-center text-muted-foreground">
                  {downloadProgress.phase === 'extracting' ? (
                    'Extracting model files...'
                  ) : downloadProgress.total > 0 ? (
                    `${formatBytes(downloadProgress.downloaded)} / ${formatBytes(downloadProgress.total)} (${progressPercent}%)`
                  ) : (
                    `${formatBytes(downloadProgress.downloaded)} downloaded...`
                  )}
                </p>
              </div>
            ) : (
              <>
                <div className="bg-background rounded-full p-2 mb-3 shadow-sm">
                  <Download className="w-5 h-5 text-primary" />
                </div>
                <p className="text-center max-w-md text-muted-foreground mb-6">
                  The model runs entirely offline, ensuring your privacy.
                </p>
              </>
            )}
          </CardContent>
          {!isDownloadingModel && (
            <CardFooter className="flex justify-center pb-8">
              <Button
                size="sm"
                onClick={handleDownloadModel}
                className="min-w-[200px]"
              >
                <Download className="mr-2 h-4 w-4" />
                Download Model
              </Button>
            </CardFooter>
          )}
        </Card>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full max-w-4xl mx-auto p-6 space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <div className="h-8 w-8 rounded-lg bg-primary/10 text-primary flex items-center justify-center">
          <Mic className="h-4 w-4" />
        </div>
        <div>
          <h1 className="text-xl font-bold tracking-tight">Transcribe</h1>
          <p className="text-muted-foreground">Convert audio and video files to text using local AI models.</p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Source File</CardTitle>
          <CardDescription>Select an audio or video file to transcribe.</CardDescription>
        </CardHeader>
        <CardContent>
          <div
            className={`border-2 border-dashed rounded-lg p-6 flex flex-col items-center justify-center transition-colors ${
              filePath ? 'border-primary/50 bg-primary/5' : 'border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50'
            }`}
          >
            {filePath ? (
              <div className="flex items-center gap-4 text-center flex-col">
                <div className="p-2 rounded-full bg-primary/10 text-primary">
                  <FileAudio className="w-5 h-5" />
                </div>
                <div>
                  <p className="font-medium text-lg breaking-all">{filePath.split(/[/\\]/).pop()}</p>
                  <p className="text-sm text-muted-foreground mt-1">{filePath}</p>
                </div>
                <Button variant="outline" onClick={handleSelectFile} className="mt-2">
                  Change File
                </Button>
              </div>
            ) : (
              <div className="text-center">
                <div className="flex gap-2 justify-center">
                  <Button onClick={handleSelectFile} size="sm" className="gap-2">
                    <Upload className="w-4 h-4" />
                    Select Media File
                  </Button>
                  <Button onClick={handleTryExample} size="sm" variant="outline" className="gap-2">
                    <Play className="w-4 h-4" />
                    Try Example
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground mt-4">
                  Supports WAV, MP3, M4A, OGG, FLAC, MP4, MOV, MKV
                </p>
              </div>
            )}
          </div>
        </CardContent>
        {filePath && (
          <CardFooter className="flex justify-end border-t pt-6 bg-muted/20">
            <Button
              onClick={handleTranscribe}
              disabled={isTranscribing}
              size="sm"
              className="gap-2 min-w-[150px]"
            >
              {isTranscribing ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Transcribing...
                </>
              ) : (
                <>
                  <Mic className="w-4 h-4" />
                  Start Transcription
                </>
              )}
            </Button>
          </CardFooter>
        )}
      </Card>

      {error && (
        <Card className="border-destructive/50 bg-destructive/5">
          <CardContent className="pt-6 flex items-start gap-4">
            <AlertCircle className="w-6 h-6 text-destructive shrink-0" />
            <div>
              <h3 className="font-medium text-destructive mb-1">Error Occurred</h3>
              <p className="text-sm text-destructive/90">{error}</p>
            </div>
          </CardContent>
        </Card>
      )}

      {transcription && (
        <Card className="flex-1 flex flex-col min-h-[300px]">
          <CardHeader className="flex flex-row items-center justify-between">
            <div className="flex items-center gap-2">
              <FileText className="w-5 h-5 text-primary" />
              <CardTitle>Transcription Result</CardTitle>
            </div>
            <Button variant="outline" size="sm" onClick={() => {
              navigator.clipboard.writeText(transcription);
              toast.success('Copied to clipboard');
            }}>
              Copy Text
            </Button>
          </CardHeader>
          <CardContent className="flex-1">
            <textarea
              className="w-full h-full min-h-[300px] p-4 rounded-md border bg-muted/30 font-mono text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary/50"
              value={transcription}
              readOnly
            />
          </CardContent>
        </Card>
      )}
    </div>
  );
}
