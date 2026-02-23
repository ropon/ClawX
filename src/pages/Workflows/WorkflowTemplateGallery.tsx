/**
 * Workflow Template Gallery
 * Browse and use pre-built workflow templates
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import { useWorkflowStore } from '@/stores/workflow';
import { workflowTemplates } from '@/data/workflow-templates';
import type { WorkflowTemplate } from '@/types/workflow';

interface WorkflowTemplateGalleryProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const CATEGORIES = ['all', 'productivity', 'content', 'code', 'research', 'automation'] as const;

export function WorkflowTemplateGallery({ open, onOpenChange }: WorkflowTemplateGalleryProps) {
  const { t } = useTranslation('workflows');
  const navigate = useNavigate();
  const createWorkflow = useWorkflowStore((s) => s.createWorkflow);
  const [category, setCategory] = useState<string>('all');

  const filtered = category === 'all'
    ? workflowTemplates
    : workflowTemplates.filter((tpl) => tpl.category === category);

  const handleUse = async (template: WorkflowTemplate) => {
    try {
      const wfId = await createWorkflow({
        name: template.name,
        description: template.description,
        icon: template.icon,
        nodes: template.nodes,
        edges: template.edges,
        triggers: template.triggers,
      });
      onOpenChange(false);
      navigate(`/workflows/${wfId}`);
      toast.success(t('templates.useTemplate'));
    } catch (err) {
      toast.error(String(err));
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh]">
        <DialogHeader>
          <DialogTitle>{t('templates.title')}</DialogTitle>
        </DialogHeader>

        {/* Category tabs */}
        <div className="flex gap-1 flex-wrap">
          {CATEGORIES.map((cat) => (
            <Button
              key={cat}
              variant={category === cat ? 'default' : 'outline'}
              size="sm"
              onClick={() => setCategory(cat)}
            >
              {t(`templates.categories.${cat}`)}
            </Button>
          ))}
        </div>

        {/* Template grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 overflow-auto max-h-[50vh]">
          {filtered.map((tpl) => (
            <div key={tpl.id} className="rounded-lg border p-4 space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-xl">{tpl.icon}</span>
                <div className="flex-1">
                  <h4 className="text-sm font-medium">{tpl.name}</h4>
                  <p className="text-xs text-muted-foreground">{tpl.description}</p>
                </div>
              </div>
              <div className="flex items-center justify-between">
                <div className="flex gap-1">
                  <Badge variant="secondary" className="text-xs">
                    {tpl.nodes.filter((n) => n.type === 'agent').length} agents
                  </Badge>
                  <Badge variant="outline" className="text-xs capitalize">
                    {tpl.category}
                  </Badge>
                </div>
                <Button size="sm" variant="outline" onClick={() => handleUse(tpl)}>
                  {t('templates.useTemplate')}
                </Button>
              </div>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
