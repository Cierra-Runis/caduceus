'use client';

import { FileIcon, ImageIcon, Trash2Icon, UploadIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { RefObject, useRef } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

/// A single entry in the project file tree.
export interface SidebarFile {
  /// Stable file id (the delete/asset key), independent of `path`.
  id: string;
  kind: 'binary' | 'text';
  path: string;
}

export interface SidebarPanelProps {
  /// Id of the file whose delete is in flight; its row shows a spinner and the
  /// controls are disabled. Null when no delete is running.
  deletingId: null | string;
  /// Path of the compile entry file, marked in the list. Null if none.
  entry: null | string;
  /// The whole project file tree, text and binary alike.
  files: SidebarFile[];
  /// Path of the file currently open in the editor.
  focus: string;
  /// Delete a file (removes its row and, for a binary asset, its bytes).
  onDelete: (file: SidebarFile) => void;
  onSelect: (path: string) => void;
  /// Upload a picked file as a project asset.
  onUpload: (file: File) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
  /// An upload is in flight; the control is disabled and shows a spinner.
  uploading: boolean;
}

export function SidebarPanel({
  deletingId,
  entry,
  files,
  focus,
  onDelete,
  onSelect,
  onUpload,
  sidebarPanelRef,
  uploading,
}: SidebarPanelProps) {
  const t = useTranslations('Project');
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <div className='flex items-center justify-between px-3 py-2'>
        <span className='text-xs font-medium opacity-60'>{t('files')}</span>
        <button
          className={cn(
            `flex items-center gap-1 rounded-sm px-2 py-1 text-xs`,
            uploading ? 'opacity-50' : 'hover:bg-accent',
          )}
          disabled={uploading}
          onClick={() => inputRef.current?.click()}
          type='button'
        >
          {uploading ? (
            <Spinner className='size-3' />
          ) : (
            <UploadIcon className='size-3' />
          )}
          {t('asset.upload')}
        </button>
        <input
          className='hidden'
          onChange={(event) => {
            const file = event.target.files?.[0];
            // Reset so re-picking the same file fires `onChange` again.
            event.target.value = '';
            if (file) onUpload(file);
          }}
          ref={inputRef}
          type='file'
        />
      </div>
      <ul className='flex flex-col pb-2'>
        {files.map((file) => {
          const isBinary = file.kind === 'binary';
          const isEntry = file.path === entry;
          const isDeleting = file.id === deletingId;
          return (
            <li
              className={cn(
                `group flex items-center pr-1`,
                file.path === focus ? 'bg-accent' : 'hover:bg-accent/50',
              )}
              key={file.id}
            >
              <button
                className={cn(
                  `flex min-w-0 flex-1 items-center gap-2 px-3 py-1 text-left text-sm`,
                  isBinary && 'cursor-default opacity-70',
                )}
                // Binary assets have no text buffer to open; the row is a
                // reference for `#image(...)`, not an editor target.
                disabled={isBinary}
                onClick={() => onSelect(file.path)}
                type='button'
              >
                {isBinary ? (
                  <ImageIcon className='size-4 shrink-0 opacity-60' />
                ) : (
                  <FileIcon className='size-4 shrink-0 opacity-60' />
                )}
                <span className='truncate'>{file.path}</span>
              </button>
              {isEntry ? (
                // The compile entry can't be deleted; label it instead.
                <span className='ml-auto shrink-0 px-1 text-xs opacity-50'>
                  entry
                </span>
              ) : (
                <button
                  aria-label={t('asset.delete', { path: file.path })}
                  className={cn(
                    `flex shrink-0 items-center rounded-sm p-1 text-xs`,
                    isDeleting
                      ? 'opacity-100'
                      : `opacity-0 group-hover:opacity-100 hover:bg-accent
                        focus-visible:opacity-100`,
                  )}
                  disabled={isDeleting}
                  onClick={() => onDelete(file)}
                  title={t('asset.delete', { path: file.path })}
                  type='button'
                >
                  {isDeleting ? (
                    <Spinner className='size-3' />
                  ) : (
                    <Trash2Icon className='size-3' />
                  )}
                </button>
              )}
            </li>
          );
        })}
      </ul>
    </Panel>
  );
}
