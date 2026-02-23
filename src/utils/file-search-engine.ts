/**
 * File Search Engine - Frontend Utilities (F-2.6)
 * Pure functions for file search UI logic
 */

/**
 * Check if input starts with "@" (file search trigger)
 * Returns the search query after "@", or null if not a trigger
 */
export function isFileSearchTrigger(input: string): string | null {
  if (input.startsWith('@') && input.length >= 2) {
    return input.slice(1);
  }
  return null;
}

/**
 * Get file icon emoji based on extension
 */
export function getFileIcon(ext: string): string {
  const lower = ext.toLowerCase();
  const iconMap: Record<string, string> = {
    // Code
    '.ts': '🟦',
    '.tsx': '🟦',
    '.js': '🟨',
    '.jsx': '🟨',
    '.py': '🐍',
    '.rs': '🦀',
    '.go': '🔵',
    '.java': '☕',
    '.c': '⚙️',
    '.cpp': '⚙️',
    '.h': '⚙️',
    '.cs': '🟣',
    '.rb': '💎',
    '.php': '🐘',
    '.swift': '🍊',
    '.kt': '🟪',
    // Web
    '.html': '🌐',
    '.css': '🎨',
    '.scss': '🎨',
    '.less': '🎨',
    '.vue': '💚',
    '.svelte': '🔶',
    // Data
    '.json': '📋',
    '.yaml': '📋',
    '.yml': '📋',
    '.xml': '📋',
    '.csv': '📊',
    '.sql': '🗃️',
    // Text
    '.md': '📝',
    '.txt': '📄',
    '.log': '📜',
    '.env': '🔒',
    // Config
    '.toml': '⚙️',
    '.ini': '⚙️',
    '.conf': '⚙️',
    // Shell
    '.sh': '🖥️',
    '.bash': '🖥️',
    '.zsh': '🖥️',
    // Package
    '.lock': '🔒',
  };

  return iconMap[lower] || '📄';
}

/**
 * Format file size in human-readable form
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
