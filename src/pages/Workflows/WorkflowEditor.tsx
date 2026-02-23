/**
 * Workflow Editor
 * ReactFlow-based visual workflow editor with drag-and-drop node creation
 */
import { useCallback, useRef, useState, useEffect, useMemo } from 'react';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  addEdge,
  useNodesState,
  useEdgesState,
  useReactFlow,
  ReactFlowProvider,
  type Connection,
  type Node,
  type Edge,
  BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Bot, GitFork, Merge, LogIn, LogOut, History } from 'lucide-react';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import { useWorkflowStore } from '@/stores/workflow';
import { WorkflowEditorToolbar } from './WorkflowEditorToolbar';
import { WorkflowRunPanel } from './WorkflowRunPanel';
import { WorkflowRunHistory } from './WorkflowRunHistory';
import { NodeConfigPanel } from './nodes/NodeConfigPanel';
import { InputNode } from './nodes/InputNode';
import { OutputNode } from './nodes/OutputNode';
import { AgentNode } from './nodes/AgentNode';
import { ConditionNode } from './nodes/ConditionNode';
import { MergeNode } from './nodes/MergeNode';
import { Button } from '@/components/ui/button';
import type { WorkflowConfig, WorkflowNode as WFNode, WorkflowNodeData } from '@/types/workflow';

const nodeTypes = {
  input: InputNode,
  output: OutputNode,
  agent: AgentNode,
  condition: ConditionNode,
  merge: MergeNode,
};

const NODE_PALETTE = [
  { type: 'agent', label: 'Agent', icon: Bot, color: 'text-blue-500' },
  { type: 'condition', label: 'Condition', icon: GitFork, color: 'text-amber-500' },
  { type: 'merge', label: 'Merge', icon: Merge, color: 'text-purple-500' },
  { type: 'input', label: 'Input', icon: LogIn, color: 'text-green-500' },
  { type: 'output', label: 'Output', icon: LogOut, color: 'text-red-500' },
] as const;

interface WorkflowEditorInnerProps {
  workflow: WorkflowConfig;
  onBack: () => void;
}

function WorkflowEditorInner({ workflow, onBack }: WorkflowEditorInnerProps) {
  const { t } = useTranslation('workflows');
  const { screenToFlowPosition, zoomIn, zoomOut, fitView } = useReactFlow();

  const updateWorkflow = useWorkflowStore((s) => s.updateWorkflow);
  const executeWorkflow = useWorkflowStore((s) => s.executeWorkflow);
  const cancelExecution = useWorkflowStore((s) => s.cancelExecution);
  const activeRunId = useWorkflowStore((s) => s.activeRunId);
  const runs = useWorkflowStore((s) => s.runs);
  const fetchRuns = useWorkflowStore((s) => s.fetchRuns);

  const activeRun = useMemo(
    () => runs.find((r) => r.id === activeRunId) || null,
    [runs, activeRunId]
  );

  const [nodes, setNodes, onNodesChange] = useNodesState(
    workflow.nodes.map((n) => ({
      ...n,
      type: n.type,
      data: { ...n.data },
    })) as Node[]
  );
  const [edges, setEdges, onEdgesChange] = useEdgesState(
    workflow.edges.map((e) => ({ ...e })) as Edge[]
  );
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [name, setName] = useState(workflow.name);
  const [saving, setSaving] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const reactFlowWrapper = useRef<HTMLDivElement>(null);

  const selectedNode = useMemo(
    () => {
      const n = nodes.find((n) => n.id === selectedNodeId);
      if (!n) return null;
      return { id: n.id, type: n.type as WFNode['type'], position: n.position, data: n.data as unknown as WorkflowNodeData };
    },
    [nodes, selectedNodeId]
  );

  // Fetch runs on mount
  useEffect(() => {
    fetchRuns(workflow.id);
  }, [workflow.id, fetchRuns]);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onSelectionChange = useCallback(({ nodes: sel }: { nodes: Node[] }) => {
    setSelectedNodeId(sel.length === 1 ? sel[0].id : null);
  }, []);

  const onNodeUpdate = useCallback(
    (nodeId: string, data: Partial<WorkflowNodeData>) => {
      setNodes((nds) =>
        nds.map((n) =>
          n.id === nodeId ? { ...n, data: { ...n.data, ...data } } : n
        )
      );
    },
    [setNodes]
  );

  // Drag and drop from palette
  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const type = event.dataTransfer.getData('application/reactflow-type');
      if (!type) return;

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const id = `${type}-${Date.now()}`;
      const defaultLabels: Record<string, string> = {
        agent: t('nodeTypes.agent'),
        condition: t('nodeTypes.condition'),
        merge: t('nodeTypes.merge'),
        input: t('nodeTypes.input'),
        output: t('nodeTypes.output'),
      };

      const newNode: Node = {
        id,
        type,
        position,
        data: { label: defaultLabels[type] || type },
      };

      setNodes((nds) => [...nds, newNode]);
    },
    [screenToFlowPosition, setNodes, t]
  );

  const handleSave = async () => {
    setSaving(true);
    try {
      const wfNodes = nodes.map((n) => ({
        id: n.id,
        type: n.type as WFNode['type'],
        position: n.position,
        data: n.data as unknown as WorkflowNodeData,
      }));
      const wfEdges = edges.map((e) => ({
        id: e.id,
        source: e.source,
        target: e.target,
        sourceHandle: e.sourceHandle || undefined,
        targetHandle: e.targetHandle || undefined,
        label: typeof e.label === 'string' ? e.label : undefined,
      }));

      await updateWorkflow(workflow.id, {
        name,
        nodes: wfNodes,
        edges: wfEdges,
      });
      toast.success(t('editor.save'));
    } catch (err) {
      toast.error(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleRun = async (input: string) => {
    try {
      // Save first
      await handleSave();
      await executeWorkflow(workflow.id, input);
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleStop = async () => {
    if (activeRunId) {
      await cancelExecution(activeRunId);
    }
  };

  const isRunning = activeRun?.status === 'running' || activeRun?.status === 'pending';

  return (
    <div className="flex flex-col h-full">
      <WorkflowEditorToolbar
        name={name}
        onNameChange={setName}
        onSave={handleSave}
        onBack={onBack}
        onRun={handleRun}
        onStop={handleStop}
        onZoomIn={() => zoomIn()}
        onZoomOut={() => zoomOut()}
        onFitView={() => fitView()}
        isRunning={isRunning}
        saving={saving}
      />

      <div className="flex flex-1 overflow-hidden">
        {/* Left: Node Palette */}
        <div className="w-48 border-r p-3 space-y-2 bg-background overflow-auto">
          <div className="flex items-center justify-between">
            <p className="text-xs font-medium text-muted-foreground">{t('editor.nodePanel')}</p>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => setHistoryOpen(true)}>
              <History className="h-3.5 w-3.5" />
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">{t('editor.dragHint')}</p>
          {NODE_PALETTE.map(({ type, label, icon: Icon, color }) => (
            <div
              key={type}
              className="flex items-center gap-2 rounded-lg border p-2 cursor-grab active:cursor-grabbing hover:bg-accent/50"
              draggable
              onDragStart={(e) => {
                e.dataTransfer.setData('application/reactflow-type', type);
                e.dataTransfer.effectAllowed = 'move';
              }}
            >
              <Icon className={`h-4 w-4 ${color}`} />
              <span className="text-sm">{label}</span>
            </div>
          ))}
        </div>

        {/* Center: ReactFlow Canvas */}
        <div className="flex-1" ref={reactFlowWrapper}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onSelectionChange={onSelectionChange}
            onDragOver={onDragOver}
            onDrop={onDrop}
            nodeTypes={nodeTypes}
            fitView
            deleteKeyCode={['Backspace', 'Delete']}
            className="bg-background"
          >
            <MiniMap
              className="!bg-muted/50"
              nodeColor="#6366f1"
              maskColor="rgba(0,0,0,0.1)"
            />
            <Controls />
            <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
          </ReactFlow>
        </div>

        {/* Right: Config Panel or Run Panel */}
        {isRunning || (activeRun && activeRun.status !== 'pending') ? (
          <WorkflowRunPanel run={activeRun} onCancel={handleStop} />
        ) : (
          <div className="w-72 border-l bg-background overflow-auto">
            <div className="px-3 py-2 border-b">
              <p className="text-sm font-medium">{t('editor.configPanel')}</p>
            </div>
            <NodeConfigPanel node={selectedNode} onUpdate={onNodeUpdate} />
          </div>
        )}
      </div>

      <WorkflowRunHistory
        workflowId={workflow.id}
        open={historyOpen}
        onOpenChange={setHistoryOpen}
      />
    </div>
  );
}

interface WorkflowEditorProps {
  workflow: WorkflowConfig;
  onBack: () => void;
}

export function WorkflowEditor({ workflow, onBack }: WorkflowEditorProps) {
  return (
    <ReactFlowProvider>
      <WorkflowEditorInner workflow={workflow} onBack={onBack} />
    </ReactFlowProvider>
  );
}
