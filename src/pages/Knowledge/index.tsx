/**
 * Knowledge Page
 * Main knowledge base management page with list and detail views.
 */
import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Plus, BookOpen } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useKnowledgeStore } from '@/stores/knowledge';
import { KnowledgeCard } from './KnowledgeCard';
import { KnowledgeEditor } from './KnowledgeEditor';
import { KnowledgeDetail } from './KnowledgeDetail';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import type { KnowledgeBase } from '@/types/knowledge';

export function Knowledge() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { t } = useTranslation('knowledge');

  const knowledgeBases = useKnowledgeStore((s) => s.knowledgeBases);
  const loading = useKnowledgeStore((s) => s.loading);
  const fetchKnowledgeBases = useKnowledgeStore((s) => s.fetchKnowledgeBases);
  const deleteKnowledgeBase = useKnowledgeStore((s) => s.deleteKnowledgeBase);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editingKB, setEditingKB] = useState<KnowledgeBase | null>(null);

  useEffect(() => {
    fetchKnowledgeBases();
  }, [fetchKnowledgeBases]);

  const selectedKB = id ? knowledgeBases.find(kb => kb.id === id) : null;

  const handleCreate = () => {
    setEditingKB(null);
    setEditorOpen(true);
  };

  const handleEdit = (kb: KnowledgeBase) => {
    setEditingKB(kb);
    setEditorOpen(true);
  };

  const handleDelete = async (kb: KnowledgeBase) => {
    if (!confirm(t('deleteConfirm.message', { name: kb.name }))) return;
    try {
      await deleteKnowledgeBase(kb.id);
      if (id === kb.id) navigate('/knowledge');
    } catch {
      toast.error('Failed to delete knowledge base');
    }
  };

  // Detail view
  if (selectedKB) {
    return (
      <div className="flex-1 overflow-auto p-6">
        <KnowledgeDetail
          kb={selectedKB}
          onBack={() => navigate('/knowledge')}
        />
      </div>
    );
  }

  // List view
  return (
    <div className="flex-1 overflow-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">{t('title')}</h1>
          <p className="text-sm text-muted-foreground mt-1">{t('subtitle')}</p>
        </div>
        <Button onClick={handleCreate}>
          <Plus className="h-4 w-4 mr-2" />
          {t('createKB')}
        </Button>
      </div>

      {loading ? (
        <div className="text-sm text-muted-foreground">{t('common:status.loading')}</div>
      ) : knowledgeBases.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 space-y-4">
          <BookOpen className="h-12 w-12 text-muted-foreground/50" />
          <h2 className="text-lg font-medium">{t('empty.title')}</h2>
          <p className="text-sm text-muted-foreground text-center max-w-md">
            {t('empty.description')}
          </p>
          <Button onClick={handleCreate}>
            <Plus className="h-4 w-4 mr-2" />
            {t('empty.cta')}
          </Button>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {knowledgeBases.map((kb) => (
            <KnowledgeCard
              key={kb.id}
              kb={kb}
              onSelect={() => navigate(`/knowledge/${kb.id}`)}
              onEdit={() => handleEdit(kb)}
              onDelete={() => handleDelete(kb)}
            />
          ))}
        </div>
      )}

      <KnowledgeEditor
        open={editorOpen}
        kb={editingKB}
        onClose={() => setEditorOpen(false)}
      />
    </div>
  );
}
