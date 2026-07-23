'use client';

import { DownloadIcon, FileQuestionIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';

import { Button } from '@/components/ui/button';
import { fileRawUrl } from '@/lib/api/project';

const IMAGE_EXTS = new Set([
  'apng',
  'avif',
  'bmp',
  'gif',
  'ico',
  'jpeg',
  'jpg',
  'png',
  'svg',
  'webp',
]);

export interface FilePreviewProps {
  fileId: string;
  path: string;
  projectId: string;
}

/// Read-only viewer that replaces the code editor when a non-text file is
/// focused. Support is intentionally incremental — images and PDFs today,
/// more as we add them; anything else offers a download.
export function FilePreview({ fileId, path, projectId }: FilePreviewProps) {
  const t = useTranslations('FileExplorer');
  const ext = extensionOf(path);
  const src = fileRawUrl(projectId, fileId);
  const name = path.split('/').pop() ?? path;

  if (IMAGE_EXTS.has(ext)) {
    return (
      <PreviewFrame name={name}>
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          alt={name}
          className='max-h-full max-w-full object-contain'
          src={src}
        />
      </PreviewFrame>
    );
  }

  if (ext === 'pdf') {
    return (
      <PreviewFrame name={name}>
        <iframe className='size-full' src={src} title={name} />
      </PreviewFrame>
    );
  }

  return (
    <div
      className={`
        flex h-full flex-col items-center justify-center gap-3 p-6 text-center
      `}
    >
      <FileQuestionIcon className='size-10 opacity-40' />
      <div className='text-sm opacity-70'>{t('preview.unsupported')}</div>
      <div className='max-w-full truncate text-xs opacity-50' title={path}>
        {path}
      </div>
      <Button asChild size='sm' variant='outline'>
        <a download={name} href={src} rel='noreferrer' target='_blank'>
          <DownloadIcon data-icon='inline-start' /> {t('actions.download')}
        </a>
      </Button>
    </div>
  );
}

function extensionOf(path: string): string {
  const dot = path.lastIndexOf('.');
  return dot === -1 ? '' : path.slice(dot + 1).toLowerCase();
}

function PreviewFrame({
  children,
  name,
}: {
  children: React.ReactNode;
  name: string;
}) {
  return (
    <div className='flex h-full flex-col'>
      <div className='truncate border-b px-3 py-2 text-xs opacity-70'>
        {name}
      </div>
      <div className='flex flex-1 items-center justify-center overflow-auto p-4'>
        {children}
      </div>
    </div>
  );
}
