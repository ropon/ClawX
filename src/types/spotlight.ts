/**
 * Spotlight Types
 * Type definitions for the Spotlight quick chat feature
 */

/** Content read from the system clipboard */
export interface ClipboardContent {
  text: string | null;
  hasImage: boolean;
  imageDataUrl: string | null;
}

/** Detected clipboard content type */
export type ClipboardContentType = 'text' | 'url' | 'code' | 'image' | 'empty';

/** Quick action available for a clipboard content type */
export interface QuickAction {
  id: string;
  labelKey: string;
  promptPrefix: string;
}
