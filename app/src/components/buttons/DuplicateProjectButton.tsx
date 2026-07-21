'use client';

import { useTranslations } from 'next-intl';
import { toast } from 'sonner';
import { mutate } from 'swr';

import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import { useDuplicateProject } from '@/hooks/api/project';
import { Project } from '@/lib/types/project';

export function DuplicateProjectButton({
  children,
  project,
  ...props
}: {
  project: Project;
} & React.ComponentProps<typeof Button>) {
  const t = useTranslations('Dashboard');
  const { isMutating, trigger } = useDuplicateProject(project.id);

  return (
    <Button
      {...props}
      disabled={isMutating}
      onClick={() =>
        trigger(undefined, {
          onError: (error) => {
            toast.error(t('duplicationFailed'), {
              description: error.message,
            });
          },
          onSuccess: () => {
            toast.success(t('duplicationSucceeded'), {
              description: t('duplicated'),
            });
            // Revalidate the dashboard's project list so the new copy shows
            // up without a manual reload.
            mutate('user/projects');
          },
        })
      }
    >
      {isMutating ? <Spinner /> : children}
    </Button>
  );
}
