/**
 * Clipboard Bar (F-2.3)
 * Shows detected clipboard content type and quick action buttons
 */
import { useTranslation } from 'react-i18next';
import { useSpotlightStore } from '@/stores/spotlight';
import { getQuickActionsForType } from '@/utils/clipboard-detect';
import type { ClipboardContentType } from '@/types/spotlight';

const TYPE_COLORS: Record<ClipboardContentType, string> = {
  text: 'bg-blue-500/15 text-blue-600 dark:text-blue-400',
  url: 'bg-green-500/15 text-green-600 dark:text-green-400',
  code: 'bg-purple-500/15 text-purple-600 dark:text-purple-400',
  image: 'bg-orange-500/15 text-orange-600 dark:text-orange-400',
  empty: '',
};

export function ClipboardBar() {
  const { t } = useTranslation('spotlight');
  const clipboardContent = useSpotlightStore((s) => s.clipboardContent);
  const clipboardType = useSpotlightStore((s) => s.clipboardType);
  const sendMessage = useSpotlightStore((s) => s.sendMessage);
  const sending = useSpotlightStore((s) => s.sending);

  if (clipboardType === 'empty' || !clipboardContent) return null;

  const actions = getQuickActionsForType(clipboardType);
  const previewText = clipboardContent.text?.slice(0, 80) || '';

  const handleAction = (promptPrefix: string) => {
    if (sending) return;
    const content = clipboardContent.text || '';
    // For code, close the code block if the prefix opens one
    const suffix = promptPrefix.includes('```\n') ? '\n```' : '';
    sendMessage(promptPrefix + content + suffix);
  };

  return (
    <div className="px-4 py-2 flex items-center gap-2 border-t border-border/50">
      <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${TYPE_COLORS[clipboardType]}`}>
        {t(`clipboard.${clipboardType}`)}
      </span>

      {previewText && (
        <span className="text-xs text-muted-foreground truncate max-w-[200px]">
          {previewText}
        </span>
      )}

      <div className="flex-1" />

      {actions.map((action) => (
        <button
          key={action.id}
          onClick={() => handleAction(action.promptPrefix)}
          disabled={sending}
          className="text-xs px-2 py-1 rounded-md bg-accent hover:bg-accent/80 text-accent-foreground transition-colors disabled:opacity-50"
        >
          {t(action.labelKey.replace('spotlight:', ''))}
        </button>
      ))}
    </div>
  );
}
