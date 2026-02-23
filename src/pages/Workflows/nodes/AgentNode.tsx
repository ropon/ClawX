/**
 * Agent Node
 * Represents an AI agent step in the workflow
 */
import { Handle, Position } from '@xyflow/react';
import { Bot, CheckCircle2, Loader2 } from 'lucide-react';

interface AgentNodeProps {
  data: {
    label: string;
    agentId?: string;
    _runStatus?: 'pending' | 'running' | 'completed' | 'failed';
  };
  selected?: boolean;
}

export function AgentNode({ data, selected }: AgentNodeProps) {
  const status = data._runStatus;
  const borderColor = selected
    ? 'border-primary'
    : status === 'running'
      ? 'border-blue-500 animate-pulse'
      : status === 'completed'
        ? 'border-green-500'
        : status === 'failed'
          ? 'border-red-500'
          : 'border-border';

  return (
    <div className={`rounded-xl border-2 bg-background px-4 py-3 shadow-sm min-w-[160px] ${borderColor}`}>
      <Handle type="target" position={Position.Top} className="!bg-blue-500 !w-3 !h-3" />
      <div className="flex items-center gap-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-full bg-blue-500/10">
          {status === 'running' ? (
            <Loader2 className="h-4 w-4 text-blue-500 animate-spin" />
          ) : status === 'completed' ? (
            <CheckCircle2 className="h-4 w-4 text-green-500" />
          ) : (
            <Bot className="h-4 w-4 text-blue-500" />
          )}
        </div>
        <div className="flex flex-col">
          <span className="text-sm font-medium">{data.label || 'Agent'}</span>
          {!data.agentId && (
            <span className="text-xs text-muted-foreground">No agent selected</span>
          )}
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-blue-500 !w-3 !h-3" />
    </div>
  );
}
