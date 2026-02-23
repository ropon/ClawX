/**
 * Knowledge Base type definitions
 * Types for knowledge base management, documents, chunks, and RAG
 */

export interface KnowledgeBase {
  id: string;                 // "kb-{timestamp}-{random6}"
  name: string;
  description: string;
  embeddingModel: string;     // "provider:default" | "provider:<id>" | "local:all-MiniLM-L6-v2"
  embeddingDimension: number; // 384 (MiniLM) / 1536 (OpenAI) / etc.
  documentCount: number;
  totalChunks: number;
  totalSize: number;          // bytes
  chunkSize: number;          // default 500
  chunkOverlap: number;       // default 100
  watchedFolder?: string;     // F-3.8
  createdAt: string;
  updatedAt: string;
}

export interface KnowledgeDocument {
  id: string;                 // "doc-{timestamp}-{random6}"
  knowledgeBaseId: string;
  fileName: string;
  filePath: string;           // original path or URL
  fileType: 'pdf' | 'txt' | 'md' | 'docx' | 'csv' | 'url';
  fileSize: number;
  chunkCount: number;
  status: 'pending' | 'processing' | 'ready' | 'error';
  error?: string;
  createdAt: string;
  updatedAt: string;
}

export interface KnowledgeChunk {
  id: string;
  documentId: string;
  knowledgeBaseId: string;
  content: string;
  metadata: { page?: number; lineStart?: number; lineEnd?: number; source?: string };
}

export interface RAGResult {
  chunkId: string;
  documentId: string;
  documentName: string;
  content: string;
  score: number;
}

export interface RAGConfig {
  topK: number;             // default 5
  scoreThreshold: number;   // default 0.3
  maxContextTokens: number; // default 4000
}
