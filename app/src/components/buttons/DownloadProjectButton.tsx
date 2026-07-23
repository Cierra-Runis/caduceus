'use client';

import { DownloadIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useState } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import { useProjectDetail } from '@/hooks/api/project';
import { fileRawUrl } from '@/lib/api/project';
import { Project, ProjectDetail } from '@/lib/types/project';
import { compileProjectToPdf, TypstAssetFile, TypstSourceFile } from '@/lib/typst';

export function DownloadProjectButton({
  project,
  ...props
}: {
  project: Project;
} & Omit<React.ComponentProps<typeof Button>, 'children'>) {
  const t = useTranslations('DownloadProject');
  const { trigger } = useProjectDetail();
  // Tracks the whole fetch-then-compile flow, not just the network step: the
  // WASM compile (`compileProjectToPdf`) can run long after `trigger`
  // resolves, and the button must keep showing "busy" for all of it or a
  // slow/stuck compile looks identical to nothing having happened.
  const [isDownloading, setIsDownloading] = useState(false);

  const download = async () => {
    setIsDownloading(true);
    try {
      const { payload } = await trigger(project.id);
      const { assets, sources } = await compileInputs(payload);
      const pdf = await compileProjectToPdf(
        entryPath(payload),
        sources,
        assets,
      );
      saveBytes(pdf, `${project.name}.pdf`);
    } catch (error) {
      toast.error(t('failed'), {
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setIsDownloading(false);
    }
  };

  return (
    // `props` spreads first: this is rendered via `TooltipTrigger asChild`,
    // which injects its own `onClick`/`ref`/focus handlers into the child's
    // props via Slot cloning. Spreading after our explicit `onClick` would
    // silently overwrite it, so the incoming handler is composed instead.
    <Button
      {...props}
      disabled={isDownloading}
      onClick={(event) => {
        props.onClick?.(event);
        void download();
      }}
    >
      {isDownloading ? <Spinner /> : <DownloadIcon />}
    </Button>
  );
}

// Split the project into what the compiler needs: `.typ` files as editable
// sources, and everything else (data text files + binary assets fetched from
// object storage) as shadowed bytes, so `#image`/`#read` resolve during export
// exactly like they do in the live preview.
async function compileInputs(
  project: ProjectDetail,
): Promise<{ assets: TypstAssetFile[]; sources: TypstSourceFile[] }> {
  const sources: TypstSourceFile[] = [];
  const assets: TypstAssetFile[] = [];
  const encoder = new TextEncoder();
  const binaryFetches: Promise<TypstAssetFile>[] = [];

  for (const file of project.files) {
    if (file.content.kind === 'text') {
      if (file.path.endsWith('.typ')) {
        sources.push({ path: file.path, text: file.content.text });
      } else {
        assets.push({ bytes: encoder.encode(file.content.text), path: file.path });
      }
    } else {
      const { id, path } = file;
      binaryFetches.push(
        fetch(fileRawUrl(project.id, id), { credentials: 'include' })
          .then((res) => res.arrayBuffer())
          .then((buffer) => ({ bytes: new Uint8Array(buffer), path })),
      );
    }
  }

  assets.push(...(await Promise.all(binaryFetches)));
  return { assets, sources };
}

// The compile root, resolved from the project's `entry` (a file id) to its
// path. Thrown as a user-facing error rather than returned as null, since the
// caller has nothing sensible to compile without it.
function entryPath(project: ProjectDetail): string {
  const entry = project.files.find((file) => file.id === project.entry);
  if (entry?.content.kind !== 'text') {
    throw new Error('This project has no entry file to compile.');
  }
  return entry.path;
}

function saveBytes(bytes: Uint8Array, filename: string) {
  const url = URL.createObjectURL(
    new Blob([Uint8Array.from(bytes)], { type: 'application/pdf' }),
  );
  const link = document.createElement('a');
  link.download = filename;
  link.href = url;
  link.click();
  URL.revokeObjectURL(url);
}
