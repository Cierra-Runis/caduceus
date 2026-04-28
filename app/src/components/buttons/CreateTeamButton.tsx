'use client';

import { PlusIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';

import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger,
} from '../ui/dialog';

export function CreateTeamButton() {
  const t = useTranslations('CreateTeam');
  // const { isOpen, onOpen, onOpenChange } = useDisclosure();
  // const { isMutating, trigger } = useCreateTeam();

  return (
    <Dialog>
      <DialogTrigger className='aspect-square h-auto w-full'>
        <Button asChild size='icon' variant='ghost'>
          <PlusIcon />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogTitle>{t('title')}</DialogTitle>
      </DialogContent>
      {/* <Button isIconOnly onPress={onOpen} {...props}>
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
                schema={CreateTeamRequestSchema}
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
      </Modal> */}
    </Dialog>
  );
}
