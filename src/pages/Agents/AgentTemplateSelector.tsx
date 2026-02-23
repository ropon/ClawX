/**
 * Agent Template Selector
 * Dialog for choosing a template when creating a new agent
 */
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import { useTranslation } from 'react-i18next';
import { AGENT_TEMPLATES, type AgentTemplate } from '@/data/agent-templates';

interface AgentTemplateSelectorProps {
  open: boolean;
  onClose: () => void;
  onSelect: (template: AgentTemplate | null) => void;
}

export function AgentTemplateSelector({ open, onClose, onSelect }: AgentTemplateSelectorProps) {
  const { t } = useTranslation('agents');

  const handleSelect = (template: AgentTemplate | null) => {
    onSelect(template);
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('templates.sectionTitle')}</DialogTitle>
          <DialogDescription>{t('templates.sectionDesc')}</DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-2 gap-3 py-4">
          {/* Blank Agent */}
          <button
            type="button"
            className="flex items-start gap-3 rounded-lg border-2 border-dashed border-border p-4 text-left transition-colors hover:border-primary hover:bg-accent/50"
            onClick={() => handleSelect(null)}
          >
            <span className="text-2xl">📄</span>
            <div className="min-w-0">
              <p className="font-medium">{t('templates.blankAgent')}</p>
              <p className="text-sm text-muted-foreground mt-0.5">
                {t('templates.blankAgentDesc')}
              </p>
            </div>
          </button>

          {/* Template Cards */}
          {AGENT_TEMPLATES.map((tmpl) => (
            <button
              key={tmpl.id}
              type="button"
              className="flex items-start gap-3 rounded-lg border border-border p-4 text-left transition-colors hover:border-primary hover:bg-accent/50"
              onClick={() => handleSelect(tmpl)}
            >
              <span className="text-2xl">{tmpl.avatar}</span>
              <div className="min-w-0">
                <p className="font-medium">{t(tmpl.nameKey)}</p>
                <p className="text-sm text-muted-foreground mt-0.5 line-clamp-2">
                  {t(tmpl.descriptionKey)}
                </p>
              </div>
            </button>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
