/**
 * Settings Page
 * Application configuration
 */
import { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Sun,
  Moon,
  Monitor,
  RefreshCw,
  Terminal,
  ExternalLink,
  Key,
  Download,
  Copy,
  FileText,
  Zap,
  Keyboard,
  Bell,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { toast } from 'sonner';
import { useSettingsStore } from '@/stores/settings';
import { useGatewayStore } from '@/stores/gateway';
import { useUpdateStore } from '@/stores/update';
import { invoke, openExternal, showItemInFolder, getPlatform, getIsDev } from '@/lib/bridge';
import { ProvidersSettings } from '@/components/settings/ProvidersSettings';
import { UpdateSettings } from '@/components/settings/UpdateSettings';
import { useTranslation } from 'react-i18next';
import { SUPPORTED_LANGUAGES } from '@/i18n';
type ControlUiInfo = {
  url: string;
  token: string;
  port: number;
};

export function Settings() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const {
    theme,
    setTheme,
    language,
    setLanguage,
    launchAtStartup,
    setLaunchAtStartup,
    gatewayAutoStart,
    setGatewayAutoStart,
    autoCheckUpdate,
    setAutoCheckUpdate,
    autoDownloadUpdate,
    setAutoDownloadUpdate,
    enableNotifications,
    setEnableNotifications,
    devModeUnlocked,
    setDevModeUnlocked,
  } = useSettingsStore();

  const { status: gatewayStatus, restart: restartGateway } = useGatewayStore();
  const currentVersion = useUpdateStore((state) => state.currentVersion);
  const updateSetAutoDownload = useUpdateStore((state) => state.setAutoDownload);
  const [controlUiInfo, setControlUiInfo] = useState<ControlUiInfo | null>(null);
  const [openclawCliCommand, setOpenclawCliCommand] = useState('');
  const [openclawCliError, setOpenclawCliError] = useState<string | null>(null);
  const [installingCli, setInstallingCli] = useState(false);

  // Spotlight shortcut
  const [currentShortcut, setCurrentShortcut] = useState('');
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    invoke<{ spotlight: string }>('shortcut:get').then((config) => {
      setCurrentShortcut(config.spotlight);
    });
  }, []);

  const handleKeyRecording = useCallback((e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    // Build accelerator string from the event
    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push('CommandOrControl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');

    const key = e.key;
    // Only accept non-modifier keys
    if (['Control', 'Meta', 'Alt', 'Shift'].includes(key)) return;

    // Map special keys
    const keyMap: Record<string, string> = {
      ' ': 'Space',
      'ArrowUp': 'Up',
      'ArrowDown': 'Down',
      'ArrowLeft': 'Left',
      'ArrowRight': 'Right',
      'Escape': 'Escape',
    };
    const mappedKey = keyMap[key] || (key.length === 1 ? key.toUpperCase() : key);

    if (mappedKey === 'Escape' && parts.length === 0) {
      // Plain Escape cancels recording
      setIsRecording(false);
      return;
    }

    parts.push(mappedKey);
    const accelerator = parts.join('+');

    // Must have at least one modifier
    if (parts.length < 2) return;

    setIsRecording(false);

    // Try to update the shortcut
    invoke<{ success: boolean; error?: string }>('shortcut:update', { config: { spotlight: accelerator } }).then((r) => {
      if (r.success) {
        setCurrentShortcut(accelerator);
        toast.success(`Shortcut updated to ${accelerator}`);
      } else {
        toast.error(r.error || 'Failed to update shortcut');
      }
    });
  }, []);

  useEffect(() => {
    if (isRecording) {
      window.addEventListener('keydown', handleKeyRecording, true);
      return () => window.removeEventListener('keydown', handleKeyRecording, true);
    }
  }, [isRecording, handleKeyRecording]);

  const platform = getPlatform();
  const isMac = platform === 'darwin';
  const isWindows = platform === 'win32';
  const isLinux = platform === 'linux';
  const isDev = getIsDev();
  const showCliTools = isMac || isWindows || isLinux;
  const [showLogs, setShowLogs] = useState(false);
  const [logContent, setLogContent] = useState('');

  const handleShowLogs = async () => {
    try {
      const logs = await invoke<string>('log:readFile', 100);
      setLogContent(logs);
      setShowLogs(true);
    } catch {
      setLogContent('(Failed to load logs)');
      setShowLogs(true);
    }
  };

  const handleOpenLogDir = async () => {
    try {
      const logDir = await invoke<string>('log:getDir');
      if (logDir) {
        await showItemInFolder(logDir);
      }
    } catch (err) {
      console.warn('Failed to open log directory:', err);
    }
  };

  // Open developer console
  const openDevConsole = async () => {
    try {
      const result = await invoke<{
        success: boolean;
        url?: string;
        token?: string;
        port?: number;
        error?: string;
      }>('gateway:getControlUiUrl');
      if (result.success && result.url && result.token && typeof result.port === 'number') {
        setControlUiInfo({ url: result.url, token: result.token, port: result.port });
        openExternal(result.url);
      } else {
        console.error('Failed to get Dev Console URL:', result.error);
      }
    } catch (err) {
      console.error('Error opening Dev Console:', err);
    }
  };

  const refreshControlUiInfo = async () => {
    try {
      const result = await invoke<{
        success: boolean;
        url?: string;
        token?: string;
        port?: number;
      }>('gateway:getControlUiUrl');
      if (result.success && result.url && result.token && typeof result.port === 'number') {
        setControlUiInfo({ url: result.url, token: result.token, port: result.port });
      }
    } catch {
      // Ignore refresh errors
    }
  };

  const handleCopyGatewayToken = async () => {
    if (!controlUiInfo?.token) return;
    try {
      await navigator.clipboard.writeText(controlUiInfo.token);
      toast.success(t('developer.tokenCopied'));
    } catch (error) {
      toast.error(`Failed to copy token: ${String(error)}`);
    }
  };

  useEffect(() => {
    if (!showCliTools) return;
    let cancelled = false;

    const loadCliCommand = async () => {
      try {
        const result = await invoke<{
          success: boolean;
          command?: string;
          error?: string;
        }>('openclaw:getCliCommand');
        if (cancelled) return;
        if (result.success && result.command) {
          setOpenclawCliCommand(result.command);
          setOpenclawCliError(null);
        } else {
          setOpenclawCliCommand('');
          setOpenclawCliError(result.error || 'OpenClaw CLI unavailable');
        }
      } catch (error) {
        if (cancelled) return;
        setOpenclawCliCommand('');
        setOpenclawCliError(String(error));
      }
    };

    loadCliCommand();
    return () => {
      cancelled = true;
    };
  }, [devModeUnlocked, showCliTools]);

  const handleCopyCliCommand = async () => {
    if (!openclawCliCommand) return;
    try {
      await navigator.clipboard.writeText(openclawCliCommand);
      toast.success(t('developer.cmdCopied'));
    } catch (error) {
      toast.error(`Failed to copy command: ${String(error)}`);
    }
  };

  const handleInstallCliCommand = async () => {
    if (!isMac || installingCli) return;
    try {
      const confirmation = await invoke<{ response: number }>('dialog:message', {
        type: 'question',
        title: t('developer.installTitle'),
        message: t('developer.installMessage'),
        detail: t('developer.installDetail'),
        buttons: ['Cancel', 'Install'],
        defaultId: 1,
        cancelId: 0,
      });

      if (confirmation.response !== 1) return;

      setInstallingCli(true);
      const result = await invoke<{
        success: boolean;
        path?: string;
        error?: string;
      }>('openclaw:installCliMac');

      if (result.success) {
        toast.success(`Installed command at ${result.path ?? '/usr/local/bin/openclaw'}`);
      } else {
        toast.error(result.error || 'Failed to install command');
      }
    } catch (error) {
      toast.error(`Install failed: ${String(error)}`);
    } finally {
      setInstallingCli(false);
    }
  };

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">{t('title')}</h1>
        <p className="text-muted-foreground">
          {t('subtitle')}
        </p>
      </div>

      {/* Appearance */}
      <Card>
        <CardHeader>
          <CardTitle>{t('appearance.title')}</CardTitle>
          <CardDescription>{t('appearance.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>{t('appearance.theme')}</Label>
            <div className="flex gap-2">
              <Button
                variant={theme === 'light' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTheme('light')}
              >
                <Sun className="h-4 w-4 mr-2" />
                {t('appearance.light')}
              </Button>
              <Button
                variant={theme === 'dark' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTheme('dark')}
              >
                <Moon className="h-4 w-4 mr-2" />
                {t('appearance.dark')}
              </Button>
              <Button
                variant={theme === 'system' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTheme('system')}
              >
                <Monitor className="h-4 w-4 mr-2" />
                {t('appearance.system')}
              </Button>
            </div>
          </div>
          <div className="space-y-2">
            <Label>{t('appearance.language')}</Label>
            <div className="flex gap-2">
              {SUPPORTED_LANGUAGES.map((lang) => (
                <Button
                  key={lang.code}
                  variant={language === lang.code ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setLanguage(lang.code)}
                >
                  {lang.label}
                </Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* AI Providers */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            {t('aiProviders.title')}
          </CardTitle>
          <CardDescription>{t('aiProviders.description')}</CardDescription>
        </CardHeader>
        <CardContent>
          <ProvidersSettings />
        </CardContent>
      </Card>

      {/* Gateway */}
      <Card>
        <CardHeader>
          <CardTitle>{t('gateway.title')}</CardTitle>
          <CardDescription>{t('gateway.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>{t('gateway.status')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('gateway.port')}: {gatewayStatus.port}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Badge
                variant={
                  gatewayStatus.state === 'running'
                    ? 'success'
                    : gatewayStatus.state === 'error'
                      ? 'destructive'
                      : 'secondary'
                }
              >
                {gatewayStatus.state}
              </Badge>
              <Button variant="outline" size="sm" onClick={restartGateway}>
                <RefreshCw className="h-4 w-4 mr-2" />
                {t('common:actions.restart')}
              </Button>
              <Button variant="outline" size="sm" onClick={handleShowLogs}>
                <FileText className="h-4 w-4 mr-2" />
                {t('gateway.logs')}
              </Button>
            </div>
          </div>

          {showLogs && (
            <div className="mt-4 p-4 rounded-lg bg-black/10 dark:bg-black/40 border border-border">
              <div className="flex items-center justify-between mb-2">
                <p className="font-medium text-sm">{t('gateway.appLogs')}</p>
                <div className="flex gap-2">
                  <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={handleOpenLogDir}>
                    <ExternalLink className="h-3 w-3 mr-1" />
                    {t('gateway.openFolder')}
                  </Button>
                  <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setShowLogs(false)}>
                    {t('common:actions.close')}
                  </Button>
                </div>
              </div>
              <pre className="text-xs text-muted-foreground bg-background/50 p-3 rounded max-h-60 overflow-auto whitespace-pre-wrap font-mono">
                {logContent || t('chat:noLogs')}
              </pre>
            </div>
          )}

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label>{t('gateway.autoStart')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('gateway.autoStartDesc')}
              </p>
            </div>
            <Switch
              checked={gatewayAutoStart}
              onCheckedChange={setGatewayAutoStart}
            />
          </div>
        </CardContent>
      </Card>

      {/* Desktop Assistant (Spotlight) */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5" />
            {t('spotlight.title')}
          </CardTitle>
          <CardDescription>{t('spotlight.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label className="flex items-center gap-2">
                <Keyboard className="h-4 w-4" />
                {t('spotlight.shortcutLabel')}
              </Label>
              <p className="text-sm text-muted-foreground">
                {t('spotlight.shortcutDesc')}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Input
                readOnly
                value={isRecording ? t('spotlight.recording') : (currentShortcut || '')}
                className="w-[220px] text-center font-mono"
              />
              {isRecording ? (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setIsRecording(false)}
                >
                  {t('spotlight.cancelBtn')}
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setIsRecording(true)}
                >
                  {t('spotlight.recordBtn')}
                </Button>
              )}
            </div>
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label className="flex items-center gap-2">
                <Bell className="h-4 w-4" />
                {t('spotlight.notificationsLabel')}
              </Label>
              <p className="text-sm text-muted-foreground">
                {t('spotlight.notificationsDesc')}
              </p>
            </div>
            <Switch
              checked={enableNotifications}
              onCheckedChange={setEnableNotifications}
            />
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label>{t('spotlight.launchAtStartup')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('spotlight.launchAtStartupDesc')}
              </p>
            </div>
            <Switch
              checked={launchAtStartup}
              onCheckedChange={setLaunchAtStartup}
            />
          </div>
        </CardContent>
      </Card>

      {/* Updates */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            {t('updates.title')}
          </CardTitle>
          <CardDescription>{t('updates.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <UpdateSettings />

          <Separator />

          <div className="flex items-center justify-between">
            <div>
              <Label>{t('updates.autoCheck')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('updates.autoCheckDesc')}
              </p>
            </div>
            <Switch
              checked={autoCheckUpdate}
              onCheckedChange={setAutoCheckUpdate}
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <Label>{t('updates.autoDownload')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('updates.autoDownloadDesc')}
              </p>
            </div>
            <Switch
              checked={autoDownloadUpdate}
              onCheckedChange={(value) => {
                setAutoDownloadUpdate(value);
                updateSetAutoDownload(value);
              }}
            />
          </div>
        </CardContent>
      </Card>

      {/* Advanced */}
      <Card>
        <CardHeader>
          <CardTitle>{t('advanced.title')}</CardTitle>
          <CardDescription>{t('advanced.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>{t('advanced.devMode')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('advanced.devModeDesc')}
              </p>
            </div>
            <Switch
              checked={devModeUnlocked}
              onCheckedChange={setDevModeUnlocked}
            />
          </div>
          <Separator />
          <div className="flex items-center justify-between gap-4">
            <div>
              <Label>{t('advanced.resetSetup')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('advanced.resetSetupDesc')}
              </p>
            </div>
            <Button
              variant="outline"
              onClick={() => {
                useSettingsStore.getState().resetSetup();
                navigate('/setup');
              }}
            >
              {t('advanced.resetSetupBtn')}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Developer */}
      {devModeUnlocked && (
        <Card>
          <CardHeader>
            <CardTitle>{t('developer.title')}</CardTitle>
            <CardDescription>{t('developer.description')}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>{t('developer.console')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('developer.consoleDesc')}
              </p>
              <Button variant="outline" onClick={openDevConsole}>
                <Terminal className="h-4 w-4 mr-2" />
                {t('developer.openConsole')}
                <ExternalLink className="h-3 w-3 ml-2" />
              </Button>
              <p className="text-xs text-muted-foreground">
                {t('developer.consoleNote')}
              </p>
              <div className="space-y-2 pt-2">
                <Label>{t('developer.gatewayToken')}</Label>
                <p className="text-sm text-muted-foreground">
                  {t('developer.gatewayTokenDesc')}
                </p>
                <div className="flex gap-2">
                  <Input
                    readOnly
                    value={controlUiInfo?.token || ''}
                    placeholder={t('developer.tokenUnavailable')}
                    className="font-mono"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={refreshControlUiInfo}
                    disabled={!devModeUnlocked}
                  >
                    <RefreshCw className="h-4 w-4 mr-2" />
                    {t('common:actions.load')}
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleCopyGatewayToken}
                    disabled={!controlUiInfo?.token}
                  >
                    <Copy className="h-4 w-4 mr-2" />
                    {t('common:actions.copy')}
                  </Button>
                </div>
              </div>
            </div>
            {showCliTools && (
              <>
                <Separator />
                <div className="space-y-2">
                  <Label>{t('developer.cli')}</Label>
                  <p className="text-sm text-muted-foreground">
                    {t('developer.cliDesc')}
                  </p>
                  {isWindows && (
                    <p className="text-xs text-muted-foreground">
                      {t('developer.cliPowershell')}
                    </p>
                  )}
                  <div className="flex gap-2">
                    <Input
                      readOnly
                      value={openclawCliCommand}
                      placeholder={openclawCliError || t('developer.cmdUnavailable')}
                      className="font-mono"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      onClick={handleCopyCliCommand}
                      disabled={!openclawCliCommand}
                    >
                      <Copy className="h-4 w-4 mr-2" />
                      {t('common:actions.copy')}
                    </Button>
                  </div>
                  {isMac && !isDev && (
                    <div className="space-y-1">
                      <Button
                        type="button"
                        variant="outline"
                        onClick={handleInstallCliCommand}
                        disabled={installingCli}
                      >
                        <Terminal className="h-4 w-4 mr-2" />
                        {t('developer.installCmd')}
                      </Button>
                      <p className="text-xs text-muted-foreground">
                        {t('developer.installCmdDesc')}
                      </p>
                    </div>
                  )}
                </div>
              </>
            )}
          </CardContent>
        </Card>
      )}

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle>{t('about.title')}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-muted-foreground">
          <p>
            <strong>{t('about.appName')}</strong> - {t('about.tagline')}
          </p>
          <p>{t('about.basedOn')}</p>
          <p>{t('about.version', { version: currentVersion })}</p>
          <div className="flex gap-4 pt-2">
            <Button
              variant="link"
              className="h-auto p-0"
              onClick={() => openExternal('https://claw-x.com')}
            >
              {t('about.docs')}
            </Button>
            <Button
              variant="link"
              className="h-auto p-0"
              onClick={() => openExternal('https://github.com/ValueCell-ai/ClawX')}
            >
              {t('about.github')}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default Settings;
