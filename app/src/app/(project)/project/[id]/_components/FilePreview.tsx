'use client';

import { DownloadIcon, FileQuestionIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useEffect, useRef, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
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

// Formats the browser's FontFace API can render for a specimen. This is wider
// than what Typst can *use* (WOFF/WOFF2 preview fine here even though Typst
// rejects them) — the preview is purely a browser capability.
const FONT_EXTS = new Set(['otf', 'ttc', 'ttf', 'woff', 'woff2']);

const SPECIMEN_SIZES = [48, 36, 24, 18, 14];

export interface FilePreviewProps {
  /// Font family names from server metadata, when the file is a font.
  families?: string[];
  fileId: string;
  path: string;
  projectId: string;
}

/// Read-only viewer that replaces the code editor when a non-text file is
/// focused. Support is intentionally incremental — images, PDFs, and font
/// specimens today, more as we add them; anything else offers a download.
export function FilePreview({
  families,
  fileId,
  path,
  projectId,
}: FilePreviewProps) {
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

  // A font either by extension or by the server having recognized it.
  if (FONT_EXTS.has(ext) || (families && families.length > 0)) {
    return <FontPreview families={families} name={name} src={src} />;
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

/// Font specimen: loads the font via the FontFace API (under a generated,
/// collision-free family id) and renders sample text at several sizes. The
/// server-parsed family names are shown as the heading.
function FontPreview({
  families,
  name,
  src,
}: {
  families?: string[];
  name: string;
  src: string;
}) {
  const t = useTranslations('FileExplorer');
  const [status, setStatus] = useState<'error' | 'loading' | 'ready'>(
    'loading',
  );
  // Stable, unique CSS family id for this preview instance.
  const familyId = useRef(
    `caduceus-font-preview-${Math.random().toString(36).slice(2)}`,
  ).current;

  useEffect(() => {
    let cancelled = false;
    let face: FontFace | null = null;
    (async () => {
      try {
        const buffer = await fetch(src, { credentials: 'include' }).then((res) =>
          res.arrayBuffer(),
        );
        face = new FontFace(familyId, buffer);
        await face.load();
        if (cancelled) return;
        document.fonts.add(face);
        setStatus('ready');
      } catch {
        if (!cancelled) setStatus('error');
      }
    })();
    return () => {
      cancelled = true;
      if (face) document.fonts.delete(face);
    };
  }, [src, familyId]);

  const heading = families && families.length > 0 ? families.join(', ') : name;

  return (
    <div className='flex h-full flex-col'>
      <div className='border-b px-3 py-2'>
        <div className='truncate text-sm font-medium' title={heading}>
          {heading}
        </div>
        <div className='truncate text-xs opacity-50'>{name}</div>
      </div>
      {status === 'loading' && (
        <div className='flex flex-1 items-center justify-center'>
          <Spinner />
        </div>
      )}
      {status === 'error' && (
        <div
          className={`
            flex flex-1 items-center justify-center p-6 text-center text-sm
            text-destructive
          `}
        >
          {t('preview.fontError')}
        </div>
      )}
      {status === 'ready' && (
        <div
          className='flex flex-1 flex-col gap-4 overflow-auto p-4'
          style={{ fontFamily: `"${familyId}", sans-serif` }}
        >
          {SPECIMEN_SIZES.map((size) => (
            <p
              className='leading-tight wrap-break-word'
              key={size}
              style={{ fontSize: size }}
            >
              The quick brown fox jumps over the lazy dog
            </p>
          ))}
          <p className='wrap-break-word' style={{ fontSize: 24 }}>
            ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz 0123456789
          </p>
          <p className='wrap-break-word' style={{ fontSize: 24 }}>
            天地玄黄 宇宙洪荒 日月盈昃 辰宿列张 —— 视之不见 听之不闻
          </p>
        </div>
      )}
    </div>
  );
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
