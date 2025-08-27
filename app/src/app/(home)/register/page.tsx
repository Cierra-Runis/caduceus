'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Form } from '@heroui/form';
import { Input } from '@heroui/input';
import { Link } from '@heroui/link';
import { addToast } from '@heroui/toast';
import axios, { AxiosError } from 'axios';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';
import { FormEvent, useState } from 'react';

export default function RegisterPage() {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    const form = new FormData(e.currentTarget);
    const payload = {
      nickname: String(form.get('nickname') || ''),
      password: String(form.get('password') || ''),
      username: String(form.get('username') || ''),
    };

    if (!payload.username || !payload.password) {
      return addToast({
        color: 'warning',
        description: 'Please fill in both username and password.',
        title: 'Register Failed',
      });
    }

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
    <section className='flex h-screen flex-col items-center justify-center p-4'>
      <Card className='w-full max-w-3xl p-4'>
        <Form onSubmit={onSubmit}>
          <CardHeader className='flex items-center justify-between'>
            <h1 className='text-2xl font-bold'>Register</h1>
            <Button as={NextLink} href='/' size='sm' variant='light'>
              Back to homepage
            </Button>
          </CardHeader>
          <CardBody>
            <div className='flex flex-col gap-4'>
              <Input
                description='Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen.'
                isRequired
                label='Username'
                labelPlacement='outside'
                name='username'
                placeholder='Username'
                variant='bordered'
              />
              <Input
                description='Nickname can contain any characters you want and it will not used for identification.'
                label='Nickname'
                labelPlacement='outside'
                name='nickname'
                placeholder='Nickname'
                variant='bordered'
              />
              <Input
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
        </Form>
      </Card>
    </section>
  );
}
