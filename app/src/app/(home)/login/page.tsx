'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Form } from '@heroui/form';
import { Input } from '@heroui/input';
import { addToast } from '@heroui/toast';
import axios from 'axios';
import NextLink from 'next/link';
import { useRouter } from 'next/navigation';
import { FormEvent, useState } from 'react';

export default function LoginPage() {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);

  const onSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    const form = new FormData(e.currentTarget);
    const payload = {
      username: String(form.get('username') || ''),
      password: String(form.get('password') || ''),
    };

    if (!payload.username || !payload.password) {
      return addToast({
        color: 'warning',
        title: 'Login Failed',
        description: 'Please fill in both username and password.',
      });
    }

    // TODO: Find a better way to avoid try-catch
    try {
      setSubmitting(true);
      const res = await axios.post('/api/login', payload, {
        headers: { 'Content-Type': 'application/json' },
        withCredentials: true,
      });
      addToast({
        color: 'success',
        title: res.data.message,
        description: 'Redirecting to homepage...',
        timeout: 3000,
        shouldShowTimeoutProgress: true,
        onClose: () => router.push('/'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
      });
    } catch (err: any) {
      const message = err?.response?.data?.message || err?.message;
      addToast({
        color: 'danger',
        title: 'Login Failed',
        description: message,
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section className='flex items-center flex-col justify-center h-screen p-4'>
      <Card className='w-full max-w-3xl p-4'>
        <Form onSubmit={onSubmit}>
          <CardHeader className='flex justify-between items-center'>
            <h1 className='text-2xl font-bold'>Login</h1>
            <Button href='/' as={NextLink} size='sm' variant='light'>
              Back to homepage
            </Button>
          </CardHeader>
          <CardBody>
            <div className='flex flex-col gap-4'>
              <Input
                label='Username'
                labelPlacement='outside'
                variant='bordered'
                placeholder='Username'
                name='username'
                isRequired
              />
              <Input
                label='Password'
                labelPlacement='outside'
                variant='bordered'
                placeholder='Password'
                description={
                  <NextLink href='/' className='text-primary'>
                    Forget Password?
                  </NextLink>
                }
                name='password'
                type='password'
                isRequired
              />
            </div>
          </CardBody>
          <CardFooter className='flex justify-end gap-4'>
            <Button href='/register' as={NextLink} variant='light'>
              New to Caduceus?
            </Button>
            <Button
              type='submit'
              color='primary'
              isLoading={submitting}
              isDisabled={submitting}
            >
              Login
            </Button>
          </CardFooter>
        </Form>
      </Card>
    </section>
  );
}
