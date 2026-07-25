'use client';

import { PlusIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useState } from 'react';
import { toast } from 'sonner';
import { mutate } from 'swr';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { FieldGroup } from '@/components/ui/field';
import { Spinner } from '@/components/ui/spinner';
import { useCreateProject } from '@/hooks/api/project';
import { teamProjectKey } from '@/hooks/api/team';
import { CreateProjectRequestSchema } from '@/lib/api/project';

export function CreateProjectButton({
  children,
  ownerId,
  ownerType,
  ...props
}: {
  ownerId: string;
  ownerType: 'team' | 'user';
} & React.ComponentProps<typeof Button>) {
  const t = useTranslations('CreateProject');
  const [open, setOpen] = useState(false);
  const { isMutating, trigger } = useCreateProject();

  return (
    <Dialog onOpenChange={setOpen} open={open}>
      <DialogTrigger asChild>
        <Button {...props}>
          {children ?? <PlusIcon data-icon='inline-start' />}
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('title')}</DialogTitle>
        </DialogHeader>
        <ZodForm
          defaultValues={{ name: '' }}
          id='create-project-form'
          onValid={(data) =>
            trigger(
              { ...data, owner_id: ownerId, owner_type: ownerType },
              {
                onError: (error) => {
                  toast.error(t('creationFailed'), {
                    description: error.message,
                  });
                },
                onSuccess: () => {
                  toast.success(t('creationSucceeded'), {
                    description: t('created'),
                  });
                  setOpen(false);
                  mutate(
                    ownerType === 'team'
                      ? teamProjectKey(ownerId)
                      : 'user/projects',
                  );
                },
              },
            )
          }
          schema={CreateProjectRequestSchema}
        >
          {(control) => (
            <FieldGroup>
              <Input
                control={control}
                inputProps={{
                  placeholder: t('placeholders.name'),
                  required: true,
                }}
                label={t('labels.name')}
                name='name'
              />
            </FieldGroup>
          )}
        </ZodForm>
        <DialogFooter>
          <Button
            disabled={isMutating}
            form='create-project-form'
            type='submit'
          >
            {isMutating && <Spinner data-icon='inline-start' />}
            {t('create')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
