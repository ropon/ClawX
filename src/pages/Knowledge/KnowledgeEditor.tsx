/**
 * Knowledge Editor
 * Dialog for creating and editing knowledge base configurations.
 */
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select } from '@/components/ui/select';
import { invoke } from '@/lib/bridge';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useKnowledgeStore } from '@/stores/knowledge';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import type { KnowledgeBase } from '@/types/knowledge';

interface KnowledgeEditorProps {
  open: boolean;
  kb: KnowledgeBase | null;
  onClose: () => void;
}

interface FormData {
  name: string;
  description: string;
  embeddingModel: string;
  chunkSize: number;
  chunkOverlap: number;
}

export function KnowledgeEditor({ open, kb, onClose }: KnowledgeEditorProps) {
  const { t } = useTranslation('knowledge');
  const { t: tc } = useTranslation('common');
  const createKB = useKnowledgeStore((s) => s.createKnowledgeBase);
  const updateKB = useKnowledgeStore((s) => s.updateKnowledgeBase);
  const embeddingOptions = useKnowledgeStore((s) => s.embeddingOptions);
  const fetchEmbeddingOptions = useKnowledgeStore((s) => s.fetchEmbeddingOptions);

  const [saving, setSaving] = useState(false);
  const [detectingDim, setDetectingDim] = useState(false);
  const [dimension, setDimension] = useState(384);
  const [form, setForm] = useState<FormData>({
    name: '',
    description: '',
    embeddingModel: 'local:all-MiniLM-L6-v2',
    chunkSize: 500,
    chunkOverlap: 100,
  });

  useEffect(() => {
    if (open) {
      fetchEmbeddingOptions();
      if (kb) {
        setForm({
          name: kb.name,
          description: kb.description,
          embeddingModel: kb.embeddingModel,
          chunkSize: kb.chunkSize,
          chunkOverlap: kb.chunkOverlap,
        });
        setDimension(kb.embeddingDimension);
      } else {
        setForm({
          name: '',
          description: '',
          embeddingModel: embeddingOptions[0]?.value || 'local:all-MiniLM-L6-v2',
          chunkSize: 500,
          chunkOverlap: 100,
        });
        setDimension(embeddingOptions[0]?.dimension || 384);
      }
    }
  }, [open, kb, fetchEmbeddingOptions]);

  // Update dimension when embedding model changes
  useEffect(() => {
    const opt = embeddingOptions.find(o => o.value === form.embeddingModel);
    if (opt) {
      setDimension(opt.dimension);
    }
  }, [form.embeddingModel, embeddingOptions]);

  const handleDetectDimension = async () => {
    setDetectingDim(true);
    try {
      const result = await invoke<{ success: boolean; dimension?: number; error?: string }>(
        'knowledge:detectDimension',
        form.embeddingModel
      );
      if (result.success && result.dimension) {
        setDimension(result.dimension);
        toast.success(`Dimension: ${result.dimension}`);
      } else {
        toast.error(result.error || 'Failed to detect dimension');
      }
    } catch {
      toast.error('Failed to detect dimension');
    } finally {
      setDetectingDim(false);
    }
  };

  const handleSave = async () => {
    if (!form.name.trim()) {
      toast.error('Name is required');
      return;
    }

    setSaving(true);
    try {
      if (kb) {
        await updateKB(kb.id, {
          name: form.name.trim(),
          description: form.description.trim(),
          chunkSize: form.chunkSize,
          chunkOverlap: form.chunkOverlap,
        });
      } else {
        await createKB({
          name: form.name.trim(),
          description: form.description.trim(),
          embeddingModel: form.embeddingModel,
          embeddingDimension: dimension,
          chunkSize: form.chunkSize,
          chunkOverlap: form.chunkOverlap,
        });
      }
      onClose();
      toast.success(tc('actions.save'));
    } catch {
      toast.error('Failed to save knowledge base');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {kb ? t('editor.editTitle') : t('editor.createTitle')}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>{t('editor.name')}</Label>
            <Input
              value={form.name}
              onChange={(e) => setForm(f => ({ ...f, name: e.target.value }))}
              placeholder={t('editor.namePlaceholder')}
            />
          </div>

          <div className="space-y-2">
            <Label>{t('editor.description')}</Label>
            <Input
              value={form.description}
              onChange={(e) => setForm(f => ({ ...f, description: e.target.value }))}
              placeholder={t('editor.descriptionPlaceholder')}
            />
          </div>

          {!kb && (
            <div className="space-y-2">
              <Label>{t('editor.embeddingModel')}</Label>
              <p className="text-xs text-muted-foreground">{t('editor.embeddingModelDesc')}</p>
              <div className="flex gap-2">
                <Select
                  value={form.embeddingModel}
                  onChange={(e) => setForm(f => ({ ...f, embeddingModel: e.target.value }))}
                  className="flex-1"
                >
                  {embeddingOptions.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </Select>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleDetectDimension}
                  disabled={detectingDim}
                >
                  {detectingDim ? t('editor.detectingDimension') : `${dimension}d`}
                </Button>
              </div>
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>{t('editor.chunkSize')}</Label>
              <p className="text-xs text-muted-foreground">{t('editor.chunkSizeDesc')}</p>
              <Input
                type="number"
                value={form.chunkSize}
                onChange={(e) => setForm(f => ({ ...f, chunkSize: parseInt(e.target.value) || 500 }))}
                min={100}
                max={4000}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('editor.chunkOverlap')}</Label>
              <p className="text-xs text-muted-foreground">{t('editor.chunkOverlapDesc')}</p>
              <Input
                type="number"
                value={form.chunkOverlap}
                onChange={(e) => setForm(f => ({ ...f, chunkOverlap: parseInt(e.target.value) || 100 }))}
                min={0}
                max={1000}
              />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {tc('actions.cancel')}
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving ? tc('status.saving') : tc('actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
