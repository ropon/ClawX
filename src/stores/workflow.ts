/**
 * Workflow State Store
 * Manages workflow orchestration configurations and execution
 */
import { create } from 'zustand';
import { invoke, on } from '@/lib/bridge';
import type { WorkflowConfig, WorkflowRun } from '@/types/workflow';

interface WorkflowState {
  workflows: WorkflowConfig[];
  currentWorkflowId: string | null;
  runs: WorkflowRun[];
  activeRunId: string | null;
  loading: boolean;
  error: string | null;

  // Actions
  fetchWorkflows: () => Promise<void>;
  createWorkflow: (config: Partial<WorkflowConfig>) => Promise<string>;
  updateWorkflow: (id: string, updates: Partial<WorkflowConfig>) => Promise<void>;
  deleteWorkflow: (id: string) => Promise<void>;
  duplicateWorkflow: (id: string) => Promise<void>;
  setCurrentWorkflow: (id: string | null) => void;
  executeWorkflow: (id: string, input?: string) => Promise<string>;
  cancelExecution: (runId: string) => Promise<void>;
  fetchRuns: (workflowId: string, limit?: number) => Promise<void>;
  deleteRun: (runId: string) => Promise<void>;
  clearRuns: (workflowId: string) => Promise<void>;
  exportWorkflow: (id: string) => Promise<string | null>;
  importWorkflow: (json: string) => Promise<void>;
  handleStepProgress: (data: { runId: string; nodeId: string; status: string; output?: string }) => void;
}

export const useWorkflowStore = create<WorkflowState>((set, get) => ({
  workflows: [],
  currentWorkflowId: null,
  runs: [],
  activeRunId: null,
  loading: false,
  error: null,

  fetchWorkflows: async () => {
    set({ loading: true, error: null });

    try {
      const result = await invoke('workflow:list') as {
        success: boolean;
        workflows: WorkflowConfig[];
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to fetch workflows');
      }

      set({
        workflows: result.workflows,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createWorkflow: async (config) => {
    try {
      const result = await invoke('workflow:create', config) as {
        success: boolean;
        workflow: WorkflowConfig;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to create workflow');
      }

      await get().fetchWorkflows();
      return result.workflow.id;
    } catch (error) {
      console.error('Failed to create workflow:', error);
      throw error;
    }
  },

  updateWorkflow: async (id, updates) => {
    try {
      const result = await invoke('workflow:update', id, updates) as {
        success: boolean;
        workflow: WorkflowConfig;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to update workflow');
      }

      await get().fetchWorkflows();
    } catch (error) {
      console.error('Failed to update workflow:', error);
      throw error;
    }
  },

  deleteWorkflow: async (id) => {
    try {
      const result = await invoke('workflow:delete', id) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to delete workflow');
      }

      // Clear current selection if the deleted workflow was selected
      if (get().currentWorkflowId === id) {
        set({ currentWorkflowId: null });
      }

      await get().fetchWorkflows();
    } catch (error) {
      console.error('Failed to delete workflow:', error);
      throw error;
    }
  },

  duplicateWorkflow: async (id) => {
    try {
      const result = await invoke('workflow:duplicate', id) as {
        success: boolean;
        workflow: WorkflowConfig;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to duplicate workflow');
      }

      await get().fetchWorkflows();
    } catch (error) {
      console.error('Failed to duplicate workflow:', error);
      throw error;
    }
  },

  setCurrentWorkflow: (id) => {
    set({ currentWorkflowId: id });
  },

  executeWorkflow: async (id, input?) => {
    try {
      const result = await invoke('workflow:execute', id, input) as {
        success: boolean;
        runId: string;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to execute workflow');
      }

      set({ activeRunId: result.runId });

      // Fetch the newly created run to add it to the list
      const runResult = await invoke('workflow:getRun', result.runId) as {
        success: boolean;
        run: WorkflowRun;
        error?: string;
      };

      if (runResult.success) {
        set((state) => ({
          runs: [runResult.run, ...state.runs],
        }));
      }

      return result.runId;
    } catch (error) {
      console.error('Failed to execute workflow:', error);
      throw error;
    }
  },

  cancelExecution: async (runId) => {
    try {
      const result = await invoke('workflow:cancel', runId) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to cancel execution');
      }

      // Update the run status locally
      set((state) => ({
        runs: state.runs.map((run) =>
          run.id === runId ? { ...run, status: 'cancelled' as const } : run
        ),
        activeRunId: state.activeRunId === runId ? null : state.activeRunId,
      }));
    } catch (error) {
      console.error('Failed to cancel execution:', error);
      throw error;
    }
  },

  fetchRuns: async (workflowId, limit?) => {
    try {
      const result = await invoke('workflow:getRuns', workflowId, limit) as {
        success: boolean;
        runs: WorkflowRun[];
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to fetch runs');
      }

      set({ runs: result.runs });
    } catch (error) {
      console.error('Failed to fetch runs:', error);
      throw error;
    }
  },

  deleteRun: async (runId) => {
    try {
      const result = await invoke('workflow:deleteRun', runId) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to delete run');
      }

      set((state) => ({
        runs: state.runs.filter((run) => run.id !== runId),
        activeRunId: state.activeRunId === runId ? null : state.activeRunId,
      }));
    } catch (error) {
      console.error('Failed to delete run:', error);
      throw error;
    }
  },

  clearRuns: async (workflowId) => {
    try {
      const result = await invoke('workflow:clearRuns', workflowId) as {
        success: boolean;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to clear runs');
      }

      set({ runs: [], activeRunId: null });
    } catch (error) {
      console.error('Failed to clear runs:', error);
      throw error;
    }
  },

  exportWorkflow: async (id) => {
    try {
      const result = await invoke('workflow:export', id) as {
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

  importWorkflow: async (json) => {
    try {
      const result = await invoke('workflow:import', json) as {
        success: boolean;
        workflow: WorkflowConfig;
        error?: string;
      };

      if (!result.success) {
        throw new Error(result.error || 'Failed to import workflow');
      }

      await get().fetchWorkflows();
    } catch (error) {
      console.error('Failed to import workflow:', error);
      throw error;
    }
  },

  handleStepProgress: (data) => {
    const { runId, nodeId, status, output } = data;

    set((state) => {
      const runs = state.runs.map((run) => {
        if (run.id !== runId) return run;

        const stepIndex = run.steps.findIndex((s) => s.nodeId === nodeId);
        const updatedSteps = [...run.steps];

        if (stepIndex >= 0) {
          // Update existing step
          updatedSteps[stepIndex] = {
            ...updatedSteps[stepIndex],
            status: status as WorkflowRun['status'],
            ...(output !== undefined ? { output } : {}),
            ...(status === 'completed' || status === 'failed'
              ? { completedAt: Date.now() }
              : {}),
          };
        } else {
          // Add new step
          updatedSteps.push({
            nodeId,
            status: status as WorkflowRun['status'],
            input: '',
            output: output || '',
            startedAt: Date.now(),
            ...(status === 'completed' || status === 'failed'
              ? { completedAt: Date.now() }
              : {}),
          });
        }

        // Determine overall run status
        const allCompleted = updatedSteps.every((s) => s.status === 'completed');
        const anyFailed = updatedSteps.some((s) => s.status === 'failed');
        let runStatus = run.status;
        if (anyFailed) {
          runStatus = 'failed';
        } else if (allCompleted) {
          runStatus = 'completed';
        } else {
          runStatus = 'running';
        }

        return {
          ...run,
          steps: updatedSteps,
          status: runStatus,
          ...(runStatus === 'completed' || runStatus === 'failed'
            ? { completedAt: Date.now() }
            : {}),
          ...(runStatus === 'completed' && output !== undefined
            ? { finalOutput: output }
            : {}),
        };
      });

      // Clear activeRunId if run is done
      const activeRun = runs.find((r) => r.id === state.activeRunId);
      const activeRunDone =
        activeRun && (activeRun.status === 'completed' || activeRun.status === 'failed');

      return {
        runs,
        ...(activeRunDone ? { activeRunId: null } : {}),
      };
    });
  },
}));

/**
 * Initialize the workflow step progress listener.
 * Call this once during app startup. Safe to call again on hot-reload.
 */
let _workflowUnsub: (() => void) | null = null;

export function initWorkflowListener(): void {
  _workflowUnsub?.();
  _workflowUnsub = on(
    'workflow:stepProgress',
    (...args: unknown[]) => {
      const data = args[1] as { runId: string; nodeId: string; status: string; output?: string };
      useWorkflowStore.getState().handleStepProgress(data);
    }
  );
}
