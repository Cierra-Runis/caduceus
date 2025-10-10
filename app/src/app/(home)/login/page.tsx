'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { addToast } from '@heroui/toast';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { useLogin } from '@/hooks/api/login';
import { LoginSchema } from '@/lib/api/login';

export default function LoginPage() {
  const router = useRouter();
  const { isMutating, trigger } = useLogin();
  const t = useTranslations('Login');

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
                  title: t('loginFailed'),
                });
              },
              onSuccess: ({
                payload: {
                  user: { username },
                },
              }) => {
                addToast({
                  color: 'success',
                  description: t('redirectingHome'),
                  onClose: () => router.push('/'),
                  shouldShowTimeoutProgress: true,
                  timeout: 3000,
                  title: t('welcome', { username }),
                });
              },
            })
          }
          schema={LoginSchema}
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
                    isRequired
                    label={t('labels.username')}
                    labelPlacement='outside'
                    name='username'
                    placeholder={t('placeholders.username')}
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    description={
                      <NextLink className='text-primary' href='/'>
                        {t('forgetPassword')}
                      </NextLink>
                    }
                    isRequired
                    label={t('labels.password')}
                    labelPlacement='outside'
                    name='password'
                    placeholder={t('placeholders.password')}
                    type='password'
                    variant='bordered'
                  />
                </div>
              </CardBody>
              <CardFooter className='flex justify-end gap-4'>
                <Button as={NextLink} href='/register' variant='light'>
                  {t('newTo')}
                </Button>
                <Button
                  color='primary'
                  isDisabled={isMutating}
                  isLoading={isMutating}
                  type='submit'
                >
                  {t('login')}
                </Button>
              </CardFooter>
            </>
          )}
        </ZodForm>
      </Card>
    </main>
  );
}
