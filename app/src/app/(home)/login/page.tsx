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
      password: String(form.get('password') || ''),
      username: String(form.get('username') || ''),
    };

    if (!payload.username || !payload.password) {
      return addToast({
        color: 'warning',
        description: 'Please fill in both username and password.',
        title: 'Login Failed',
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
        description: 'Redirecting to homepage...',
        onClose: () => router.push('/'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
        shouldShowTimeoutProgress: true,
        timeout: 3000,
        title: res.data.message,
      });
    } catch (err: any) {
      const message = err?.response?.data?.message || err?.message;
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
    <section className='flex items-center flex-col justify-center h-screen p-4'>
      <Card className='w-full max-w-3xl p-4'>
        <Form onSubmit={onSubmit}>
          <CardHeader className='flex justify-between items-center'>
            <h1 className='text-2xl font-bold'>Login</h1>
            <Button as={NextLink} href='/' size='sm' variant='light'>
              Back to homepage
            </Button>
          </CardHeader>
          <CardBody>
            <div className='flex flex-col gap-4'>
              <Input
                isRequired
                label='Username'
                labelPlacement='outside'
                name='username'
                placeholder='Username'
                variant='bordered'
              />
              <Input
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
        </Form>
      </Card>
    </section>
  );
}
