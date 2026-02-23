/**
 * File Search Panel (F-2.6)
 * Displays file search results triggered by "@" prefix in Spotlight input
 */
import { useRef, useEffect } from 'react';
import { Loader2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useSpotlightStore } from '@/stores/spotlight';
import { getFileIcon, formatFileSize } from '@/utils/file-search-engine';

export function FileSearchPanel() {
  const { t } = useTranslation('spotlight');
  const fileSearch = useSpotlightStore((s) => s.fileSearch);
  const selectFileSearchResult = useSpotlightStore((s) => s.selectFileSearchResult);
  const setFileSearchHighlight = useSpotlightStore((s) => s.setFileSearchHighlight);
  const listRef = useRef<HTMLDivElement>(null);

  // Auto-scroll highlighted item into view
  useEffect(() => {
    if (!fileSearch.isOpen) return;
    const el = listRef.current?.querySelector(`[data-file-index="${fileSearch.highlightIndex}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  }, [fileSearch.highlightIndex, fileSearch.isOpen]);

  if (!fileSearch.isOpen) return null;

  // Searching state
  if (fileSearch.searching) {
    return (
      <div className="border-t border-border/50 px-4 py-3 flex items-center gap-2 text-muted-foreground text-sm">
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
        <span>{t('fileSearch.searching')}</span>
      </div>
    );
  }

  // Hint: need at least 2 chars
  if (fileSearch.filter.length < 2) {
    return (
      <div className="border-t border-border/50 px-4 py-3 text-muted-foreground text-sm">
        {t('fileSearch.hint')}
      </div>
    );
  }

  // No results
  if (fileSearch.results.length === 0) {
    return (
      <div className="border-t border-border/50 px-4 py-3 text-muted-foreground text-sm">
        {t('fileSearch.noResults')}
      </div>
    );
  }

  return (
    <div
      ref={listRef}
      className="border-t border-border/50 max-h-[240px] overflow-y-auto"
    >
      {fileSearch.results.map((result, index) => {
        const isHighlighted = index === fileSearch.highlightIndex;
        // Extract directory from filePath
        const dir = result.filePath.replace(/\/[^/]+$/, '');
        // Shorten home dir
        const shortDir = dir.replace(/^\/Users\/[^/]+/, '~');

        return (
          <div
            key={result.filePath}
            data-file-index={index}
            className={`flex items-center gap-2.5 px-3 py-1.5 cursor-pointer transition-colors ${
              isHighlighted ? 'bg-accent' : 'hover:bg-accent/50'
            }`}
            onMouseEnter={() => setFileSearchHighlight(index)}
            onClick={() => selectFileSearchResult(result)}
          >
            <span className="text-sm flex-shrink-0 w-5 text-center">
              {getFileIcon(result.fileType)}
            </span>
            <span className="text-sm font-medium text-foreground truncate">
              {result.fileName}
            </span>
            <span className="text-xs text-muted-foreground truncate flex-1">
              {shortDir}
            </span>
            <span className="text-[11px] text-muted-foreground/50 flex-shrink-0 font-mono">
              {formatFileSize(result.fileSize)}
            </span>
          </div>
        );
      })}
    </div>
  );
}
