/**
 * Workflow Run History
 * Dialog showing all historical runs for a workflow
 */
import { useEffect, useState } from 'react';
import { Trash2, Eye, AlertTriangle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useTranslation } from 'react-i18next';
import { useWorkflowStore } from '@/stores/workflow';
import type { WorkflowRun } from '@/types/workflow';

interface WorkflowRunHistoryProps {
  workflowId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function statusColor(status: string): string {
  switch (status) {
    case 'completed': return 'text-green-500';
    case 'failed': return 'text-red-500';
    case 'running': return 'text-blue-500';
    case 'cancelled': return 'text-amber-500';
    default: return 'text-muted-foreground';
  }
}

function RunDetailDialog({ run, open, onClose }: { run: WorkflowRun | null; open: boolean; onClose: () => void }) {
  const { t } = useTranslation('workflows');
  if (!run) return null;

  return (
    <Dialog open={open} onOpenChange={() => onClose()}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>Run {run.id.slice(0, 12)}...</DialogTitle>
        </DialogHeader>
        <div className="space-y-3 max-h-[60vh] overflow-auto">
          <div className="flex gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">{t('execution.startedAt')}: </span>
              {new Date(run.startedAt).toLocaleString()}
            </div>
            {run.completedAt && (
              <div>
                <span className="text-muted-foreground">{t('execution.duration')}: </span>
                {((run.completedAt - run.startedAt) / 1000).toFixed(1)}s
              </div>
            )}
          </div>
          {run.error && (
            <div className="rounded bg-red-500/10 p-2 text-sm text-red-500">{run.error}</div>
          )}
          {run.steps.map((step) => (
            <div key={step.nodeId} className="rounded border p-2 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">{step.nodeId}</span>
                <Badge variant="outline" className={statusColor(step.status)}>
                  {t(`execution.${step.status}`)}
                </Badge>
              </div>
              {step.output && (
                <p className="text-xs text-muted-foreground whitespace-pre-wrap max-h-20 overflow-auto">
                  {step.output.slice(0, 500)}
                </p>
              )}
            </div>
          ))}
          {run.finalOutput && (
            <div className="rounded bg-muted p-2">
              <p className="text-xs font-medium mb-1">{t('execution.finalOutput')}</p>
              <p className="text-sm whitespace-pre-wrap">{run.finalOutput}</p>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

export function WorkflowRunHistory({ workflowId, open, onOpenChange }: WorkflowRunHistoryProps) {
  const { t } = useTranslation('workflows');
  const runs = useWorkflowStore((s) => s.runs);
  const fetchRuns = useWorkflowStore((s) => s.fetchRuns);
  const deleteRun = useWorkflowStore((s) => s.deleteRun);
  const clearRuns = useWorkflowStore((s) => s.clearRuns);

  const [selectedRun, setSelectedRun] = useState<WorkflowRun | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);

  useEffect(() => {
    if (open && workflowId) {
      fetchRuns(workflowId);
    }
  }, [open, workflowId, fetchRuns]);

  const handleClear = async () => {
    await clearRuns(workflowId);
    setConfirmClear(false);
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-xl">
          <DialogHeader>
            <DialogTitle>{t('history.title')}</DialogTitle>
          </DialogHeader>
          <ScrollArea className="max-h-[60vh]">
            {runs.length === 0 ? (
              <p className="text-sm text-muted-foreground text-center py-8">{t('history.empty')}</p>
            ) : (
              <div className="space-y-2">
                {runs.map((run) => (
                  <div key={run.id} className="flex items-center gap-3 rounded border p-2">
                    <Badge variant="outline" className={statusColor(run.status)}>
                      {t(`execution.${run.status}`)}
                    </Badge>
                    <div className="flex-1 min-w-0">
                      <p className="text-xs text-muted-foreground">
                        {new Date(run.startedAt).toLocaleString()}
                        {run.completedAt && ` (${((run.completedAt - run.startedAt) / 1000).toFixed(1)}s)`}
                      </p>
                    </div>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setSelectedRun(run)}>
                      <Eye className="h-3.5 w-3.5" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => deleteRun(run.id)}>
                      <Trash2 className="h-3.5 w-3.5 text-destructive" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>
          {runs.length > 0 && (
            <DialogFooter>
              {confirmClear ? (
                <div className="flex items-center gap-2">
                  <AlertTriangle className="h-4 w-4 text-amber-500" />
                  <span className="text-sm">{t('history.confirmClear')}</span>
                  <Button size="sm" variant="destructive" onClick={handleClear}>
                    {t('history.clearAll')}
                  </Button>
                  <Button size="sm" variant="outline" onClick={() => setConfirmClear(false)}>
                    Cancel
                  </Button>
                </div>
              ) : (
                <Button variant="outline" size="sm" onClick={() => setConfirmClear(true)}>
                  {t('history.clearAll')}
                </Button>
              )}
            </DialogFooter>
          )}
        </DialogContent>
      </Dialog>

      <RunDetailDialog run={selectedRun} open={!!selectedRun} onClose={() => setSelectedRun(null)} />
    </>
  );
}
