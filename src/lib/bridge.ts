/**
 * IPC Bridge — Tauri Native
 *
 * All IPC calls and events go through Tauri.
 * All IPC calls and events go through Tauri native commands.
 *
 * Usage:
 *   import { invoke, on, openExternal } from '@/lib/bridge'
 *   const result = await invoke<MyType>('agent:list')
 *   const unsub = on('gateway:status-changed', handler)
 */

type IpcHandler = (...args: unknown[]) => void;

// Commands that take a single object parameter (not positional _args)
const NATIVE_COMMANDS = new Set([
  'window_minimize', 'window_maximize', 'window_close', 'window_is_maximized',
  'clipboard_read', 'clipboard_write',
  'spotlight_toggle', 'spotlight_hide',
  'shortcut_get', 'shortcut_update',
  'app_version', 'app_platform',
  'screenshot_capture',
]);

/**
 * Map frontend channel names to Tauri event names.
 *
 * Frontend uses colon-separated: "gateway:status-changed"
 * Rust emits camelCase: "gateway_statusChanged"
 */
const EVENT_MAP: Record<string, string> = {
  'gateway:status-changed': 'gateway_statusChanged',
  'gateway:chat-message': 'gateway_chatMessage',
  'gateway:notification': 'gateway_notification',
  'gateway:channel-status': 'gateway_channelStatus',
  'gateway:error': 'gateway_error',
  'gateway:exit': 'gateway_exit',
  'gateway:message': 'gateway_message',
  'gateway:startup-progress': 'gateway_startupProgress',
  'knowledge:documentProgress': 'knowledge_documentProgress',
  'workflow:stepProgress': 'workflow_stepProgress',
};

function toCommand(channel: string): string {
  // Convert "provider:validateKey" → "provider_validate_key"
  // 1. Replace : and - with _
  // 2. Convert camelCase to snake_case (Rust functions use snake_case)
  return channel
    .replace(':', '_')
    .replace(/-/g, '_')
    .replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

function toEvent(channel: string): string {
  return EVENT_MAP[channel] ?? channel.replace(/:/g, '_').replace(/-/g, '_');
}

// ─── Core Functions ─────────────────────────────────────────

export async function invoke<T = unknown>(channel: string, ...args: unknown[]): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  const command = toCommand(channel);

  if (NATIVE_COMMANDS.has(command)) {
    return tauriInvoke<T>(command, args[0] as Record<string, unknown> | undefined);
  }

  // Tauri 2 #[tauri::command] uses serde(rename_all = "camelCase")
  // which transforms Rust param `_args` → JSON key `args`
  return tauriInvoke<T>(command, { args: args });
}

export function on(channel: string, handler: IpcHandler): () => void {
  let cancelled = false;
  let unlisten: (() => void) | null = null;
  const tauriEvent = toEvent(channel);

  import('@tauri-apps/api/event').then(({ listen }) => {
    if (cancelled) return;
    listen<unknown>(tauriEvent, (event) => {
      handler(event.payload);
    }).then((fn) => {
      if (cancelled) { fn(); return; }
      unlisten = fn;
    }).catch((err) => {
      console.warn(`[bridge] Failed to listen for ${tauriEvent}:`, err);
    });
  }).catch((err) => {
    console.warn('[bridge] Failed to import event module:', err);
  });

  return () => {
    cancelled = true;
    unlisten?.();
  };
}

export function once(channel: string, handler: IpcHandler): () => void {
  let unsub: (() => void) | null = null;
  unsub = on(channel, (...args: unknown[]) => {
    unsub?.();
    handler(...args);
  });
  return () => unsub?.();
}

export async function openExternal(url: string): Promise<void> {
  const { openUrl } = await import('@tauri-apps/plugin-opener');
  await openUrl(url);
}

export async function showItemInFolder(path: string): Promise<void> {
  const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
  await revealItemInDir(path);
}

export function getPlatform(): string {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes('mac')) return 'darwin';
  if (ua.includes('win')) return 'win32';
  return 'linux';
}

export function getIsDev(): boolean {
  return import.meta.env.DEV;
}
