/**
 * Knowledge Search
 * Search within a knowledge base and preview results (F-3.6).
 */
import { useState } from 'react';
import { Search, FileText } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { useKnowledgeStore } from '@/stores/knowledge';
import { useTranslation } from 'react-i18next';

interface KnowledgeSearchProps {
  kbId: string;
}

export function KnowledgeSearch({ kbId }: KnowledgeSearchProps) {
  const { t } = useTranslation('knowledge');
  const searchResults = useKnowledgeStore((s) => s.searchResults);
  const searching = useKnowledgeStore((s) => s.searching);
  const searchKB = useKnowledgeStore((s) => s.searchKB);
  const [query, setQuery] = useState('');

  const handleSearch = () => {
    if (query.trim()) {
      searchKB(kbId, query.trim());
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleSearch();
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-medium">{t('search.title')}</h3>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('search.placeholder')}
            className="pl-9"
          />
        </div>
      </div>

      {searching && (
        <div className="text-sm text-muted-foreground animate-pulse">
          {t('detail.processing')}
        </div>
      )}

      {searchResults.length > 0 && (
        <div className="space-y-3">
          {searchResults.map((result) => (
            <div
              key={result.chunkId}
              className="rounded-lg border p-3 space-y-2"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm">
                  <FileText className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="font-medium">{result.documentName}</span>
                </div>
                <Badge variant="outline" className="text-xs">
                  {t('search.score')}: {(result.score * 100).toFixed(0)}%
                </Badge>
              </div>
              <p className="text-sm text-muted-foreground whitespace-pre-wrap line-clamp-4">
                {result.content}
              </p>
            </div>
          ))}
        </div>
      )}

      {!searching && searchResults.length === 0 && query && (
        <p className="text-sm text-muted-foreground">{t('search.noResults')}</p>
      )}
    </div>
  );
}
