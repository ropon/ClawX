/**
 * Clipboard Content Detection
 * Detects clipboard content type and provides quick actions
 */
import type { ClipboardContent, ClipboardContentType, QuickAction } from '@/types/spotlight';

// URL pattern
const URL_REGEX = /^https?:\/\/[^\s]+$/i;

// Code detection heuristics
const CODE_INDICATORS = [
  /^(import|export|const|let|var|function|class|interface|type|enum|def|fn|pub|use|package|module)\s/m,
  /[{}\[\]();].*[{}\[\]();]/,
  /^\s*(\/\/|#|\/\*|\*|--)/m,
  /=>/,
  /\.(then|catch|map|filter|reduce)\(/,
  /\b(if|else|for|while|return|switch|case|try|catch)\s*[({]/,
];

/**
 * Detect the type of clipboard content
 */
export function detectClipboardType(content: ClipboardContent): ClipboardContentType {
  if (content.hasImage && content.imageDataUrl) {
    return 'image';
  }

  const text = content.text?.trim();
  if (!text) {
    return 'empty';
  }

  if (URL_REGEX.test(text)) {
    return 'url';
  }

  // Check for code indicators
  const codeScore = CODE_INDICATORS.reduce(
    (score, regex) => score + (regex.test(text) ? 1 : 0),
    0
  );
  // Multiple lines + at least 2 code indicators = likely code
  if (text.includes('\n') && codeScore >= 2) {
    return 'code';
  }

  return 'text';
}

/**
 * Get quick actions for a given clipboard content type
 */
export function getQuickActionsForType(type: ClipboardContentType): QuickAction[] {
  switch (type) {
    case 'text':
      return [
        { id: 'translate', labelKey: 'spotlight:actions.translate', promptPrefix: 'Translate the following text:\n\n' },
        { id: 'summarize', labelKey: 'spotlight:actions.summarize', promptPrefix: 'Summarize the following text:\n\n' },
        { id: 'explain', labelKey: 'spotlight:actions.explain', promptPrefix: 'Explain the following text:\n\n' },
      ];
    case 'url':
      return [
        { id: 'summarize', labelKey: 'spotlight:actions.summarizePage', promptPrefix: 'Summarize the content at this URL:\n\n' },
        { id: 'explain', labelKey: 'spotlight:actions.explain', promptPrefix: 'What is this link about?\n\n' },
      ];
    case 'code':
      return [
        { id: 'explain', labelKey: 'spotlight:actions.explainCode', promptPrefix: 'Explain this code:\n\n```\n' },
        { id: 'review', labelKey: 'spotlight:actions.review', promptPrefix: 'Review this code for bugs and improvements:\n\n```\n' },
        { id: 'optimize', labelKey: 'spotlight:actions.optimize', promptPrefix: 'Optimize this code:\n\n```\n' },
      ];
    case 'image':
      return [
        { id: 'describe', labelKey: 'spotlight:actions.describe', promptPrefix: 'Describe this image:\n\n' },
      ];
    case 'empty':
    default:
      return [];
  }
}
