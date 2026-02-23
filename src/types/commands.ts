/**
 * Quick Command Types
 * Type definitions for the "/" command palette system
 */

export type CommandCategory = 'general' | 'code' | 'writing' | 'analysis';

export interface QuickCommand {
  id: string;
  command: string;
  nameKey: string;
  descriptionKey: string;
  icon: string;
  promptTemplate: string;
  requiresInput: boolean;
  inputPlaceholderKey?: string;
  category: CommandCategory;
}
