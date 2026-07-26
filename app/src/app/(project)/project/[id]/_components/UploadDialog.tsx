'use client';

import { CheckIcon, Loader2Icon, TriangleAlertIcon, XIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useRef, useState } from 'react';

import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { uploadBlobWithProgress } from '@/lib/api/blob';
import { cn } from '@/lib/utils';

export interface UploadDialogProps {
  onOpenChange: (open: boolean) => void;
  /// Called for each blob once uploaded, to create its file node. May throw
  /// (e.g. a duplicate name); the error is surfaced on that row.
  onUploaded: (name: string, sha256: string, size: number) => void;
  open: boolean;
  projectId: string;
}

interface UploadItem {
  error?: string;
  file: File;
  id: number;
  progress: number;
  status: 'done' | 'error' | 'queued' | 'uploading';
}

/// Stages picked files, then uploads each as a content-addressed blob with a
/// per-file progress bar, creating a file node for each on success.
export function UploadDialog({
  onOpenChange,
  onUploaded,
  open,
  projectId,
}: UploadDialogProps) {
  const t = useTranslations('Editor');
  const [items, setItems] = useState<UploadItem[]>([]);
  const nextId = useRef(0);
  const fileInput = useRef<HTMLInputElement>(null);

  const uploading = items.some((item) => item.status === 'uploading');
  const pendingCount = items.filter(
    (item) => item.status === 'error' || item.status === 'queued',
  ).length;

  const patch = (id: number, next: Partial<UploadItem>) =>
    setItems((prev) =>
      prev.map((item) => (item.id === id ? { ...item, ...next } : item)),
    );

  const addFiles = (files: FileList) => {
    const staged = [...files].map<UploadItem>((file) => ({
      file,
      id: nextId.current++,
      progress: 0,
      status: 'queued',
    }));
    setItems((prev) => [...prev, ...staged]);
  };

  const startUpload = async () => {
    // Snapshot what needs sending (queued or a previous failure to retry).
    const pending = items.filter(
      (item) => item.status === 'error' || item.status === 'queued',
    );
    for (const item of pending) {
      patch(item.id, { error: undefined, progress: 0, status: 'uploading' });
      try {
        const { sha256, size } = await uploadBlobWithProgress(
          projectId,
          item.file,
          (fraction) => patch(item.id, { progress: fraction }),
        );
        // Create the node; a duplicate name throws and lands on the row.
        onUploaded(item.file.name, sha256, size);
        patch(item.id, { progress: 1, status: 'done' });
      } catch (error) {
        patch(item.id, {
          error: error instanceof Error ? error.message : undefined,
          status: 'error',
        });
      }
    }
  };

  const handleOpenChange = (next: boolean) => {
    if (!next) setItems([]); // reset the staging list when the dialog closes
    onOpenChange(next);
  };

  return (
    <Dialog onOpenChange={handleOpenChange} open={open}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('upload.title')}</DialogTitle>
        </DialogHeader>

        <input
          className='hidden'
          multiple
          onChange={(event) => {
            if (event.target.files) addFiles(event.target.files);
            event.target.value = ''; // allow re-picking the same file
          }}
          ref={fileInput}
          type='file'
        />
        <Button
          onClick={() => fileInput.current?.click()}
          type='button'
          variant='outline'
        >
          {t('upload.choose')}
        </Button>

        {items.length === 0 ? (
          <p className='py-4 text-center text-sm opacity-60'>
            {t('upload.empty')}
          </p>
        ) : (
          <ul className='flex max-h-64 flex-col gap-2 overflow-auto'>
            {items.map((item) => (
              <li className='flex flex-col gap-1' key={item.id}>
                <div className='flex items-center gap-2 text-sm'>
                  <StatusIcon status={item.status} />
                  <span className='min-w-0 flex-1 truncate' title={item.file.name}>
                    {item.file.name}
                  </span>
                  {item.status === 'queued' && (
                    <button
                      aria-label={t('upload.remove', { name: item.file.name })}
                      className='rounded-sm p-1 hover:bg-accent'
                      onClick={() =>
                        setItems((prev) =>
                          prev.filter((other) => other.id !== item.id),
                        )
                      }
                    >
                      <XIcon className='size-3.5' />
                    </button>
                  )}
                </div>
                <div className='h-1 overflow-hidden rounded-full bg-muted'>
                  <div
                    className={cn(
                      'h-full transition-[width]',
                      item.status === 'error' ? 'bg-destructive' : 'bg-primary',
                    )}
                    style={{ width: `${Math.round(item.progress * 100)}%` }}
                  />
                </div>
                {item.error && (
                  <span className='text-xs text-destructive'>{item.error}</span>
                )}
              </li>
            ))}
          </ul>
        )}

        <DialogFooter>
          <Button
            disabled={pendingCount === 0 || uploading}
            onClick={startUpload}
            type='button'
          >
            {t('upload.start')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function StatusIcon({ status }: { status: UploadItem['status'] }) {
  if (status === 'done') return <CheckIcon className='size-4 text-primary' />;
  if (status === 'error') {
    return <TriangleAlertIcon className='size-4 text-destructive' />;
  }
  if (status === 'uploading') {
    return <Loader2Icon className='size-4 animate-spin opacity-70' />;
  }
  return <span className='size-4 shrink-0' />;
}
