'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Link } from '@heroui/link';
import { addToast } from '@heroui/toast';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { useRegister } from '@/hooks/api/register';
import { RegisterRequestSchema } from '@/lib/api/register';

export default function RegisterPage() {
  const router = useRouter();
  const { isMutating, trigger } = useRegister();
  const t = useTranslations('Register');

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-3xl p-4'>
        <ZodForm
          onValid={(data) =>
            trigger(data, {
              onError: (error) => {
                addToast({
                  color: 'danger',
                  description: error.message,
                  title: t('creationFailed'),
                });
              },
              onSuccess: ({
                payload: {
                  user: { username },
                },
              }) => {
                addToast({
                  color: 'success',
                  description: t('redirectingLogin'),
                  onClose: () => router.push('/login'),
                  shouldShowTimeoutProgress: true,
                  timeout: 3000,
                  title: t('welcome', { username }),
                });
              },
            })
          }
          schema={RegisterRequestSchema}
        >
          {(control) => (
            <>
              <CardHeader className='flex items-center justify-between'>
                <h1 className='text-2xl font-bold'>{t('title')}</h1>
                <Button as={NextLink} href='/' size='sm' variant='light'>
                  {t('backToHome')}
                </Button>
              </CardHeader>
              <CardBody>
                <div className='flex flex-col gap-4'>
                  <Input
                    control={control}
                    description={t('descriptions.username')}
                    isRequired
                    label={t('labels.username')}
                    labelPlacement='outside'
                    name='username'
                    placeholder={t('placeholders.username')}
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    description={t('descriptions.nickname')}
                    label={t('labels.nickname')}
                    labelPlacement='outside'
                    name='nickname'
                    placeholder={t('placeholders.nickname')}
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    description={t('descriptions.password')}
                    isRequired
                    label={t('labels.password')}
                    labelPlacement='outside'
                    name='password'
                    placeholder={t('placeholders.password')}
                    type='password'
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    isRequired
                    label={t('labels.confirmPassword')}
                    labelPlacement='outside'
                    name='confirmPassword'
                    placeholder={t('placeholders.confirmPassword')}
                    type='password'
                    variant='bordered'
                  />
                </div>
                <p className='mt-4 text-sm'>
                  {t.rich('agree', {
                    privacy: (chunks) => (
                      <Link className='text-sm' href='/privacy'>
                        {chunks}
                      </Link>
                    ),
                  })}
                </p>
              </CardBody>
              <CardFooter className='flex justify-end gap-4'>
                <Button as={NextLink} href='/login' variant='light'>
                  {t('alreadyHave')}
                </Button>
                <Button
                  color='primary'
                  isDisabled={isMutating}
                  isLoading={isMutating}
                  type='submit'
                >
                  {t('register')}
                </Button>
              </CardFooter>
            </>
          )}
        </ZodForm>
      </Card>
    </main>
  );
}
