/**
 * Agent State Store
 * Manages AI agent role configurations
 */
import { create } from 'zustand';
import { invoke } from '@/lib/bridge';
import type { AgentConfig } from '@/types/agent';

interface AgentState {
  agents: AgentConfig[];
  activeAgentId: string | null;
  loading: boolean;
  error: string | null;

  // Actions
  fetchAgents: () => Promise<void>;
  createAgent: (config: Omit<AgentConfig, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  updateAgent: (id: string, updates: Partial<AgentConfig>) => Promise<void>;
  deleteAgent: (id: string) => Promise<void>;
  setActiveAgent: (id: string) => Promise<void>;
  getActiveAgent: () => AgentConfig | null;
  cloneAgent: (id: string) => Promise<void>;
  exportAgent: (id: string) => Promise<string | null>;
  importAgent: (json: string) => Promise<void>;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  agents: [],
  activeAgentId: null,
  loading: false,
  error: null,

  fetchAgents: async () => {
    set({ loading: true, error: null });

    try {
      const result = await invoke('agent:list') as {
        success: boolean;
        agents: AgentConfig[];
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to fetch agents');
      }

      const activeResult = await invoke('agent:getActive') as {
        success: boolean;
        id: string | null;
      };

      set({
        agents: result.agents,
        activeAgentId: activeResult.id,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createAgent: async (config) => {
    try {
      const result = await invoke('agent:create', config) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to create agent');
      }

      await get().fetchAgents();
    } catch (error) {
      console.error('Failed to create agent:', error);
      throw error;
    }
  },

  updateAgent: async (id, updates) => {
    try {
      const result = await invoke('agent:update', id, updates) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to update agent');
      }

      await get().fetchAgents();
    } catch (error) {
      console.error('Failed to update agent:', error);
      throw error;
    }
  },

  deleteAgent: async (id) => {
    try {
      const result = await invoke('agent:delete', id) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to delete agent');
      }

      await get().fetchAgents();
    } catch (error) {
      console.error('Failed to delete agent:', error);
      throw error;
    }
  },

  setActiveAgent: async (id) => {
    try {
      const result = await invoke('agent:setActive', id) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to set active agent');
      }

      set({ activeAgentId: id });
    } catch (error) {
      console.error('Failed to set active agent:', error);
      throw error;
    }
  },

  getActiveAgent: () => {
    const { agents, activeAgentId } = get();
    if (!agents.length) return null;

    // Try active ID first
    if (activeAgentId) {
      const found = agents.find((a) => a.id === activeAgentId);
      if (found) return found;
    }

    // Fallback to default agent
    const defaultAgent = agents.find((a) => a.isDefault);
    if (defaultAgent) return defaultAgent;

    // Fallback to first agent
    return agents[0] || null;
  },

  cloneAgent: async (id) => {
    const agent = get().agents.find((a) => a.id === id);
    if (!agent) return;

    const { id: _id, createdAt: _c, updatedAt: _u, ...rest } = agent;
    await get().createAgent({
      ...rest,
      name: `${agent.name} (Copy)`,
      isDefault: false,
    });
  },

  exportAgent: async (id) => {
    try {
      const result = await invoke('agent:export', id) as {
        success: boolean;
        json?: string;
        error?: string;
      };

      if (!result.success) return null;
      return result.json || null;
    } catch {
      return null;
    }
  },

  importAgent: async (json) => {
    try {
      const result = await invoke('agent:import', json) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to import agent');
      }

      await get().fetchAgents();
    } catch (error) {
      console.error('Failed to import agent:', error);
      throw error;
    }
  },
}));
