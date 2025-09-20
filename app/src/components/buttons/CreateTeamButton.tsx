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
import { mutate } from 'swr';

import { useCreateTeam } from '@/hooks/useCreateTeam';
import { CreateTeamRequest } from '@/lib/api/team';

import { Input } from '../forms/Input';
import { ZodForm } from '../forms/ZodForm';

export function CreateTeamButton({ ...props }: ButtonProps) {
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
                Create Team
              </ModalHeader>
              <ZodForm
                className='contents' // Prevent extra div breaking Modal layout
                onValid={(data) =>
                  trigger(data, {
                    onError: (error) => {
                      addToast({
                        color: 'danger',
                        description: error.message,
                        title: 'Creation Failed',
                      });
                    },
                    onSuccess: () => {
                      addToast({
                        color: 'success',
                        description: 'Team created successfully!',
                        timeout: 3000,
                        title: 'Creation Successful',
                      });
                      onClose();
                      mutate('/api/user/teams');
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
                        label='Team Name'
                        name='name'
                        placeholder='Enter your team name'
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
                        Create
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
