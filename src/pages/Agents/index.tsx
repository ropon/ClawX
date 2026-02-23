/**
 * Agents Page
 * Create and manage AI agent roles
 */
import { useEffect, useState, useCallback } from 'react';
import { Search, Bot, Plus, Upload } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useAgentStore } from '@/stores/agents';
import { LoadingSpinner } from '@/components/common/LoadingSpinner';
import { AgentCard } from './AgentCard';
import { AgentEditor } from './AgentEditor';
import { AgentTemplateSelector } from './AgentTemplateSelector';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import type { AgentConfig } from '@/types/agent';
import type { AgentTemplate } from '@/data/agent-templates';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

export function Agents() {
  const { t } = useTranslation('agents');
  const { t: tc } = useTranslation('common');
  const agents = useAgentStore((s) => s.agents);
  const activeAgentId = useAgentStore((s) => s.activeAgentId);
  const loading = useAgentStore((s) => s.loading);
  const fetchAgents = useAgentStore((s) => s.fetchAgents);
  const deleteAgent = useAgentStore((s) => s.deleteAgent);
  const cloneAgent = useAgentStore((s) => s.cloneAgent);
  const setActiveAgent = useAgentStore((s) => s.setActiveAgent);
  const updateAgent = useAgentStore((s) => s.updateAgent);
  const exportAgent = useAgentStore((s) => s.exportAgent);
  const importAgent = useAgentStore((s) => s.importAgent);

  const [search, setSearch] = useState('');
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingAgent, setEditingAgent] = useState<AgentConfig | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AgentConfig | null>(null);
  const [templateSelectorOpen, setTemplateSelectorOpen] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<AgentTemplate | null>(null);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  const filteredAgents = agents.filter((a) =>
    a.name.toLowerCase().includes(search.toLowerCase()) ||
    a.description.toLowerCase().includes(search.toLowerCase())
  );

  const handleEdit = useCallback((agent: AgentConfig) => {
    setEditingAgent(agent);
    setEditorOpen(true);
  }, []);

  const handleCreate = useCallback(() => {
    setTemplateSelectorOpen(true);
  }, []);

  const handleTemplateSelect = useCallback((template: AgentTemplate | null) => {
    setSelectedTemplate(template);
    setEditingAgent(null);
    setEditorOpen(true);
  }, []);

  const handleClone = useCallback(async (id: string) => {
    try {
      await cloneAgent(id);
      toast.success(t('actions.clone'));
    } catch {
      toast.error('Failed to clone agent');
    }
  }, [cloneAgent, t]);

  const handleDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteAgent(deleteTarget.id);
      setDeleteTarget(null);
      toast.success(t('actions.delete'));
    } catch {
      toast.error('Failed to delete agent');
    }
  }, [deleteTarget, deleteAgent, t]);

  const handleSetDefault = useCallback(async (id: string) => {
    try {
      await updateAgent(id, { isDefault: true });
      // Unset other defaults
      for (const a of agents) {
        if (a.id !== id && a.isDefault) {
          await updateAgent(a.id, { isDefault: false });
        }
      }
    } catch {
      toast.error('Failed to set default');
    }
  }, [updateAgent, agents]);

  const handleExport = useCallback(async (id: string) => {
    try {
      const json = await exportAgent(id);
      if (json) {
        await navigator.clipboard.writeText(json);
        toast.success('Agent config copied to clipboard');
      }
    } catch {
      toast.error('Failed to export agent');
    }
  }, [exportAgent]);

  const handleImport = useCallback(async () => {
    try {
      const json = await navigator.clipboard.readText();
      await importAgent(json);
      toast.success('Agent imported');
    } catch {
      toast.error('Failed to import agent. Make sure clipboard contains valid agent JSON.');
    }
  }, [importAgent]);

  const handleSelect = useCallback(async (id: string) => {
    try {
      await setActiveAgent(id);
    } catch {
      toast.error('Failed to select agent');
    }
  }, [setActiveAgent]);

  const handleEditorClose = useCallback(() => {
    setEditorOpen(false);
    setEditingAgent(null);
    setSelectedTemplate(null);
  }, []);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('title')}</h1>
          <p className="text-muted-foreground">{t('subtitle')}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleImport}>
            <Upload className="h-4 w-4 mr-2" />
            {t('actions.import')}
          </Button>
          <Button size="sm" onClick={handleCreate}>
            <Plus className="h-4 w-4 mr-2" />
            {t('createAgent')}
          </Button>
        </div>
      </div>

      {/* Search */}
      <div className="relative max-w-sm">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder={t('searchPlaceholder')}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="pl-10"
        />
      </div>

      {/* Content */}
      {loading ? (
        <LoadingSpinner className="py-12" />
      ) : filteredAgents.length === 0 ? (
        agents.length === 0 ? (
          /* Empty state */
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <Bot className="h-16 w-16 text-muted-foreground/50 mb-4" />
            <h3 className="text-lg font-semibold">{t('empty.title')}</h3>
            <p className="text-muted-foreground mt-1 max-w-md">{t('empty.description')}</p>
            <Button className="mt-6" onClick={handleCreate}>
              <Plus className="h-4 w-4 mr-2" />
              {t('empty.cta')}
            </Button>
          </div>
        ) : (
          /* No search results */
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <Search className="h-12 w-12 text-muted-foreground/50 mb-4" />
            <p className="text-muted-foreground">No agents match your search</p>
          </div>
        )
      ) : (
        /* Agent grid */
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredAgents.map((agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              isActive={agent.id === activeAgentId}
              onEdit={handleEdit}
              onClone={handleClone}
              onDelete={setDeleteTarget}
              onSetDefault={handleSetDefault}
              onExport={handleExport}
              onSelect={handleSelect}
            />
          ))}
        </div>
      )}

      {/* Template Selector Dialog */}
      <AgentTemplateSelector
        open={templateSelectorOpen}
        onClose={() => setTemplateSelectorOpen(false)}
        onSelect={handleTemplateSelect}
      />

      {/* Editor Dialog */}
      <AgentEditor
        open={editorOpen}
        agent={editingAgent}
        template={selectedTemplate}
        onClose={handleEditorClose}
      />

      {/* Delete Confirmation Dialog */}
      <Dialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('deleteConfirm.title')}</DialogTitle>
            <DialogDescription>
              {t('deleteConfirm.message', { name: deleteTarget?.name })}
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
    </div>
  );
}
