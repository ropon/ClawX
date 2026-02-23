/**
 * Knowledge Card
 * Displays a knowledge base summary in the list view.
 */
import { BookOpen, FileText, Layers, Trash2, Edit, FolderSync } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useTranslation } from 'react-i18next';
import type { KnowledgeBase } from '@/types/knowledge';

interface KnowledgeCardProps {
  kb: KnowledgeBase;
  onSelect: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function KnowledgeCard({ kb, onSelect, onEdit, onDelete }: KnowledgeCardProps) {
  const { t } = useTranslation('knowledge');

  return (
    <div
      className="group rounded-lg border bg-card p-4 hover:shadow-md transition-all cursor-pointer"
      onClick={onSelect}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3 min-w-0">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
            <BookOpen className="h-5 w-5 text-primary" />
          </div>
          <div className="min-w-0">
            <h3 className="font-medium truncate">{kb.name}</h3>
            <p className="text-sm text-muted-foreground truncate">
              {kb.description || t('card.noDescription')}
            </p>
          </div>
        </div>

        <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={(e) => { e.stopPropagation(); onEdit(); }}
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-destructive"
            onClick={(e) => { e.stopPropagation(); onDelete(); }}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="mt-3 flex items-center gap-3 text-sm text-muted-foreground">
        <span className="flex items-center gap-1">
          <FileText className="h-3.5 w-3.5" />
          {t('card.documents', { count: kb.documentCount })}
        </span>
        <span className="flex items-center gap-1">
          <Layers className="h-3.5 w-3.5" />
          {t('card.chunks', { count: kb.totalChunks })}
        </span>
        {kb.watchedFolder && (
          <Badge variant="secondary" className="text-xs gap-1">
            <FolderSync className="h-3 w-3" />
            {t('card.watchActive')}
          </Badge>
        )}
      </div>

      <div className="mt-2">
        <Badge variant="outline" className="text-xs">
          {kb.embeddingModel}
        </Badge>
      </div>
    </div>
  );
}
