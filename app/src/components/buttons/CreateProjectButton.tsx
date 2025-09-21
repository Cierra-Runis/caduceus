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

import { useCreateProject } from '@/hooks/useCreateProject';
import { useUserMe } from '@/hooks/useUserMe';
import { CreateProjectRequest } from '@/lib/api/project';

import { Input } from '../forms/Input';
import { ZodForm } from '../forms/ZodForm';

export function CreateProjectButton({ ...props }: ButtonProps) {
  const { data: user } = useUserMe();
  const { isOpen, onOpen, onOpenChange } = useDisclosure();

  const { isMutating, trigger } = useCreateProject();

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
                Create Project
              </ModalHeader>
              <ZodForm
                className='contents' // Prevent extra div breaking Modal layout
                onValid={(data) => {
                  if (!user?.payload.id) return;

                  trigger(
                    {
                      ...data,
                      owner_id: user.payload.id,
                      owner_type: 'user',
                    },
                    {
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
                          description: 'Project created successfully!',
                          timeout: 3000,
                          title: 'Creation Successful',
                        });
                        onClose();
                        mutate('/api/user/teams');
                      },
                    },
                  );
                }}
                schema={CreateProjectRequest}
              >
                {(control) => (
                  <>
                    <ModalBody>
                      <Input
                        control={control}
                        label='Project Name'
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
