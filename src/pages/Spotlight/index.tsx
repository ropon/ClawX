/**
 * Spotlight Page
 * Spotlight-style quick chat window — loaded at #/spotlight route
 */
import { useEffect } from 'react';
import { motion } from 'framer-motion';
import { useSpotlightStore } from '@/stores/spotlight';
import { useGatewayStore } from '@/stores/gateway';
import { on } from '@/lib/bridge';
import { SpotlightInput } from './SpotlightInput';
import { SpotlightResponse } from './SpotlightResponse';
import { ClipboardBar } from './ClipboardBar';
import i18n from '@/i18n';
import { useSettingsStore } from '@/stores/settings';

export function Spotlight() {
  const initGateway = useGatewayStore((s) => s.init);
  const readClipboard = useSpotlightStore((s) => s.readClipboard);
  const setVisible = useSpotlightStore((s) => s.setVisible);
  const clearConversation = useSpotlightStore((s) => s.clearConversation);
  const handleChatEvent = useSpotlightStore((s) => s.handleChatEvent);
  const sessionKey = useSpotlightStore((s) => s.sessionKey);
  const language = useSettingsStore((s) => s.language);

  // Sync language setting
  useEffect(() => {
    if (language && language !== i18n.language) {
      i18n.changeLanguage(language);
    }
  }, [language]);

  // Initialize Gateway connection (each BrowserWindow has its own JS context)
  useEffect(() => {
    initGateway();
  }, [initGateway]);

  // Listen for spotlight:shown → read clipboard + focus
  useEffect(() => {
    const unsub = on('spotlight:shown', () => {
      setVisible(true);
      readClipboard();
    });
    return () => {
      unsub();
    };
  }, [setVisible, readClipboard]);

  // Listen for spotlight:hidden → optionally clear
  useEffect(() => {
    const unsub = on('spotlight:hidden', () => {
      setVisible(false);
      // Clear conversation on hide for a fresh start next time
      clearConversation();
    });
    return () => {
      unsub();
    };
  }, [setVisible, clearConversation]);

  // Listen for Gateway notifications and forward spotlight session events
  useEffect(() => {
    const unsub = on('gateway:notification', (notification) => {
      const payload = notification as { method?: string; params?: Record<string, unknown> } | undefined;
      if (!payload || payload.method !== 'agent' || !payload.params) return;

      const p = payload.params;
      const data = (p.data && typeof p.data === 'object') ? (p.data as Record<string, unknown>) : {};

      // Only handle events for the spotlight session
      const evtSession = (p.sessionKey ?? data.sessionKey) as string | undefined;
      if (!evtSession || !evtSession.includes(':spotlight')) return;

      const normalizedEvent: Record<string, unknown> = {
        ...data,
        runId: p.runId ?? data.runId,
        sessionKey: p.sessionKey ?? data.sessionKey,
        state: p.state ?? data.state,
        message: p.message ?? data.message,
        errorMessage: p.errorMessage ?? data.errorMessage,
      };

      handleChatEvent(normalizedEvent);
    });
    return () => {
      unsub();
    };
  }, [handleChatEvent, sessionKey]);

  // Also listen for direct chat-message events
  useEffect(() => {
    const unsub = on('gateway:chat-message', (data) => {
      const chatData = data as Record<string, unknown>;
      const payload = ('message' in chatData && typeof chatData.message === 'object')
        ? chatData.message as Record<string, unknown>
        : chatData;

      // Only handle spotlight session events
      const evtSession = (chatData.sessionKey ?? payload.sessionKey) as string | undefined;
      if (!evtSession || !evtSession.includes(':spotlight')) return;

      if (payload.state) {
        handleChatEvent(payload);
      } else {
        handleChatEvent({
          state: 'final',
          message: payload,
          runId: chatData.runId ?? payload.runId,
        });
      }
    });
    return () => {
      unsub();
    };
  }, [handleChatEvent]);

  return (
    <div className="h-screen w-screen overflow-hidden select-none" style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}>
      <motion.div
        initial={{ opacity: 0, y: -10, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.15, ease: 'easeOut' }}
        className="mx-auto mt-0 w-full max-w-[680px] rounded-2xl border border-border/60 bg-background/80 backdrop-blur-xl shadow-2xl overflow-hidden"
      >
        <SpotlightInput />
        <ClipboardBar />
        <SpotlightResponse />
      </motion.div>
    </div>
  );
}

export default Spotlight;
