/**
 * Authentication Store — OneClaw Backend Proxy Architecture
 *
 * Flow:
 *   1. Login to OneClaw backend → get uid + token
 *   2. Auto-configure: fetch provider list from backend (apiKey = placeholder)
 *   3. All model requests go through backend proxy with x-auth-uid / x-auth-token
 */
import { create } from 'zustand';
import {
  login as apiLogin,
  logout as apiLogout,
  setAuthSession,
  getAuthToken,
  getAuthUid,
  getUser as getStoredUser,
  fetchUserProviders,
  type LoginResponse,
  type BackendProvider,
} from '@/lib/auth-api';

interface AuthState {
  uid: string | null;
  token: string | null;
  user: LoginResponse['user'] | null;
  isAuthenticated: boolean;
  loading: boolean;
  error: string | null;
  login: (email: string, password: string) => Promise<boolean>;
  logout: () => void;
  checkAuth: () => void;
  fetchAndAutoConfigureProviders: () => Promise<{
    success: boolean;
    providers?: BackendProvider[];
  }>;
}

export const useAuthStore = create<AuthState>((set) => ({
  uid: null,
  token: null,
  user: null,
  isAuthenticated: false,
  loading: false,
  error: null,

  login: async (email, password) => {
    set({ loading: true, error: null });
    try {
      const response = await apiLogin(email, password);
      if (response.success && response.uid && response.token) {
        setAuthSession(response.uid, response.token, response.user);
        set({
          uid: response.uid,
          token: response.token,
          user: response.user,
          isAuthenticated: true,
          loading: false,
          error: null,
        });
        return true;
      } else {
        set({ loading: false, error: response.error || 'Login failed' });
        return false;
      }
    } catch (error) {
      set({ loading: false, error: String(error) });
      return false;
    }
  },

  logout: () => {
    apiLogout();
    set({ uid: null, token: null, user: null, isAuthenticated: false, error: null });
  },

  checkAuth: () => {
    const token = getAuthToken();
    const uid = getAuthUid();
    if (token && uid) {
      const user = getStoredUser();
      set({ uid, token, user, isAuthenticated: true });
    } else {
      set({ uid: null, token: null, user: null, isAuthenticated: false });
    }
  },

  fetchAndAutoConfigureProviders: async () => {
    const result = await fetchUserProviders();
    if (result.success && result.providers) {
      return { success: true, providers: result.providers };
    }
    return { success: false };
  },
}));
