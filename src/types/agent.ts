/**
 * Agent type definitions
 * Defines the structure for AI agent roles/personas
 */

export interface AgentConfig {
  id: string;
  name: string;
  avatar: string; // emoji character
  description: string;
  systemPrompt: string;
  providerId: string | null;
  model: string;
  temperature: number;
  maxTokens: number | null;
  skillIds: string[];
  knowledgeBaseIds: string[];
  channelBindings: string[];
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
}

export const DEFAULT_AGENT: AgentConfig = {
  id: 'default',
  name: 'Assistant',
  avatar: '🤖',
  description: 'Default AI assistant',
  systemPrompt: '',
  providerId: null,
  model: '',
  temperature: 0.7,
  maxTokens: null,
  skillIds: [],
  knowledgeBaseIds: [],
  channelBindings: [],
  isDefault: true,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};
