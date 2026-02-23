/**
 * Spotlight Input
 * Single-line input for quick chat queries with screenshot + command palette + file search support
 */
import { useRef, useEffect } from 'react';
import { Send, Square, Camera, Paperclip, Loader2, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useSpotlightStore } from '@/stores/spotlight';
import { on } from '@/lib/bridge';
import { isCommandTrigger, filterCommands } from '@/utils/command-engine';
import { isFileSearchTrigger } from '@/utils/file-search-engine';
import { CommandPalette } from './CommandPalette';
import { FileSearchPanel } from './FileSearchPanel';
import { FileAttachmentBar } from './FileAttachmentBar';

export function SpotlightInput() {
  const { t } = useTranslation('spotlight');
  const inputRef = useRef<HTMLInputElement>(null);
  const query = useSpotlightStore((s) => s.query);
  const setQuery = useSpotlightStore((s) => s.setQuery);
  const sendMessage = useSpotlightStore((s) => s.sendMessage);
  const sending = useSpotlightStore((s) => s.sending);
  const abortRun = useSpotlightStore((s) => s.abortRun);
  const hideAndReset = useSpotlightStore((s) => s.hideAndReset);
  const takeScreenshot = useSpotlightStore((s) => s.takeScreenshot);
  const clearScreenshot = useSpotlightStore((s) => s.clearScreenshot);
  const screenshotPreview = useSpotlightStore((s) => s.screenshotPreview);
  const isCapturing = useSpotlightStore((s) => s.isCapturing);

  // Command palette state
  const commandPalette = useSpotlightStore((s) => s.commandPalette);
  const pendingCommand = useSpotlightStore((s) => s.pendingCommand);
  const openCommandPalette = useSpotlightStore((s) => s.openCommandPalette);
  const closeCommandPalette = useSpotlightStore((s) => s.closeCommandPalette);
  const setCommandFilter = useSpotlightStore((s) => s.setCommandFilter);
  const setHighlightIndex = useSpotlightStore((s) => s.setHighlightIndex);
  const selectCommand = useSpotlightStore((s) => s.selectCommand);
  const executeCommand = useSpotlightStore((s) => s.executeCommand);
  const cancelPendingCommand = useSpotlightStore((s) => s.cancelPendingCommand);

  // File search state (F-2.6)
  const fileSearch = useSpotlightStore((s) => s.fileSearch);
  const fileAttachments = useSpotlightStore((s) => s.fileAttachments);
  const pickFileFromDialog = useSpotlightStore((s) => s.pickFileFromDialog);
  const openFileSearch = useSpotlightStore((s) => s.openFileSearch);
  const closeFileSearch = useSpotlightStore((s) => s.closeFileSearch);
  const setFileSearchFilter = useSpotlightStore((s) => s.setFileSearchFilter);
  const setFileSearchHighlight = useSpotlightStore((s) => s.setFileSearchHighlight);
  const selectFileSearchResult = useSpotlightStore((s) => s.selectFileSearchResult);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Re-focus when spotlight becomes visible
  useEffect(() => {
    const unsub = on('spotlight:shown', () => {
      setTimeout(() => inputRef.current?.focus(), 50);
    });
    return () => {
      unsub();
    };
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;

    if (pendingCommand) {
      // In pending command mode, just update query (no "/" or "@" detection)
      setQuery(value);
      return;
    }

    setQuery(value);

    // Priority: "/" command trigger > "@" file search trigger > normal
    if (isCommandTrigger(value)) {
      if (!commandPalette.isOpen) openCommandPalette();
      setCommandFilter(value);
      if (fileSearch.isOpen) closeFileSearch();
    } else if (isFileSearchTrigger(value) !== null) {
      const searchQuery = isFileSearchTrigger(value)!;
      if (!fileSearch.isOpen) openFileSearch();
      setFileSearchFilter(searchQuery);
      if (commandPalette.isOpen) closeCommandPalette();
    } else {
      if (commandPalette.isOpen) closeCommandPalette();
      if (fileSearch.isOpen) closeFileSearch();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    // Command Palette open — handle navigation
    if (commandPalette.isOpen) {
      const filtered = filterCommands(commandPalette.filter);
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setHighlightIndex(
          commandPalette.highlightIndex < filtered.length - 1
            ? commandPalette.highlightIndex + 1
            : 0
        );
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setHighlightIndex(
          commandPalette.highlightIndex > 0
            ? commandPalette.highlightIndex - 1
            : filtered.length - 1
        );
        return;
      }
      if ((e.key === 'Enter' || e.key === 'Tab') && filtered.length > 0) {
        e.preventDefault();
        selectCommand(filtered[commandPalette.highlightIndex]);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        closeCommandPalette();
        setQuery('');
        return;
      }
      return;
    }

    // File Search open — handle navigation (F-2.6)
    if (fileSearch.isOpen) {
      const results = fileSearch.results;
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setFileSearchHighlight(
          fileSearch.highlightIndex < results.length - 1
            ? fileSearch.highlightIndex + 1
            : 0
        );
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setFileSearchHighlight(
          fileSearch.highlightIndex > 0
            ? fileSearch.highlightIndex - 1
            : Math.max(0, results.length - 1)
        );
        return;
      }
      if (e.key === 'Enter' && results.length > 0) {
        e.preventDefault();
        selectFileSearchResult(results[fileSearch.highlightIndex]);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        closeFileSearch();
        setQuery('');
        return;
      }
      return;
    }

    // Pending command mode — handle execute/cancel
    if (pendingCommand) {
      if (e.key === 'Escape') {
        e.preventDefault();
        cancelPendingCommand();
        return;
      }
      if (e.key === 'Enter' && !e.shiftKey && !sending) {
        e.preventDefault();
        executeCommand(query);
        return;
      }
      return;
    }

    // Normal mode
    if (e.key === 'Escape') {
      e.preventDefault();
      hideAndReset();
      return;
    }
    if (e.key === 'Enter' && !e.shiftKey && !sending) {
      e.preventDefault();
      if (query.trim() || screenshotPreview || fileAttachments.length > 0) {
        sendMessage(query);
      }
    }
  };

  const canSend = pendingCommand
    ? !!query.trim()
    : !!(query.trim() || screenshotPreview || fileAttachments.length > 0);

  // Determine placeholder
  const getPlaceholder = () => {
    if (pendingCommand?.inputPlaceholderKey) {
      return t(pendingCommand.inputPlaceholderKey);
    }
    if (screenshotPreview) {
      return t('screenshot.analyzeThis');
    }
    return t('input.placeholder');
  };

  return (
    <div className="flex flex-col">
      {/* Screenshot preview */}
      {screenshotPreview && (
        <div className="px-4 pt-3 pb-1">
          <div className="relative inline-block">
            <img
              src={screenshotPreview.preview}
              alt={t('screenshot.attached')}
              className="h-16 rounded-md border border-border object-cover"
            />
            <button
              onClick={clearScreenshot}
              className="absolute -top-1.5 -right-1.5 p-0.5 rounded-full bg-destructive text-destructive-foreground hover:bg-destructive/80 transition-colors"
            >
              <X className="h-3 w-3" />
            </button>
          </div>
        </div>
      )}

      {/* File attachment chips (F-2.6) */}
      <FileAttachmentBar />

      {/* Input row */}
      <div className="flex items-center gap-3 px-4 py-3">
        {/* Screenshot button */}
        <button
          onClick={takeScreenshot}
          disabled={isCapturing || sending}
          className="flex-shrink-0 p-1.5 rounded-lg text-muted-foreground hover:text-primary hover:bg-primary/10 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title={t('screenshot.capture')}
        >
          {isCapturing ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Camera className="h-4 w-4" />
          )}
        </button>

        {/* Attach file button (F-2.6) */}
        <button
          onClick={pickFileFromDialog}
          disabled={sending}
          className="flex-shrink-0 p-1.5 rounded-lg text-muted-foreground hover:text-primary hover:bg-primary/10 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title={t('fileSearch.attachFile')}
        >
          <Paperclip className="h-4 w-4" />
        </button>

        {/* Pending command badge */}
        {pendingCommand && (
          <div className="flex items-center gap-1 px-2 py-0.5 rounded-md bg-primary/10 text-primary text-xs font-medium flex-shrink-0">
            <span>{pendingCommand.icon}</span>
            <span>{t(pendingCommand.nameKey)}</span>
            <button
              onClick={cancelPendingCommand}
              className="ml-0.5 p-0.5 rounded hover:bg-primary/20 transition-colors"
            >
              <X className="h-3 w-3" />
            </button>
          </div>
        )}

        <input
          ref={inputRef}
          type="text"
          className="flex-1 bg-transparent text-base text-foreground placeholder:text-muted-foreground outline-none"
          placeholder={getPlaceholder()}
          value={query}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          autoFocus
        />
        {sending ? (
          <button
            onClick={abortRun}
            className="flex-shrink-0 p-1.5 rounded-lg text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
          >
            <Square className="h-4 w-4" />
          </button>
        ) : (
          <button
            onClick={() => {
              if (!canSend) return;
              if (pendingCommand) {
                executeCommand(query);
              } else {
                sendMessage(query);
              }
            }}
            disabled={!canSend}
            className="flex-shrink-0 p-1.5 rounded-lg text-muted-foreground hover:text-primary hover:bg-primary/10 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          >
            <Send className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Command Palette */}
      <CommandPalette />

      {/* File Search Panel (F-2.6) */}
      <FileSearchPanel />
    </div>
  );
}
