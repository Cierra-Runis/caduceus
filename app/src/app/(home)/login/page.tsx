'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import NextLink from 'next/link';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { useLogin } from '@/hooks/useLogin';
import { LoginSchema } from '@/lib/api/login';

export default function LoginPage() {
  const { isMutating, trigger } = useLogin();

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-3xl p-4'>
        <ZodForm onValid={(data) => trigger(data)} schema={LoginSchema}>
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
                  isDisabled={isMutating}
                  isLoading={isMutating}
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
