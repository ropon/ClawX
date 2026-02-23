/**
 * Output Node
 * Exit point for workflow results
 */
import { Handle, Position } from '@xyflow/react';
import { LogOut } from 'lucide-react';

interface OutputNodeProps {
  data: { label: string };
  selected?: boolean;
}

export function OutputNode({ data, selected }: OutputNodeProps) {
  return (
    <div
      className={`rounded-xl border-2 bg-background px-4 py-3 shadow-sm min-w-[140px] ${
        selected ? 'border-primary' : 'border-red-500/50'
      }`}
    >
      <Handle type="target" position={Position.Top} className="!bg-red-500 !w-3 !h-3" />
      <div className="flex items-center gap-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-full bg-red-500/10">
          <LogOut className="h-4 w-4 text-red-500" />
        </div>
        <span className="text-sm font-medium">{data.label || 'Output'}</span>
      </div>
    </div>
  );
}
