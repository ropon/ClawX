/**
 * Gateway State Store
 * Manages Gateway connection state and communication
 */
import { create } from 'zustand';
import { invoke, on } from '@/lib/bridge';
import type { GatewayStatus, GatewayStartupProgress } from '../types/gateway';

let gatewayInitPromise: Promise<void> | null = null;

interface GatewayHealth {
  ok: boolean;
  error?: string;
  uptime?: number;
}

interface GatewayState {
  status: GatewayStatus;
  health: GatewayHealth | null;
  isInitialized: boolean;
  lastError: string | null;
  startupProgress: GatewayStartupProgress | null;
  _cleanups: Array<() => void>;

  // Actions
  init: () => Promise<void>;
  destroy: () => void;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  restart: () => Promise<void>;
  checkHealth: () => Promise<GatewayHealth>;
  rpc: <T>(method: string, params?: unknown, timeoutMs?: number) => Promise<T>;
  setStatus: (status: GatewayStatus) => void;
  clearError: () => void;
}

export const useGatewayStore = create<GatewayState>((set, get) => ({
  status: {
    state: 'stopped',
    port: 18789,
  },
  health: null,
  isInitialized: false,
  lastError: null,
  startupProgress: null,
  _cleanups: [],

  init: async () => {
    if (get().isInitialized) return;
    if (gatewayInitPromise) {
      await gatewayInitPromise;
      return;
    }

    gatewayInitPromise = (async () => {
      const cleanups: Array<() => void> = [];
      try {
        // Register all event listeners FIRST — before any async operations
        // that could trigger events. This prevents race conditions with
        // non-blocking gateway:start which emits events from a background task.
        cleanups.push(on('gateway:status-changed', (newStatus) => {
          set({ status: newStatus as GatewayStatus });
        }));

        cleanups.push(on('gateway:error', (error) => {
          set({ lastError: String(error) });
        }));

        cleanups.push(on('gateway:startup-progress', (data) => {
          const progress = data as GatewayStartupProgress;
          set({ startupProgress: progress });
          if (progress.phase === 'ready') {
            setTimeout(() => set({ startupProgress: null }), 1500);
          }
        }));

        // Some Gateway builds stream chat events via generic "agent" notifications.
        // Normalize and forward them to the chat store.
        // The Gateway may put event fields (state, message, etc.) either inside
        // params.data or directly on params — we must handle both layouts.
        cleanups.push(on('gateway:notification', (notification) => {
          const payload = notification as { method?: string; params?: Record<string, unknown> } | undefined;
          if (!payload || payload.method !== 'agent' || !payload.params || typeof payload.params !== 'object') {
            return;
          }

          const p = payload.params;
          const data = (p.data && typeof p.data === 'object') ? (p.data as Record<string, unknown>) : {};
          const normalizedEvent: Record<string, unknown> = {
            // Spread data sub-object first (nested layout)
            ...data,
            // Then override with top-level params fields (flat layout takes precedence)
            runId: p.runId ?? data.runId,
            sessionKey: p.sessionKey ?? data.sessionKey,
            stream: p.stream ?? data.stream,
            seq: p.seq ?? data.seq,
            // Critical: also pick up state and message from params (flat layout)
            state: p.state ?? data.state,
            message: p.message ?? data.message,
          };

          import('./chat')
            .then(({ useChatStore }) => {
              useChatStore.getState().handleChatEvent(normalizedEvent);
            })
            .catch((err) => {
              console.warn('Failed to forward gateway notification event:', err);
            });
        }));

        // Listen for chat events from the gateway and forward to chat store.
        // The data arrives as { message: payload } from handleProtocolEvent.
        // The payload may be a full event wrapper ({ state, runId, message })
        // or the raw chat message itself. We need to handle both.
        cleanups.push(on('gateway:chat-message', (data) => {
          try {
            // Dynamic import to avoid circular dependency
            import('./chat').then(({ useChatStore }) => {
              const chatData = data as Record<string, unknown>;
              // Unwrap the { message: payload } wrapper from handleProtocolEvent
              const payload = ('message' in chatData && typeof chatData.message === 'object')
                ? chatData.message as Record<string, unknown>
                : chatData;

              // If payload has a 'state' field, it's already a proper event wrapper
              if (payload.state) {
                useChatStore.getState().handleChatEvent(payload);
                return;
              }

              // Otherwise, payload is the raw message — wrap it as a 'final' event
              // so handleChatEvent can process it (this happens when the Gateway
              // sends protocol events with the message directly as payload).
              const syntheticEvent: Record<string, unknown> = {
                state: 'final',
                message: payload,
                runId: chatData.runId ?? payload.runId,
              };
              useChatStore.getState().handleChatEvent(syntheticEvent);
            });
          } catch (err) {
            console.warn('Failed to forward chat event:', err);
          }
        }));

        // Yield to let the async event listener imports resolve before triggering events
        await new Promise(resolve => setTimeout(resolve, 50));

        // NOW get initial status and auto-start
        const status = await invoke('gateway:status') as GatewayStatus;
        set({ status, isInitialized: true, _cleanups: cleanups });

        // Auto-start Gateway if not running
        if (status.state === 'stopped' || status.state === 'error') {
          const { useSettingsStore } = await import('./settings');
          const gatewayAutoStart = useSettingsStore.getState().gatewayAutoStart;
          if (gatewayAutoStart) {
            get().start();
          }
        }

      } catch (error) {
        console.error('Failed to initialize Gateway:', error);
        cleanups.forEach(fn => fn());
        set({ lastError: String(error), _cleanups: [] });
      } finally {
        gatewayInitPromise = null;
      }
    })();

    await gatewayInitPromise;
  },

  destroy: () => {
    const { _cleanups } = get();
    _cleanups.forEach(fn => fn());
    set({ _cleanups: [], isInitialized: false });
    gatewayInitPromise = null;
  },

  start: async () => {
    try {
      set({ status: { ...get().status, state: 'starting' }, lastError: null, startupProgress: null });
      const result = await invoke('gateway:start') as { success: boolean; state?: string; error?: string };

      // Non-blocking: if state is "starting", the backend will send progress events
      // If state is "running", gateway was already running
      if (!result.success) {
        set({
          status: { ...get().status, state: 'error', error: result.error },
          lastError: result.error || 'Failed to start Gateway'
        });
      } else if (result.state === 'running') {
        set({ status: { ...get().status, state: 'running' } });
      }
      // state === 'starting' → UI updates via gateway:startup-progress events
    } catch (error) {
      set({
        status: { ...get().status, state: 'error', error: String(error) },
        lastError: String(error)
      });
    }
  },

  stop: async () => {
    try {
      await invoke('gateway:stop');
      set({ status: { ...get().status, state: 'stopped' }, lastError: null });
    } catch (error) {
      console.error('Failed to stop Gateway:', error);
      set({ lastError: String(error) });
    }
  },

  restart: async () => {
    try {
      set({ status: { ...get().status, state: 'starting' }, lastError: null, startupProgress: null });
      const result = await invoke('gateway:restart') as { success: boolean; state?: string; error?: string };

      if (!result.success) {
        set({
          status: { ...get().status, state: 'error', error: result.error },
          lastError: result.error || 'Failed to restart Gateway'
        });
      }
      // Non-blocking: progress events will update the UI
    } catch (error) {
      set({
        status: { ...get().status, state: 'error', error: String(error) },
        lastError: String(error)
      });
    }
  },

  checkHealth: async () => {
    try {
      const result = await invoke('gateway:health') as {
        success: boolean;
        ok: boolean;
        error?: string;
        uptime?: number
      };

      const health: GatewayHealth = {
        ok: result.ok,
        error: result.error,
        uptime: result.uptime,
      };

      set({ health });
      return health;
    } catch (error) {
      const health: GatewayHealth = { ok: false, error: String(error) };
      set({ health });
      return health;
    }
  },

  rpc: async <T>(method: string, params?: unknown, timeoutMs?: number): Promise<T> => {
    const result = await invoke('gateway:rpc', method, params, timeoutMs) as {
      success: boolean;
      result?: T;
      error?: string;
    };

    if (!result.success) {
      throw new Error(result.error || `RPC call failed: ${method}`);
    }

    return result.result as T;
  },

  setStatus: (status) => set({ status }),

  clearError: () => set({ lastError: null }),
}));
