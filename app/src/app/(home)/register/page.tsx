'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Form } from '@heroui/form';
import { Input } from '@heroui/input';
import { Link } from '@heroui/link';
import { addToast } from '@heroui/toast';
import axios from 'axios';
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
      username: String(form.get('username') || ''),
      nickname: String(form.get('nickname') || ''),
      password: String(form.get('password') || ''),
    };

    if (!payload.username || !payload.password) {
      return addToast({
        color: 'warning',
        title: 'Register Failed',
        description: 'Please fill in both username and password.',
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
        title: res.data.message,
        description: 'Redirecting to login page...',
        timeout: 3000,
        shouldShowTimeoutProgress: true,
        onClose: () => router.push('/login'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
      });
    } catch (err: any) {
      const message = err?.response?.data?.message || err?.message;
      addToast({
        color: 'danger',
        title: 'Register Failed',
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
            <h1 className='text-2xl font-bold'>Register</h1>
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
                description='Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen.'
                placeholder='Username'
                name='username'
                isRequired
              />
              <Input
                label='Nickname'
                labelPlacement='outside'
                variant='bordered'
                description='Nickname can contain any characters you want and it will not used for identification.'
                placeholder='Nickname'
                name='nickname'
              />
              <Input
                label='Password'
                labelPlacement='outside'
                variant='bordered'
                description='Password should be at least 15 characters OR at least 8 characters including a number and a lowercase letter.'
                placeholder='Password'
                name='password'
                type='password'
                isRequired
              />
            </div>
            <p className='mt-4 text-sm'>
              By signing up, you confirm that you have read and accepted our{' '}
              <Link href='/privacy' className='text-sm'>
                Privacy Policy
              </Link>
              .
            </p>
          </CardBody>
          <CardFooter className='flex justify-end gap-4'>
            <Button href='/login' as={NextLink} variant='light'>
              Already have an account?
            </Button>
            <Button
              type='submit'
              color='primary'
              isLoading={submitting}
              isDisabled={submitting}
            >
              Register
            </Button>
          </CardFooter>
        </Form>
      </Card>
    </section>
  );
}
