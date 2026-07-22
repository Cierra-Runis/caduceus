'use client';

import { CheckIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useState } from 'react';
import { Controller } from 'react-hook-form';
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
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';
import { useUpdateProject } from '@/hooks/api/project';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserTeams } from '@/hooks/api/user/team';
import { UpdateProjectRequestSchema } from '@/lib/api/project';
import { Project } from '@/lib/types/project';

export function UpdateProjectButton({
  children,
  project,
  ...props
}: {
  project: Project;
} & React.ComponentProps<typeof Button>) {
  const t = useTranslations('UpdateProject');
  const [open, setOpen] = useState(false);
  const { data: user } = useUserMe();
  const { data: teams } = useUserTeams();
  const { isMutating, trigger } = useUpdateProject(project.id);

  return (
    <Dialog onOpenChange={setOpen} open={open}>
      <DialogTrigger asChild>
        <Button {...props}>{children}</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('title')}</DialogTitle>
        </DialogHeader>
        <ZodForm
          defaultValues={project}
          id='update-project-form'
          onValid={(data) =>
            trigger(data, {
              onError: (error) => {
                toast.error(t('updateFailed'), {
                  description: error.message,
                });
              },
              onSuccess: () => {
                toast.success(t('updateSucceeded'), {
                  description: t('updated'),
                });
                setOpen(false);
                // The project may have been renamed or moved between owners,
                // so refresh both the personal and the team project lists.
                mutate('user/projects');
                mutate('team/projects');
              },
            })
          }
          schema={UpdateProjectRequestSchema}
        >
          {(control, setValue) => (
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
              <Controller
                control={control}
                name='owner_id'
                render={({ field: { onChange, value } }) => {
                  const selectedLabel =
                    value === user?.payload.id
                      ? user?.payload.username
                      : (teams?.payload.find((team) => team.id === value)
                          ?.name ?? t('placeholders.location'));

                  const select = (
                    ownerId: string,
                    ownerType: 'team' | 'user',
                  ) => {
                    onChange(ownerId);
                    setValue('owner_type', ownerType);
                  };

                  return (
                    <Field>
                      <FieldLabel>{t('labels.location')}</FieldLabel>
                      <Popover>
                        <PopoverTrigger asChild>
                          <Button
                            className='justify-between'
                            type='button'
                            variant='outline'
                          >
                            {selectedLabel}
                          </Button>
                        </PopoverTrigger>
                        <PopoverContent align='start' className='gap-1 p-1'>
                          <span className='px-2 py-1 text-xs text-muted-foreground'>
                            User
                          </span>
                          <LocationItem
                            onSelect={() =>
                              user?.payload.id &&
                              select(user.payload.id, 'user')
                            }
                            selected={value === user?.payload.id}
                          >
                            {user?.payload.username}
                          </LocationItem>
                          {!!teams?.payload.length && (
                            <>
                              <Separator className='my-1' />
                              <span className='px-2 py-1 text-xs text-muted-foreground'>
                                Teams
                              </span>
                              {teams.payload.map((team) => (
                                <LocationItem
                                  key={team.id}
                                  onSelect={() => select(team.id, 'team')}
                                  selected={value === team.id}
                                >
                                  {team.name}
                                </LocationItem>
                              ))}
                            </>
                          )}
                        </PopoverContent>
                      </Popover>
                    </Field>
                  );
                }}
              />
            </FieldGroup>
          )}
        </ZodForm>
        <DialogFooter className='sm:justify-between'>
          <Button onClick={() => setOpen(false)} variant='destructive'>
            {t('delete')}
          </Button>
          <Button
            disabled={isMutating}
            form='update-project-form'
            type='submit'
          >
            {isMutating && <Spinner data-icon='inline-start' />}
            {t('save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function LocationItem({
  children,
  onSelect,
  selected,
}: {
  children: React.ReactNode;
  onSelect: () => void;
  selected: boolean;
}) {
  return (
    <Button
      className='justify-between'
      onClick={onSelect}
      type='button'
      variant='ghost'
    >
      {children}
      {selected && <CheckIcon data-icon='inline-end' />}
    </Button>
  );
}
