/**
 * Device Identity Management (Electron Main Process)
 * Generates Ed25519 key pair, stores in ~/.oneclaw/identity/device.json
 */
import { randomBytes, generateKeyPairSync } from 'crypto';
import { join } from 'path';
import { homedir } from 'os';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';

const IDENTITY_DIR = join(homedir(), '.oneclaw', 'identity');
const DEVICE_FILE = join(IDENTITY_DIR, 'device.json');

export interface DeviceIdentity {
  deviceId: string;
  publicKey: string;
  privateKey: string;
  paired: boolean;
  pairedAt?: string;
}

function ensureDir() {
  if (!existsSync(IDENTITY_DIR)) {
    mkdirSync(IDENTITY_DIR, { recursive: true });
  }
}

function readIdentity(): DeviceIdentity | null {
  if (!existsSync(DEVICE_FILE)) return null;
  try {
    return JSON.parse(readFileSync(DEVICE_FILE, 'utf-8'));
  } catch { return null; }
}

function writeIdentity(identity: DeviceIdentity): void {
  ensureDir();
  writeFileSync(DEVICE_FILE, JSON.stringify(identity, null, 2), 'utf-8');
}

function generateEd25519KeyPair(): { publicKey: string; privateKey: string } {
  const { publicKey, privateKey } = generateKeyPairSync('ed25519', {
    publicKeyEncoding: { type: 'spki', format: 'der' },
    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
  });
  return {
    publicKey: publicKey.toString('base64'),
    privateKey: privateKey.toString('base64'),
  };
}

export function getOrCreateDeviceIdentity(): DeviceIdentity {
  let identity = readIdentity();
  if (!identity) {
    const { publicKey, privateKey } = generateEd25519KeyPair();
    identity = {
      deviceId: `device-${Date.now()}-${randomBytes(8).toString('hex')}`,
      publicKey,
      privateKey,
      paired: false,
    };
    writeIdentity(identity);
  }
  return identity;
}

export function markDevicePaired(): void {
  const identity = readIdentity();
  if (identity && !identity.paired) {
    identity.paired = true;
    identity.pairedAt = new Date().toISOString();
    writeIdentity(identity);
  }
}

export function getDeviceAuthHeaders(): Record<string, string> {
  const identity = getOrCreateDeviceIdentity();
  return {
    'x-device-id': identity.deviceId,
    'x-device-pubkey': identity.publicKey,
  };
}
