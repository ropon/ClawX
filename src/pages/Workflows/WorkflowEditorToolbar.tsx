/**
 * Workflow Editor Toolbar
 * Top toolbar with save, run, stop, and zoom controls
 */
import { useState } from 'react';
import { ArrowLeft, Play, Square, Save, ZoomIn, ZoomOut, Maximize } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { useTranslation } from 'react-i18next';

interface WorkflowEditorToolbarProps {
  name: string;
  onNameChange: (name: string) => void;
  onSave: () => void;
  onBack: () => void;
  onRun: (input: string) => void;
  onStop: () => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitView: () => void;
  isRunning: boolean;
  saving: boolean;
}

export function WorkflowEditorToolbar({
  name,
  onNameChange,
  onSave,
  onBack,
  onRun,
  onStop,
  onZoomIn,
  onZoomOut,
  onFitView,
  isRunning,
  saving,
}: WorkflowEditorToolbarProps) {
  const { t } = useTranslation('workflows');
  const [runDialogOpen, setRunDialogOpen] = useState(false);
  const [runInput, setRunInput] = useState('');

  const handleRun = () => {
    onRun(runInput);
    setRunDialogOpen(false);
    setRunInput('');
  };

  return (
    <>
      <div className="flex items-center gap-2 border-b px-3 py-2 bg-background">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-4 w-4" />
        </Button>

        <Input
          className="w-64 h-8"
          value={name}
          onChange={(e) => onNameChange(e.target.value)}
        />

        <div className="flex-1" />

        {/* Zoom controls */}
        <div className="flex items-center gap-1 border-r pr-2 mr-1">
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onZoomIn} title={t('editor.zoomIn')}>
            <ZoomIn className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onZoomOut} title={t('editor.zoomOut')}>
            <ZoomOut className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onFitView} title={t('editor.fitView')}>
            <Maximize className="h-4 w-4" />
          </Button>
        </div>

        {/* Run/Stop */}
        {isRunning ? (
          <Button variant="destructive" size="sm" onClick={onStop}>
            <Square className="h-4 w-4 mr-1" />
            {t('execution.stop')}
          </Button>
        ) : (
          <Button variant="default" size="sm" onClick={() => setRunDialogOpen(true)}>
            <Play className="h-4 w-4 mr-1" />
            {t('execution.run')}
          </Button>
        )}

        {/* Save */}
        <Button variant="outline" size="sm" onClick={onSave} disabled={saving}>
          <Save className="h-4 w-4 mr-1" />
          {t('editor.save')}
        </Button>
      </div>

      {/* Run input dialog */}
      <Dialog open={runDialogOpen} onOpenChange={setRunDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('execution.runWorkflow')}</DialogTitle>
          </DialogHeader>
          <Textarea
            rows={4}
            value={runInput}
            onChange={(e) => setRunInput(e.target.value)}
            placeholder={t('execution.inputPlaceholder')}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setRunDialogOpen(false)}>
              {t('execution.noInput')}
            </Button>
            <Button onClick={handleRun}>
              <Play className="h-4 w-4 mr-1" />
              {t('execution.run')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
