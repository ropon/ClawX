/**
 * Spotlight Response
 * Displays AI responses with markdown rendering and streaming
 */
import { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import ReactMarkdown from 'react-markdown';
import { useSpotlightStore } from '@/stores/spotlight';
import { invoke } from '@/lib/bridge';

export function SpotlightResponse() {
  const { t } = useTranslation('spotlight');
  const messages = useSpotlightStore((s) => s.messages);
  const streamingText = useSpotlightStore((s) => s.streamingText);
  const sending = useSpotlightStore((s) => s.sending);
  const error = useSpotlightStore((s) => s.error);

  // Only show the latest assistant message + streaming
  const lastAssistantMsg = [...messages].reverse().find((m) => m.role === 'assistant');
  const displayText = streamingText || lastAssistantMsg?.content || '';
  const isStreaming = sending && streamingText;

  if (!displayText && !error && !sending) return null;

  return (
    <div className="px-4 pb-3 max-h-[320px] overflow-y-auto">
      {error && (
        <div className="text-sm text-destructive bg-destructive/10 rounded-lg px-3 py-2 mb-2">
          {error}
        </div>
      )}

      {sending && !streamingText && !error && (
        <div className="text-sm text-muted-foreground animate-pulse py-2">
          {t('response.thinking')}
        </div>
      )}

      {displayText && (
        <div className="relative group">
          <div className="prose prose-sm dark:prose-invert max-w-none text-sm leading-relaxed">
            <ReactMarkdown>{displayText}</ReactMarkdown>
            {isStreaming && (
              <span className="inline-block w-1.5 h-4 bg-primary animate-pulse ml-0.5 align-text-bottom" />
            )}
          </div>
          {!isStreaming && displayText && (
            <CopyButton text={displayText} />
          )}
        </div>
      )}
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const { t } = useTranslation('spotlight');
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // fallback: use IPC
      await invoke('clipboard:write', text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="absolute top-0 right-0 opacity-0 group-hover:opacity-100 transition-opacity p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
      title={copied ? t('response.copied') : t('response.copy')}
    >
      {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
    </button>
  );
}
