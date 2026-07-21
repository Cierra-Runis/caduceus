'use client';

import { DownloadIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import { useProjectDetail } from '@/hooks/api/project';
import { Project, ProjectDetail } from '@/lib/types/project';
import { compileProjectToPdf, TypstSourceFile } from '@/lib/typst';

export function DownloadProjectButton({
  project,
  ...props
}: {
  project: Project;
} & Omit<React.ComponentProps<typeof Button>, 'children' | 'onClick'>) {
  const t = useTranslations('DownloadProject');
  const { isMutating, trigger } = useProjectDetail();

  const download = async () => {
    try {
      const { payload } = await trigger(project.id);
      const pdf = await compileProjectToPdf(
        entryPath(payload),
        textSources(payload),
      );
      saveBytes(pdf, `${project.name}.pdf`);
    } catch (error) {
      toast.error(t('failed'), {
        description: error instanceof Error ? error.message : String(error),
      });
    }
  };

  return (
    <Button disabled={isMutating} onClick={download} {...props}>
      {isMutating ? <Spinner /> : <DownloadIcon />}
    </Button>
  );
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

function textSources(project: ProjectDetail): TypstSourceFile[] {
  return project.files.flatMap((file) =>
    file.content.kind === 'text'
      ? [{ path: file.path, text: file.content.text }]
      : [],
  );
}
