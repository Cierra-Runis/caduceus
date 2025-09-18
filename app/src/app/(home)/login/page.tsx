'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { addToast } from '@heroui/toast';
import axios, { AxiosError } from 'axios';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { SubmitHandler } from 'react-hook-form';
import z from 'zod';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';

const LoginSchema = z.object({
  password: z.string().min(1, 'Password is required'),
  username: z.string().min(1, 'Username is required'),
});

export default function LoginPage() {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);

  const onValid: SubmitHandler<z.infer<typeof LoginSchema>> = async (
    payload,
  ) => {
    try {
      setSubmitting(true);
      const res = await axios.post('/api/login', payload, {
        headers: { 'Content-Type': 'application/json' },
        withCredentials: true,
      });
      addToast({
        color: 'success',
        description: 'Redirecting to homepage...',
        onClose: () => router.push('/'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
        shouldShowTimeoutProgress: true,
        timeout: 3000,
        title: res.data.message,
      });
    } catch (err: unknown) {
      let message = 'An unexpected error occurred';
      if (err instanceof AxiosError) {
        message = err.response?.data?.message || err.message;
      } else if (err instanceof Error) {
        message = err.message;
      }
      addToast({
        color: 'danger',
        description: message,
        title: 'Login Failed',
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-3xl p-4'>
        <ZodForm onValid={onValid} schema={LoginSchema}>
          {(control) => (
            <>
              <CardHeader className='flex items-center justify-between'>
                <h1 className='text-2xl font-bold'>Login</h1>
                <Button as={NextLink} href='/' size='sm' variant='light'>
                  Back to homepage
                </Button>
              </CardHeader>
              <CardBody>
                <div className='flex flex-col gap-4'>
                  <Input
                    control={control}
                    isRequired
                    label='Username'
                    labelPlacement='outside'
                    name='username'
                    placeholder='Username'
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    description={
                      <NextLink className='text-primary' href='/'>
                        Forget Password?
                      </NextLink>
                    }
                    isRequired
                    label='Password'
                    labelPlacement='outside'
                    name='password'
                    placeholder='Password'
                    type='password'
                    variant='bordered'
                  />
                </div>
              </CardBody>
              <CardFooter className='flex justify-end gap-4'>
                <Button as={NextLink} href='/register' variant='light'>
                  New to Caduceus?
                </Button>
                <Button
                  color='primary'
                  isDisabled={submitting}
                  isLoading={submitting}
                  type='submit'
                >
                  Login
                </Button>
              </CardFooter>
            </>
          )}
        </ZodForm>
      </Card>
    </main>
  );
}
