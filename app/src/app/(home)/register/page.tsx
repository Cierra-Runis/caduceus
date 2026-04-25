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
import { useRegister } from '@/hooks/api/register';
import { useRegisterRequestSchema } from '@/lib/api/register';

export default function RegisterPage() {
  const router = useRouter();
  const { isMutating, trigger } = useRegister();
  const t = useTranslations('Register');
  const RegisterRequestSchema = useRegisterRequestSchema();

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-lg'>
        <CardHeader>
          <CardTitle>{t('title')}</CardTitle>
        </CardHeader>
        <CardContent>
          <ZodForm
            id='register-form'
            onValid={(data) =>
              trigger(data, {
                onError: (error) => {
                  toast.error(t('creationFailed'), {
                    description: error.message,
                  });
                },
                onSuccess: ({
                  payload: {
                    user: { username },
                  },
                }) => {
                  toast.success(t('welcome', { username }), {
                    description: t('redirectingLogin'),
                    dismissible: false,
                    onAutoClose: () => router.push('/login'),
                  });
                },
              })
            }
            schema={RegisterRequestSchema}
          >
            {(control) => (
              <FieldGroup>
                <Input
                  control={control}
                  description={t('descriptions.username')}
                  inputProps={{
                    placeholder: t('placeholders.username'),
                    required: true,
                  }}
                  label={t('labels.username')}
                  name='username'
                />
                <Input
                  control={control}
                  description={t('descriptions.nickname')}
                  inputProps={{
                    placeholder: t('placeholders.nickname'),
                  }}
                  label={t('labels.nickname')}
                  name='nickname'
                />
                <Input
                  control={control}
                  description={t('descriptions.password')}
                  inputProps={{
                    placeholder: t('placeholders.password'),
                    required: true,
                    type: 'password',
                  }}
                  label={t('labels.password')}
                  name='password'
                />
                <Input
                  control={control}
                  inputProps={{
                    placeholder: t('placeholders.confirmPassword'),
                    required: true,
                    type: 'password',
                  }}
                  label={t('labels.confirmPassword')}
                  name='confirmPassword'
                />
              </FieldGroup>
            )}
          </ZodForm>
        </CardContent>
        <CardFooter>
          <Field className='justify-end' orientation='horizontal'>
            <Button asChild variant='link'>
              <NextLink href='/login'>{t('alreadyHave')}</NextLink>
            </Button>
            <Button disabled={isMutating} form='register-form' type='submit'>
              {isMutating && <Spinner data-icon='inline-start' />}
              {t('register')}
            </Button>
          </Field>
        </CardFooter>
      </Card>
    </main>
  );
}
