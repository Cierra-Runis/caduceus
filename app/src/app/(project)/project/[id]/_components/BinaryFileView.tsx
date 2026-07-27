'use client';

import { useEffect, useState } from 'react';

import { Spinner } from '@/components/ui/spinner';
import { fetchBlob } from '@/lib/api/blob';

export interface BinaryFileViewProps {
  path: string;
  projectId: string;
  sha256: string;
  size: number;
}

const IMAGE = /\.(avif|bmp|gif|ico|jpe?g|png|webp)$/i;

/// Shown in place of the code editor for a binary file (which has no text
/// overlay and must never be opened in Monaco). Images are previewed via an
/// object URL fetched from the blob endpoint; anything else shows a summary.
export function BinaryFileView({
  path,
  projectId,
  sha256,
  size,
}: BinaryFileViewProps) {
  const isImage = IMAGE.test(path);
  const [url, setUrl] = useState<null | string>(null);

  useEffect(() => {
    if (!isImage) return;
    let objectUrl: null | string = null;
    let alive = true;
    fetchBlob(projectId, sha256)
      .then((blob) => {
        if (!alive) return;
        objectUrl = URL.createObjectURL(blob);
        setUrl(objectUrl);
      })
      .catch(() => setUrl(null));
    return () => {
      alive = false;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [projectId, sha256, isImage]);

  return (
    <div className='flex h-full flex-col items-center justify-center gap-3 p-6 text-sm opacity-70'>
      {isImage ? (
        url ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            alt={path}
            className='max-h-full max-w-full object-contain'
            src={url}
          />
        ) : (
          <Spinner />
        )
      ) : (
        <>
          <span className='font-medium'>{path}</span>
          <span>Binary file · {size} bytes · not editable as text</span>
        </>
      )}
    </div>
  );
}
