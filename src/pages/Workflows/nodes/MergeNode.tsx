/**
 * Merge Node
 * Combines outputs from multiple branches
 */
import { Handle, Position } from '@xyflow/react';
import { Merge } from 'lucide-react';

interface MergeNodeProps {
  data: {
    label: string;
    mergeStrategy?: 'concat' | 'first' | 'custom';
  };
  selected?: boolean;
}

export function MergeNode({ data, selected }: MergeNodeProps) {
  return (
    <div
      className={`rounded-xl border-2 bg-background px-4 py-3 shadow-sm min-w-[140px] ${
        selected ? 'border-primary' : 'border-purple-500/50'
      }`}
    >
      <Handle type="target" position={Position.Top} id="in_0" className="!bg-purple-500 !w-3 !h-3" style={{ left: '33%' }} />
      <Handle type="target" position={Position.Top} id="in_1" className="!bg-purple-500 !w-3 !h-3" style={{ left: '66%' }} />
      <div className="flex items-center gap-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-full bg-purple-500/10">
          <Merge className="h-4 w-4 text-purple-500" />
        </div>
        <div className="flex flex-col">
          <span className="text-sm font-medium">{data.label || 'Merge'}</span>
          <span className="text-xs text-muted-foreground capitalize">{data.mergeStrategy || 'concat'}</span>
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-purple-500 !w-3 !h-3" />
    </div>
  );
}
