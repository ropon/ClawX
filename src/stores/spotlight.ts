/**
 * Spotlight State Store
 * Manages the Spotlight quick chat window state
 */
import { create } from 'zustand';
import { invoke } from '@/lib/bridge';
import { useAgentStore } from './agents';
import type { ClipboardContent, ClipboardContentType } from '@/types/spotlight';
import type { QuickCommand } from '@/types/commands';
import type { FileAttachment, FileSearchResult, FileSearchState } from '@/types/file-attachment';
import { detectClipboardType } from '@/utils/clipboard-detect';
import { filterCommands, resolveTemplate } from '@/utils/command-engine';

/** Simplified message for Spotlight conversations */
interface SpotlightMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

/** Screenshot preview data */
interface ScreenshotPreview {
  stagedPath: string;
  preview: string;
  fileName: string;
  mimeType: string;
  fileSize: number;
}

/** Command Palette state */
interface CommandPaletteState {
  isOpen: boolean;
  filter: string;
  highlightIndex: number;
}

interface SpotlightState {
  // Visibility
  visible: boolean;

  // Input
  query: string;

  // Conversation
  messages: SpotlightMessage[];
  sending: boolean;
  streamingText: string;
  streamingMessage: unknown | null;
  error: string | null;
  activeRunId: string | null;

  // Clipboard
  clipboardContent: ClipboardContent | null;
  clipboardType: ClipboardContentType;

  // Screenshot
  screenshotPreview: ScreenshotPreview | null;
  isCapturing: boolean;

  // Command Palette
  commandPalette: CommandPaletteState;
  pendingCommand: QuickCommand | null;

  // File Attachments (F-2.6)
  fileAttachments: FileAttachment[];
  fileSearch: FileSearchState;

  // Session
  sessionKey: string;

  // Actions
  setVisible: (visible: boolean) => void;
  setQuery: (query: string) => void;
  sendMessage: (text: string) => Promise<void>;
  abortRun: () => Promise<void>;
  readClipboard: () => Promise<void>;
  takeScreenshot: () => Promise<void>;
  clearScreenshot: () => void;
  clearConversation: () => void;
  hideAndReset: () => void;
  handleChatEvent: (event: Record<string, unknown>) => void;

  // Command Palette Actions
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  setCommandFilter: (filter: string) => void;
  setHighlightIndex: (index: number) => void;
  selectCommand: (cmd: QuickCommand) => void;
  executeCommand: (userInput?: string) => void;
  cancelPendingCommand: () => void;

  // File Attachment Actions (F-2.6)
  addFileAttachment: (file: Omit<FileAttachment, 'id'>) => boolean;
  removeFileAttachment: (id: string) => void;
  clearFileAttachments: () => void;
  pickFileFromDialog: () => Promise<void>;
  openFileSearch: () => void;
  closeFileSearch: () => void;
  setFileSearchFilter: (filter: string) => void;
  setFileSearchHighlight: (index: number) => void;
  selectFileSearchResult: (result: FileSearchResult) => void;
}

function generateSessionKey(): string {
  const activeAgent = useAgentStore.getState().getActiveAgent();
  const agentId = activeAgent?.id || 'main';
  return `agent:${agentId}:spotlight`;
}

export const useSpotlightStore = create<SpotlightState>((set, get) => ({
  visible: false,
  query: '',
  messages: [],
  sending: false,
  streamingText: '',
  streamingMessage: null,
  error: null,
  activeRunId: null,
  clipboardContent: null,
  clipboardType: 'empty',
  screenshotPreview: null,
  isCapturing: false,
  commandPalette: { isOpen: false, filter: '', highlightIndex: 0 },
  pendingCommand: null,
  fileAttachments: [],
  fileSearch: { isOpen: false, filter: '', results: [], highlightIndex: 0, searching: false },
  sessionKey: 'agent:main:spotlight',

  setVisible: (visible) => set({ visible }),

  setQuery: (query) => set({ query }),

  sendMessage: async (text: string) => {
    const { screenshotPreview, fileAttachments } = get();
    const trimmed = text.trim();

    // Need either text, a screenshot, or file attachments to send
    if (!trimmed && !screenshotPreview && fileAttachments.length === 0) return;

    // Default prompt when sending screenshot without text
    const message = trimmed || (screenshotPreview ? 'Analyze this screenshot' : 'Analyze these files');

    const sessionKey = generateSessionKey();

    // Build user message display text
    const fileNames = fileAttachments.map((f) => f.fileName);
    let displayContent = message;
    if (screenshotPreview) displayContent = `📷 ${displayContent}`;
    if (fileNames.length > 0) displayContent = `📎 [${fileNames.join(', ')}] ${displayContent}`;

    const userMsg: SpotlightMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: displayContent,
      timestamp: Date.now(),
    };

    // Capture attachments before clearing
    const attachmentsToSend = [...fileAttachments];

    set((s) => ({
      messages: [...s.messages, userMsg],
      sending: true,
      error: null,
      query: '',
      streamingText: '',
      streamingMessage: null,
      screenshotPreview: null,
      fileAttachments: [],
      fileSearch: { isOpen: false, filter: '', results: [], highlightIndex: 0, searching: false },
      sessionKey,
    }));

    try {
      const idempotencyKey = crypto.randomUUID();

      // Get agent overrides
      const activeAgent = useAgentStore.getState().getActiveAgent();
      const agentOverrides: Record<string, unknown> = {};
      if (activeAgent?.systemPrompt) {
        agentOverrides.systemPrompt = activeAgent.systemPrompt;
      }
      if (activeAgent?.model) {
        agentOverrides.model = activeAgent.model;
      }

      // RAG injection (F-3.5): if the agent has knowledge bases, retrieve context
      let ragPrefix = '';
      if (activeAgent?.knowledgeBaseIds?.length && message) {
        try {
          const ragResult = await invoke(
            'knowledge:rag',
            activeAgent.knowledgeBaseIds,
            message,
            { topK: 5 }
          ) as { success: boolean; results?: Array<{ documentName: string; content: string }>; error?: string };
          if (ragResult.success && ragResult.results?.length) {
            const chunks = ragResult.results.map((r, i) =>
              `[${i + 1}] (Source: ${r.documentName})\n${r.content}`
            ).join('\n---\n');
            ragPrefix = `<knowledge>\n${chunks}\n</knowledge>\n\n`;
          }
        } catch (err) {
          console.warn('[RAG] Spotlight: Failed to retrieve context:', err);
        }
      }

      // Read file attachment contents and inline into the message
      let finalMessage = ragPrefix + message;
      if (attachmentsToSend.length > 0) {
        const fileParts: string[] = [];
        for (const att of attachmentsToSend) {
          try {
            const readResult = await invoke(
              'filesearch:readContent',
              { filePath: att.filePath }
            ) as { success: boolean; content?: string; error?: string };
            if (readResult.success && readResult.content) {
              fileParts.push(`--- File: ${att.fileName} (${att.filePath}) ---\n${readResult.content}`);
            } else {
              fileParts.push(`--- File: ${att.fileName} (${att.filePath}) ---\n[Error: ${readResult.error || 'Failed to read'}]`);
            }
          } catch {
            fileParts.push(`--- File: ${att.fileName} (${att.filePath}) ---\n[Error: Failed to read file]`);
          }
        }
        finalMessage = `${message}\n\n${fileParts.join('\n\n')}`;
      }

      let result: { success: boolean; result?: { runId?: string }; error?: string };

      if (screenshotPreview) {
        // Send with media attachment (reuse chat:sendWithMedia)
        result = await invoke(
          'chat:sendWithMedia',
          {
            sessionKey,
            message: finalMessage,
            deliver: false,
            idempotencyKey,
            media: [{
              filePath: screenshotPreview.stagedPath,
              mimeType: screenshotPreview.mimeType,
              fileName: screenshotPreview.fileName,
            }],
            ...agentOverrides,
          }
        ) as { success: boolean; result?: { runId?: string }; error?: string };
      } else {
        result = await invoke(
          'gateway:rpc',
          'chat.send',
          {
            sessionKey,
            message: finalMessage,
            deliver: false,
            idempotencyKey,
            ...agentOverrides,
          }
        ) as { success: boolean; result?: { runId?: string }; error?: string };
      }

      if (!result.success) {
        set({ error: result.error || 'Failed to send message', sending: false });
      } else if (result.result?.runId) {
        set({ activeRunId: result.result.runId });
      }
    } catch (err) {
      set({ error: String(err), sending: false });
    }
  },

  abortRun: async () => {
    const { sessionKey } = get();
    set({ sending: false, streamingText: '', streamingMessage: null });

    try {
      await invoke(
        'gateway:rpc',
        'chat.abort',
        { sessionKey }
      );
    } catch (err) {
      set({ error: String(err) });
    }
  },

  readClipboard: async () => {
    try {
      const content = await invoke('clipboard:read') as ClipboardContent;
      const clipboardType = detectClipboardType(content);
      set({ clipboardContent: content, clipboardType });
    } catch (err) {
      console.warn('Failed to read clipboard:', err);
      set({ clipboardContent: null, clipboardType: 'empty' });
    }
  },

  takeScreenshot: async () => {
    set({ isCapturing: true });
    try {
      // Use Rust screenshot:capture command (implemented via macOS screencapture / platform tools)
      const result = await invoke('screenshot:capture') as
        | { cancelled: true }
        | ScreenshotPreview;

      if ('cancelled' in result) {
        set({ isCapturing: false });
        return;
      }

      set({
        isCapturing: false,
        screenshotPreview: result,
      });
    } catch (err) {
      console.warn('Screenshot not available:', err);
      set({ isCapturing: false });
    }
  },

  clearScreenshot: () => {
    set({ screenshotPreview: null });
  },

  clearConversation: () => {
    set({
      messages: [],
      streamingText: '',
      streamingMessage: null,
      error: null,
      activeRunId: null,
      sending: false,
      screenshotPreview: null,
      commandPalette: { isOpen: false, filter: '', highlightIndex: 0 },
      pendingCommand: null,
      fileAttachments: [],
      fileSearch: { isOpen: false, filter: '', results: [], highlightIndex: 0, searching: false },
    });
  },

  hideAndReset: () => {
    invoke('spotlight:hide');
    set({ visible: false });
  },

  handleChatEvent: (event: Record<string, unknown>) => {
    const runId = String(event.runId || '');
    const eventState = String(event.state || '');
    const { activeRunId } = get();

    if (activeRunId && runId && runId !== activeRunId) return;

    // Infer state if missing
    let resolvedState = eventState;
    if (!resolvedState && event.message && typeof event.message === 'object') {
      const msg = event.message as Record<string, unknown>;
      const stopReason = msg.stopReason ?? msg.stop_reason;
      if (stopReason) {
        resolvedState = 'final';
      } else if (msg.role || msg.content) {
        resolvedState = 'delta';
      }
    }

    switch (resolvedState) {
      case 'delta': {
        const msg = event.message as Record<string, unknown> | undefined;
        if (!msg) break;
        // Extract text content from the streaming message
        let text = '';
        if (typeof msg.content === 'string') {
          text = msg.content;
        } else if (Array.isArray(msg.content)) {
          text = (msg.content as Array<{ type?: string; text?: string }>)
            .filter(b => b.type === 'text' && b.text)
            .map(b => b.text!)
            .join('');
        }
        if (typeof msg.text === 'string') {
          text = msg.text;
        }
        set({ streamingMessage: event.message, streamingText: text });
        break;
      }
      case 'final': {
        const msg = event.message as Record<string, unknown> | undefined;
        if (!msg) {
          set({ sending: false, activeRunId: null, streamingText: '', streamingMessage: null });
          break;
        }
        // Extract final text
        let text = '';
        if (typeof msg.content === 'string') {
          text = msg.content;
        } else if (Array.isArray(msg.content)) {
          text = (msg.content as Array<{ type?: string; text?: string }>)
            .filter(b => b.type === 'text' && b.text)
            .map(b => b.text!)
            .join('');
        }
        if (typeof msg.text === 'string') {
          text = msg.text;
        }

        // Skip tool-only messages
        const role = String(msg.role || '').toLowerCase();
        if (role === 'toolresult' || role === 'tool_result') break;

        // Skip if no text content (tool use only)
        if (!text.trim()) break;

        const assistantMsg: SpotlightMessage = {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: text,
          timestamp: Date.now(),
        };
        set((s) => ({
          messages: [...s.messages, assistantMsg],
          sending: false,
          activeRunId: null,
          streamingText: '',
          streamingMessage: null,
        }));
        break;
      }
      case 'error': {
        set({
          error: String(event.errorMessage || 'An error occurred'),
          sending: false,
          activeRunId: null,
          streamingText: '',
          streamingMessage: null,
        });
        break;
      }
      case 'aborted': {
        set({
          sending: false,
          activeRunId: null,
          streamingText: '',
          streamingMessage: null,
        });
        break;
      }
    }
  },

  // Command Palette Actions
  openCommandPalette: () => {
    set({ commandPalette: { isOpen: true, filter: '', highlightIndex: 0 } });
  },

  closeCommandPalette: () => {
    set({ commandPalette: { isOpen: false, filter: '', highlightIndex: 0 } });
  },

  setCommandFilter: (filter: string) => {
    const filtered = filterCommands(filter);
    set((s) => ({
      commandPalette: {
        ...s.commandPalette,
        filter,
        highlightIndex: Math.min(s.commandPalette.highlightIndex, Math.max(0, filtered.length - 1)),
      },
    }));
  },

  setHighlightIndex: (index: number) => {
    set((s) => ({
      commandPalette: { ...s.commandPalette, highlightIndex: index },
    }));
  },

  selectCommand: (cmd: QuickCommand) => {
    if (cmd.requiresInput) {
      // Enter pending command mode — user needs to type additional input
      set({
        pendingCommand: cmd,
        query: '',
        commandPalette: { isOpen: false, filter: '', highlightIndex: 0 },
      });
    } else {
      // Execute immediately with clipboard content
      const { clipboardContent, sendMessage } = get();
      const clipboardText = clipboardContent?.text || '';
      const resolved = resolveTemplate(cmd, clipboardText, '');
      set({
        commandPalette: { isOpen: false, filter: '', highlightIndex: 0 },
        query: '',
      });
      sendMessage(resolved);
    }
  },

  executeCommand: (userInput?: string) => {
    const { pendingCommand, clipboardContent, sendMessage } = get();
    if (!pendingCommand) return;
    const clipboardText = clipboardContent?.text || '';
    const resolved = resolveTemplate(pendingCommand, clipboardText, userInput || '');
    set({ pendingCommand: null, query: '' });
    sendMessage(resolved);
  },

  cancelPendingCommand: () => {
    set({ pendingCommand: null, query: '' });
  },

  // File Attachment Actions (F-2.6)
  addFileAttachment: (file) => {
    const { fileAttachments } = get();
    // Deduplicate by filePath
    if (fileAttachments.some((f) => f.filePath === file.filePath)) return false;
    // Total size check (200KB limit)
    const totalSize = fileAttachments.reduce((sum, f) => sum + f.fileSize, 0) + file.fileSize;
    if (totalSize > 204800) return false;
    const attachment: FileAttachment = { ...file, id: crypto.randomUUID() };
    set({ fileAttachments: [...fileAttachments, attachment] });
    return true;
  },

  removeFileAttachment: (id) => {
    set((s) => ({
      fileAttachments: s.fileAttachments.filter((f) => f.id !== id),
    }));
  },

  clearFileAttachments: () => {
    set({ fileAttachments: [] });
  },

  pickFileFromDialog: async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: true,
        filters: [
          { name: 'Text Files', extensions: ['txt', 'md', 'json', 'yaml', 'yml', 'toml', 'xml', 'csv', 'log', 'env', 'ini', 'conf'] },
          { name: 'Code', extensions: ['ts', 'tsx', 'js', 'jsx', 'py', 'rs', 'go', 'java', 'c', 'cpp', 'h', 'cs', 'rb', 'php', 'swift', 'kt', 'html', 'css', 'scss', 'vue', 'svelte', 'sh', 'bash', 'zsh', 'sql'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (!selected) return;
      const filePaths = Array.isArray(selected) ? selected : [selected];

      for (const filePath of filePaths) {
        const fileName = filePath.split('/').pop() || filePath;
        const ext = fileName.includes('.') ? '.' + fileName.split('.').pop() : '';
        get().addFileAttachment({ filePath, fileName, fileSize: 0, fileType: ext });
      }
    } catch (err) {
      console.warn('Failed to pick file:', err);
    }
  },

  openFileSearch: () => {
    set({ fileSearch: { isOpen: true, filter: '', results: [], highlightIndex: 0, searching: false } });
  },

  closeFileSearch: () => {
    set({ fileSearch: { isOpen: false, filter: '', results: [], highlightIndex: 0, searching: false } });
  },

  setFileSearchFilter: (filter: string) => {
    set((s) => ({
      fileSearch: { ...s.fileSearch, filter, searching: filter.length >= 2 },
    }));

    // Debounce search
    if (filter.length < 2) {
      set((s) => ({ fileSearch: { ...s.fileSearch, results: [], searching: false } }));
      return;
    }

    // Use a simple timeout-based debounce
    const searchId = Date.now();
    (window as unknown as Record<string, unknown>).__fileSearchId = searchId;
    setTimeout(async () => {
      if ((window as unknown as Record<string, unknown>).__fileSearchId !== searchId) return;
      try {
        const res = await invoke('filesearch:search', { query: filter }) as {
          success: boolean;
          results: FileSearchResult[];
        };
        // Only update if still the active search
        if ((window as unknown as Record<string, unknown>).__fileSearchId !== searchId) return;
        if (res.success) {
          set((s) => ({
            fileSearch: { ...s.fileSearch, results: res.results, searching: false, highlightIndex: 0 },
          }));
        } else {
          set((s) => ({ fileSearch: { ...s.fileSearch, results: [], searching: false } }));
        }
      } catch {
        set((s) => ({ fileSearch: { ...s.fileSearch, results: [], searching: false } }));
      }
    }, 200);
  },

  setFileSearchHighlight: (index: number) => {
    set((s) => ({ fileSearch: { ...s.fileSearch, highlightIndex: index } }));
  },

  selectFileSearchResult: (result: FileSearchResult) => {
    const added = get().addFileAttachment({
      filePath: result.filePath,
      fileName: result.fileName,
      fileSize: result.fileSize,
      fileType: result.fileType,
    });
    if (added) {
      // Close file search and clear @ prefix from query
      set({
        fileSearch: { isOpen: false, filter: '', results: [], highlightIndex: 0, searching: false },
        query: '',
      });
    }
  },
}));
