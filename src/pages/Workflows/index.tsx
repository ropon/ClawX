/**
 * Workflows Page
 * List and manage workflow orchestrations
 */
import { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Search, Plus, Upload, LayoutGrid } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { LoadingSpinner } from '@/components/common/LoadingSpinner';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import { useWorkflowStore } from '@/stores/workflow';
import { WorkflowCard } from './WorkflowCard';
import { WorkflowEditor } from './WorkflowEditor';
import { WorkflowTemplateGallery } from './WorkflowTemplateGallery';
import type { WorkflowConfig } from '@/types/workflow';

export function Workflows() {
  const { t } = useTranslation('workflows');
  const { t: tc } = useTranslation('common');
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();

  const workflows = useWorkflowStore((s) => s.workflows);
  const loading = useWorkflowStore((s) => s.loading);
  const fetchWorkflows = useWorkflowStore((s) => s.fetchWorkflows);
  const createWorkflow = useWorkflowStore((s) => s.createWorkflow);
  const deleteWorkflow = useWorkflowStore((s) => s.deleteWorkflow);
  const duplicateWorkflow = useWorkflowStore((s) => s.duplicateWorkflow);
  const exportWorkflow = useWorkflowStore((s) => s.exportWorkflow);
  const importWorkflow = useWorkflowStore((s) => s.importWorkflow);
  const executeWorkflow = useWorkflowStore((s) => s.executeWorkflow);
  const setCurrentWorkflow = useWorkflowStore((s) => s.setCurrentWorkflow);

  const [search, setSearch] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<WorkflowConfig | null>(null);
  const [templateGalleryOpen, setTemplateGalleryOpen] = useState(false);

  useEffect(() => {
    fetchWorkflows();
  }, [fetchWorkflows]);

  useEffect(() => {
    setCurrentWorkflow(id || null);
  }, [id, setCurrentWorkflow]);

  const currentWorkflow = id ? workflows.find((w) => w.id === id) : null;

  const filteredWorkflows = workflows.filter(
    (w) =>
      w.name.toLowerCase().includes(search.toLowerCase()) ||
      w.description.toLowerCase().includes(search.toLowerCase())
  );

  const handleCreate = async () => {
    try {
      const wfId = await createWorkflow({
        name: 'New Workflow',
        description: '',
        icon: '🔄',
        nodes: [
          { id: 'input-1', type: 'input', position: { x: 250, y: 50 }, data: { label: t('nodeTypes.input') } },
          { id: 'output-1', type: 'output', position: { x: 250, y: 400 }, data: { label: t('nodeTypes.output') } },
        ],
        edges: [],
      });
      navigate(`/workflows/${wfId}`);
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleEdit = useCallback(
    (wfId: string) => navigate(`/workflows/${wfId}`),
    [navigate]
  );

  const handleRun = useCallback(
    async (wfId: string) => {
      try {
        await executeWorkflow(wfId, '');
        toast.success(t('execution.running'));
      } catch (err) {
        toast.error(String(err));
      }
    },
    [executeWorkflow, t]
  );

  const handleDuplicate = useCallback(
    async (wfId: string) => {
      try {
        await duplicateWorkflow(wfId);
        toast.success(t('duplicate'));
      } catch (err) {
        toast.error(String(err));
      }
    },
    [duplicateWorkflow, t]
  );

  const handleExport = useCallback(
    async (wfId: string) => {
      try {
        const json = await exportWorkflow(wfId);
        if (json) {
          await navigator.clipboard.writeText(json);
          toast.success(t('export'));
        }
      } catch (err) {
        toast.error(String(err));
      }
    },
    [exportWorkflow, t]
  );

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteWorkflow(deleteTarget.id);
      setDeleteTarget(null);
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleImport = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim().startsWith('{')) {
        toast.error('Invalid JSON');
        return;
      }
      await importWorkflow(text);
      toast.success(t('import'));
    } catch (err) {
      toast.error(String(err));
    }
  };

  // Editor view
  if (currentWorkflow) {
    return (
      <WorkflowEditor
        workflow={currentWorkflow}
        onBack={() => navigate('/workflows')}
      />
    );
  }

  // List view
  if (loading && workflows.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold">{t('title')}</h1>
            <p className="text-sm text-muted-foreground">{t('description')}</p>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={handleImport}>
              <Upload className="h-4 w-4 mr-1" />
              {t('import')}
            </Button>
            <Button variant="outline" size="sm" onClick={() => setTemplateGalleryOpen(true)}>
              <LayoutGrid className="h-4 w-4 mr-1" />
              {t('templates.title')}
            </Button>
            <Button size="sm" onClick={handleCreate}>
              <Plus className="h-4 w-4 mr-1" />
              {t('create')}
            </Button>
          </div>
        </div>

        {workflows.length > 0 && (
          <div className="mt-3 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              className="pl-9"
              placeholder={t('search')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {filteredWorkflows.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="text-4xl mb-3">🔄</div>
            <p className="text-sm font-medium">{t('empty')}</p>
            <p className="text-xs text-muted-foreground mt-1">{t('emptyHint')}</p>
            <Button className="mt-4" size="sm" onClick={handleCreate}>
              <Plus className="h-4 w-4 mr-1" />
              {t('create')}
            </Button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredWorkflows.map((wf) => (
              <WorkflowCard
                key={wf.id}
                workflow={wf}
                onEdit={handleEdit}
                onRun={handleRun}
                onDuplicate={handleDuplicate}
                onExport={handleExport}
                onDelete={(wfId) => setDeleteTarget(workflows.find((w) => w.id === wfId) || null)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Delete dialog */}
      <Dialog open={!!deleteTarget} onOpenChange={() => setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('delete')}</DialogTitle>
            <DialogDescription>
              {deleteTarget?.name}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              {tc('actions.cancel')}
            </Button>
            <Button variant="destructive" onClick={handleDelete}>
              {tc('actions.delete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Template Gallery */}
      <WorkflowTemplateGallery
        open={templateGalleryOpen}
        onOpenChange={setTemplateGalleryOpen}
      />
    </div>
  );
}
