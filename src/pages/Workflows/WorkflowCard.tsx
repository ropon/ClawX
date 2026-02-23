/**
 * Workflow Card
 * Displays workflow summary in the list view
 */
import { Play, MoreHorizontal, Copy, Download, Trash2, Pencil } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useTranslation } from 'react-i18next';
import type { WorkflowConfig } from '@/types/workflow';

interface WorkflowCardProps {
  workflow: WorkflowConfig;
  onEdit: (id: string) => void;
  onRun: (id: string) => void;
  onDuplicate: (id: string) => void;
  onExport: (id: string) => void;
  onDelete: (id: string) => void;
}

export function WorkflowCard({
  workflow,
  onEdit,
  onRun,
  onDuplicate,
  onExport,
  onDelete,
}: WorkflowCardProps) {
  const { t } = useTranslation('workflows');

  const agentCount = workflow.nodes.filter((n) => n.type === 'agent').length;
  const triggerCount = workflow.triggers.filter((t) => t.enabled).length;

  return (
    <div
      className="group rounded-lg border bg-card p-4 shadow-sm transition-colors hover:bg-accent/50 cursor-pointer"
      onClick={() => onEdit(workflow.id)}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{workflow.icon}</span>
          <div>
            <h3 className="font-medium text-sm">{workflow.name}</h3>
            {workflow.description && (
              <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">
                {workflow.description}
              </p>
            )}
          </div>
        </div>

        <DropdownMenu>
          <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
            <Button variant="ghost" size="icon" className="h-8 w-8 opacity-0 group-hover:opacity-100">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onEdit(workflow.id); }}>
              <Pencil className="h-4 w-4 mr-2" />
              {t('edit')}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onDuplicate(workflow.id); }}>
              <Copy className="h-4 w-4 mr-2" />
              {t('duplicate')}
            </DropdownMenuItem>
            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onExport(workflow.id); }}>
              <Download className="h-4 w-4 mr-2" />
              {t('export')}
            </DropdownMenuItem>
            <DropdownMenuItem
              className="text-destructive"
              onClick={(e) => { e.stopPropagation(); onDelete(workflow.id); }}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              {t('delete')}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="flex items-center gap-2 mt-3">
        {agentCount > 0 && (
          <Badge variant="secondary" className="text-xs">
            {agentCount} {t('nodeTypes.agent')}
          </Badge>
        )}
        {triggerCount > 0 && (
          <Badge variant="outline" className="text-xs">
            {triggerCount} {t('triggers.title')}
          </Badge>
        )}
        {!workflow.enabled && (
          <Badge variant="outline" className="text-xs text-muted-foreground">
            {t('disabled')}
          </Badge>
        )}
        <div className="flex-1" />
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={(e) => {
            e.stopPropagation();
            onRun(workflow.id);
          }}
        >
          <Play className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
