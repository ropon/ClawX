/**
 * Command Palette (F-2.5)
 * Displays available quick commands grouped by category
 */
import { useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useSpotlightStore } from '@/stores/spotlight';
import { filterCommands } from '@/utils/command-engine';
import type { CommandCategory } from '@/types/commands';

const CATEGORY_ORDER: CommandCategory[] = ['general', 'code', 'writing', 'analysis'];

export function CommandPalette() {
  const { t } = useTranslation('spotlight');
  const { isOpen, filter, highlightIndex } = useSpotlightStore((s) => s.commandPalette);
  const selectCommand = useSpotlightStore((s) => s.selectCommand);
  const setHighlightIndex = useSpotlightStore((s) => s.setHighlightIndex);
  const listRef = useRef<HTMLDivElement>(null);

  const filtered = filterCommands(filter);

  // Auto-scroll highlighted item into view
  useEffect(() => {
    if (!isOpen) return;
    const el = listRef.current?.querySelector(`[data-cmd-index="${highlightIndex}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  }, [highlightIndex, isOpen]);

  if (!isOpen || filtered.length === 0) return null;

  // Group commands by category, preserving order
  const grouped = CATEGORY_ORDER
    .map((cat) => ({
      category: cat,
      commands: filtered.filter((cmd) => cmd.category === cat),
    }))
    .filter((g) => g.commands.length > 0);

  // Build flat index for highlight tracking
  let flatIndex = 0;

  return (
    <div
      ref={listRef}
      className="border-t border-border/50 max-h-[240px] overflow-y-auto"
    >
      {grouped.map((group) => (
        <div key={group.category}>
          <div className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/70">
            {t(`commands.categories.${group.category}`)}
          </div>
          {group.commands.map((cmd) => {
            const idx = flatIndex++;
            const isHighlighted = idx === highlightIndex;
            return (
              <div
                key={cmd.id}
                data-cmd-index={idx}
                className={`flex items-center gap-2.5 px-3 py-1.5 cursor-pointer transition-colors ${
                  isHighlighted ? 'bg-accent' : 'hover:bg-accent/50'
                }`}
                onMouseEnter={() => setHighlightIndex(idx)}
                onClick={() => selectCommand(cmd)}
              >
                <span className="text-sm flex-shrink-0 w-5 text-center">{cmd.icon}</span>
                <span className="text-sm font-medium text-foreground truncate">
                  {t(cmd.nameKey)}
                </span>
                <span className="text-xs text-muted-foreground truncate flex-1">
                  {t(cmd.descriptionKey)}
                </span>
                <span className="text-[11px] text-muted-foreground/50 flex-shrink-0 font-mono">
                  {cmd.command}
                </span>
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}
