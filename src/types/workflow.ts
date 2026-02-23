/**
 * Workflow Types
 * Type definitions for Agent collaboration & workflow orchestration (Phase 4)
 */

// Node types
export type WorkflowNodeType = 'agent' | 'condition' | 'merge' | 'input' | 'output';

export interface WorkflowNodeData {
  label: string;
  agentId?: string;
  promptTemplate?: string;
  conditionType?: 'keyword' | 'regex' | 'ai-classify';
  conditionRules?: ConditionRule[];
  mergeStrategy?: 'concat' | 'first' | 'custom';
  mergeTemplate?: string;
}

export interface ConditionRule {
  id: string;
  handle: string;
  type: 'keyword' | 'regex' | 'ai-classify';
  value: string;
  isDefault?: boolean;
}

export interface WorkflowNode {
  id: string;
  type: WorkflowNodeType;
  position: { x: number; y: number };
  data: WorkflowNodeData;
}

export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  label?: string;
}

// Trigger types
export type TriggerType = 'manual' | 'cron' | 'file-change' | 'clipboard' | 'shortcut';

export interface WorkflowTrigger {
  id: string;
  type: TriggerType;
  cronExpr?: string;
  watchPath?: string;
  filePattern?: string;
  clipboardPattern?: string;
  shortcutAccelerator?: string;
  enabled: boolean;
}

// Workflow configuration (persisted)
export interface WorkflowConfig {
  id: string;
  name: string;
  description: string;
  icon: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  triggers: WorkflowTrigger[];
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

// Execution state
export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface StepResult {
  nodeId: string;
  status: ExecutionStatus;
  input: string;
  output: string;
  startedAt: number;
  completedAt?: number;
  error?: string;
  agentId?: string;
  sessionKey?: string;
}

export interface WorkflowRun {
  id: string;
  workflowId: string;
  status: ExecutionStatus;
  triggerType: TriggerType | 'manual';
  triggerInput?: string;
  startedAt: number;
  completedAt?: number;
  steps: StepResult[];
  finalOutput?: string;
  error?: string;
}

// Template
export interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: 'productivity' | 'content' | 'code' | 'research' | 'automation';
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  triggers: WorkflowTrigger[];
}
