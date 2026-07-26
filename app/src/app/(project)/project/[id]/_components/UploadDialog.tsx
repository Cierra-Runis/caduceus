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
import { isBinaryPath } from '@/lib/yjs/tree';

/// How many files upload at once (bounded so a folder of hundreds doesn't open
/// hundreds of sockets).
const UPLOAD_CONCURRENCY = 4;

export interface UploadDialogProps {
  onOpenChange: (open: boolean) => void;
  /// Called for each uploaded blob to create its node. `relativePath` may include
  /// folders (recreated by the caller); `text` is the decoded content when the
  /// file was detected as text, else undefined (binary). May throw (e.g. a
  /// duplicate name); the error is surfaced on that row.
  onUploaded: (
    relativePath: string,
    sha256: string,
    size: number,
    text?: string,
  ) => void;
  open: boolean;
  projectId: string;
}

interface UploadItem {
  error?: string;
  file: File;
  id: number;
  /// Path relative to the drop (folder uploads keep their structure).
  path: string;
  progress: number;
  status: 'done' | 'error' | 'queued' | 'uploading';
}

/// Stages picked files (or a whole folder), then uploads them in parallel as
/// content-addressed blobs with per-file progress, creating a node for each —
/// editable text when detected as text, otherwise a binary blob.
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
  const folderInput = useRef<HTMLInputElement>(null);

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
      // Folder uploads carry a relative path; plain files just their name.
      path: file.webkitRelativePath || file.name,
      progress: 0,
      status: 'queued',
    }));
    setItems((prev) => [...prev, ...staged]);
  };

  const uploadOne = async (item: UploadItem) => {
    patch(item.id, { error: undefined, progress: 0, status: 'uploading' });
    try {
      const { sha256, size } = await uploadBlobWithProgress(
        projectId,
        item.file,
        (fraction) => patch(item.id, { progress: fraction }),
      );
      const text = await detectText(item.file);
      // Create the node; a duplicate name throws and lands on the row.
      onUploaded(item.path, sha256, size, text ?? undefined);
      patch(item.id, { progress: 1, status: 'done' });
    } catch (error) {
      patch(item.id, {
        error: error instanceof Error ? error.message : undefined,
        status: 'error',
      });
    }
  };

  const startUpload = async () => {
    // Snapshot what needs sending (queued or a previous failure to retry) and
    // drain it through a bounded pool of workers.
    const queue = items.filter(
      (item) => item.status === 'error' || item.status === 'queued',
    );
    const worker = async () => {
      for (let item = queue.shift(); item; item = queue.shift()) {
        await uploadOne(item);
      }
    };
    await Promise.all(
      Array.from({ length: Math.min(UPLOAD_CONCURRENCY, queue.length) }, worker),
    );
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
        <input
          className='hidden'
          onChange={(event) => {
            if (event.target.files) addFiles(event.target.files);
            event.target.value = '';
          }}
          // `webkitdirectory` turns the picker into a folder chooser; the picked
          // files carry a `webkitRelativePath` we use to recreate the tree.
          ref={(el) => {
            if (el) el.webkitdirectory = true;
            folderInput.current = el;
          }}
          type='file'
        />
        <div className='flex gap-2'>
          <Button
            className='flex-1'
            onClick={() => fileInput.current?.click()}
            type='button'
            variant='outline'
          >
            {t('upload.choose')}
          </Button>
          <Button
            className='flex-1'
            onClick={() => folderInput.current?.click()}
            type='button'
            variant='outline'
          >
            {t('upload.chooseFolder')}
          </Button>
        </div>

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
                  <span className='min-w-0 flex-1 truncate' title={item.path}>
                    {item.path}
                  </span>
                  {item.status === 'queued' && (
                    <button
                      aria-label={t('upload.remove', { name: item.path })}
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

/// Decode `file` as UTF-8 text, or `null` if it looks binary — a known-binary
/// extension, a NUL byte in its head, or invalid UTF-8. Mirrors how GitHub
/// decides whether an uploaded file is editable text.
async function detectText(file: File): Promise<null | string> {
  if (isBinaryPath(file.name)) return null;
  const head = new Uint8Array(await file.slice(0, 4096).arrayBuffer());
  if (head.includes(0)) return null;
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(
      await file.arrayBuffer(),
    );
  } catch {
    return null;
  }
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
