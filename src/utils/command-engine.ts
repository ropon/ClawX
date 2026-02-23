/**
 * Command Engine
 * Pure functions for command trigger detection, filtering, and template resolution
 */
import type { QuickCommand } from '@/types/commands';
import { builtinCommands } from '@/data/builtin-commands';

/**
 * Check if input starts with "/" (command trigger)
 */
export function isCommandTrigger(input: string): boolean {
  return input.startsWith('/');
}

/**
 * Filter commands by fuzzy matching against command name
 * Returns all commands if filter is just "/"
 */
export function filterCommands(filter: string): QuickCommand[] {
  const query = filter.startsWith('/') ? filter.slice(1).toLowerCase() : filter.toLowerCase();
  if (!query) return builtinCommands;
  return builtinCommands.filter(
    (cmd) =>
      cmd.command.toLowerCase().includes(query) ||
      cmd.id.toLowerCase().includes(query)
  );
}

/**
 * Resolve a prompt template by replacing {{clipboard}} and {{input}} placeholders
 */
export function resolveTemplate(
  cmd: QuickCommand,
  clipboardText: string,
  userInput: string
): string {
  return cmd.promptTemplate
    .replace(/\{\{clipboard\}\}/g, clipboardText)
    .replace(/\{\{input\}\}/g, userInput);
}
