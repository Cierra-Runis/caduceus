'use client';

import { Button, ButtonProps } from '@heroui/button';
import { Listbox, ListboxItem, ListboxSection } from '@heroui/listbox';
import {
  Modal,
  ModalBody,
  ModalContent,
  ModalFooter,
  ModalHeader,
  useDisclosure,
} from '@heroui/modal';
import { addToast } from '@heroui/toast';
import { useTranslations } from 'next-intl';
import { Controller } from 'react-hook-form';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserTeams } from '@/hooks/api/user/team';
import { UpdateProjectRequestSchema } from '@/lib/api/project';
import { Project } from '@/lib/types/project';

export function UpdateProjectButton({
  project,
  ...props
}: {
  project: Project;
} & ButtonProps) {
  const t = useTranslations('UpdateProject');
  const { data: user, error } = useUserMe();
  const { data: teams } = useUserTeams();
  const { isOpen, onOpen, onOpenChange } = useDisclosure();

  return (
    <>
      <Button {...props} onPress={onOpen} />
      <Modal isOpen={isOpen} onOpenChange={onOpenChange}>
        <ModalContent>
          {(onClose) => (
            <>
              <ModalHeader className='flex flex-col gap-1'>
                {t('title')}
              </ModalHeader>
              <ZodForm
                className='contents' // Prevent extra div breaking Modal layout
                defaultValues={project}
                onValid={(data) => {
                  if (!user?.payload.id)
                    return addToast({
                      color: 'danger',
                      description: error?.message,
                    });
                  console.log(data);
                }}
                schema={UpdateProjectRequestSchema}
              >
                {(control, setValue) => (
                  <>
                    <ModalBody>
                      <Input
                        control={control}
                        label={t('labels.name')}
                        name='name'
                        placeholder={t('placeholders.name')}
                        variant='bordered'
                      />
                      <Controller
                        control={control}
                        name='owner_id'
                        render={({ field: { onChange, value } }) => (
                          <Listbox
                            label={t('labels.location')}
                            selectedKeys={[value]}
                            selectionMode='single'
                          >
                            <ListboxSection title='User'>
                              <ListboxItem
                                key={user?.payload.id}
                                onPress={() => {
                                  onChange(user?.payload.id);
                                  setValue('owner_type', 'user');
                                }}
                              >
                                {project.owner_id}
                              </ListboxItem>
                            </ListboxSection>
                            <ListboxSection
                              items={teams?.payload || []}
                              title='Teams'
                            >
                              {(team) => (
                                <ListboxItem
                                  key={team.id}
                                  onPress={() => {
                                    onChange(team.id);
                                    setValue('owner_type', 'team');
                                  }}
                                >
                                  {team.name}
                                </ListboxItem>
                              )}
                            </ListboxSection>
                          </Listbox>
                        )}
                      />
                    </ModalBody>
                    <ModalFooter>
                      <Button color='danger' onPress={onClose} variant='light'>
                        {t('delete')}
                      </Button>
                      <Button color='primary' type='submit'>
                        {t('save')}
                      </Button>
                    </ModalFooter>
                  </>
                )}
              </ZodForm>
            </>
          )}
        </ModalContent>
      </Modal>
    </>
  );
}
