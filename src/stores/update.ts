/**
 * Update State Store
 * Manages application update state via @tauri-apps/plugin-updater
 */
import { create } from 'zustand';
import { useSettingsStore } from './settings';

export interface UpdateInfo {
  version: string;
  releaseDate?: string;
  releaseNotes?: string | null;
}

export interface ProgressInfo {
  total: number;
  delta: number;
  transferred: number;
  percent: number;
  bytesPerSecond: number;
}

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'not-available'
  | 'downloading'
  | 'downloaded'
  | 'error';

interface UpdateState {
  status: UpdateStatus;
  currentVersion: string;
  updateInfo: UpdateInfo | null;
  progress: ProgressInfo | null;
  error: string | null;
  isInitialized: boolean;

  // Actions
  init: () => Promise<void>;
  checkForUpdates: () => Promise<void>;
  downloadUpdate: () => Promise<void>;
  installUpdate: () => void;
  setChannel: (channel: 'stable' | 'beta' | 'dev') => Promise<void>;
  setAutoDownload: (enable: boolean) => Promise<void>;
  clearError: () => void;
}

// Cache the pending Update object between check and download/install
let _pendingUpdate: Awaited<ReturnType<typeof import('@tauri-apps/plugin-updater').check>> = null;

export const useUpdateStore = create<UpdateState>((set, get) => ({
  status: 'idle',
  currentVersion: '0.0.0',
  updateInfo: null,
  progress: null,
  error: null,
  isInitialized: false,

  init: async () => {
    if (get().isInitialized) return;

    // Get current version via Tauri API
    try {
      const { getVersion } = await import('@tauri-apps/api/app');
      const version = await getVersion();
      set({ currentVersion: version });
    } catch (error) {
      console.error('Failed to get version:', error);
    }

    set({ isInitialized: true });

    // Auto-check for updates on startup (respects user toggle)
    const { autoCheckUpdate } = useSettingsStore.getState();
    if (autoCheckUpdate) {
      setTimeout(() => {
        get().checkForUpdates().catch((err) => {
          console.warn('Auto-check for updates failed:', err);
        });
      }, 10000);
    }
  },

  checkForUpdates: async () => {
    set({ status: 'checking', error: null });

    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();

      if (update) {
        _pendingUpdate = update;
        set({
          status: 'available',
          updateInfo: {
            version: update.version,
            releaseDate: update.date ?? undefined,
            releaseNotes: typeof update.body === 'string' ? update.body : null,
          },
        });

        // Auto-download if enabled
        const { autoDownloadUpdate } = useSettingsStore.getState();
        if (autoDownloadUpdate) {
          get().downloadUpdate().catch((err) => {
            console.warn('Auto-download update failed:', err);
          });
        }
      } else {
        set({ status: 'not-available' });
      }
    } catch (error) {
      const msg = String(error);
      // In dev mode, the updater may not be configured
      if (msg.includes('no updater') || msg.includes('not configured')) {
        set({ status: 'error', error: 'Updater not available in dev mode' });
      } else {
        set({ status: 'error', error: msg });
      }
    }
  },

  downloadUpdate: async () => {
    if (!_pendingUpdate) {
      set({ status: 'error', error: 'No update available to download' });
      return;
    }

    set({ status: 'downloading', error: null });

    try {
      let totalBytes = 0;
      let transferredBytes = 0;

      await _pendingUpdate.downloadAndInstall((event) => {
        if (event.event === 'Started' && event.data.contentLength) {
          totalBytes = event.data.contentLength;
        } else if (event.event === 'Progress') {
          transferredBytes += event.data.chunkLength;
          const percent = totalBytes > 0 ? (transferredBytes / totalBytes) * 100 : 0;
          set({
            progress: {
              total: totalBytes,
              delta: event.data.chunkLength,
              transferred: transferredBytes,
              percent,
              bytesPerSecond: 0,
            },
          });
        } else if (event.event === 'Finished') {
          set({ status: 'downloaded', progress: null });
        }
      });

      // downloadAndInstall will restart the app on success
    } catch (error) {
      set({ status: 'error', error: String(error) });
    }
  },

  installUpdate: () => {
    // downloadAndInstall handles both download + install in Tauri 2
    // If already downloaded, re-trigger
    if (_pendingUpdate) {
      get().downloadUpdate().catch((err) => {
        console.warn('Install update failed:', err);
      });
    }
  },

  setChannel: async (_channel) => {
    // Channel management is handled via Tauri updater config (tauri.conf.json endpoints)
    // No runtime API needed — this is a no-op at runtime
    console.info('Update channel is configured in tauri.conf.json');
  },

  setAutoDownload: async (_enable) => {
    // Auto-download preference is managed by settings store
    // and checked in checkForUpdates() — no backend call needed
  },

  clearError: () => set({ error: null, status: 'idle' }),
}));
