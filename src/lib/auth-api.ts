/**
 * Auth API — OneClaw Backend Proxy Architecture
 *
 * Architecture:
 *   Client ←→ OneClaw Backend (api.oneclaw.net) ←→ Model Providers
 *
 * 1. Client logs into OneClaw backend → gets uid + token
 * 2. Backend returns available model providers (apiKey = placeholder)
 * 3. All chat requests go through backend proxy with x-auth-uid / x-auth-token headers
 * 4. Backend holds real API keys and proxies to actual model providers
 */

import { hostApiFetch } from './host-api';

// ── Types ────────────────────────────────────────────────────

export interface LoginResponse {
  success: boolean;
  token?: string;          // JWT token (contains uid, user info)
  user?: {
    id: string;
    email: string;
    name?: string;
    avatar?: string;
  };
  error?: string;
}

/**
 * Backend-hosted provider — client never sees the real API key.
 */
export interface BackendProvider {
  vendorId: string;
  label: string;
  model: string;
  baseUrl: string;        // Proxy endpoint on OneClaw backend
  apiKey: string;          // Always "oneclaw-placeholder"
  enabled: boolean;
  isDefault?: boolean;
}

export interface FetchProvidersResponse {
  success: boolean;
  providers?: BackendProvider[];
  error?: string;
}

/** Device pairing response */
export interface DevicePairResponse {
  success: boolean;
  deviceId?: string;
  error?: string;
}

// ── Config ───────────────────────────────────────────────────

/** OneClaw backend base URL */
const BACKEND_URL = 'https://api.oneclaw.net';

/** Placeholder API key — replaced by backend with real key */
export const ONECLAW_PLACEHOLDER_API_KEY = 'oneclaw-placeholder';

/** localStorage keys */
const TOKEN_KEY = 'oneclaw:auth:token';
const UID_KEY = 'oneclaw:auth:uid';
const USER_KEY = 'oneclaw:auth:user';

// ── Auth API ─────────────────────────────────────────────────

/** Login with email + password to OneClaw backend */
export async function login(email: string, password: string): Promise<LoginResponse> {
  // Route through local Host API which proxies to OneClaw backend
  // This keeps the backend URL configurable and allows offline fallback
  try {
    const response = await hostApiFetch<LoginResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password, backendUrl: BACKEND_URL }),
    });
    return response;
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Login failed',
    };
  }
}

/** Fetch user's available providers from OneClaw backend */
export async function fetchUserProviders(): Promise<FetchProvidersResponse> {
  const token = getAuthToken();
  const uid = getAuthUid();
  if (!token || !uid) {
    return { success: false, error: 'Not authenticated' };
  }
  try {
    const response = await hostApiFetch<FetchProvidersResponse>('/api/auth/providers', {
      method: 'POST',
      body: JSON.stringify({ backendUrl: BACKEND_URL, uid, token }),
    });
    return response;
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Failed to fetch providers',
    };
  }
}

/** Pair this device with the user's account (Ed25519 key exchange) */
export async function pairDevice(
  uid: string,
  token: string,
  devicePublicKey: string,
): Promise<DevicePairResponse> {
  try {
    const response = await hostApiFetch<DevicePairResponse>('/api/auth/pair-device', {
      method: 'POST',
      body: JSON.stringify({
        backendUrl: BACKEND_URL,
        uid,
        token,
        devicePublicKey,
      }),
    });
    return response;
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Device pairing failed',
    };
  }
}

// ── Token Management ─────────────────────────────────────────

export function setAuthSession(uid: string, token: string, user: LoginResponse['user']): void {
  localStorage.setItem(UID_KEY, uid);
  localStorage.setItem(TOKEN_KEY, token);
  if (user) {
    localStorage.setItem(USER_KEY, JSON.stringify(user));
  }
}

export function getAuthToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function getAuthUid(): string | null {
  return localStorage.getItem(UID_KEY);
}

export function getUser(): LoginResponse['user'] | null {
  const raw = localStorage.getItem(USER_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function isLoggedIn(): boolean {
  return !!getAuthToken() && !!getAuthUid();
}

export function logout(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(UID_KEY);
  localStorage.removeItem(USER_KEY);
}

// ── Auth Headers (injected into every model request) ──────────

/** Build standard auth headers for backend-proxied requests */
export function buildAuthHeaders(): Record<string, string> {
  const uid = getAuthUid();
  const token = getAuthToken();
  if (!uid || !token) return {};
  return {
    'x-auth-uid': uid,
    'x-auth-token': token,
  };
}

// ── Backend URL ──────────────────────────────────────────────

export function getBackendUrl(): string {
  return BACKEND_URL;
}
