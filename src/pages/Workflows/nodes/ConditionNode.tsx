/**
 * Condition Node
 * Routes workflow based on conditions
 */
import { Handle, Position } from '@xyflow/react';
import { GitFork } from 'lucide-react';
import type { ConditionRule } from '@/types/workflow';

interface ConditionNodeProps {
  data: {
    label: string;
    conditionRules?: ConditionRule[];
  };
  selected?: boolean;
}

export function ConditionNode({ data, selected }: ConditionNodeProps) {
  const rules = data.conditionRules || [];

  return (
    <div
      className={`rounded-xl border-2 bg-background px-4 py-3 shadow-sm min-w-[160px] ${
        selected ? 'border-primary' : 'border-amber-500/50'
      }`}
      style={{ transform: 'rotate(0deg)' }}
    >
      <Handle type="target" position={Position.Top} className="!bg-amber-500 !w-3 !h-3" />
      <div className="flex items-center gap-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-full bg-amber-500/10">
          <GitFork className="h-4 w-4 text-amber-500" />
        </div>
        <span className="text-sm font-medium">{data.label || 'Condition'}</span>
      </div>
      {/* Output handles for each rule + default */}
      <div className="flex justify-around mt-2 relative" style={{ minHeight: 12 }}>
        {rules.length > 0 ? (
          rules.map((rule, i) => (
            <Handle
              key={rule.id}
              type="source"
              position={Position.Bottom}
              id={rule.handle}
              className="!bg-amber-500 !w-3 !h-3"
              style={{ left: `${((i + 1) / (rules.length + 1)) * 100}%` }}
            />
          ))
        ) : (
          <>
            <Handle
              type="source"
              position={Position.Bottom}
              id="true"
              className="!bg-green-500 !w-3 !h-3"
              style={{ left: '33%' }}
            />
            <Handle
              type="source"
              position={Position.Bottom}
              id="false"
              className="!bg-red-500 !w-3 !h-3"
              style={{ left: '66%' }}
            />
          </>
        )}
      </div>
    </div>
  );
}
