'use client';

import { Button, ButtonProps } from '@heroui/button';
import { Form } from '@heroui/form';
import { Input } from '@heroui/input';
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
import { FormEvent, useState } from 'react';

export function CreateTeamButton({ ...props }: ButtonProps) {
  const { isOpen, onOpen, onOpenChange } = useDisclosure();
  const [submitting, setSubmitting] = useState(false);

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    const form = new FormData(e.currentTarget);
    const payload = {
      name: String(form.get('name') || ''),
    };

    if (!payload.name) {
      return addToast({
        color: 'warning',
        description: 'Please provide a team name.',
        title: 'Creation Failed',
      });
    }

    // try {
    //   setSubmitting(true);
    //   const res = await axios.post('/api/team', payload, {
    //     headers: { 'Content-Type': 'application/json' },
    //     withCredentials: true,
    //   });
    //   addToast({
    //     color: 'success',
    //     description: 'Team created successfully!',
    //     timeout: 3000,
    //     title: res.data.message,
    //   });
    // } catch (err: unknown) {
    //   let message = 'An unexpected error occurred';
    //   if (err instanceof Error) {
    //     message = err.message;
    //   }
    //   addToast({
    //     color: 'danger',
    //     description: message,
    //     title: 'Creation Failed',
    //   });
    // } finally {
    //   setSubmitting(false);
    // }
  };

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
              <Form className='contents' onSubmit={onSubmit}>
                <ModalBody>
                  <Input
                    label='Team Name'
                    name='name'
                    placeholder='Enter your team name'
                    variant='bordered'
                  />
                </ModalBody>
                <ModalFooter>
                  <Button
                    color='primary'
                    isDisabled={submitting}
                    isLoading={submitting}
                    onPress={onClose}
                    type='submit'
                  >
                    Create
                  </Button>
                </ModalFooter>
              </Form>
            </>
          )}
        </ModalContent>
      </Modal>
    </>
  );
}
