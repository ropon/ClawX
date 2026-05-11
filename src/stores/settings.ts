/**
 * Settings State Store
 * Manages application settings
 */
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import i18n from '@/i18n';

type Theme = 'light' | 'dark' | 'system';
type UpdateChannel = 'stable' | 'beta' | 'dev';

interface SettingsState {
  // General
  theme: Theme;
  language: string;
  startMinimized: boolean;
  launchAtStartup: boolean;

  // Gateway
  gatewayAutoStart: boolean;
  gatewayPort: number;

  // Update
  updateChannel: UpdateChannel;
  autoCheckUpdate: boolean;
  autoDownloadUpdate: boolean;

  // Notifications
  enableNotifications: boolean;

  // UI State
  sidebarCollapsed: boolean;
  devModeUnlocked: boolean;

  // Setup
  setupComplete: boolean;
  setupVersion: string;

  // Actions
  setTheme: (theme: Theme) => void;
  setLanguage: (language: string) => void;
  setStartMinimized: (value: boolean) => void;
  setLaunchAtStartup: (value: boolean) => void;
  setGatewayAutoStart: (value: boolean) => void;
  setGatewayPort: (port: number) => void;
  setUpdateChannel: (channel: UpdateChannel) => void;
  setAutoCheckUpdate: (value: boolean) => void;
  setAutoDownloadUpdate: (value: boolean) => void;
  setEnableNotifications: (value: boolean) => void;
  setSidebarCollapsed: (value: boolean) => void;
  setDevModeUnlocked: (value: boolean) => void;
  markSetupComplete: (version: string) => void;
  resetSetup: () => void;
  checkSetupVersion: (currentVersion: string) => void;
  resetSettings: () => void;
}

const defaultSettings = {
  theme: 'system' as Theme,
  language: (() => {
    const lang = navigator.language.toLowerCase();
    if (lang.startsWith('zh')) return 'zh';
    if (lang.startsWith('ja')) return 'ja';
    return 'en';
  })(),
  startMinimized: false,
  launchAtStartup: false,
  gatewayAutoStart: true,
  gatewayPort: 18789,
  updateChannel: 'stable' as UpdateChannel,
  autoCheckUpdate: true,
  autoDownloadUpdate: false,
  enableNotifications: true,
  sidebarCollapsed: false,
  devModeUnlocked: false,
  setupComplete: false,
  setupVersion: '',
};

// Increment SETTINGS_VERSION to force re-setup after breaking changes (e.g. Tauri migration)
const SETTINGS_VERSION = 2;

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      ...defaultSettings,

      setTheme: (theme) => set({ theme }),
      setLanguage: (language) => { i18n.changeLanguage(language); set({ language }); },
      setStartMinimized: (startMinimized) => set({ startMinimized }),
      setLaunchAtStartup: (launchAtStartup) => {
        set({ launchAtStartup });
        // Toggle system autostart via Tauri plugin
        import('@tauri-apps/plugin-autostart').then(({ enable, disable }) => {
          (launchAtStartup ? enable() : disable()).catch((err) => {
            console.warn('Failed to toggle autostart:', err);
          });
        }).catch(() => { /* dev mode — plugin not available */ });
      },
      setGatewayAutoStart: (gatewayAutoStart) => set({ gatewayAutoStart }),
      setGatewayPort: (gatewayPort) => set({ gatewayPort }),
      setUpdateChannel: (updateChannel) => set({ updateChannel }),
      setAutoCheckUpdate: (autoCheckUpdate) => set({ autoCheckUpdate }),
      setAutoDownloadUpdate: (autoDownloadUpdate) => set({ autoDownloadUpdate }),
      setEnableNotifications: (enableNotifications) => set({ enableNotifications }),
      setSidebarCollapsed: (sidebarCollapsed) => set({ sidebarCollapsed }),
      setDevModeUnlocked: (devModeUnlocked) => set({ devModeUnlocked }),
      markSetupComplete: (version: string) => set({ setupComplete: true, setupVersion: version }),
      resetSetup: () => set({ setupComplete: false, setupVersion: '' }),
      checkSetupVersion: (currentVersion: string) => {
        const { setupVersion, setupComplete } = useSettingsStore.getState();
        if (setupComplete && setupVersion !== currentVersion) {
          set({ setupComplete: false });
        }
      },
      resetSettings: () => set(defaultSettings),
    }),
    {
      name: 'clawx-settings',
      version: SETTINGS_VERSION,
      migrate: (persistedState, version) => {
        const state = persistedState as Record<string, unknown>;
        if (version < SETTINGS_VERSION) {
          // Force re-setup on major settings schema change
          return { ...state, setupComplete: false, setupVersion: '' };
        }
        return state;
      },
    }
  )
);
