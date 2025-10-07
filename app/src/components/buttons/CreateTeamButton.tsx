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
import { IconPlus } from '@tabler/icons-react';
import { useTranslations } from 'next-intl';
import { mutate } from 'swr';

import { CreateTeamRequest, useCreateTeam } from '@/lib/api/team';

import { Input } from '../forms/Input';
import { ZodForm } from '../forms/ZodForm';

export function CreateTeamButton({ ...props }: ButtonProps) {
  const t = useTranslations('CreateTeam');
  const { isOpen, onOpen, onOpenChange } = useDisclosure();

  const { isMutating, trigger } = useCreateTeam();

  return (
    <>
      <Button isIconOnly onPress={onOpen} {...props}>
        <IconPlus />
      </Button>
      <Modal isOpen={isOpen} onOpenChange={onOpenChange}>
        <ModalContent>
          {(onClose) => (
            <>
              <ModalHeader className='flex flex-col gap-1'>
                {t('title')}
              </ModalHeader>
              <ZodForm
                className='contents' // Prevent extra div breaking Modal layout
                onValid={(data) =>
                  trigger(data, {
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
                      mutate('user/teams');
                    },
                  })
                }
                schema={CreateTeamRequest}
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
