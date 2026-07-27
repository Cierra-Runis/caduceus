'use client';

import { DownloadIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useState } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import { useProjectDetail } from '@/hooks/api/project';
import { fetchBlob } from '@/lib/api/blob';
import { Project, ProjectDetail } from '@/lib/types/project';
import { compileProjectToPdf, TypstSourceFile } from '@/lib/typst';
import { isBinaryPath } from '@/lib/yjs/tree';

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
      const pdf = await compileProjectToPdf(
        entryPath(payload),
        await textSources(payload),
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

// The compile root, resolved from the project's `entry` (a file id) to its
// path against the tree. Thrown as a user-facing error rather than returned as
// null, since the caller has nothing sensible to compile without it.
function entryPath(project: ProjectDetail): string {
  const entry = project.entry ? project.tree[project.entry] : undefined;
  if (entry?.kind !== 'file') {
    throw new Error('This project has no entry file to compile.');
  }
  return entry.path;
}

// Binary assets aren't wired into the compiler yet (M3, see lib/typst.ts), so
// a download only ever needs to push bytes through a throwaway <a> element.
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

// The compiler needs each text file's source. Text is no longer inlined in the
// payload, so fetch it from the referenced blob. Binary assets (by extension)
// aren't wired into the compiler yet, so they're skipped.
async function textSources(project: ProjectDetail): Promise<TypstSourceFile[]> {
  // Narrow `blob` inside the flatMap so no non-null assertion is needed.
  const sources = Object.values(project.tree).flatMap((node) =>
    node.kind === 'file' && node.blob && !isBinaryPath(node.path)
      ? [{ path: node.path, sha256: node.blob.sha256 }]
      : [],
  );
  return Promise.all(
    sources.map(async ({ path, sha256 }) => ({
      path,
      text: await (await fetchBlob(project.id, sha256)).text(),
    })),
  );
}
