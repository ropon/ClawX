/**
 * Auth Routes — Proxy authentication requests to OneClaw backend
 *
 * These routes act as a secure proxy between the Electron renderer
 * and the external OneClaw backend (api.oneclaw.net).
 *
 * The Electron main process adds device identity headers before forwarding.
 */
import type { IncomingMessage, ServerResponse } from 'http';
import type { HostApiContext } from '../context';
import { parseJsonBody, sendJson } from '../route-utils';
import { getOrCreateDeviceIdentity } from '../../utils/device-identity';

const BACKEND_BASE = 'https://api.oneclaw.net';

async function proxyToBackend(
  path: string,
  body: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const device = getOrCreateDeviceIdentity();
  const response = await fetch(`${BACKEND_BASE}${path}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-device-id': device.deviceId,
      'x-device-pubkey': device.publicKey,
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const errorText = await response.text().catch(() => '');
    return {
      success: false,
      error: `Backend returned ${response.status}: ${errorText}`,
    };
  }

  const data = await response.json();
  return data as Record<string, unknown>;
}

export async function handleAuthRoutes(
  req: IncomingMessage,
  res: ServerResponse,
  url: URL,
  _ctx: HostApiContext,
): Promise<boolean> {
  // POST /api/auth/login
  if (url.pathname === '/api/auth/login' && req.method === 'POST') {
    const body = await parseJsonBody<{ email: string; password: string }>(req);
    if (!body.email || !body.password) {
      sendJson(res, 400, { success: false, error: 'Email and password are required' });
      return true;
    }
    const result = await proxyToBackend('/api/auth/login', {
      email: body.email,
      password: body.password,
    });
    sendJson(res, 200, result);
    return true;
  }

  // POST /api/auth/providers
  if (url.pathname === '/api/auth/providers' && req.method === 'POST') {
    const body = await parseJsonBody<{ uid: string; token: string }>(req);
    if (!body.uid || !body.token) {
      sendJson(res, 401, { success: false, error: 'Authentication required' });
      return true;
    }
    const result = await proxyToBackend('/api/auth/providers', {
      uid: body.uid,
      token: body.token,
    });
    sendJson(res, 200, result);
    return true;
  }

  // POST /api/auth/pair-device
  if (url.pathname === '/api/auth/pair-device' && req.method === 'POST') {
    const body = await parseJsonBody<{ uid: string; token: string; devicePublicKey: string }>(req);
    if (!body.uid || !body.token || !body.devicePublicKey) {
      sendJson(res, 400, { success: false, error: 'Missing fields' });
      return true;
    }
    const result = await proxyToBackend('/api/auth/pair-device', {
      uid: body.uid,
      token: body.token,
      devicePublicKey: body.devicePublicKey,
    });
    sendJson(res, 200, result);
    return true;
  }

  return false;
}
