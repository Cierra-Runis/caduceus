'use client';

import { Button, ButtonProps } from '@heroui/button';
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
import { mutate } from 'swr';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { useCreateProject } from '@/hooks/api/project';
import { CreateProjectRequestSchema } from '@/lib/api/project';

export function CreateProjectButton({
  ownerId,
  ownerType,
  ...props
}: {
  ownerId: string;
  ownerType: 'team' | 'user';
} & ButtonProps) {
  const t = useTranslations('CreateProject');
  const { isOpen, onOpen, onOpenChange } = useDisclosure();

  const { isMutating, trigger } = useCreateProject();

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
                onValid={(data) => {
                  trigger(
                    {
                      ...data,
                      owner_id: ownerId,
                      owner_type: ownerType,
                    },
                    {
                      onError: (error) => {
                        addToast({
                          color: 'danger',
                          description: error.message,
                          title: t('creationFailed'),
                        });
                      },
                      onSuccess: () => {
                        addToast({
                          color: 'success',
                          description: t('created'),
                          timeout: 3000,
                          title: t('creationSucceeded'),
                        });
                        onClose();
                        mutate(`${ownerType}/projects`);
                      },
                    },
                  );
                }}
                schema={CreateProjectRequestSchema}
              >
                {(control) => (
                  <>
                    <ModalBody>
                      <Input
                        control={control}
                        label={t('labels.name')}
                        name='name'
                        placeholder={t('placeholders.name')}
                        variant='bordered'
                      />
                    </ModalBody>
                    <ModalFooter>
                      <Button
                        color='primary'
                        isDisabled={isMutating}
                        isLoading={isMutating}
                        type='submit'
                      >
                        {t('create')}
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
