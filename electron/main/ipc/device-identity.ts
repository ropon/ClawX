/**
 * IPC handlers for device identity
 */
import { ipcMain } from 'electron';
import { getOrCreateDeviceIdentity, markDevicePaired, getDeviceAuthHeaders } from '../../utils/device-identity';

export function registerDeviceIdentityHandlers(): void {
  ipcMain.handle('device:get-or-create-identity', async () => {
    return getOrCreateDeviceIdentity();
  });

  ipcMain.handle('device:mark-paired', async () => {
    markDevicePaired();
    return { success: true };
  });

  ipcMain.handle('device:get-auth-headers', async () => {
    return getDeviceAuthHeaders();
  });
}
