'use client';

import { Dialog } from '../ui/dialog';

export function CreateTeamButton() {
  // const t = useTranslations('CreateTeam');
  // const { isOpen, onOpen, onOpenChange } = useDisclosure();
  // const { isMutating, trigger } = useCreateTeam();

  return (
    <Dialog>
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
