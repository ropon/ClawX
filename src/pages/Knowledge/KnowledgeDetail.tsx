/**
 * Knowledge Detail
 * Shows documents list, upload controls, search, and folder watcher for a KB.
 */
import { useEffect } from 'react';
import { ArrowLeft, RefreshCw, Trash2, FolderOpen, FolderSync } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useKnowledgeStore } from '@/stores/knowledge';
import { invoke, on } from '@/lib/bridge';
import { DocumentUpload } from './DocumentUpload';
import { KnowledgeSearch } from './KnowledgeSearch';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import type { KnowledgeBase } from '@/types/knowledge';

interface KnowledgeDetailProps {
  kb: KnowledgeBase;
  onBack: () => void;
}

const STATUS_COLORS: Record<string, string> = {
  ready: 'bg-green-500/10 text-green-600',
  processing: 'bg-yellow-500/10 text-yellow-600',
  pending: 'bg-gray-500/10 text-gray-500',
  error: 'bg-red-500/10 text-red-600',
};

export function KnowledgeDetail({ kb, onBack }: KnowledgeDetailProps) {
  const { t } = useTranslation('knowledge');
  const documents = useKnowledgeStore((s) => s.documents);
  const fetchDocuments = useKnowledgeStore((s) => s.fetchDocuments);
  const removeDocument = useKnowledgeStore((s) => s.removeDocument);
  const reprocessDocument = useKnowledgeStore((s) => s.reprocessDocument);
  const setWatchFolder = useKnowledgeStore((s) => s.setWatchFolder);

  useEffect(() => {
    fetchDocuments(kb.id);

    // Listen for document progress events
    const unsub = on('knowledge:documentProgress', (...args: unknown[]) => {
      const data = args[0] as { kbId: string; documentId: string; step: string; pct: number };
      if (data.kbId === kb.id && data.step === 'done') {
        fetchDocuments(kb.id);
      }
    });
    return () => { unsub(); };
  }, [kb.id, fetchDocuments]);

  const handleRemove = async (docId: string, docName: string) => {
    if (!confirm(t('detail.removeConfirm', { name: docName }))) return;
    try {
      await removeDocument(kb.id, docId);
    } catch {
      toast.error('Failed to remove document');
    }
  };

  const handleReprocess = async (docId: string) => {
    try {
      await reprocessDocument(kb.id, docId);
      toast.success('Reprocessing started');
    } catch {
      toast.error('Failed to reprocess');
    }
  };

  const handleSelectWatchFolder = async () => {
    try {
      const result = await invoke<{ canceled: boolean; filePaths: string[] }>('dialog:open', {
        properties: ['openDirectory'],
      });

      if (result.canceled || result.filePaths.length === 0) return;
      await setWatchFolder(kb.id, result.filePaths[0]);
      toast.success('Folder watcher started');
    } catch {
      toast.error('Failed to set watch folder');
    }
  };

  const handleStopWatching = async () => {
    try {
      await setWatchFolder(kb.id, null);
      toast.success('Folder watcher stopped');
    } catch {
      toast.error('Failed to stop watching');
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1 min-w-0">
          <h2 className="text-lg font-semibold truncate">{kb.name}</h2>
          <p className="text-sm text-muted-foreground truncate">
            {kb.description} &middot; {kb.embeddingModel}
          </p>
        </div>
      </div>

      {/* Upload Controls */}
      <DocumentUpload kbId={kb.id} />

      {/* Folder Watcher (F-3.8) */}
      <div className="rounded-lg border p-3 space-y-2">
        <h3 className="text-sm font-medium flex items-center gap-2">
          <FolderSync className="h-4 w-4" />
          {t('watch.title')}
        </h3>
        <p className="text-xs text-muted-foreground">{t('watch.description')}</p>
        {kb.watchedFolder ? (
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="text-xs truncate max-w-[300px]">
              {t('watch.watching', { path: kb.watchedFolder })}
            </Badge>
            <Button variant="outline" size="sm" onClick={handleStopWatching}>
              {t('watch.stopWatching')}
            </Button>
          </div>
        ) : (
          <Button variant="outline" size="sm" onClick={handleSelectWatchFolder}>
            <FolderOpen className="h-4 w-4 mr-1" />
            {t('watch.selectFolder')}
          </Button>
        )}
      </div>

      {/* Documents Table */}
      <div>
        <h3 className="text-sm font-medium mb-3">{t('detail.documents')}</h3>
        {documents.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('detail.noDocuments')}</p>
        ) : (
          <div className="rounded-lg border overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="text-left p-3 font-medium">{t('detail.fileName')}</th>
                  <th className="text-left p-3 font-medium w-20">{t('detail.fileType')}</th>
                  <th className="text-left p-3 font-medium w-24">{t('detail.status')}</th>
                  <th className="text-left p-3 font-medium w-20">{t('detail.chunks')}</th>
                  <th className="text-right p-3 font-medium w-24">{t('detail.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {documents.map((doc) => (
                  <tr key={doc.id} className="border-b last:border-0">
                    <td className="p-3 truncate max-w-[200px]">{doc.fileName}</td>
                    <td className="p-3">
                      <Badge variant="outline" className="text-xs">{doc.fileType}</Badge>
                    </td>
                    <td className="p-3">
                      <Badge className={`text-xs ${STATUS_COLORS[doc.status] || ''}`}>
                        {doc.status}
                      </Badge>
                    </td>
                    <td className="p-3">{doc.chunkCount}</td>
                    <td className="p-3 text-right">
                      <div className="flex justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7"
                          onClick={() => handleReprocess(doc.id)}
                          disabled={doc.status === 'processing'}
                        >
                          <RefreshCw className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7 text-destructive"
                          onClick={() => handleRemove(doc.id, doc.fileName)}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Search (F-3.6) */}
      <KnowledgeSearch kbId={kb.id} />
    </div>
  );
}
