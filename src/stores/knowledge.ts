/**
 * Knowledge State Store
 * Manages knowledge bases, documents, and search state.
 */
import { create } from 'zustand';
import { invoke } from '@/lib/bridge';
import type { KnowledgeBase, KnowledgeDocument, RAGResult } from '@/types/knowledge';

interface KnowledgeState {
  knowledgeBases: KnowledgeBase[];
  currentKBId: string | null;
  documents: KnowledgeDocument[];
  loading: boolean;
  error: string | null;
  searchResults: RAGResult[];
  searchQuery: string;
  searching: boolean;
  embeddingOptions: Array<{ label: string; value: string; dimension: number }>;

  // Actions
  fetchKnowledgeBases: () => Promise<void>;
  createKnowledgeBase: (config: {
    name: string;
    description: string;
    embeddingModel: string;
    embeddingDimension: number;
    chunkSize: number;
    chunkOverlap: number;
  }) => Promise<void>;
  updateKnowledgeBase: (id: string, updates: Partial<KnowledgeBase>) => Promise<void>;
  deleteKnowledgeBase: (id: string) => Promise<void>;
  setCurrentKB: (id: string | null) => void;
  fetchDocuments: (kbId: string) => Promise<void>;
  addDocuments: (kbId: string, filePaths: string[]) => Promise<void>;
  addUrl: (kbId: string, url: string) => Promise<void>;
  removeDocument: (kbId: string, docId: string) => Promise<void>;
  reprocessDocument: (kbId: string, docId: string) => Promise<void>;
  searchKB: (kbId: string, query: string) => Promise<void>;
  clearSearch: () => void;
  fetchEmbeddingOptions: () => Promise<void>;
  setWatchFolder: (kbId: string, folderPath: string | null) => Promise<void>;
}

export const useKnowledgeStore = create<KnowledgeState>((set, get) => ({
  knowledgeBases: [],
  currentKBId: null,
  documents: [],
  loading: false,
  error: null,
  searchResults: [],
  searchQuery: '',
  searching: false,
  embeddingOptions: [],

  fetchKnowledgeBases: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invoke('knowledge:list') as {
        success: boolean;
        knowledgeBases: KnowledgeBase[];
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to fetch knowledge bases');
      set({ knowledgeBases: result.knowledgeBases, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createKnowledgeBase: async (config) => {
    try {
      const result = await invoke('knowledge:create', config) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to create knowledge base');
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to create knowledge base:', error);
      throw error;
    }
  },

  updateKnowledgeBase: async (id, updates) => {
    try {
      const result = await invoke('knowledge:update', id, updates) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to update knowledge base');
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to update knowledge base:', error);
      throw error;
    }
  },

  deleteKnowledgeBase: async (id) => {
    try {
      const result = await invoke('knowledge:delete', id) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to delete knowledge base');
      if (get().currentKBId === id) set({ currentKBId: null, documents: [] });
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to delete knowledge base:', error);
      throw error;
    }
  },

  setCurrentKB: (id) => {
    set({ currentKBId: id, documents: [], searchResults: [], searchQuery: '' });
    if (id) get().fetchDocuments(id);
  },

  fetchDocuments: async (kbId) => {
    try {
      const result = await invoke('knowledge:listDocuments', kbId) as {
        success: boolean;
        documents: KnowledgeDocument[];
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to fetch documents');
      set({ documents: result.documents });
    } catch (error) {
      console.error('Failed to fetch documents:', error);
    }
  },

  addDocuments: async (kbId, filePaths) => {
    try {
      const result = await invoke('knowledge:addDocument', kbId, filePaths) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to add documents');
      await get().fetchDocuments(kbId);
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to add documents:', error);
      throw error;
    }
  },

  addUrl: async (kbId, url) => {
    try {
      const result = await invoke('knowledge:addUrl', kbId, url) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to add URL');
      await get().fetchDocuments(kbId);
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to add URL:', error);
      throw error;
    }
  },

  removeDocument: async (kbId, docId) => {
    try {
      const result = await invoke('knowledge:removeDocument', kbId, docId) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to remove document');
      await get().fetchDocuments(kbId);
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to remove document:', error);
      throw error;
    }
  },

  reprocessDocument: async (kbId, docId) => {
    try {
      const result = await invoke('knowledge:reprocessDocument', kbId, docId) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to reprocess document');
      await get().fetchDocuments(kbId);
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to reprocess document:', error);
      throw error;
    }
  },

  searchKB: async (kbId, query) => {
    if (!query.trim()) {
      set({ searchResults: [], searchQuery: '' });
      return;
    }
    set({ searching: true, searchQuery: query });
    try {
      const result = await invoke('knowledge:search', kbId, query) as {
        success: boolean;
        results: RAGResult[];
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Search failed');
      set({ searchResults: result.results, searching: false });
    } catch (error) {
      console.error('Search failed:', error);
      set({ searchResults: [], searching: false });
    }
  },

  clearSearch: () => set({ searchResults: [], searchQuery: '', searching: false }),

  fetchEmbeddingOptions: async () => {
    try {
      const result = await invoke('knowledge:getEmbeddingOptions') as {
        success: boolean;
        options: Array<{ label: string; value: string; dimension: number }>;
        error?: string;
      };
      if (result.success) {
        set({ embeddingOptions: result.options });
      }
    } catch (error) {
      console.error('Failed to fetch embedding options:', error);
    }
  },

  setWatchFolder: async (kbId, folderPath) => {
    try {
      const result = await invoke('knowledge:setWatchFolder', kbId, folderPath) as {
        success: boolean;
        error?: string;
      };
      if (!result.success) throw new Error(result.error || 'Failed to set watch folder');
      await get().fetchKnowledgeBases();
    } catch (error) {
      console.error('Failed to set watch folder:', error);
      throw error;
    }
  },
}));
