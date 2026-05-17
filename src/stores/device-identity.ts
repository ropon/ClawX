/**
 * Device Identity Store — Ed25519 Key Pair Management
 *
 * Each OneClaw installation gets a unique Ed25519 key pair.
 * The public key is registered with the OneClaw backend during device pairing.
 * This binds the device to the user's account for secure access.
 *
 * Storage: ~/.oneclaw/identity/device.json
 */
import { create } from 'zustand';

const DEVICE_ID_KEY = 'oneclaw:device:id';
const DEVICE_PUBKEY_KEY = 'oneclaw:device:pubkey';

interface DeviceIdentity {
  deviceId: string;
  publicKey: string;
  paired: boolean;
  pairedAt?: string;
}

interface DeviceIdentityState {
  identity: DeviceIdentity | null;
  ready: boolean;
  init: () => Promise<void>;
  getOrCreateIdentity: () => Promise<DeviceIdentity>;
  markPaired: () => void;
  getAuthHeaders: () => Record<string, string>;
}

export const useDeviceIdentityStore = create<DeviceIdentityState>((set, get) => ({
  identity: null,
  ready: false,

  init: async () => {
    const deviceId = localStorage.getItem(DEVICE_ID_KEY);
    const publicKey = localStorage.getItem(DEVICE_PUBKEY_KEY);
    if (deviceId && publicKey) {
      set({
        identity: { deviceId, publicKey, paired: true },
        ready: true,
      });
    } else {
      // Generate new identity via IPC
      try {
        const { invokeIpc } = await import('@/lib/api-client');
        const result = await invokeIpc('device:get-or-create-identity') as {
          deviceId: string;
          publicKey: string;
          paired: boolean;
          pairedAt?: string;
        };
        if (result?.deviceId) {
          localStorage.setItem(DEVICE_ID_KEY, result.deviceId);
          localStorage.setItem(DEVICE_PUBKEY_KEY, result.publicKey);
          set({
            identity: {
              deviceId: result.deviceId,
              publicKey: result.publicKey,
              paired: result.paired,
              pairedAt: result.pairedAt,
            },
            ready: true,
          });
          return;
        }
      } catch {
        // Fallback: generate client-side identity
        const fallbackId = `device-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
        localStorage.setItem(DEVICE_ID_KEY, fallbackId);
        localStorage.setItem(DEVICE_PUBKEY_KEY, '');
        set({
          identity: { deviceId: fallbackId, publicKey: '', paired: false },
          ready: true,
        });
      }
    }
  },

  getOrCreateIdentity: async () => {
    const state = get();
    if (state.identity) return state.identity;
    await state.init();
    return get().identity!;
  },

  markPaired: () => {
    const state = get();
    if (state.identity) {
      const updated = { ...state.identity, paired: true, pairedAt: new Date().toISOString() };
      set({ identity: updated });
    }
  },

  getAuthHeaders: () => {
    const state = get();
    if (!state.identity?.deviceId) return {};
    return {
      'x-device-id': state.identity.deviceId,
    };
  },
}));
