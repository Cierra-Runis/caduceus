'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Link } from '@heroui/link';
import NextLink from 'next/link';

import { Input } from '@/components/forms/Input';
import { ZodForm } from '@/components/forms/ZodForm';
import { useRegister } from '@/hooks/useRegister';
import { RegisterRequest } from '@/lib/api/register';

export default function RegisterPage() {
  const { isMutating, trigger } = useRegister();

  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <Card className='w-full max-w-3xl p-4'>
        <ZodForm onValid={(data) => trigger(data)} schema={RegisterRequest}>
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
                  <Input
                    control={control}
                    isRequired
                    label='Confirm Password'
                    labelPlacement='outside'
                    name='confirmPassword'
                    placeholder='Confirm Password'
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
                  isDisabled={isMutating}
                  isLoading={isMutating}
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
