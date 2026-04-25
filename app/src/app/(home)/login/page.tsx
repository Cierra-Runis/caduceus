'use client';

import { useTranslations } from 'next-intl';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Field, FieldGroup } from '@/components/ui/field';
import { Spinner } from '@/components/ui/spinner';
import { useLogin } from '@/hooks/api/login';
import { useLoginRequestSchema } from '@/lib/api/login';

export default function LoginPage() {
  const router = useRouter();
  const { isMutating, trigger } = useLogin();
  const t = useTranslations('Login');
  const LoginSchema = useLoginRequestSchema();

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-sm'>
        <CardHeader>
          <CardTitle>{t('title')}</CardTitle>
        </CardHeader>
        <CardContent>
          <ZodForm
            defaultValues={{
              password: '',
              username: '',
            }}
            id='login-form'
            onValid={(data) =>
              trigger(data, {
                onError: (error) => {
                  toast.error(t('loginFailed'), {
                    description: error.message,
                  });
                },
                onSuccess: ({
                  payload: {
                    user: { username },
                  },
                }) => {
                  toast.success(t('welcome', { username }), {
                    description: t('redirectingHome'),
                    dismissible: false,
                    onAutoClose: () => router.push('/'),
                  });
                },
              })
            }
            schema={LoginSchema}
          >
            {(control) => (
              <FieldGroup>
                <Input
                  control={control}
                  inputProps={{
                    placeholder: t('validation.username'),
                    required: true,
                  }}
                  label={t('labels.username')}
                  name='username'
                />
                <Input
                  control={control}
                  helper={
                    <NextLink
                      className={`
                        ml-auto
                        hover:underline
                      `}
                      href='/'
                    >
                      {t('forgetPassword')}
                    </NextLink>
                  }
                  inputProps={{
                    placeholder: t('placeholders.password'),
                    required: true,
                    type: 'password',
                  }}
                  label={t('labels.password')}
                  name='password'
                />
              </FieldGroup>
            )}
          </ZodForm>
        </CardContent>
        <CardFooter>
          <Field className='justify-end' orientation='horizontal'>
            <Button asChild variant='link'>
              <NextLink href='/register'>{t('newTo')}</NextLink>
            </Button>
            <Button disabled={isMutating} form='login-form' type='submit'>
              {isMutating && <Spinner data-icon='inline-start' />}
              {t('login')}
            </Button>
          </Field>
        </CardFooter>
      </Card>
    </main>
  );
}
