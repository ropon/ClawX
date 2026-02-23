/**
 * Workflow Run Panel
 * Shows execution progress as a vertical timeline
 */
import { CheckCircle2, XCircle, Loader2, Clock, Square } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useTranslation } from 'react-i18next';
import type { WorkflowRun, StepResult } from '@/types/workflow';
import { useState } from 'react';

interface WorkflowRunPanelProps {
  run: WorkflowRun | null;
  onCancel: () => void;
}

function StepIcon({ status }: { status: string }) {
  switch (status) {
    case 'completed':
      return <CheckCircle2 className="h-4 w-4 text-green-500" />;
    case 'failed':
      return <XCircle className="h-4 w-4 text-red-500" />;
    case 'running':
      return <Loader2 className="h-4 w-4 text-blue-500 animate-spin" />;
    default:
      return <Clock className="h-4 w-4 text-muted-foreground" />;
  }
}

function StepItem({ step }: { step: StepResult }) {
  const [expanded, setExpanded] = useState(false);
  const duration = step.completedAt
    ? `${((step.completedAt - step.startedAt) / 1000).toFixed(1)}s`
    : null;

  return (
    <div className="flex gap-3">
      <div className="flex flex-col items-center">
        <StepIcon status={step.status} />
        <div className="flex-1 w-px bg-border mt-1" />
      </div>
      <div className="flex-1 pb-4">
        <button
          className="text-sm font-medium text-left w-full hover:underline"
          onClick={() => setExpanded(!expanded)}
        >
          {step.nodeId}
          {duration && <span className="text-xs text-muted-foreground ml-2">{duration}</span>}
        </button>
        {step.error && (
          <p className="text-xs text-red-500 mt-1">{step.error}</p>
        )}
        {expanded && (
          <div className="mt-2 space-y-1">
            {step.input && (
              <div className="rounded bg-muted/50 p-2">
                <p className="text-xs text-muted-foreground mb-0.5">Input</p>
                <p className="text-xs whitespace-pre-wrap max-h-24 overflow-auto">{step.input}</p>
              </div>
            )}
            {step.output && (
              <div className="rounded bg-muted/50 p-2">
                <p className="text-xs text-muted-foreground mb-0.5">Output</p>
                <p className="text-xs whitespace-pre-wrap max-h-24 overflow-auto">{step.output}</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export function WorkflowRunPanel({ run, onCancel }: WorkflowRunPanelProps) {
  const { t } = useTranslation('workflows');

  if (!run) return null;

  const isActive = run.status === 'running' || run.status === 'pending';

  return (
    <div className="border-l w-80 flex flex-col bg-background">
      <div className="flex items-center justify-between px-3 py-2 border-b">
        <h3 className="text-sm font-medium">{t('execution.stepProgress')}</h3>
        {isActive && (
          <Button variant="ghost" size="sm" onClick={onCancel}>
            <Square className="h-3 w-3 mr-1" />
            {t('execution.stop')}
          </Button>
        )}
      </div>

      <ScrollArea className="flex-1 p-3">
        {run.steps.map((step) => (
          <StepItem key={step.nodeId} step={step} />
        ))}
      </ScrollArea>

      {run.finalOutput && (
        <div className="border-t p-3">
          <p className="text-xs font-medium text-muted-foreground mb-1">{t('execution.finalOutput')}</p>
          <div className="rounded bg-muted/50 p-2 max-h-32 overflow-auto">
            <p className="text-xs whitespace-pre-wrap">{run.finalOutput}</p>
          </div>
        </div>
      )}
    </div>
  );
}
