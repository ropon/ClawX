/**
 * Document Upload
 * Handles uploading documents and URLs to a knowledge base.
 */
import { useState } from 'react';
import { Upload, Globe, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
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

interface DocumentUploadProps {
  kbId: string;
}

export function DocumentUpload({ kbId }: DocumentUploadProps) {
  const { t } = useTranslation('knowledge');
  const { t: tc } = useTranslation('common');
  const addDocuments = useKnowledgeStore((s) => s.addDocuments);
  const addUrl = useKnowledgeStore((s) => s.addUrl);
  const [uploading, setUploading] = useState(false);
  const [urlDialogOpen, setUrlDialogOpen] = useState(false);
  const [url, setUrl] = useState('');
  const [addingUrl, setAddingUrl] = useState(false);

  const handleUploadFiles = async () => {
    try {
      const result = await invoke<{ canceled: boolean; filePaths: string[] }>('dialog:open', {
        properties: ['openFile', 'multiSelections'],
        filters: [
          { name: 'Documents', extensions: ['pdf', 'docx', 'txt', 'md', 'csv'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (result.canceled || result.filePaths.length === 0) return;

      setUploading(true);
      await addDocuments(kbId, result.filePaths);
      toast.success(`Added ${result.filePaths.length} document(s)`);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setUploading(false);
    }
  };

  const handleAddUrl = async () => {
    if (!url.trim()) return;

    setAddingUrl(true);
    try {
      await addUrl(kbId, url.trim());
      toast.success('URL added successfully');
      setUrlDialogOpen(false);
      setUrl('');
    } catch (err) {
      toast.error(String(err));
    } finally {
      setAddingUrl(false);
    }
  };

  return (
    <>
      <div className="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleUploadFiles}
          disabled={uploading}
        >
          {uploading ? (
            <Loader2 className="h-4 w-4 mr-1 animate-spin" />
          ) : (
            <Upload className="h-4 w-4 mr-1" />
          )}
          {t('detail.addDocuments')}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setUrlDialogOpen(true)}
        >
          <Globe className="h-4 w-4 mr-1" />
          {t('detail.addUrl')}
        </Button>
      </div>

      <Dialog open={urlDialogOpen} onOpenChange={setUrlDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t('detail.urlDialogTitle')}</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">{t('detail.urlDialogDesc')}</p>
          <Input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder={t('detail.urlPlaceholder')}
            onKeyDown={(e) => e.key === 'Enter' && handleAddUrl()}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setUrlDialogOpen(false)}>
              {tc('actions.cancel')}
            </Button>
            <Button onClick={handleAddUrl} disabled={addingUrl || !url.trim()}>
              {addingUrl ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : null}
              {tc('actions.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
