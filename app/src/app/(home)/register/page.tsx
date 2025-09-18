'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Link } from '@heroui/link';
import { addToast } from '@heroui/toast';
import axios, { AxiosError } from 'axios';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { SubmitHandler } from 'react-hook-form';
import z from 'zod';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';

const RegisterSchema = z.object({
  nickname: z.string().optional(),
  password: z
    .string()
    .min(1, 'Password is required')
    .regex(
      /^(?=.{15,}$)|(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/,
      'Password should be at least 15 characters OR at least 8 characters including a number and a lowercase letter.',
    ),
  username: z
    .string()
    .min(1, 'Username is required')
    .regex(
      /^(?!-)[a-zA-Z0-9-]{1,39}(?<!-)$/,
      'Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen.',
    ),
});

export default function RegisterPage() {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);

  const onValid: SubmitHandler<z.infer<typeof RegisterSchema>> = async (
    payload,
  ) => {
    // TODO: Find a better way to avoid try-catch
    try {
      setSubmitting(true);
      const res = await axios.post('/api/register', payload, {
        headers: { 'Content-Type': 'application/json' },
        withCredentials: true,
      });
      addToast({
        color: 'success',
        description: 'Redirecting to login page...',
        onClose: () => router.push('/login'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
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
        title: 'Register Failed',
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-3xl p-4'>
        <ZodForm onValid={onValid} schema={RegisterSchema}>
          {(control) => (
            <>
              <CardHeader className='flex items-center justify-between'>
                <h1 className='text-2xl font-bold'>Register</h1>
                <Button as={NextLink} href='/' size='sm' variant='light'>
                  Back to homepage
                </Button>
              </CardHeader>
              <CardBody>
                <div className='flex flex-col gap-4'>
                  <Input
                    control={control}
                    description='Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen.'
                    isRequired
                    label='Username'
                    labelPlacement='outside'
                    name='username'
                    placeholder='Username'
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    description='Nickname can contain any characters you want and it will not used for identification.'
                    label='Nickname'
                    labelPlacement='outside'
                    name='nickname'
                    placeholder='Nickname'
                    variant='bordered'
                  />
                  <Input
                    control={control}
                    description='Password should be at least 15 characters OR at least 8 characters including a number and a lowercase letter.'
                    isRequired
                    label='Password'
                    labelPlacement='outside'
                    name='password'
                    placeholder='Password'
                    type='password'
                    variant='bordered'
                  />
                </div>
                <p className='mt-4 text-sm'>
                  By signing up, you confirm that you have read and accepted our{' '}
                  <Link className='text-sm' href='/privacy'>
                    Privacy Policy
                  </Link>
                  .
                </p>
              </CardBody>
              <CardFooter className='flex justify-end gap-4'>
                <Button as={NextLink} href='/login' variant='light'>
                  Already have an account?
                </Button>
                <Button
                  color='primary'
                  isDisabled={submitting}
                  isLoading={submitting}
                  type='submit'
                >
                  Register
                </Button>
              </CardFooter>
            </>
          )}
        </ZodForm>
      </Card>
    </main>
  );
}
