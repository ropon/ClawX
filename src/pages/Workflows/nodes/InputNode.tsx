/**
 * Input Node
 * Entry point for workflow data
 */
import { Handle, Position } from '@xyflow/react';
import { LogIn } from 'lucide-react';

interface InputNodeProps {
  data: { label: string };
  selected?: boolean;
}

export function InputNode({ data, selected }: InputNodeProps) {
  return (
    <div
      className={`rounded-xl border-2 bg-background px-4 py-3 shadow-sm min-w-[140px] ${
        selected ? 'border-primary' : 'border-green-500/50'
      }`}
    >
      <div className="flex items-center gap-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-full bg-green-500/10">
          <LogIn className="h-4 w-4 text-green-500" />
        </div>
        <span className="text-sm font-medium">{data.label || 'Input'}</span>
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-green-500 !w-3 !h-3" />
    </div>
  );
}
