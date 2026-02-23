/**
 * File Attachment Types (F-2.6)
 * Types for local file search and attachment in Spotlight
 */

export interface FileSearchResult {
  filePath: string;
  fileName: string;
  fileSize: number;
  modifiedAt: number;
  fileType: string; // extension e.g. ".ts"
}

export interface FileAttachment {
  id: string; // crypto.randomUUID()
  filePath: string;
  fileName: string;
  fileSize: number;
  fileType: string;
}

export interface FileSearchState {
  isOpen: boolean;
  filter: string;
  results: FileSearchResult[];
  highlightIndex: number;
  searching: boolean;
}
