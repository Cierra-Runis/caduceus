'use client';

import {
    FileIcon,
    FolderUpIcon,
    TriangleAlertIcon,
    UploadIcon,
    XIcon,
} from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Spinner } from '@/components/ui/spinner';
import { apiErrorMessage, uploadFiles } from '@/lib/api/project';
import { ancestorDirectories, isValidPath } from '@/lib/path';
import { ProjectFile } from '@/lib/types/project';

export interface UploadDialogProps {
  directories: string[];
  files: ProjectFile[];
  projectId: string;
  refresh: () => Promise<unknown>;
}

interface StagedFile {
  file: File;
  id: string;
  /// Target path relative to the project root, e.g. `assets/images/logo.png`.
  path: string;
}

export function UploadDialog({
  directories,
  files,
  projectId,
  refresh,
}: UploadDialogProps) {
  const t = useTranslations('FileExplorer');
  const [open, setOpen] = useState(false);
  const [staged, setStaged] = useState<StagedFile[]>([]);
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);

  const fileInput = useRef<HTMLInputElement>(null);
  const folderInput = useRef<HTMLInputElement>(null);

  // The set of paths already occupied in the project (files, and every folder
  // implied by a file or explicit directory). Mirrors the server's rule so the
  // user sees conflicts *before* pressing upload — the server still has the
  // final say.
  const occupied = useMemo(() => {
    const filePaths = new Set<string>();
    const dirPaths = new Set<string>();
    for (const file of files) {
      filePaths.add(file.path);
      for (const dir of ancestorDirectories(file.path)) dirPaths.add(dir);
    }
    for (const dir of directories) {
      dirPaths.add(dir);
      for (const parent of ancestorDirectories(dir)) dirPaths.add(parent);
    }
    return { dirPaths, filePaths };
  }, [files, directories]);

  const conflicts = useMemo(() => {
    const seen = new Set<string>();
    const result: Record<string, string> = {};
    for (const item of staged) {
      const path = item.path;
      let reason: null | string = null;
      if (!isValidPath(path)) reason = t('upload.conflictInvalid');
      else if (occupied.filePaths.has(path) || occupied.dirPaths.has(path))
        reason = t('upload.conflictExists');
      else if (seen.has(path)) reason = t('upload.conflictDuplicate');
      else if (
        ancestorDirectories(path).some((a) => occupied.filePaths.has(a))
      )
        reason = t('upload.conflictParentIsFile');
      seen.add(path);
      if (reason) result[item.id] = reason;
    }
    return result;
  }, [staged, occupied, t]);

  const hasConflicts = Object.keys(conflicts).length > 0;

  function addFiles(incoming: StagedFile[]) {
    setStaged((prev) => {
      const byPath = new Map(prev.map((s) => [s.path, s]));
      for (const item of incoming) byPath.set(item.path, item);
      return [...byPath.values()];
    });
  }

  function onPick(list: FileList | null, useRelative: boolean) {
    if (!list) return;
    const items: StagedFile[] = Array.from(list).map((file) => ({
      file,
      id: `${file.name}-${file.size}-${Math.random().toString(36).slice(2)}`,
      path:
        (useRelative &&
          (file as { webkitRelativePath?: string } & File)
            .webkitRelativePath) ||
        file.name,
    }));
    addFiles(items);
  }

  async function onDrop(event: React.DragEvent) {
    event.preventDefault();
    setDragOver(false);
    const entries = Array.from(event.dataTransfer.items)
      .map((item) => item.webkitGetAsEntry?.())
      .filter((e): e is FileSystemEntry => Boolean(e));
    if (entries.length > 0) {
      const collected: StagedFile[] = [];
      await Promise.all(entries.map((entry) => walkEntry(entry, collected)));
      addFiles(collected);
    } else {
      onPick(event.dataTransfer.files, false);
    }
  }

  async function commit() {
    if (staged.length === 0 || hasConflicts) return;
    setUploading(true);
    try {
      const form = new FormData();
      for (const item of staged) form.append(item.path, item.file);
      await uploadFiles(projectId, form);
      await refresh();
      toast.success(t('upload.succeeded'), {
        description: t('upload.uploadedCount', { count: staged.length }),
      });
      setStaged([]);
      setOpen(false);
    } catch (error) {
      toast.error(t('upload.failed'), {
        description: await apiErrorMessage(error),
      });
    } finally {
      setUploading(false);
    }
  }

  return (
    <Dialog
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) setStaged([]);
      }}
      open={open}
    >
      <DialogTrigger asChild>
        <Button
          aria-label={t('actions.upload')}
          size='icon'
          title={t('actions.upload')}
          variant='ghost'
        >
          <UploadIcon className='size-4' />
        </Button>
      </DialogTrigger>
      <DialogContent className='sm:max-w-lg'>
        <DialogHeader>
          <DialogTitle>{t('upload.title')}</DialogTitle>
          <DialogDescription>{t('upload.description')}</DialogDescription>
        </DialogHeader>

        <div
          className={`
            flex flex-col items-center justify-center gap-2 rounded-md border-2
            border-dashed px-4 py-8 text-center text-sm transition-colors
            ${dragOver ? 'border-primary bg-accent/50' : 'border-muted'}
          `}
          onDragLeave={() => setDragOver(false)}
          onDragOver={(e) => {
            e.preventDefault();
            setDragOver(true);
          }}
          onDrop={onDrop}
        >
          <UploadIcon className='size-6 opacity-60' />
          <p className='opacity-70'>{t('upload.dropHint')}</p>
          <div className='flex gap-2'>
            <Button
              onClick={() => fileInput.current?.click()}
              size='sm'
              type='button'
              variant='outline'
            >
              <FileIcon data-icon='inline-start' /> {t('upload.selectFiles')}
            </Button>
            <Button
              onClick={() => folderInput.current?.click()}
              size='sm'
              type='button'
              variant='outline'
            >
              <FolderUpIcon data-icon='inline-start' />{' '}
              {t('upload.selectFolder')}
            </Button>
          </div>
          <input
            className='hidden'
            multiple
            onChange={(e) => {
              onPick(e.target.value ? e.target.files : null, false);
              e.target.value = '';
            }}
            ref={fileInput}
            type='file'
          />
          <input
            className='hidden'
            multiple
            onChange={(e) => {
              onPick(e.target.value ? e.target.files : null, true);
              e.target.value = '';
            }}
            ref={folderInput}
            type='file'
            // `webkitdirectory` is non-standard (not in React's input typings)
            // but widely supported; spread it so the pick returns a folder.
            {...{ webkitdirectory: '' }}
          />
        </div>

        {staged.length > 0 && (
          <ScrollArea className='max-h-56'>
            <ul className='flex flex-col gap-1 pr-2'>
              {staged.map((item) => {
                const conflict = conflicts[item.id];
                return (
                  <li
                    className={`
                      flex items-center gap-2 rounded-sm border px-2 py-1
                      text-sm
                      ${conflict ? 'border-destructive/50' : 'border-transparent'}
                    `}
                    key={item.id}
                  >
                    <FileIcon className='size-4 shrink-0 opacity-60' />
                    <div className='min-w-0 flex-1'>
                      <div className='truncate' title={item.path}>
                        {item.path}
                      </div>
                      {conflict && (
                        <div
                          className={`
                            flex items-center gap-1 text-xs text-destructive
                          `}
                        >
                          <TriangleAlertIcon className='size-3' /> {conflict}
                        </div>
                      )}
                    </div>
                    <span className='shrink-0 text-xs opacity-50'>
                      {formatSize(item.file.size)}
                    </span>
                    <Button
                      aria-label={t('upload.remove')}
                      className='size-6'
                      onClick={() =>
                        setStaged((prev) =>
                          prev.filter((s) => s.id !== item.id),
                        )
                      }
                      size='icon'
                      variant='ghost'
                    >
                      <XIcon className='size-3.5' />
                    </Button>
                  </li>
                );
              })}
            </ul>
          </ScrollArea>
        )}

        <DialogFooter>
          <span className='mr-auto self-center text-xs opacity-60'>
            {t('upload.stagedCount', { count: staged.length })}
          </span>
          <Button
            disabled={staged.length === 0 || hasConflicts || uploading}
            onClick={commit}
            type='button'
          >
            {uploading && <Spinner data-icon='inline-start' />}
            {t('upload.upload')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/// Recursively collect files from a dropped filesystem entry, preserving the
/// dropped folder's structure as the target path.
async function walkEntry(
  entry: FileSystemEntry,
  out: StagedFile[],
): Promise<void> {
  if (entry.isFile) {
    const fileEntry = entry as FileSystemFileEntry;
    const file = await new Promise<File>((resolve, reject) =>
      fileEntry.file(resolve, reject),
    );
    out.push({
      file,
      id: `${entry.fullPath}-${Math.random().toString(36).slice(2)}`,
      // fullPath is like `/folder/file.png`; drop the leading slash.
      path: entry.fullPath.replace(/^\/+/, ''),
    });
  } else if (entry.isDirectory) {
    const reader = (entry as FileSystemDirectoryEntry).createReader();
    const children = await new Promise<FileSystemEntry[]>((resolve, reject) =>
      reader.readEntries(resolve, reject),
    );
    await Promise.all(children.map((child) => walkEntry(child, out)));
  }
}
