/**
 * Voice Call Dialog Component
 * Handles real-time voice calling with AI agents using Gemini Live API
 */

import React, { useState, useEffect, useRef } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Phone, PhoneOff, Mic, MicOff, Volume2, VolumeX, Loader2 } from 'lucide-react';
import { anycoworkApi } from '@/lib/anycowork-api';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import { useAIConfig } from '@/lib/hooks/use-anycowork';

interface VoiceCallDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  agentId: string;
  agentName: string;
  agentAvatar?: string;
}

type CallStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface TranscriptMessage {
  speaker: 'user' | 'agent';
  text: string;
  timestamp: Date;
}

export function VoiceCallDialog({
  open,
  onOpenChange,
  agentId,
  agentName,
  agentAvatar,
}: VoiceCallDialogProps) {
  const { data: aiConfig } = useAIConfig();
  const [callStatus, setCallStatus] = useState<CallStatus>('connecting');
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [isMuted, setIsMuted] = useState(false);
  const [isSpeakerOn, setIsSpeakerOn] = useState(true);
  const [transcript, setTranscript] = useState<TranscriptMessage[]>([]);
  const [isAgentSpeaking, setIsAgentSpeaking] = useState(false);

  const audioContextRef = useRef<AudioContext | null>(null);
  const mediaStreamRef = useRef<MediaStream | null>(null);
  const audioWorkletNodeRef = useRef<AudioWorkletNode | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const playbackQueueRef = useRef<Float32Array[]>([]);
  const isPlayingRef = useRef(false);

  // Initialize audio context and capture
  useEffect(() => {
    if (!open) return;

    const initializeCall = async () => {
      try {
        setCallStatus('connecting');

        // Get API key from backend settings
        const apiKey = aiConfig?.gemini_api_key;
        if (!apiKey) {
          toast.error('Gemini API key not configured. Please add it in Settings → AI Providers → Google (Gemini).');
          setCallStatus('error');
          return;
        }

        // Start voice call session
        const newSessionId = await anycoworkApi.startVoiceCall(agentId, apiKey);
        setSessionId(newSessionId);

        // Listen for voice call events
        const unlisten = await listen<any>('voice_call_event', (event) => {
          const { session_id, event: voiceEvent } = event.payload;

          if (session_id !== newSessionId) return;

          switch (voiceEvent.type) {
            case 'connected':
              setCallStatus('connected');
              toast.success(`Connected to ${agentName}`);
              break;

            case 'audio_data':
              // Play audio from agent
              if (isSpeakerOn && voiceEvent.data) {
                playAudioData(voiceEvent.data);
              }
              break;

            case 'transcription':
              // Add transcription to transcript
              if (voiceEvent.text) {
                setTranscript(prev => [
                  ...prev,
                  { speaker: 'agent', text: voiceEvent.text, timestamp: new Date() }
                ]);
              }
              break;

            case 'error':
              setCallStatus('error');
              toast.error(`Call error: ${voiceEvent.message}`);
              break;

            case 'disconnected':
              setCallStatus('disconnected');
              toast.info('Call ended');
              break;
          }
        });

        unlistenRef.current = unlisten;

        // Initialize audio context
        audioContextRef.current = new AudioContext({ sampleRate: 48000 });

        // Request microphone access
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: {
            channelCount: 1,
            sampleRate: 48000,
            echoCancellation: true,
            noiseSuppression: true,
          },
        });
        mediaStreamRef.current = stream;

        // Setup audio processing
        await setupAudioProcessing(stream, newSessionId);

      } catch (error) {
        console.error('Failed to initialize call:', error);
        toast.error('Failed to start call. Please check microphone permissions.');
        setCallStatus('error');
      }
    };

    initializeCall();

    // Cleanup on unmount
    return () => {
      cleanup();
    };
  }, [open, agentId]);

  // Setup audio processing pipeline
  const setupAudioProcessing = async (stream: MediaStream, sessionId: string) => {
    if (!audioContextRef.current) return;

    const audioContext = audioContextRef.current;
    const source = audioContext.createMediaStreamSource(stream);

    // Create script processor for audio capture
    // Note: ScriptProcessorNode is deprecated but AudioWorklet requires more setup
    const processor = audioContext.createScriptProcessor(4096, 1, 1);

    processor.onaudioprocess = (e) => {
      if (isMuted) return;

      const inputData = e.inputBuffer.getChannelData(0);

      // Convert Float32 to Int16 PCM
      const pcmData = new Int16Array(inputData.length);
      for (let i = 0; i < inputData.length; i++) {
        const s = Math.max(-1, Math.min(1, inputData[i]));
        pcmData[i] = s < 0 ? s * 0x8000 : s * 0x7FFF;
      }

      // Resample from 48kHz to 16kHz (simple decimation)
      const downsampledLength = Math.floor(pcmData.length / 3);
      const downsampled = new Int16Array(downsampledLength);
      for (let i = 0; i < downsampledLength; i++) {
        downsampled[i] = pcmData[i * 3];
      }

      // Convert to byte array
      const byteArray = new Uint8Array(downsampled.buffer);

      // Send to backend
      anycoworkApi.sendVoiceAudio(sessionId, Array.from(byteArray)).catch((err) => {
        console.error('Failed to send audio:', err);
      });
    };

    source.connect(processor);
    processor.connect(audioContext.destination);
  };

  // Play audio data from agent (24kHz PCM)
  const playAudioData = async (audioData: number[]) => {
    if (!audioContextRef.current) return;

    setIsAgentSpeaking(true);

    const audioContext = audioContextRef.current;

    // Convert byte array to Int16Array
    const int16Data = new Int16Array(new Uint8Array(audioData).buffer);

    // Convert Int16 PCM to Float32
    const float32Data = new Float32Array(int16Data.length);
    for (let i = 0; i < int16Data.length; i++) {
      float32Data[i] = int16Data[i] / (int16Data[i] < 0 ? 0x8000 : 0x7FFF);
    }

    // Add to playback queue
    playbackQueueRef.current.push(float32Data);

    // Start playback if not already playing
    if (!isPlayingRef.current) {
      playNextInQueue();
    }
  };

  // Play next audio chunk in queue
  const playNextInQueue = async () => {
    if (playbackQueueRef.current.length === 0) {
      isPlayingRef.current = false;
      setIsAgentSpeaking(false);
      return;
    }

    isPlayingRef.current = true;
    const audioData = playbackQueueRef.current.shift()!;

    if (!audioContextRef.current) return;

    const audioContext = audioContextRef.current;

    // Create audio buffer (24kHz output from Gemini)
    const audioBuffer = audioContext.createBuffer(1, audioData.length, 24000);
    audioBuffer.getChannelData(0).set(audioData);

    // Create source and play
    const source = audioContext.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(audioContext.destination);

    source.onended = () => {
      playNextInQueue();
    };

    source.start();
  };

  // Cleanup function
  const cleanup = () => {
    // Stop media stream
    if (mediaStreamRef.current) {
      mediaStreamRef.current.getTracks().forEach(track => track.stop());
      mediaStreamRef.current = null;
    }

    // Close audio context
    if (audioContextRef.current) {
      audioContextRef.current.close();
      audioContextRef.current = null;
    }

    // Unlisten from events
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }

    // Stop voice call session
    if (sessionId) {
      anycoworkApi.stopVoiceCall(sessionId).catch(console.error);
    }
  };

  // Handle hang up
  const handleHangUp = () => {
    cleanup();
    onOpenChange(false);
  };

  // Toggle mute
  const toggleMute = () => {
    setIsMuted(!isMuted);
  };

  // Toggle speaker
  const toggleSpeaker = () => {
    setIsSpeakerOn(!isSpeakerOn);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Voice Call</DialogTitle>
        </DialogHeader>

        <div className="space-y-6">
          {/* Agent Info */}
          <div className="flex flex-col items-center gap-4 py-4">
            <Avatar className="h-24 w-24">
              <AvatarFallback className="text-4xl">
                {agentAvatar || agentName[0]}
              </AvatarFallback>
            </Avatar>
            <div className="text-center">
              <h3 className="text-xl font-semibold">{agentName}</h3>
              <Badge variant={callStatus === 'connected' ? 'default' : 'secondary'} className="mt-2">
                {callStatus === 'connecting' && (
                  <>
                    <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                    Connecting...
                  </>
                )}
                {callStatus === 'connected' && (
                  <>
                    <div className="mr-2 h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                    Connected
                  </>
                )}
                {callStatus === 'disconnected' && 'Disconnected'}
                {callStatus === 'error' && 'Error'}
              </Badge>
            </div>

            {/* Speaking Indicator */}
            {isAgentSpeaking && (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <div className="flex gap-1">
                  <div className="h-3 w-1 bg-primary animate-pulse rounded" style={{ animationDelay: '0ms' }} />
                  <div className="h-3 w-1 bg-primary animate-pulse rounded" style={{ animationDelay: '100ms' }} />
                  <div className="h-3 w-1 bg-primary animate-pulse rounded" style={{ animationDelay: '200ms' }} />
                </div>
                <span>{agentName} is speaking...</span>
              </div>
            )}
          </div>

          {/* Transcript */}
          {transcript.length > 0 && (
            <div className="border rounded-lg p-3">
              <h4 className="text-sm font-semibold mb-2">Conversation</h4>
              <ScrollArea className="h-[200px]">
                <div className="space-y-2">
                  {transcript.map((msg, idx) => (
                    <div
                      key={idx}
                      className={`text-sm p-2 rounded ${
                        msg.speaker === 'agent'
                          ? 'bg-muted'
                          : 'bg-primary/10 text-right'
                      }`}
                    >
                      <div className="font-semibold text-xs mb-1">
                        {msg.speaker === 'agent' ? agentName : 'You'}
                      </div>
                      <div>{msg.text}</div>
                    </div>
                  ))}
                </div>
              </ScrollArea>
            </div>
          )}

          {/* Call Controls */}
          <div className="flex items-center justify-center gap-4">
            <Button
              variant={isMuted ? 'destructive' : 'outline'}
              size="icon"
              className="h-12 w-12 rounded-full"
              onClick={toggleMute}
              disabled={callStatus !== 'connected'}
            >
              {isMuted ? <MicOff className="h-5 w-5" /> : <Mic className="h-5 w-5" />}
            </Button>

            <Button
              variant="destructive"
              size="icon"
              className="h-16 w-16 rounded-full"
              onClick={handleHangUp}
            >
              <PhoneOff className="h-6 w-6" />
            </Button>

            <Button
              variant={!isSpeakerOn ? 'destructive' : 'outline'}
              size="icon"
              className="h-12 w-12 rounded-full"
              onClick={toggleSpeaker}
              disabled={callStatus !== 'connected'}
            >
              {isSpeakerOn ? <Volume2 className="h-5 w-5" /> : <VolumeX className="h-5 w-5" />}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
