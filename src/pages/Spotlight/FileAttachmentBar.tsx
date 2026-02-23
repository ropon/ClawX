/**
 * File Attachment Bar (F-2.6)
 * Displays selected file attachment chips in the Spotlight input area
 */
import { X } from 'lucide-react';
import { useSpotlightStore } from '@/stores/spotlight';
import { getFileIcon, formatFileSize } from '@/utils/file-search-engine';

export function FileAttachmentBar() {
  const fileAttachments = useSpotlightStore((s) => s.fileAttachments);
  const removeFileAttachment = useSpotlightStore((s) => s.removeFileAttachment);

  if (fileAttachments.length === 0) return null;

  return (
    <div className="px-4 pt-1 pb-1 flex flex-wrap gap-1.5">
      {fileAttachments.map((file) => (
        <div
          key={file.id}
          className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-primary/10 text-primary text-xs font-medium max-w-[200px]"
        >
          <span className="flex-shrink-0">{getFileIcon(file.fileType)}</span>
          <span className="truncate">{file.fileName}</span>
          {file.fileSize > 0 && (
            <span className="text-[10px] text-muted-foreground flex-shrink-0">
              {formatFileSize(file.fileSize)}
            </span>
          )}
          <button
            onClick={() => removeFileAttachment(file.id)}
            className="ml-0.5 p-0.5 rounded hover:bg-primary/20 transition-colors flex-shrink-0"
          >
            <X className="h-3 w-3" />
          </button>
        </div>
      ))}
    </div>
  );
}
