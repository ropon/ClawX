/**
 * Workflow Trigger Editor
 * Dialog for configuring workflow triggers (cron, file-change, clipboard, shortcut)
 */
import { useState } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { useTranslation } from 'react-i18next';
import type { WorkflowTrigger, TriggerType } from '@/types/workflow';

interface WorkflowTriggerEditorProps {
  triggers: WorkflowTrigger[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (triggers: WorkflowTrigger[]) => void;
}

export function WorkflowTriggerEditor({
  triggers: initialTriggers,
  open,
  onOpenChange,
  onSave,
}: WorkflowTriggerEditorProps) {
  const { t } = useTranslation('workflows');
  const [triggers, setTriggers] = useState<WorkflowTrigger[]>(initialTriggers);

  const addTrigger = (type: TriggerType) => {
    const newTrigger: WorkflowTrigger = {
      id: `trigger-${Date.now()}`,
      type,
      enabled: true,
    };
    setTriggers([...triggers, newTrigger]);
  };

  const updateTrigger = (id: string, updates: Partial<WorkflowTrigger>) => {
    setTriggers(triggers.map((tr) => (tr.id === id ? { ...tr, ...updates } : tr)));
  };

  const removeTrigger = (id: string) => {
    setTriggers(triggers.filter((tr) => tr.id !== id));
  };

  const handleSave = () => {
    onSave(triggers);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('triggers.title')}</DialogTitle>
        </DialogHeader>

        <div className="space-y-3 max-h-[60vh] overflow-auto">
          {triggers.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-4">
              {t('triggers.noTriggers')}
            </p>
          )}
          {triggers.map((tr) => (
            <div key={tr.id} className="rounded-lg border p-3 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium capitalize">{t(`triggers.${tr.type === 'file-change' ? 'fileChange' : tr.type}`)}</span>
                <div className="flex items-center gap-2">
                  <Switch
                    checked={tr.enabled}
                    onCheckedChange={(checked) => updateTrigger(tr.id, { enabled: checked })}
                  />
                  <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => removeTrigger(tr.id)}>
                    <Trash2 className="h-3.5 w-3.5 text-destructive" />
                  </Button>
                </div>
              </div>

              {tr.type === 'cron' && (
                <div className="space-y-1">
                  <Label className="text-xs">{t('triggers.cronExpr')}</Label>
                  <Input
                    value={tr.cronExpr || ''}
                    onChange={(e) => updateTrigger(tr.id, { cronExpr: e.target.value })}
                    placeholder={t('triggers.cronPlaceholder')}
                  />
                </div>
              )}

              {tr.type === 'file-change' && (
                <>
                  <div className="space-y-1">
                    <Label className="text-xs">{t('triggers.watchPath')}</Label>
                    <Input
                      value={tr.watchPath || ''}
                      onChange={(e) => updateTrigger(tr.id, { watchPath: e.target.value })}
                      placeholder="/path/to/watch"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs">{t('triggers.filePattern')}</Label>
                    <Input
                      value={tr.filePattern || ''}
                      onChange={(e) => updateTrigger(tr.id, { filePattern: e.target.value })}
                      placeholder="*.txt"
                    />
                  </div>
                </>
              )}

              {tr.type === 'clipboard' && (
                <div className="space-y-1">
                  <Label className="text-xs">{t('triggers.clipboardPattern')}</Label>
                  <Input
                    value={tr.clipboardPattern || ''}
                    onChange={(e) => updateTrigger(tr.id, { clipboardPattern: e.target.value })}
                    placeholder="https?://.*"
                  />
                </div>
              )}

              {tr.type === 'shortcut' && (
                <div className="space-y-1">
                  <Label className="text-xs">{t('triggers.shortcutKey')}</Label>
                  <Input
                    value={tr.shortcutAccelerator || ''}
                    onChange={(e) => updateTrigger(tr.id, { shortcutAccelerator: e.target.value })}
                    placeholder="CommandOrControl+Shift+W"
                  />
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Add trigger buttons */}
        <div className="flex flex-wrap gap-2">
          {(['cron', 'file-change', 'clipboard', 'shortcut'] as TriggerType[]).map((type) => (
            <Button key={type} variant="outline" size="sm" onClick={() => addTrigger(type)}>
              <Plus className="h-3 w-3 mr-1" />
              {t(`triggers.${type === 'file-change' ? 'fileChange' : type}`)}
            </Button>
          ))}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('triggers.title')}
          </Button>
          <Button onClick={handleSave}>{t('editor.save')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
